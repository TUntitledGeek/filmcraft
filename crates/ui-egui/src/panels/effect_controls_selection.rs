//! Keyframe selection gestures. The project changes only on a completed key drag; a marquee,
//! modifier click, or cancelled preview only changes the UI selection.
use egui::{Color32, Id, Pos2, Rect, Response, Sense, Stroke, vec2};
use filmcraft_project::{Interpolation, TrackItem};
use filmcraft_time::{FrameRate, Tick};
use serde_json::json;

use crate::{
    FilmcraftApp,
    icons::{self, Icon},
    state::KeyframeRef,
};

fn state_id() -> Id {
    Id::new("effect-controls-keyframe-selection")
}

#[derive(Clone)]
struct Point {
    key: KeyframeRef,
    rect: Rect,
    time: Tick,
    pixels_per_tick: f64,
    interp: Interpolation,
}

#[derive(Clone)]
struct Group {
    anchor: KeyframeRef,
    keys: Vec<KeyframeRef>,
    press: Pos2,
    time: Tick,
    ticks_per_pixel: f64,
    low: i64,
    high: i64,
    delta: Tick,
    snap: Option<(Tick, f32)>,
    ended: bool,
}

#[derive(Clone)]
enum Gesture {
    Marquee { origin: Pos2, end: Pos2, base: Vec<KeyframeRef> },
    Group(Group),
}

#[derive(Clone, Default)]
struct State {
    clip: u64,
    points: Vec<Point>,
    gesture: Option<Gesture>,
    revision: u64,
    generation: u64,
}

pub(crate) struct Interaction {
    state: State,
    points: Vec<Point>,
    area: Rect,
    storage: Id,
    inline: bool,
    marquee_ended: bool,
    cancelled: bool,
}

fn limits(app: &FilmcraftApp, keys: &[KeyframeRef]) -> Option<(i64, i64)> {
    let q = app.session.active_sequence()?;
    let frame = q.settings.frame_rate.frame_duration();
    let (mut low, mut high) = (i64::MIN, i64::MAX);
    for key in keys {
        let (_, it) = q.find_item(filmcraft_project::ClipId(key.clip))?;
        let speed = it.speed.abs() * if it.reverse { -1.0 } else { 1.0 };
        if !speed.is_finite() || speed == 0.0 || it.duration < frame {
            return None;
        }
        let da = (it.moving_source_time_at(it.start) - key.time).0 as f64 / speed;
        let db = (it.moving_source_time_at(it.end() - frame) - key.time).0 as f64 / speed;
        low = low.max(da.min(db).ceil() as i64);
        high = high.min(da.max(db).floor() as i64);
    }
    (low <= high).then_some((low, high))
}

fn update(group: &mut Group, points: &[Point], pos: Pos2, shift: bool, playhead: Tick, rate: FrameRate) {
    let offset = f64::from(pos.x - group.press.x) * group.ticks_per_pixel;
    if !offset.is_finite() {
        return;
    }
    let requested = rate.snap_nearest(group.time + Tick(offset.round() as i64)) - group.time;
    group.delta = Tick(requested.0.clamp(group.low, group.high));
    group.snap = None;
    if !shift {
        return;
    }
    let mut closest = 8.0;
    for point in points.iter().filter(|p| group.keys.contains(&p.key)) {
        for target in std::iter::once(playhead).chain(points.iter().filter(|p| !group.keys.contains(&p.key)).map(|p| p.time)) {
            let distance = ((target - point.time - requested).0 as f64 * point.pixels_per_tick).abs();
            let delta = (target - point.time).0;
            if distance <= closest && (group.low..=group.high).contains(&delta) {
                closest = distance;
                group.delta = Tick(delta);
                group.snap = Some((target, point.rect.center().x + (delta as f64 * point.pixels_per_tick) as f32));
            }
        }
    }
}

/// Normalize index hints by stable identity; a deleted effect/key is dropped, never rebound.
/// The scope includes document generation because demo/open can reuse numeric IDs.
pub(crate) fn refresh(app: &mut FilmcraftApp, ui: &egui::Ui) {
    let scope = (app.session.project_generation(), app.session.state.active_sequence);
    let id = Id::new("shared-animation-selection-scope");
    if ui.data(|d| d.get_temp::<(u64, Option<filmcraft_project::ItemId>)>(id)).is_some_and(|old| old != scope) {
        app.ui.keyframe_selection.clear();
        app.ui.mask_pen = None;
        app.ui.eyedropper = None;
        app.ui.save_preset = None;
        app.ui.audio_fx_editors.clear();
    }
    ui.data_mut(|d| d.insert_temp(id, scope));
    let Some(q) = app.session.active_sequence() else {
        app.ui.keyframe_selection.clear();
        return;
    };
    let resolve = |clip: u64, index: usize, id: Option<u64>| {
        let effects = &q.find_item(filmcraft_project::ClipId(clip))?.1.effects;
        let index = match id {
            Some(id) => filmcraft_project::EffectInstance::instance_index(effects, id)?,
            None => {
                effects.get(index)?;
                index
            }
        };
        Some((index, effects[index].instance_id))
    };
    app.ui.mask_pen = app.ui.mask_pen.take().and_then(|mut target| {
        let (index, id) = resolve(target.clip, target.effect, target.effect_id)?;
        target.effect = index;
        target.effect_id = Some(id);
        Some(target)
    });
    app.ui.eyedropper = app.ui.eyedropper.take().and_then(|mut target| {
        let (index, id) = resolve(target.clip, target.effect, target.effect_id)?;
        target.effect = index;
        target.effect_id = Some(id);
        Some(target)
    });
    app.ui.save_preset = app.ui.save_preset.take().and_then(|mut target| {
        if target.effect_ids.is_empty() {
            target.effect_ids = target.effects.iter().map(|index| resolve(target.clip, *index, None).map(|(_, id)| id)).collect::<Option<Vec<_>>>()?;
        }
        target.effects = target.effect_ids.iter().map(|id| resolve(target.clip, 0, Some(*id)).map(|(index, _)| index)).collect::<Option<Vec<_>>>()?;
        Some(target)
    });
    app.ui.keyframe_selection.retain_mut(|key| {
        let Some((_, it)) = q.find_item(filmcraft_project::ClipId(key.clip)) else {
            return false;
        };
        let Some(index) = key.effect_index(it) else {
            return false;
        };
        key.effect = index;
        key.effect_id = Some(it.effects[index].instance_id).filter(|id| *id != 0);
        super::keyframe_exists(it, key)
    });
}

impl Interaction {
    fn start(app: &mut FilmcraftApp, ui: &mut egui::Ui, area: Rect, clip: u64, inline: bool) -> Self {
        refresh(app, ui);
        let storage = if inline { Id::new("inline-animation-keyframe-selection") } else { state_id() };
        let mut state = ui.data(|d| d.get_temp::<State>(storage)).unwrap_or_default();
        if state.clip != clip || state.generation != app.session.project_generation() {
            state = State { clip, generation: app.session.project_generation(), ..Default::default() };
        }
        let cancelled = ui.input(|i| i.key_pressed(egui::Key::Escape)) || state.revision != app.session.revision;
        if cancelled {
            state.gesture = None;
        }
        state.revision = app.session.revision;
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            match &mut state.gesture {
                Some(Gesture::Marquee { end, .. }) => *end = pos,
                Some(Gesture::Group(group)) => {
                    update(group, &state.points, pos, ui.input(|i| i.modifiers.shift), app.session.playhead(), app.session.sequence_rate())
                }
                None => {}
            }
        }
        Self { state, points: Vec::new(), area, storage, inline, marquee_ended: false, cancelled }
    }

    pub(crate) fn begin(app: &mut FilmcraftApp, ui: &mut egui::Ui, area: Rect, clip: u64) -> Self {
        let mut interaction = Self::start(app, ui, area, clip, false);
        interaction.background(app, ui, area);
        interaction
    }

    pub(crate) fn inline(app: &mut FilmcraftApp, ui: &mut egui::Ui, area: Rect) -> Self {
        Self::start(app, ui, area, 0, true)
    }

    // Only actual key lanes register a background. Clip bodies, trim edges, property fields
    // and the gaps between drawers keep their normal timeline hit testing.
    pub(crate) fn background(&mut self, app: &mut FilmcraftApp, ui: &egui::Ui, area: Rect) {
        let response = ui.interact(area, self.storage.with((area.min.y.to_bits(), area.max.y.to_bits())), Sense::click_and_drag());
        let additive = ui.input(|i| i.modifiers.shift || i.modifiers.ctrl || i.modifiers.command);
        if response.clicked_by(egui::PointerButton::Primary) && !additive {
            app.ui.keyframe_selection.clear();
        }
        if !self.cancelled
            && response.drag_started_by(egui::PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            let base = if additive { app.ui.keyframe_selection.clone() } else { Vec::new() };
            self.state.gesture = Some(Gesture::Marquee { origin, end: origin, base });
        }
        self.marquee_ended |= response.drag_stopped_by(egui::PointerButton::Primary);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn key(
        &mut self,
        app: &mut FilmcraftApp,
        ui: &egui::Ui,
        key: KeyframeRef,
        it: &TrackItem,
        lane: Rect,
        rect: Rect,
        response: &Response,
        interp: Interpolation,
    ) -> Rect {
        let shift = ui.input(|i| i.modifiers.shift);
        let additive = ui.input(|i| i.modifiers.shift || i.modifiers.ctrl || i.modifiers.command);
        if self.inline && (response.clicked_by(egui::PointerButton::Primary) || response.drag_started_by(egui::PointerButton::Primary)) {
            if !app.session.state.selection.iter().any(|clip| clip.0 == key.clip)
                && let Err(error) = app.session.execute("timeline.select", json!({"clips":[key.clip],"linked":false}))
            {
                app.ui.status = error.to_string();
            }
            if let Err(error) = app.session.execute("effects.select", json!({"clip":key.clip,"effect":key.selector()})) {
                app.ui.status = error.to_string();
            }
        }
        if response.clicked_by(egui::PointerButton::Primary) {
            if additive {
                if let Some(index) = app.ui.keyframe_selection.iter().position(|k| *k == key) {
                    app.ui.keyframe_selection.remove(index);
                } else {
                    app.ui.keyframe_selection.push(key.clone());
                }
            } else {
                app.ui.keyframe_selection = vec![key.clone()];
            }
        }
        let time = super::timeline_time_of(it, key.time);
        let pixels_per_tick = f64::from(lane.width().max(1.0)) / it.duration.0.max(1) as f64;
        if self.area.contains(rect.center()) && ui.clip_rect().contains(rect.center()) {
            self.points.push(Point { key: key.clone(), rect, time, pixels_per_tick, interp });
        }
        if !self.cancelled && response.drag_started_by(egui::PointerButton::Primary) {
            if !app.ui.keyframe_selection.contains(&key) {
                if !additive {
                    app.ui.keyframe_selection.clear();
                }
                app.ui.keyframe_selection.push(key.clone());
            }
            let keys = app.ui.keyframe_selection.clone();
            let supported = !self.inline
                || keys.iter().all(|key| {
                    app.session.active_sequence().and_then(|q| q.find_item(filmcraft_project::ClipId(key.clip))).is_some_and(|(_, it)| {
                        it.speed == 1.0
                            && !it.reverse
                            && it.frame_hold.is_none()
                            && key.mask.is_none()
                            && key.effect_index(it).is_some_and(|i| it.effects[i].effect != "time_remap")
                    })
                });
            if !supported {
                app.ui.status = "Inline key dragging requires forward, unretimed effect properties".into();
            }
            if let Some((low, high)) = supported.then(|| limits(app, &keys)).flatten() {
                let press = ui.input(|i| i.pointer.press_origin()).unwrap_or(rect.center());
                let mut group = Group {
                    anchor: key.clone(),
                    keys,
                    press,
                    time,
                    ticks_per_pixel: 1.0 / pixels_per_tick,
                    low,
                    high,
                    delta: Tick::ZERO,
                    snap: None,
                    ended: false,
                };
                if let Some(pos) = response.interact_pointer_pos() {
                    update(&mut group, &self.state.points, pos, shift, app.session.playhead(), app.session.sequence_rate());
                }
                self.state.gesture = Some(Gesture::Group(group));
            }
        }
        if let Some(Gesture::Group(group)) = &mut self.state.gesture {
            if group.anchor == key && response.drag_stopped_by(egui::PointerButton::Primary) {
                group.ended = true;
            }
            if group.keys.contains(&key) {
                return rect.translate(vec2((group.delta.0 as f64 * pixels_per_tick) as f32, 0.0));
            }
        }
        rect
    }

    pub(crate) fn finish(mut self, app: &mut FilmcraftApp, ui: &egui::Ui) {
        if let Some(Gesture::Marquee { origin, end, base }) = &self.state.gesture {
            let rect = Rect::from_two_pos(*origin, *end).intersect(self.area);
            let mut selected = base.clone();
            for point in &self.points {
                if rect.contains(point.rect.center()) && !selected.contains(&point.key) {
                    selected.push(point.key.clone());
                }
            }
            app.ui.keyframe_selection = selected;
            let painter = ui.painter().with_clip_rect(self.area);
            painter.rect_filled(rect, 0.0, app.tokens.accent.linear_multiply(0.15));
            painter.rect_stroke(rect, 0.0, Stroke::new(1.0, app.tokens.accent), egui::StrokeKind::Inside);
            app.auto.add("effectControls.marquee", rect, "keyframe selection");
            for point in &self.points {
                let color = if app.ui.keyframe_selection.contains(&point.key) { app.tokens.hot_text } else { Color32::from_rgb(0xb0, 0xb0, 0xb0) };
                paint(&painter, point.rect, point.interp, color);
            }
            if self.marquee_ended || ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                self.state.gesture = None;
            }
        }
        if let Some(Gesture::Group(group)) = &self.state.gesture {
            if let Some((time, x)) = group.snap {
                let rect = Rect::from_min_max(egui::pos2(x - 1.0, self.area.min.y), egui::pos2(x + 1.0, self.area.max.y));
                ui.painter().with_clip_rect(self.area).line_segment([rect.center_top(), rect.center_bottom()], Stroke::new(1.0, app.tokens.accent));
                app.auto.add("effectControls.snapTarget", rect, &time.0.to_string());
            }
            if group.ended || ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
                if group.delta != Tick::ZERO {
                    match app.session.execute(
                        "effects.moveKeyframes",
                        json!({"keyframes":group.keys,"delta":group.delta.0,"collision":"reject","strictForward":self.inline}),
                    ) {
                        Ok(result) => {
                            if let Ok(keys) = serde_json::from_value(result["keyframes"].clone()) {
                                app.ui.keyframe_selection = keys;
                            }
                        }
                        Err(error) => app.ui.status = error.to_string(),
                    }
                }
                self.state.gesture = None;
            }
        }
        self.state.points = self.points;
        ui.data_mut(|d| d.insert_temp(self.storage, self.state));
    }
}

pub(crate) fn paint(painter: &egui::Painter, rect: Rect, interp: Interpolation, color: Color32) {
    match interp {
        Interpolation::Hold => {
            painter.rect_filled(Rect::from_center_size(rect.center(), vec2(9.0, 9.0)), 0.0, color);
        }
        Interpolation::Linear => icons::paint(painter, rect, Icon::Keyframe, color),
        _ => {
            painter.circle_filled(rect.center(), 5.5, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyframe_timeline_coordinates_follow_trim_speed_and_reverse() {
        let mut session = filmcraft_engine::Session::default();
        session.execute("file.openDemoProject", json!({})).unwrap();
        let mut it = session.active_sequence().unwrap().video_tracks[0].items[0].clone();
        it.start += Tick::from_seconds_f64(4.0);
        it.source_in += Tick::from_seconds_f64(3.0);
        for speed in [0.5, 1.0, 1.5, 2.0] {
            for reverse in [false, true] {
                it.speed = speed;
                it.reverse = reverse;
                for time in [it.start, it.start + Tick(it.duration.0 / 4), it.end() - session.sequence_rate().frame_duration()] {
                    let media = it.moving_source_time_at(time);
                    assert!((super::super::timeline_time_of(&it, media) - time).abs() <= Tick(2), "speed={speed}, reverse={reverse}");
                }
            }
        }
    }

    #[test]
    fn clip_clamping_does_not_extend_the_screen_space_snap_threshold() {
        let key = KeyframeRef { clip: 1, effect: 0, effect_id: None, param: "scale".into(), mask: None, time: Tick::ZERO };
        let other = KeyframeRef { param: "rotation".into(), time: Tick::from_seconds_f64(0.95), ..key.clone() };
        let points = [
            Point {
                key: key.clone(),
                rect: Rect::from_center_size(Pos2::ZERO, vec2(15.0, 15.0)),
                time: Tick::ZERO,
                pixels_per_tick: 100.0 / filmcraft_time::TICKS_PER_SECOND as f64,
                interp: Interpolation::Linear,
            },
            Point {
                key: other,
                rect: Rect::from_center_size(egui::pos2(95.0, 30.0), vec2(15.0, 15.0)),
                time: Tick::from_seconds_f64(0.95),
                pixels_per_tick: 100.0 / filmcraft_time::TICKS_PER_SECOND as f64,
                interp: Interpolation::Linear,
            },
        ];
        let mut group = Group {
            anchor: key.clone(),
            keys: vec![key],
            press: Pos2::ZERO,
            time: Tick::ZERO,
            ticks_per_pixel: filmcraft_time::TICKS_PER_SECOND as f64 / 100.0,
            low: 0,
            high: Tick::from_seconds_f64(1.0).0,
            delta: Tick::ZERO,
            snap: None,
            ended: false,
        };
        update(&mut group, &points, egui::pos2(120.0, 0.0), true, Tick::from_seconds_f64(3.0), FrameRate::from_f64(100.0));
        assert_eq!(group.delta, Tick::from_seconds_f64(1.0));
        assert!(group.snap.is_none(), "the pointer is 25 px from the target, even though the clamped preview is only 5 px away");
    }
}
