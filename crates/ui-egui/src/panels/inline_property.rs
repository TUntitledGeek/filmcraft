//! Typed inline property rows. Values and keyframes always come from the project snapshot;
//! all mutations go through the engine registry, resolving a unique effect immediately.
use super::super::timeline::Layout;
use crate::{
    FilmcraftApp,
    icons::{self, Icon},
    theme::Tokens,
};
use egui::{Align2, Rect, Sense, Stroke, pos2, vec2};
use filmcraft_project::{ClipId, EffectInstance, Interpolation, Keyframe, Param, ParamKind, ParamValue, TrackItem};
use filmcraft_time::Tick;
use serde_json::{Value, json};

pub(super) fn execute(app: &mut FilmcraftApp, clip: ClipId, effect: u64, command: &str, mut args: Value) {
    let exists =
        app.session.active_sequence().and_then(|q| q.find_item(clip)).and_then(|(_, it)| EffectInstance::instance_index(&it.effects, effect)).is_some();
    if !exists {
        app.ui.status = "The inline effect instance no longer exists".into();
        return;
    }
    if !app.session.state.selection.contains(&clip)
        && let Err(error) = app.session.execute("timeline.select", json!({"clips":[clip.0],"linked":false}))
    {
        app.ui.status = error.to_string();
        return;
    }
    if command != "effects.select"
        && let Err(error) = app.session.execute("effects.select", json!({"clip":clip.0,"effect":{"instanceId":effect}}))
    {
        app.ui.status = error.to_string();
        return;
    }
    args["clip"] = json!(clip.0);
    args["effect"] = json!({"instanceId":effect});
    if let Err(error) = app.session.execute(command, args) {
        app.ui.status = error.to_string();
    }
}

fn editable(it: &TrackItem) -> bool {
    it.speed == 1.0 && !it.reverse && it.frame_hold.is_none() && it.duration > Tick::ZERO
}
fn at(app: &FilmcraftApp, it: &TrackItem) -> Tick {
    it.effect_time_at(app.session.playhead().clamp(it.start, (it.end() - Tick(1)).max(it.start)))
}
fn timeline_time(it: &TrackItem, source: Tick) -> Option<Tick> {
    editable(it).then(|| it.start + source - it.source_in).filter(|t| *t >= it.start && *t < it.end())
}

fn number(v: f64, decimals: usize) -> String {
    if v.is_finite() { format!("{v:.decimals$}") } else { "Auto".into() }
}
fn display(value: &ParamValue, kind: Option<&ParamKind>) -> String {
    match value {
        ParamValue::Float(v) => {
            let (d, u) = match kind {
                Some(ParamKind::Float { decimals, unit, .. }) => (*decimals as usize, *unit),
                Some(ParamKind::Angle) => (1, "°"),
                _ => (2, ""),
            };
            format!("{}{}", number(*v, d), u)
        }
        ParamValue::Vec2(v) => format!("{}, {}", number(v.x, 1), number(v.y, 1)),
        ParamValue::Bool(v) => if *v { "On" } else { "Off" }.into(),
        ParamValue::Choice(v) => match kind {
            Some(ParamKind::Choice(choices)) => choices.get(*v as usize).copied().unwrap_or("Unknown").into(),
            _ => v.to_string(),
        },
        ParamValue::Color(c) => format!("RGBA {:.2}, {:.2}, {:.2}, {:.2}", c[0], c[1], c[2], c[3]),
        ParamValue::Text(s) => {
            if s.is_empty() {
                "None".into()
            } else {
                s.clone()
            }
        }
        ParamValue::Curve(points) => format!("Curve · {} points", points.len()),
        ParamValue::Path(_) => "Mask path".into(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn row(app: &mut FilmcraftApp, ui: &mut egui::Ui, layout: &Layout, row: Rect, clip: ClipId, it: &TrackItem, e: &EffectInstance, param_id: &str) {
    let Some(param) = e.param(param_id) else { return };
    let def = e.def().and_then(|d| d.param(param_id));
    let label = def.map_or(param_id, |d| d.label);
    let value = param.value_at(at(app, it));
    let t = app.tokens;
    let column = layout.content.min.x - row.min.x;
    let value_x = row.min.x + (column * 0.54).max(86.0);
    let text = display(&value, def.map(|d| &d.kind));
    super::super::effect_controls::row_label(ui, pos2(row.min.x + 43.0, row.center().y), label, value_x - row.min.x - 48.0, Tokens::ui(10.5), t.text_dim);
    let id = egui::Id::new(("inline-value", clip.0, e.instance_id, param_id));
    let auto = format!("timeline.animation.clip.{}.effect.{}.param.{}", clip.0, e.instance_id, param_id);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(pos2(value_x, row.min.y + 1.0), pos2(layout.content.min.x - 6.0, row.max.y - 1.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(child.max_rect().intersect(ui.clip_rect()));
    child.spacing_mut().item_spacing.x = 2.0;
    child.spacing_mut().interact_size.y = 18.0;
    child.spacing_mut().button_padding = vec2(2.0, 0.0);
    let mut changed = None;
    let mut begin = false;
    child
        .add_enabled_ui(editable(it) && e.effect != "time_remap", |ui| match (def.map(|d| &d.kind), &value) {
            (Some(ParamKind::Float { min, max, soft_min, soft_max, unit, decimals }), ParamValue::Float(v)) if v.is_finite() => {
                let mut val = *v;
                let response = ui
                    .push_id(id, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut val)
                                .range(*min..=*max)
                                .speed(((soft_max - soft_min) / 400.0).max(0.01))
                                .max_decimals(*decimals as usize)
                                .suffix(*unit),
                        )
                    })
                    .inner;
                app.auto.add_response(&format!("{auto}.value"), &response, label);
                begin = response.drag_started() || response.gained_focus();
                if response.changed() {
                    changed = Some(json!(val));
                }
            }
            (Some(ParamKind::Angle), ParamValue::Float(v)) if v.is_finite() => {
                let mut val = *v;
                let response =
                    ui.push_id(id, |ui| ui.add(egui::DragValue::new(&mut val).range(-36000.0..=36000.0).speed(0.5).max_decimals(1).suffix("°"))).inner;
                app.auto.add_response(&format!("{auto}.value"), &response, label);
                begin = response.drag_started() || response.gained_focus();
                if response.changed() {
                    changed = Some(json!(val));
                }
            }
            (Some(ParamKind::Point | ParamKind::Wheel), ParamValue::Vec2(v)) if v.x.is_finite() && v.y.is_finite() => {
                let (mut x, mut y) = (v.x, v.y);
                let range = if matches!(def.map(|d| &d.kind), Some(ParamKind::Wheel)) { -1.0..=1.0 } else { -100000.0..=100000.0 };
                let a = ui
                    .push_id(id.with("x"), |ui| ui.add(egui::DragValue::new(&mut x).range(range.clone()).speed(1.0).max_decimals(1)))
                    .inner
                    .on_hover_text("X");
                let b = ui.push_id(id.with("y"), |ui| ui.add(egui::DragValue::new(&mut y).range(range).speed(1.0).max_decimals(1))).inner.on_hover_text("Y");
                app.auto.add_response(&format!("{auto}.x"), &a, "X");
                app.auto.add_response(&format!("{auto}.y"), &b, "Y");
                begin = a.drag_started() || b.drag_started() || a.gained_focus() || b.gained_focus();
                if a.changed() || b.changed() {
                    changed = Some(json!([x, y]));
                }
            }
            (Some(ParamKind::Bool), ParamValue::Bool(v)) => {
                let mut val = *v;
                let response = ui.push_id(id, |ui| ui.checkbox(&mut val, "")).inner;
                app.auto.add_response(&format!("{auto}.value"), &response, label);
                if response.changed() {
                    begin = true;
                    changed = Some(json!(val));
                }
            }
            (Some(ParamKind::Choice(options)), ParamValue::Choice(v)) => {
                let mut val = *v;
                let response = egui::ComboBox::from_id_salt(id)
                    .width(ui.available_width())
                    .selected_text(options.get(val as usize).copied().unwrap_or("Unknown"))
                    .show_ui(ui, |ui| {
                        for (n, option) in options.iter().enumerate() {
                            if ui.selectable_value(&mut val, n as u32, *option).changed() {
                                changed = Some(json!(val));
                                begin = true;
                            }
                        }
                    })
                    .response;
                app.auto.add_response(&format!("{auto}.value"), &response, label);
            }
            (Some(ParamKind::Color), ParamValue::Color(c)) => {
                let mut rgba = egui::Rgba::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
                let response = egui::color_picker::color_edit_button_rgba(ui, &mut rgba, egui::color_picker::Alpha::Opaque);
                app.auto.add_response(&format!("{auto}.value"), &response, label);
                if response.changed() {
                    changed = Some(json!([rgba.r(), rgba.g(), rgba.b(), rgba.a()]));
                }
            }
            _ => {
                ui.label(egui::RichText::new(&text).size(10.5).color(t.text));
            }
        })
        .response
        .on_hover_text(if editable(it) && e.effect != "time_remap" {
            format!("{label}: {text}")
        } else {
            "Inline editing for reverse, speed changes and frame holds is pending; use Effect Controls. This value is evaluated from the canonical project."
                .into()
        });
    if let Some(value) = changed {
        execute(app, clip, e.instance_id, "effects.setParam", json!({"param":param_id,"value":value,"merge":true,"begin":begin}));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn keys(
    app: &mut FilmcraftApp,
    ui: &mut egui::Ui,
    layout: &Layout,
    row: Rect,
    clip: ClipId,
    it: &TrackItem,
    e: &EffectInstance,
    param_id: &str,
    keys: &mut super::super::effect_controls::Interaction,
) {
    let Some(param) = e.param(param_id) else { return };
    let t = app.tokens;
    let mt = at(app, it);
    let auto = format!("timeline.animation.clip.{}.effect.{}.param.{}", clip.0, e.instance_id, param_id);
    let supported = editable(it) && e.effect != "time_remap";
    let nav = [
        (Icon::Stopwatch, "animation", "Toggle property animation"),
        (Icon::StepBack, "previous", "Previous keyframe"),
        (Icon::Keyframe, "addKey", "Add/remove keyframe"),
        (Icon::StepFwd, "next", "Next keyframe"),
    ];
    for (n, (icon, suffix, tip)) in nav.iter().enumerate() {
        let rect = Rect::from_min_size(pos2(row.min.x + 30.0 + n as f32 * 24.0, row.min.y + 2.0), vec2(20.0, 18.0));
        let target = match *suffix {
            "previous" => param.prev_keyframe(mt),
            "next" => param.next_keyframe(mt),
            _ => None,
        };
        let enabled = supported && (!matches!(*suffix, "previous" | "next") || target.and_then(|k| timeline_time(it, k)).is_some());
        let response = ui.interact(rect, egui::Id::new((&auto, suffix)), if enabled { Sense::click() } else { Sense::hover() }).on_hover_text(*tip);
        if response.hovered() && enabled {
            ui.painter().rect_filled(rect, 2.0, t.hover);
        }
        let active = match *suffix {
            "animation" => param.is_animated(),
            "addKey" => param.keyframes.iter().any(|k| k.time == mt),
            _ => false,
        };
        icons::paint(
            ui.painter(),
            rect.shrink(3.0),
            *icon,
            if !enabled {
                t.text_faint
            } else if active {
                t.accent
            } else {
                t.text_dim
            },
        );
        if enabled {
            app.auto.add_response(&format!("{auto}.{suffix}"), &response, tip);
        }
        if response.clicked() && enabled {
            match *suffix {
                "animation" => execute(app, clip, e.instance_id, "effects.toggleAnimation", json!({"param":param_id})),
                "addKey" => execute(app, clip, e.instance_id, "effects.addKeyframe", json!({"param":param_id})),
                _ => {
                    if let Some(time) = target.and_then(|k| timeline_time(it, k)) {
                        app.stop();
                        if let Err(error) = app.session.execute("playhead.set", json!({"time":time.0})) {
                            app.ui.status = error.to_string();
                        }
                    }
                }
            }
        }
    }
    ui.painter().text(pos2(row.min.x + 132.0, row.center().y), Align2::LEFT_CENTER, format!("{} keys", param.keyframes.len()), Tokens::ui(10.0), t.text_faint);
    let lane = Rect::from_min_max(pos2(layout.content.min.x, row.min.y), row.max).intersect(ui.clip_rect());
    let p = ui.painter().with_clip_rect(lane);
    if !supported {
        p.text(
            pos2(lane.min.x + 8.0, row.center().y),
            Align2::LEFT_CENTER,
            "Timeline key mapping unavailable for this retime/hold",
            Tokens::ui(10.0),
            t.text_dim,
        );
        return;
    }
    keys.background(app, ui, lane);
    let span = Rect::from_min_max(pos2(layout.x_of(it.start), row.min.y), pos2(layout.x_of(it.end()), row.max.y));
    for key in &param.keyframes {
        let Some(time) = timeline_time(it, key.time) else { continue };
        let rect = Rect::from_center_size(pos2(layout.x_of(time), row.center().y), vec2(13.0, 16.0));
        if !rect.intersects(lane) {
            continue;
        }
        let rate = app.session.sequence_rate();
        let drop_frame = app.session.active_sequence().is_some_and(|q| q.settings.drop_frame);
        let label = filmcraft_time::format_time(time, rate, drop_frame, filmcraft_time::TimeDisplay::Timecode, 48000);
        let response =
            ui.interact(rect, egui::Id::new((&auto, key.time.0)), Sense::click_and_drag()).on_hover_text(format!("{label} · {}", key.interp.label()));
        let reference = crate::state::KeyframeRef {
            clip: clip.0,
            effect: EffectInstance::instance_index(&it.effects, e.instance_id).unwrap_or(usize::MAX),
            effect_id: Some(e.instance_id),
            param: param_id.into(),
            mask: None,
            time: key.time,
        };
        let rect = keys.key(app, ui, reference.clone(), it, span, rect, &response, key.interp);
        let col = if app.ui.keyframe_selection.contains(&reference) {
            t.hot_text
        } else if key.time == mt {
            t.accent
        } else {
            t.text_dim
        };
        super::super::effect_controls::paint_keyframe(&p, rect.shrink2(vec2(1.0, 1.0)), key.interp, col);
        app.auto.add_response(&format!("{auto}.key.{}", key.time.0), &response, "Keyframe");
        if response.clicked() {
            execute(app, clip, e.instance_id, "effects.select", json!({}));
            app.stop();
            if let Err(error) = app.session.execute("playhead.set", json!({"time":time.0})) {
                app.ui.status = error.to_string();
            }
        }
        response.context_menu(|ui| key_menu(app, ui, clip, it, e, param_id, param, key, &auto));
    }
    let x = layout.x_of(app.session.playhead());
    p.line_segment([pos2(x, row.min.y), pos2(x, row.max.y)], Stroke::new(1.0, t.playhead));
}

#[allow(clippy::too_many_arguments)]
fn key_menu(
    app: &mut FilmcraftApp,
    ui: &mut egui::Ui,
    clip: ClipId,
    it: &TrackItem,
    e: &EffectInstance,
    param: &str,
    value: &Param,
    key: &Keyframe,
    auto: &str,
) {
    let prefix = format!("{auto}.key.{}.menu", key.time.0);
    ui.label(egui::RichText::new("Temporal interpolation").color(app.tokens.text_dim));
    for (kind, wire) in [
        (Interpolation::Linear, "linear"),
        (Interpolation::Bezier, "bezier"),
        (Interpolation::AutoBezier, "autoBezier"),
        (Interpolation::ContinuousBezier, "continuousBezier"),
        (Interpolation::Hold, "hold"),
        (Interpolation::EaseIn, "easeIn"),
        (Interpolation::EaseOut, "easeOut"),
    ] {
        let response = ui.selectable_label(key.interp == kind, kind.label());
        let label = if key.interp == kind { format!("{} (current)", kind.label()) } else { kind.label().into() };
        app.auto.add_response(&format!("{prefix}.{wire}"), &response, &label);
        if response.clicked() {
            execute(app, clip, e.instance_id, "effects.setInterpolation", json!({"param":param,"mediaTime":key.time.0,"interpolation":wire}));
            ui.close();
        }
    }
    ui.separator();
    let ph = app.session.playhead();
    let target = it.effect_time_at(ph);
    let inside = ph >= it.start && ph < it.end();
    let occupied = value.keyframes.iter().any(|k| k.time == target && k.time != key.time);
    let response = ui.add_enabled(inside && !occupied && target != key.time, egui::Button::new("Move to playhead")).on_hover_text(if !inside {
        "Place the playhead inside this clip"
    } else if occupied {
        "Another keyframe already occupies the playhead"
    } else {
        "Move this key without changing its value or interpolation"
    });
    app.auto.add_response(&format!("{prefix}.moveToPlayhead"), &response, "Move to playhead");
    if response.clicked() {
        execute(app, clip, e.instance_id, "effects.moveKeyframe", json!({"param":param,"mediaTime":key.time.0,"to":target.0}));
        ui.close();
    }
    let response = ui.button("Delete keyframe");
    app.auto.add_response(&format!("{prefix}.delete"), &response, "Delete keyframe");
    if response.clicked() {
        execute(app, clip, e.instance_id, "effects.deleteKeyframe", json!({"param":param,"mediaTime":key.time.0}));
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_values_are_readable_and_use_the_schema() {
        assert_eq!(display(&ParamValue::Bool(true), Some(&ParamKind::Bool)), "On");
        assert_eq!(display(&ParamValue::Choice(1), Some(&ParamKind::Choice(&["First", "Second"]))), "Second");
        assert_eq!(display(&ParamValue::Vec2(filmcraft_geom::Vec2::new(960.0, 540.0)), Some(&ParamKind::Point)), "960.0, 540.0");
        assert_eq!(number(f64::NAN, 1), "Auto");
    }
}
