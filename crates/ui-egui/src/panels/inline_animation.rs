//! First bounded inline slice: one transient clip drawer, canonical read-only hierarchy and
//! a unique intrinsic Opacity editor. No effect instance identity or second animation store.
use egui::{Rect, Sense, Stroke, pos2, vec2};
use filmcraft_project::{ClipId, ItemId, ParamValue, Sequence, TrackItem};
use filmcraft_time::Tick;
use serde_json::{Value, json};

use super::timeline::Layout;
use crate::{FilmcraftApp, theme::Tokens};

pub const HEIGHT: f32 = 112.0;

pub fn toggle(app: &mut FilmcraftApp) -> Result<Value, String> {
    let sequence = app.session.state.active_sequence.ok_or("Open a sequence first")?;
    let clip = app
        .session
        .state
        .selection
        .iter()
        .copied()
        .find(|c| app.session.active_sequence().is_some_and(|q| q.find_item(*c).is_some()))
        .ok_or("Select a clip to show animation")?;
    if app.tl.animation.as_ref().is_some_and(|(q, c, _)| *q == sequence && *c == clip) {
        app.tl.animation = None;
    } else {
        app.tl.animation = Some((sequence, clip, app.ui.workspace.clone()));
    }
    Ok(json!({"expanded": app.tl.animation.is_some(), "clip": clip.0}))
}

pub fn clip(app: &mut FilmcraftApp, sequence: ItemId, seq: &Sequence) -> Option<ClipId> {
    let valid = app.tl.animation.as_ref().is_none_or(|(q, c, w)| *q == sequence && *w == app.ui.workspace && seq.find_item(*c).is_some());
    if !valid {
        app.tl.animation = None;
    }
    app.tl.animation.as_ref().map(|(_, c, _)| *c)
}

fn linear(it: &TrackItem) -> bool {
    it.speed == 1.0 && !it.reverse && it.frame_hold.is_none() && it.duration > Tick::ZERO
}

fn lane(ui: &egui::Ui, row: Rect, layout: &Layout, it: &TrackItem, param: &filmcraft_project::Param, app: &mut FilmcraftApp, id: &str) {
    let r = Rect::from_min_max(pos2(layout.content.min.x, row.min.y), row.max);
    let p = ui.painter().with_clip_rect(r.intersect(ui.clip_rect()));
    let x0 = layout.x_of(it.start);
    let x1 = layout.x_of(it.end());
    p.rect_filled(Rect::from_min_max(pos2(x0, r.min.y + 2.0), pos2(x1, r.max.y - 2.0)), 0.0, app.tokens.accent.gamma_multiply(0.08));
    if linear(it) {
        for key in &param.keyframes {
            let time = it.start + key.time - it.source_in;
            if time < it.start || time >= it.end() {
                continue;
            }
            let at = pos2(layout.x_of(time), r.center().y);
            crate::icons::paint(&p, Rect::from_center_size(at, vec2(10.0, 10.0)), crate::icons::Icon::Keyframe, app.tokens.hot_text);
            let visible = Rect::from_center_size(at, vec2(12.0, 16.0)).intersect(r).intersect(ui.clip_rect());
            if id == "opacity" && visible.is_positive() {
                app.auto.add(&format!("timeline.animation.{id}.key.{}", key.time.0), visible, "Source-time keyframe (read only)");
            }
        }
    }
    let x = layout.x_of(app.session.playhead());
    p.line_segment([pos2(x, r.min.y), pos2(x, r.max.y)], Stroke::new(1.0, app.tokens.playhead));
}

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, layout: &Layout, rect: Rect, clip: ClipId, it: &TrackItem) {
    let t = app.tokens;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).id_salt(("inline-animation", clip.0)));
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child.spacing_mut().item_spacing.y = 0.0;
    child.painter().rect_filled(rect, 0.0, t.panel_bg);
    app.auto.add("timeline.animation.drawer", rect, &it.name);
    let (head, _) = child.allocate_exact_size(vec2(rect.width(), 24.0), Sense::hover());
    let close = Rect::from_min_max(pos2(head.max.x - 30.0, head.min.y), head.max);
    let response = child.interact(close, egui::Id::new("inline-close"), Sense::click()).on_hover_text("Close clip animation");
    crate::icons::paint(child.painter(), close.shrink(7.0), crate::icons::Icon::Close, t.text_dim);
    app.auto.add_response("timeline.animation.close", &response, "Close clip animation");
    if response.clicked() {
        app.tl.animation = None;
    }
    super::effect_controls::row_label(
        &child,
        pos2(head.min.x + 10.0, head.center().y),
        &format!("Animation · {}", it.name),
        head.width() - 44.0,
        Tokens::semibold(11.0),
        t.text,
    );
    let mt = it.effect_time_at(app.session.playhead().clamp(it.start, (it.end() - Tick(1)).max(it.start)));
    let unique = it.effects.iter().filter(|e| e.effect == "opacity").count() == 1;
    let opacity = it.effect("opacity").and_then(|e| e.param("opacity"));
    let mut actions = Vec::new();
    if let Some(param) = opacity {
        let (row, _) = child.allocate_exact_size(vec2(rect.width(), 24.0), Sense::hover());
        let label = Rect::from_min_max(row.min + vec2(8.0, 0.0), pos2(layout.content.min.x - 4.0, row.max.y));
        let mut controls = child.new_child(egui::UiBuilder::new().max_rect(label).layout(egui::Layout::left_to_right(egui::Align::Center)));
        controls.add_enabled_ui(unique && linear(it), |ui| {
            ui.label("Opacity");
            let mut value = param.f64_at(mt);
            let response = ui.add(egui::DragValue::new(&mut value).range(0.0..=100.0).speed(0.5).suffix("%"));
            app.auto.add_response("timeline.animation.opacity.value", &response, "Opacity");
            if response.changed() {
                actions.push((
                    "effects.setParam",
                    json!({"clip":clip.0,"effect":"opacity","param":"opacity","value":value,"merge":true,"begin":response.drag_started()}),
                ));
            }
            let key = ui.button("◇").on_hover_text("Add/remove an Opacity keyframe at the playhead");
            app.auto.add_response("timeline.animation.opacity.addKey", &key, "Add/remove Opacity keyframe");
            if key.clicked() {
                actions.push(("effects.addKeyframe", json!({"clip":clip.0,"effect":"opacity","param":"opacity"})));
            }
        });
        lane(&child, row, layout, it, param, app, "opacity");
    }
    if !unique || !linear(it) {
        child.label(egui::RichText::new("Opacity editing requires one Opacity effect and a normal-speed, non-held clip.").size(10.0).color(t.text_dim));
    }
    egui::ScrollArea::vertical().id_salt(("inline-properties", clip.0)).auto_shrink([false, false]).show(&mut child, |ui| {
        for e in &it.effects {
            let name = e.def().map_or(e.effect.as_str(), |d| d.name);
            let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
            super::effect_controls::row_label(
                ui,
                pos2(row.min.x + 10.0, row.center().y),
                &format!("{}{}", name, if e.enabled { "" } else { " (bypassed)" }),
                layout.content.min.x - row.min.x - 14.0,
                Tokens::semibold(11.0),
                t.text,
            );
            for (id, param) in &e.params {
                let (row, response) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
                let label = e.def().and_then(|d| d.param(id)).map_or(id.as_str(), |p| p.label);
                let value = match param.value_at(mt) {
                    ParamValue::Float(v) => format!("{v:.2}"),
                    value => format!("{value:?}"),
                };
                response.on_hover_text(format!("{name} / {label}: {value}"));
                let width = layout.content.min.x - row.min.x;
                super::effect_controls::row_label(ui, pos2(row.min.x + 20.0, row.center().y), label, width * 0.56 - 20.0, Tokens::ui(10.5), t.text_dim);
                super::effect_controls::row_label(ui, pos2(row.min.x + width * 0.56, row.center().y), &value, width * 0.44 - 6.0, Tokens::ui(10.5), t.text);
                lane(ui, row, layout, it, param, app, "property");
            }
        }
    });
    for (command, params) in actions {
        if let Err(error) = app.session.execute(command, params) {
            app.ui.status = error.to_string();
        }
    }
    child.painter().line_segment([pos2(layout.content.min.x, rect.min.y), pos2(layout.content.min.x, rect.max.y)], Stroke::new(1.0, t.separator));
}
