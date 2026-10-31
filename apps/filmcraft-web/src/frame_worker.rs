//! Isolated frame-production host. The UI sends immutable project snapshots and
//! exact-time tasks; the worker uses the existing frame server and media pipeline.
//! There is one canonical edit/keyframe store, on the UI thread.

use filmcraft_gpu::transport::{Decoder, Encoder, MAX_PACKET_BYTES, Packet};
use filmcraft_render::plan::{FramePlan, PlanLayer, PlanStep};
use filmcraft_ui_egui::frames::{ExternalFrameJob, ExternalFrameResult, FrameServer, FrameTask, FrameTimings, GpuPlan, Rgba};
use js_sys::{Array, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cell::RefCell, collections::BTreeSet, sync::Arc, time::Duration};
use wasm_bindgen::{JsCast, prelude::*};

fn get(obj: &JsValue, key: &str) -> JsValue {
    Reflect::get(obj, &key.into()).unwrap_or(JsValue::UNDEFINED)
}
fn set(obj: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(obj, &key.into(), value).map(|_| ())
}
fn message(kind: &str) -> Result<JsValue, JsValue> {
    let obj = js_sys::Object::new();
    set(&obj, "type", &kind.into())?;
    Ok(obj.into())
}
fn text_error(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}

fn append_packet(result: &JsValue, buffers: &Array, packet: Packet) -> Result<(), JsValue> {
    set(result, "metadata", &packet.metadata.into())?;
    for bytes in packet.buffers {
        buffers.push(&Uint8Array::from(bytes.as_slice()));
    }
    Ok(())
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
struct Capabilities {
    gpu: bool,
    effects: bool,
    max_side: u32,
    proxies: bool,
}

impl Capabilities {
    fn accepts(&self, plan: &FramePlan) -> bool {
        if !self.gpu {
            return false;
        }
        let layers = |ls: &[PlanLayer]| {
            ls.iter().all(|l| {
                l.frame.width <= self.max_side
                    && l.frame.height <= self.max_side
                    && l.fx.as_ref().is_none_or(|fx| self.effects && fx.size.0 <= self.max_side && fx.size.1 <= self.max_side)
            })
        };
        match plan {
            FramePlan::Image(_) => true,
            FramePlan::Layers { width, height, layers: ls } => *width <= self.max_side as usize && *height <= self.max_side as usize && layers(ls),
            FramePlan::Composite { width, height, steps } => {
                *width <= self.max_side as usize
                    && *height <= self.max_side as usize
                    && steps.iter().all(|s| match s {
                        PlanStep::Layers(ls) => layers(ls),
                        PlanStep::Transition { inputs, effect, .. } => effect.effect == "push" && layers(&inputs[0]) && layers(&inputs[1]),
                    })
            }
        }
    }
}

#[derive(Default)]
pub struct Client {
    worker: Option<web_sys::Worker>,
    on_message: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
    on_error: Option<Closure<dyn FnMut(web_sys::ErrorEvent)>>,
    generation: u32,
    next_id: u32,
    ready: bool,
    booted_at: f64,
    failed: bool,
    manifest: u64,
    active: Option<(u32, ExternalFrameJob, f64, Option<f64>)>,
    snapshot: Option<(u64, Arc<filmcraft_project::Project>)>,
    decoder: Decoder,
    completed: u64,
    cancelled: u64,
}

impl Client {
    fn retire(&mut self) {
        if let Some(w) = self.worker.take() {
            w.set_onmessage(None);
            w.set_onerror(None);
            w.terminate();
        }
        self.on_message = None;
        self.on_error = None;
        self.ready = false;
        self.snapshot = None;
        self.decoder = Decoder::default();
        self.active = None;
    }

    fn fail(&mut self, error: String, app: &mut filmcraft_ui_egui::FilmcraftApp) {
        self.retire();
        self.failed = true;
        crate::set_info("frameWorkerError", json!(error));
        crate::set_info("frameWorkers", json!("failed"));
        app.status(format!("Preview worker stopped: {error}. Reload to retry."));
    }

    pub fn receive(&mut self, generation: u32, data: JsValue, app: &mut filmcraft_ui_egui::FilmcraftApp) {
        if generation != self.generation || self.failed {
            return;
        }
        match get(&data, "type").as_string().as_deref() {
            Some("ready") => {
                self.ready = true;
                crate::set_info("frameWorkers", json!("dedicated"));
            }
            Some("error") => self.fail(get(&data, "error").as_string().unwrap_or_else(|| "unknown worker error".into()), app),
            Some("frame") => {
                let Some((id, job, started, _)) = self.active.take() else { return };
                if get(&data, "id").as_f64() != Some(id as f64) {
                    self.active = Some((id, job, started, None));
                    return;
                }
                let cancelled = job.cancelled();
                let result = if cancelled {
                    self.cancelled = self.cancelled.saturating_add(1);
                    Err("cancelled".into())
                } else {
                    self.decode(&data)
                };
                let mut timings = get(&data, "timings").as_string().and_then(|s| serde_json::from_str::<FrameTimings>(&s).ok()).unwrap_or_default();
                timings.job_ms = timings.job_ms.max((js_sys::Date::now() - started).clamp(0.0, 600_000.0));
                let failed = if cancelled { None } else { result.as_ref().err().cloned() };
                if job.complete(result, timings) {
                    self.completed = self.completed.saturating_add(1);
                }
                crate::set_info(
                    "frameWorker",
                    json!({"completed":self.completed,"cancelled":self.cancelled,"transferCacheBytes":self.decoder.bytes(),"roundTripMs":js_sys::Date::now()-started,
                    "media":get(&data,"media").as_string().and_then(|s|serde_json::from_str::<serde_json::Value>(&s).ok()),"transportFallback":get(&data,"transportFallback").as_string()}),
                );
                if let Some(error) = failed {
                    self.fail(error, app);
                }
            }
            _ => {}
        }
    }

    fn decode(&mut self, data: &JsValue) -> Result<ExternalFrameResult, String> {
        let array = Array::from(&get(data, "buffers"));
        if array.length() > 2048 {
            return Err("too many worker pixel planes".into());
        }
        let mut total = 0usize;
        let mut buffers = Vec::new();
        for buffer in array.iter() {
            let bytes = buffer.dyn_into::<Uint8Array>().map_err(|_| "invalid worker pixel plane")?;
            total = total.checked_add(bytes.length() as usize).filter(|n| *n <= MAX_PACKET_BYTES).ok_or("worker pixels exceed transfer budget")?;
            buffers.push(bytes.to_vec());
        }
        if get(data, "kind").as_string().as_deref() == Some("rgba") {
            let w =
                get(data, "width").as_f64().filter(|v| v.is_finite() && *v >= 1.0 && *v <= 16384.0 && v.fract() == 0.0).ok_or("invalid worker width")? as usize;
            let h = get(data, "height").as_f64().filter(|v| v.is_finite() && *v >= 1.0 && *v <= 16384.0 && v.fract() == 0.0).ok_or("invalid worker height")?
                as usize;
            if buffers.len() != 1 || w.checked_mul(h).and_then(|n| n.checked_mul(4)) != buffers.first().map(Vec::len) {
                return Err("invalid worker image length".into());
            }
            Ok(ExternalFrameResult::Image(Rgba { w, h, px: buffers.remove(0) }))
        } else {
            let metadata = get(data, "metadata").as_string().ok_or("missing worker frame metadata")?;
            let (plan, prepared) = self.decoder.decode(Packet { metadata, buffers })?;
            Ok(ExternalFrameResult::Plan(GpuPlan { plan, prepared }))
        }
    }

    pub fn tick(&mut self, app: &mut filmcraft_ui_egui::FilmcraftApp, ctx: &egui::Context) {
        if !app.frames.is_external() || self.failed {
            return;
        }
        if let Err(error) = self.tick_inner(app, ctx) {
            self.fail(text_error(error), app);
        }
    }

    fn tick_inner(&mut self, app: &filmcraft_ui_egui::FilmcraftApp, ctx: &egui::Context) -> Result<(), JsValue> {
        let now = js_sys::Date::now();
        if self.worker.is_some() && !self.ready && now - self.booted_at > 30_000.0 {
            return Err("frame worker initialization timed out".into());
        }
        // FrameKey alone intentionally omits document identity. Never reuse a
        // worker's decoder/plan caches for a replacement document with reused IDs.
        if self.snapshot.as_ref().is_some_and(|(epoch, _)| *epoch != app.frames.document_epoch()) {
            self.retire();
        }
        if let Some((_, job, started, cancelled_at)) = &mut self.active {
            if job.cancelled() {
                let since = *cancelled_at.get_or_insert(now);
                // Do not continuously restart an expensive decoder during playback.
                // Paused edits/scrubs can preempt work that no longer has a consumer.
                if !app.playback.playing && now - since > 250.0 {
                    self.cancelled = self.cancelled.saturating_add(1);
                    self.retire();
                }
            } else if !app.playback.playing
                && now - *started > 1000.0
                && matches!(job.task.key.target, filmcraft_ui_egui::frames::Target::Sequence(_) | filmcraft_ui_egui::frames::Target::SequencePlan(_))
                && !Arc::ptr_eq(&job.project, &app.session.project)
            {
                self.cancelled = self.cancelled.saturating_add(1);
                self.retire();
            } else if now - *started > 120_000.0 {
                return Err("frame worker response timed out".into());
            }
        }
        // Imports/relinks and preview files are immutable handles, synchronized before dispatch.
        let previews = app.session.previews.manifest();
        let mut paths = BTreeSet::new();
        for item in app.session.project.items.values() {
            if let filmcraft_project::ItemKind::Media(media) = &item.kind {
                for reference in std::iter::once(&media.media).chain(media.proxy.as_ref()) {
                    if let filmcraft_project::MediaRef::File { path } = reference {
                        paths.insert(path.clone());
                    }
                }
            }
        }
        if let Some(dir) = &previews.0 {
            for name in &previews.1 {
                paths.insert(dir.join(name).to_string_lossy().into_owned());
            }
        }
        let (signature, files) = crate::fs::worker_manifest(&paths, if self.worker.is_some() { self.manifest } else { 0 })?;
        if self.worker.is_none() || files.is_some() {
            self.retire();
            self.manifest = signature;
            self.generation = self.generation.wrapping_add(1);
            let generation = self.generation;
            let assets = get(&js_sys::global(), "filmcraftWorkerAssets");
            let url = get(&assets, "worker").as_string().ok_or("missing frame worker URL")?;
            let opts = web_sys::WorkerOptions::new();
            opts.set_type(web_sys::WorkerType::Module);
            opts.set_name("FilmCraft frame production");
            let worker = web_sys::Worker::new_with_options(&url, &opts)?;
            let on_message = Closure::new(move |event: web_sys::MessageEvent| {
                let data = event.data();
                crate::post(move |host, ctx| {
                    host.frame_worker.receive(generation, data, &mut host.app);
                    ctx.request_repaint();
                });
            });
            worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            let on_error = Closure::new(move |event: web_sys::ErrorEvent| {
                let error = event.message();
                crate::post(move |host, _| {
                    if host.frame_worker.generation == generation {
                        host.frame_worker.fail(error, &mut host.app);
                    }
                });
            });
            worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            let msg = message("init")?;
            set(&msg, "glue", &get(&assets, "glue"))?;
            set(&msg, "search", &get(&assets, "search"))?;
            set(&msg, "module", &wasm_bindgen::module())?;
            set(&msg, "files", &files.unwrap_or_default())?;
            worker.post_message(&msg)?;
            self.booted_at = now;
            self.worker = Some(worker);
            self.on_message = Some(on_message);
            self.on_error = Some(on_error);
        }
        if self.ready
            && self.active.is_none()
            && let Some(job) = app.frames.take_external_job()
        {
            self.next_id = self.next_id.wrapping_add(1);
            let id = self.next_id;
            let msg = message("frame")?;
            set(&msg, "id", &id.into())?;
            set(&msg, "task", &serde_json::to_string(&job.task).map_err(|e| JsValue::from_str(&e.to_string()))?.into())?;
            if self.snapshot.as_ref().is_none_or(|(document, project)| *document != job.task.document || !Arc::ptr_eq(project, &job.project)) {
                set(&msg, "project", &serde_json::to_string(&*job.project).map_err(|e| JsValue::from_str(&e.to_string()))?.into())?;
                self.snapshot = Some((job.task.document, job.project.clone()));
            }
            set(&msg, "previews", &serde_json::to_string(&previews).map_err(|e| JsValue::from_str(&e.to_string()))?.into())?;
            let mut caps = app.gpu.as_ref().map_or(Capabilities::default(), |gpu| Capabilities {
                gpu: true,
                effects: gpu.compositor.effects_status() == "ready",
                max_side: gpu.max_texture,
                proxies: false,
            });
            caps.proxies = app.session.media.use_proxies();
            set(&msg, "capabilities", &serde_json::to_string(&caps).map_err(|e| JsValue::from_str(&e.to_string()))?.into())?;
            set(&msg, "retained", &Array::from_iter(self.decoder.retained_frames().into_iter().map(JsValue::from)))?;
            if let Some(worker) = &self.worker {
                worker.post_message(&msg)?;
            }
            self.active = Some((id, job, now, None));
        }
        if self.active.is_some() || !self.ready {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.retire();
    }
}

struct WorkerState {
    session: filmcraft_engine::Session,
    frames: FrameServer,
    encoder: Encoder,
    active: Option<(FrameTask, Vec<u32>, Capabilities, FrameTimings)>,
    previews: String,
}
thread_local! { static WORKER: RefCell<Option<WorkerState>> = const { RefCell::new(None) }; }

#[wasm_bindgen]
pub async fn frame_worker_initialize(files: Array, search: String) -> Result<(), JsValue> {
    crate::WORKER_SEARCH.with(|s| *s.borrow_mut() = Some(search));
    crate::fs::install_worker_manifest(files)?;
    crate::webcodecs::probe().await;
    crate::webcodecs::install();
    let session = filmcraft_engine::Session::new(Arc::new(crate::fs::WebServices));
    session.media.set_idle_source_limit(12);
    let frames = FrameServer::new(session.media.clone(), session.services.clone(), session.previews.clone(), 1);
    WORKER.with(|s| *s.borrow_mut() = Some(WorkerState { session, frames, encoder: Encoder::default(), active: None, previews: String::new() }));
    Ok(())
}

#[wasm_bindgen]
pub fn frame_worker_begin(task: String, project: Option<String>, previews: String, retained: Array, capabilities: String) -> Result<(), JsValue> {
    WORKER.with(|state| {
        let mut state = state.borrow_mut();
        let state = state.as_mut().ok_or("frame worker is not initialized")?;
        if state.active.is_some() {
            return Err("frame worker is busy".into());
        }
        let parse = |e: serde_json::Error| JsValue::from_str(&e.to_string());
        let task: FrameTask = serde_json::from_str(&task).map_err(parse)?;
        if let Some(project) = project {
            state.session.project = Arc::new(serde_json::from_str(&project).map_err(parse)?);
        }
        if state.previews != previews {
            let (dir, names): (Option<std::path::PathBuf>, Vec<String>) = serde_json::from_str(&previews).map_err(parse)?;
            state.session.previews.install_manifest(dir, &names).map_err(|e| JsValue::from_str(&e))?;
            state.previews = previews;
        }
        let caps: Capabilities = serde_json::from_str(&capabilities).map_err(parse)?;
        state.session.media.set_use_proxies(caps.proxies);
        if retained.length() > 256 {
            return Err("too many retained worker frames".into());
        }
        let retained = retained
            .iter()
            .map(|v| {
                v.as_f64()
                    .filter(|v| v.is_finite() && *v >= 0.0 && *v <= u32::MAX as f64 && v.fract() == 0.0)
                    .map(|v| v as u32)
                    .ok_or("invalid retained frame id")
            })
            .collect::<Result<Vec<_>, _>>()?;
        state.frames.retain_queue(|_| false);
        let stats = state.frames.stats();
        state.frames.enqueue_task(&task, &state.session.project).map_err(|e| JsValue::from_str(&e))?;
        state.active = Some((task, retained, caps, FrameTimings { source_ms: stats.source_ms, render_ms: stats.render_ms, job_ms: stats.job_ms }));
        Ok(())
    })
}

#[wasm_bindgen]
pub fn frame_worker_poll() -> Result<JsValue, JsValue> {
    WORKER.with(|state| {
        let mut state = state.borrow_mut();
        let state = state.as_mut().ok_or("frame worker is not initialized")?;
        let Some((task, retained, caps, previous)) = &state.active else { return Err("no active frame worker request".into()) };
        state.frames.pump(Duration::from_millis(6));
        if !state.frames.is_ready(&task.key) {
            if state.frames.queue_len() == 0 {
                state.active = None;
                return Err("frame production failed (no picture and no pending media)".into());
            }
            return Ok(JsValue::NULL);
        }
        let result = js_sys::Object::new();
        let post_started = web_time::Instant::now();
        let buffers = Array::new();
        if let Some(plan) = state.frames.get_plan(&task.key) {
            let packet = if caps.accepts(&plan.plan) {
                match state.encoder.encode(&plan.plan, &plan.prepared, retained) {
                    Ok(packet) => Some(packet),
                    Err(error) if filmcraft_gpu::transport::capacity_error(&error) => {
                        set(&result, "transportFallback", &error.into())?;
                        None
                    }
                    Err(error) => return Err(JsValue::from_str(&error)),
                }
            } else {
                None
            };
            if let Some(packet) = packet {
                append_packet(&result, &buffers, packet)?;
            } else {
                // Never send work that would make the UI compositor execute a CPU
                // effect/transition fallback while shaders initialize or on WebGL.
                let image = filmcraft_render::plan::execute_cpu(&plan.plan);
                if caps.gpu {
                    // Match the compositor's CPU fallback: retain working float
                    // values/alpha until the normal GPU display stage.
                    let plan = FramePlan::Image(image);
                    let prepared = filmcraft_gpu::prepare(&plan);
                    let packet = state.encoder.encode(&plan, &prepared, retained).map_err(|e| JsValue::from_str(&e))?;
                    append_packet(&result, &buffers, packet)?;
                } else {
                    set(&result, "kind", &"rgba".into())?;
                    set(&result, "width", &(image.w as u32).into())?;
                    set(&result, "height", &(image.h as u32).into())?;
                    buffers.push(&Uint8Array::from(image.to_rgba8().as_slice()));
                }
            }
        } else if let Some(image) = state.frames.get(&task.key) {
            set(&result, "kind", &"rgba".into())?;
            set(&result, "width", &(image.w as u32).into())?;
            set(&result, "height", &(image.h as u32).into())?;
            buffers.push(&Uint8Array::from(image.px.as_slice()));
        }
        let stats = state.frames.stats();
        let post_ms = post_started.elapsed().as_secs_f64() * 1000.0;
        let timings = FrameTimings {
            source_ms: stats.source_ms - previous.source_ms,
            render_ms: stats.render_ms - previous.render_ms + post_ms,
            job_ms: stats.job_ms - previous.job_ms + post_ms,
        };
        set(&result, "timings", &serde_json::to_string(&timings).map_err(|e| JsValue::from_str(&e.to_string()))?.into())?;
        let mut media_stats = crate::webcodecs::stats();
        media_stats["blobCacheBytes"] = json!(crate::fs::cache_bytes());
        media_stats["frameCaches"] = json!(state.frames.cache_entries());
        media_stats["wasmMemoryBytes"] = json!(crate::wasm_memory_bytes());
        set(&result, "media", &media_stats.to_string().into())?;
        set(&result, "buffers", &buffers)?;
        state.session.media.trim_idle_sources();
        state.active = None;
        Ok(result.into())
    })
}
