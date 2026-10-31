//! Host-owned asynchronous frame production. Completion holds a weak cache handle,
//! so a retired document cannot be resurrected by a late browser worker message.

use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FrameTask {
    pub document: u64,
    pub key: FrameKey,
    pub time: Tick,
    pub scale: f32,
    pub priority: u32,
    pub prefetch: bool,
    #[serde(default)]
    pub playback: bool,
    pub catch_up: Option<Tick>,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FrameTimings {
    pub source_ms: f64,
    pub render_ms: f64,
    pub job_ms: f64,
}

pub enum ExternalFrameResult {
    Plan(GpuPlan),
    Image(Rgba),
}

struct Completion {
    shared: std::sync::Weak<Shared>,
    cancel: Arc<AtomicBool>,
}

pub struct ExternalFrameJob {
    pub task: FrameTask,
    pub project: Arc<Project>,
    completion: Option<Completion>,
}

impl FrameServer {
    pub fn document_epoch(&self) -> u64 {
        self.shared.epoch
    }
    pub fn is_external(&self) -> bool {
        self.shared.external.load(Ordering::Relaxed)
    }

    pub fn set_external(&self, enabled: bool) {
        self.shared.external.store(enabled, Ordering::Relaxed);
        self.shared.cv.notify_all();
    }

    /// One bounded host request at a time. The host chooses when it has capacity;
    /// queued foreground edits therefore remain ahead of speculative work.
    pub fn take_external_job(&self) -> Option<ExternalFrameJob> {
        if !self.is_external() || self.shared.stop.load(Ordering::Relaxed) {
            return None;
        }
        let mut queue = self.shared.queue.lock().unwrap_or_else(|e| e.into_inner());
        coalesce_monitor_revisions(&mut queue, 3);
        queue.retain(|j| !j.cancel.load(Ordering::Relaxed));
        let best = queue.iter().enumerate().min_by_key(|(i, j)| (j.prio, *i)).map(|(i, _)| i)?;
        let job = queue.remove(best)?;
        drop(queue);
        self.shared.in_flight.lock().unwrap_or_else(|e| e.into_inner()).push((job.key, job.cancel.clone(), job.prefetch));
        Some(ExternalFrameJob {
            task: FrameTask {
                document: self.shared.epoch,
                key: job.key,
                time: job.time,
                scale: job.scale,
                priority: job.prio,
                prefetch: job.prefetch,
                playback: job.playback,
                catch_up: job.catch_up,
            },
            project: job.project,
            completion: Some(Completion { shared: Arc::downgrade(&self.shared), cancel: job.cancel }),
        })
    }

    /// Reconstruct an immutable job inside a separate worker instance, preserving
    /// source-time, draft, prefetch and decoder catch-up semantics.
    pub fn enqueue_task(&self, task: &FrameTask, project: &Arc<Project>) -> Result<(), String> {
        if !task.scale.is_finite() || !(1.0 / 65536.0..=64.0).contains(&task.scale) || task.key.size == 0 || task.key.size > 16_384_000 {
            return Err("invalid external frame scale or size".into());
        }
        self.request_job(
            task.key,
            task.time,
            task.scale,
            project,
            task.priority,
            RequestMode { prefetch: task.prefetch, playback: task.playback, catch_up: task.catch_up },
        );
        Ok(())
    }
}

impl ExternalFrameJob {
    pub fn cancelled(&self) -> bool {
        self.completion.as_ref().is_none_or(|c| c.cancel.load(Ordering::Relaxed) || c.shared.upgrade().is_none_or(|s| s.stop.load(Ordering::Relaxed)))
    }

    pub fn complete(mut self, result: Result<ExternalFrameResult, String>, timings: FrameTimings) -> bool {
        let Some(c) = self.completion.take() else { return false };
        let Some(shared) = c.shared.upgrade() else { return false };
        shared.in_flight.lock().unwrap_or_else(|e| e.into_inner()).retain(|(k, flag, _)| !(*k == self.task.key && Arc::ptr_eq(flag, &c.cancel)));
        let cancelled = c.cancel.load(Ordering::Relaxed) || shared.stop.load(Ordering::Relaxed);
        let valid = [timings.source_ms, timings.render_ms, timings.job_ms].iter().all(|v| v.is_finite() && (0.0..=600_000.0).contains(v));
        // Source production and asynchronous decoder waits matter just as much
        // as effect time. Learn even from a late result, or every initial slow
        // frame can be cancelled before scheduling ever discovers its cost.
        if valid && (self.task.prefetch || self.task.playback) {
            let mut cost = shared.render_cost.lock().unwrap_or_else(|e| e.into_inner());
            let next = timings.job_ms / 1000.0;
            *cost = if *cost == 0.0 { next } else { *cost * 0.8 + next * 0.2 };
        }
        let mut stats = shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.jobs = stats.jobs.saturating_add(1);
        stats.cancelled = stats.cancelled.saturating_add(cancelled as u64);
        if cancelled {
            return false;
        }
        let result = match result {
            Ok(result) if valid => result,
            _ => {
                stats.failed = stats.failed.saturating_add(1);
                return false;
            }
        };
        if let ExternalFrameResult::Image(image) = &result
            && (image.w == 0
                || image.h == 0
                || image.w > 16384
                || image.h > 16384
                || image.w.checked_mul(image.h).and_then(|n| n.checked_mul(4)).filter(|n| *n <= 256 << 20) != Some(image.px.len()))
        {
            stats.failed = stats.failed.saturating_add(1);
            return false;
        }
        stats.source_ms += timings.source_ms;
        stats.render_ms += timings.render_ms;
        stats.job_ms += timings.job_ms;
        stats.prefetch = stats.prefetch.saturating_add(self.task.prefetch as u64);
        if stats.recent.len() >= FrameStats::RECENT {
            stats.recent.pop_front();
        }
        stats.recent.push_back((timings.source_ms as f32, timings.render_ms as f32));
        drop(stats);
        match result {
            ExternalFrameResult::Plan(plan) => shared.plans.lock().unwrap_or_else(|e| e.into_inner()).insert(self.task.key, plan),
            ExternalFrameResult::Image(img) if matches!(self.task.key.target, Target::SequencePlan(_)) => {
                let frame = Arc::new(filmcraft_frame::VideoFrame::rgba8(img.w as u32, img.h as u32, img.px));
                let plan = filmcraft_render::plan::FramePlan::Layers {
                    width: img.w,
                    height: img.h,
                    layers: vec![filmcraft_render::plan::PlanLayer::new(frame, filmcraft_geom::Affine::IDENTITY, 1.0, filmcraft_render::Blend::Normal)],
                };
                shared.plans.lock().unwrap_or_else(|e| e.into_inner()).insert(self.task.key, GpuPlan { plan, prepared: Default::default() });
            }
            ExternalFrameResult::Image(img) => shared.done.lock().unwrap_or_else(|e| e.into_inner()).insert(self.task.key, Arc::new(img)),
        }
        true
    }
}

impl Drop for ExternalFrameJob {
    fn drop(&mut self) {
        if let Some(c) = self.completion.take() {
            c.cancel.store(true, Ordering::Relaxed);
            if let Some(shared) = c.shared.upgrade() {
                shared.in_flight.lock().unwrap_or_else(|e| e.into_inner()).retain(|(k, flag, _)| !(*k == self.task.key && Arc::ptr_eq(flag, &c.cancel)));
                let mut stats = shared.stats.lock().unwrap_or_else(|e| e.into_inner());
                stats.cancelled = stats.cancelled.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_program_and_source_playback_keep_useful_work_without_forward_catchup() {
        for target in [Target::SequencePlan(filmcraft_project::ItemId(3)), Target::Item(filmcraft_project::ItemId(12))] {
            let session = filmcraft_engine::Session::default();
            let frames = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
            frames.set_external(true);
            let key = FrameKey { target, frame: 10, size: 500, revision: 1, draft: true };
            frames.schedule_playback(key, filmcraft_time::FrameRate::FPS_24, 0.5, &session.project, -1.0, false);
            let job = frames.take_external_job().unwrap();
            assert!(job.task.playback);
            assert_eq!(job.task.catch_up, None, "reverse playback must not skip frames using forward decoder catch-up");
            frames.schedule_playback(FrameKey { frame: 9, ..key }, filmcraft_time::FrameRate::FPS_24, 0.5, &session.project, -1.0, false);
            assert!(!job.cancelled());
            assert!(job.complete(
                Ok(ExternalFrameResult::Image(Rgba { w: 1, h: 1, px: vec![0; 4] })),
                FrameTimings { source_ms: 95.0, render_ms: 5.0, job_ms: 100.0 }
            ));
            assert!((frames.render_cost() - 0.1).abs() < 1e-6);
        }
    }

    #[test]
    fn paused_scrubs_cancel_obsolete_work_and_do_not_change_playback_cost_estimates() {
        let session = filmcraft_engine::Session::default();
        let frames = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
        frames.set_external(true);
        let key = FrameKey { target: Target::SequencePlan(filmcraft_project::ItemId(3)), frame: 10, size: 500, revision: 1, draft: false };
        frames.request(key, Tick::ZERO, 0.5, &session.project, 0);
        let job = frames.take_external_job().unwrap();
        assert!(!job.task.playback);
        assert!(job.task.catch_up.is_some(), "scrub decoding has its own keep margin, independent of playback mode");
        frames.request(FrameKey { frame: 11, ..key }, Tick::ZERO, 0.5, &session.project, 0);
        assert!(job.cancelled(), "an old paused seek must not block the new playhead request");
        assert!(!job.complete(Err("cancelled".into()), FrameTimings { source_ms: 95.0, render_ms: 5.0, job_ms: 100.0 }));
        assert_eq!(frames.render_cost(), 0.0);
    }

    #[test]
    fn playback_clock_does_not_starve_external_frames_and_learns_source_cost() {
        let session = filmcraft_engine::Session::default();
        let frames = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
        frames.set_external(true);
        let key = FrameKey { target: Target::SequencePlan(filmcraft_project::ItemId(3)), frame: 0, size: 500, revision: 1, draft: true };
        frames.schedule_playback(key, filmcraft_time::FrameRate::FPS_24, 0.5, &session.project, 1.0, false);
        let job = frames.take_external_job().unwrap();
        assert_eq!(job.task.key.frame, 0);
        frames.schedule_playback(FrameKey { frame: 1, ..key }, filmcraft_time::FrameRate::FPS_24, 0.5, &session.project, 1.0, false);
        assert!(!job.cancelled(), "clock advanced one frame while the worker was still producing a useful picture");
        assert!(
            job.complete(Ok(ExternalFrameResult::Image(Rgba { w: 1, h: 1, px: vec![0; 4] })), FrameTimings { source_ms: 95.0, render_ms: 5.0, job_ms: 100.0 })
        );
        assert!((frames.render_cost() - 0.1).abs() < 1e-6, "slow source work must determine lookahead, not just effect time");
        frames.schedule_playback(FrameKey { frame: 2, ..key }, filmcraft_time::FrameRate::FPS_24, 0.5, &session.project, 1.0, false);
        let job = frames.take_external_job().unwrap();
        assert!(job.task.key.frame >= 5, "do not start an obsolete queued frame that cannot meet its deadline");
        frames.retain_queue(|_| false);
        assert!(!job.complete(Err("cancelled".into()), FrameTimings { source_ms: 180.0, render_ms: 20.0, job_ms: 200.0 }));
        assert!(frames.render_cost() > 0.1, "late work must still teach the scheduler its cost");
        assert!(frames.get_plan(&FrameKey { frame: 5, ..key }).is_none(), "cancelled work cannot publish");
    }

    #[test]
    fn external_jobs_prioritize_foreground_and_validate_pixels_before_caching() {
        let session = filmcraft_engine::Session::default();
        let frames = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
        frames.set_external(true);
        let thumbnail = FrameKey { target: Target::Item(filmcraft_project::ItemId(12)), frame: 0, size: 64, revision: 1, draft: false };
        let monitor = FrameKey { target: Target::SequencePlan(filmcraft_project::ItemId(3)), frame: 0, size: 500, revision: 1, draft: false };
        frames.request(thumbnail, Tick::ZERO, 0.5, &session.project, 50);
        frames.request(monitor, Tick::ZERO, 0.5, &session.project, 0);
        let job = frames.take_external_job().unwrap();
        assert_eq!(job.task.key, monitor);
        assert!(job.complete(Ok(ExternalFrameResult::Image(Rgba { w: 1, h: 1, px: vec![10, 20, 30, 255] })), FrameTimings::default()));
        assert!(frames.get_plan(&monitor).is_some());
        let job = frames.take_external_job().unwrap();
        assert_eq!(job.task.key, thumbnail);
        assert!(!job.complete(Ok(ExternalFrameResult::Image(Rgba { w: usize::MAX, h: 2, px: vec![] })), FrameTimings::default()));
        assert!(frames.get(&thumbnail).is_none());
        assert_eq!(frames.stats().failed, 1);
        assert!(frames.take_external_job().is_none());
        let other = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
        assert_ne!(frames.document_epoch(), other.document_epoch());
    }

    #[test]
    fn late_external_completion_cannot_publish_into_a_retired_or_cancelled_document() {
        let mut session = filmcraft_engine::Session::default();
        session.execute("file.openDemoProject", serde_json::json!({})).unwrap();
        let frames = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
        frames.set_external(true);
        let key = FrameKey { target: Target::Item(filmcraft_project::ItemId(999)), frame: 0, size: 64, revision: 1, draft: false };
        frames.request(key, Tick::ZERO, 0.1, &session.project, 0);
        let job = frames.take_external_job().unwrap();
        frames.retain_queue(|_| false);
        assert!(!job.complete(Ok(ExternalFrameResult::Image(Rgba { w: 1, h: 1, px: vec![0; 4] })), FrameTimings::default()));
        assert!(frames.get(&key).is_none());
        frames.request(key, Tick::ZERO, 0.1, &session.project, 0);
        let job = frames.take_external_job().unwrap();
        drop(frames);
        assert!(job.cancelled());
        assert!(!job.complete(Ok(ExternalFrameResult::Image(Rgba { w: 1, h: 1, px: vec![0; 4] })), FrameTimings::default()));
    }
}
