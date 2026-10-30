//! Progressive inline animation rows owned by clips and measured with the track layout.
//! Only transient disclosure is stored. A unique effect type is resolved afresh for each action;
//! duplicate instances stay closed until the model provides persistent instance identities.
use std::collections::{BTreeMap, BTreeSet};

use egui::{Align2, Rect, Sense, Stroke, pos2, vec2};
use filmcraft_project::{ClipId, ItemId, Sequence, TrackId, TrackItem};
use serde_json::{Value, json};

use super::timeline::Layout;
use crate::{
    FilmcraftApp,
    icons::{self, Icon},
    theme::Tokens,
};
#[path = "inline_property.rs"]
mod property;

pub const ROW_H: f32 = 22.0;

#[derive(Default)]
pub struct State {
    scope: Option<(ItemId, String)>,
    pub clips: BTreeMap<u64, Disclosure>,
    pub anchor: Option<Anchor>,
    pub active: Option<ClipId>,
    saved_scroll: Option<(f32, f32)>,
}

#[derive(Default)]
pub struct Disclosure {
    effects: BTreeSet<String>,
    sections: BTreeSet<(String, String)>,
    properties: BTreeSet<(String, String)>,
}

#[derive(Clone, Copy)]
pub struct Anchor {
    pub clip: ClipId,
    pub align_top: bool,
}

#[derive(Clone, Debug)]
pub enum Entry {
    Clip,
    // These offsets address the immutable frame snapshot, never disclosure identity.
    Effect { slot: usize, unique: bool },
    Section { slot: usize, name: String },
    Property { slot: usize, param: String },
    Keys { slot: usize, param: String },
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub clip: ClipId,
    pub track: TrackId,
    pub rows: Vec<Entry>,
}
impl Plan {
    pub fn height(&self) -> f32 {
        self.rows.len() as f32 * ROW_H
    }
}

impl State {
    pub fn inspect(&self) -> Value {
        json!({"clips":self.clips.iter().map(|(id,v)|json!({"clip":id,"effects":v.effects,"sections":v.sections,"properties":v.properties})).collect::<Vec<_>>()})
    }
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }
    pub fn contains(&self, clip: ClipId) -> bool {
        self.clips.contains_key(&clip.0)
    }
    fn scope(&mut self, sequence: ItemId, workspace: &str) {
        if self.scope.as_ref().is_none_or(|(q, w)| *q != sequence || w != workspace) {
            *self = Self { scope: Some((sequence, workspace.to_string())), ..Self::default() };
        }
    }
}

pub fn toggle(app: &mut FilmcraftApp, params: &Value) -> Result<Value, String> {
    if !params.is_object() {
        return Err("Animation view parameters must be an object".into());
    }
    if params.get("clip").is_some_and(|v| v.as_u64().is_none()) {
        return Err("Animation clip must be a clip ID".into());
    }
    for field in ["effect", "param", "section"] {
        if params.get(field).is_some_and(|v| v.as_str().is_none()) {
            return Err(format!("Animation {field} must be a string"));
        }
    }
    let sequence = app.session.state.active_sequence.ok_or("Open a sequence first")?;
    let clip = params
        .get("clip")
        .and_then(Value::as_u64)
        .map(ClipId)
        .or_else(|| app.session.state.selection.first().copied())
        .ok_or("Select a clip to show animation")?;
    let seq = app.session.active_sequence().ok_or("Open a sequence first")?;
    let (_, it) = seq.find_item(clip).ok_or("The animation clip no longer exists")?;
    let effect = params.get("effect").and_then(Value::as_str);
    if let Some(effect) = effect {
        let mut matching = it.effects.iter().filter(|e| e.effect == effect);
        let e = matching.next().ok_or("The animation effect no longer exists")?;
        if matching.next().is_some() {
            return Err("Duplicate effect instances require stable model IDs before independent inline disclosure is available".into());
        }
        if let Some(param) = params.get("param").and_then(Value::as_str)
            && e.param(param).is_none()
        {
            return Err("The animation property no longer exists".into());
        }
    }
    app.tl.animation.scope(sequence, &app.ui.workspace);
    let state = &mut app.tl.animation;
    let opening = !state.contains(clip);
    if opening && state.is_empty() {
        state.saved_scroll = Some((app.ui.timeline.v_scroll, app.ui.timeline.a_scroll));
    }
    if let Some(effect) = effect {
        let view = state.clips.entry(clip.0).or_default();
        if let Some(param) = params.get("param").and_then(Value::as_str) {
            view.effects.insert(effect.into());
            toggle_set(&mut view.properties, (effect.to_string(), param.to_string()));
        } else if let Some(section) = params.get("section").and_then(Value::as_str) {
            view.effects.insert(effect.into());
            toggle_set(&mut view.sections, (effect.to_string(), section.to_string()));
        } else {
            toggle_set(&mut view.effects, effect.to_string());
        }
    } else if state.clips.remove(&clip.0).is_none() {
        state.clips.insert(clip.0, Disclosure::default());
    }
    state.anchor = Some(Anchor { clip, align_top: opening });
    state.active = Some(clip);
    if !state.contains(clip) {
        state.active = state.clips.keys().next_back().copied().map(ClipId);
    }
    if state.is_empty() {
        if let Some((v, a)) = state.saved_scroll.take() {
            app.ui.timeline.v_scroll = v;
            app.ui.timeline.a_scroll = a;
        }
        state.anchor = None;
        state.active = None;
    }
    Ok(json!({"clip":clip.0,"expanded":state.contains(clip),"clips":state.clips.len()}))
}

fn toggle_set<T: Ord>(set: &mut BTreeSet<T>, key: T) {
    if !set.remove(&key) {
        set.insert(key);
    }
}

pub fn prepare(app: &mut FilmcraftApp, sequence: ItemId, seq: &Sequence) -> Vec<Plan> {
    app.tl.animation.scope(sequence, &app.ui.workspace);
    app.tl.animation.clips.retain(|id, _| seq.find_item(ClipId(*id)).is_some());
    let mut plans = Vec::new();
    if app.tl.animation.is_empty() {
        return plans;
    }
    for track in seq.all_tracks() {
        for it in &track.items {
            let Some(view) = app.tl.animation.clips.get_mut(&it.id.0) else { continue };
            let unique = |id: &str| it.effects.iter().filter(|e| e.effect == id).count() == 1;
            view.effects.retain(|id| unique(id));
            view.sections.retain(|(id, _)| unique(id));
            view.properties.retain(|(id, param)| unique(id) && it.effect(id).is_some_and(|e| e.params.contains_key(param)));
            let mut rows = vec![Entry::Clip];
            // Match Effect Controls: fixed clip attributes precede the ordered added-effects
            // stack. This is presentation only; the project and processing order never change.
            let mut order: Vec<_> = it.effects.iter().enumerate().collect();
            order.sort_by_key(|(_, e)| !e.def().is_some_and(|d| d.intrinsic));
            for (slot, e) in order {
                let single = unique(&e.effect);
                rows.push(Entry::Effect { slot, unique: single });
                if !single || !view.effects.contains(&e.effect) {
                    continue;
                }
                if let Some(def) = e.def() {
                    let mut sections = BTreeSet::new();
                    for pd in &def.params {
                        if let Some(group) = pd.group {
                            if sections.insert(group) {
                                rows.push(Entry::Section { slot, name: group.into() });
                            }
                            if !view.sections.contains(&(e.effect.clone(), group.into())) {
                                continue;
                            }
                        }
                        if !e.params.contains_key(pd.id) {
                            continue;
                        }
                        rows.push(Entry::Property { slot, param: pd.id.into() });
                        if pd.animatable && view.properties.contains(&(e.effect.clone(), pd.id.into())) {
                            rows.push(Entry::Keys { slot, param: pd.id.into() });
                        }
                    }
                } else {
                    for param in e.params.keys() {
                        rows.push(Entry::Property { slot, param: param.clone() });
                    }
                }
            }
            plans.push(Plan { clip: it.id, track: track.id, rows });
        }
    }
    plans
}

fn command(app: &mut FilmcraftApp, ui: &egui::Ui, params: Value) {
    if let Err(error) = crate::menus::invoke(app, ui.ctx(), "timeline.animation.toggle", params) {
        app.ui.status = error;
    }
}

pub fn chevron_rect(body: Rect) -> Option<Rect> {
    (body.width() >= 36.0 && body.height() >= 16.0).then(|| Rect::from_min_size(body.min + vec2(8.0, 1.0), vec2(14.0, 15.0)))
}

pub fn chevron(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect, clip: ClipId) {
    let response = ui.interact(rect, egui::Id::new(("clip-animation-disclosure", clip.0)), Sense::click()).on_hover_text("Show/hide clip animation");
    if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, app.tokens.hover);
    }
    icons::paint(ui.painter(), rect.shrink(2.0), if app.tl.animation.contains(clip) { Icon::ChevronDown } else { Icon::ChevronRight }, app.tokens.text);
    crate::widgets::paint_focus(ui.painter(), &response, &app.tokens);
    app.auto.add_response(&format!("timeline.animation.clip.{}.toggle", clip.0), &response, "Show/hide clip animation");
    if response.clicked() {
        command(app, ui, json!({"clip":clip.0}));
    }
}

#[allow(clippy::too_many_arguments)]
fn fold(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect, open: bool, enabled: bool, id: &str, params: Value, label: &str) {
    let response = ui.interact(rect, egui::Id::new(id), if enabled { Sense::click() } else { Sense::hover() });
    if response.hovered() {
        ui.painter().rect_filled(rect, 1.0, app.tokens.hover);
    }
    icons::paint(
        ui.painter(),
        rect.shrink(3.0),
        if open { Icon::ChevronDown } else { Icon::ChevronRight },
        if enabled { app.tokens.text_dim } else { app.tokens.text_faint },
    );
    if enabled {
        app.auto.add_response(id, &response, label);
    }
    if response.clicked() && enabled {
        command(app, ui, params);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, layout: &Layout, plan: &Plan, area: Rect, visible: Rect, it: &TrackItem) {
    let t = app.tokens;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area).id_salt(("clip-animation", plan.clip.0)));
    child.set_clip_rect(visible.intersect(ui.clip_rect()));
    child.spacing_mut().item_spacing.y = 0.0;
    let p = child.painter().clone();
    p.rect_filled(area, 0.0, t.tl_track_bg);
    app.auto.add(&format!("timeline.animation.clip.{}.drawer", plan.clip.0), visible, &it.name);
    for (n, entry) in plan.rows.iter().enumerate() {
        let row = Rect::from_min_size(area.min + vec2(0.0, n as f32 * ROW_H), vec2(area.width(), ROW_H));
        if row.max.y <= visible.min.y || row.min.y >= visible.max.y {
            continue;
        }
        let label_right = layout.content.min.x;
        let span = Rect::from_min_max(pos2(layout.x_of(it.start), row.min.y), pos2(layout.x_of(it.end()), row.max.y));
        p.with_clip_rect(Rect::from_min_max(pos2(label_right, visible.min.y), visible.max)).rect_filled(
            span.shrink2(vec2(0.0, 1.0)),
            0.0,
            t.accent.gamma_multiply(0.055),
        );
        match entry {
            Entry::Clip => {
                p.rect_filled(row, 0.0, t.tl_header_bg);
                super::effect_controls::row_label(
                    &child,
                    pos2(row.min.x + 8.0, row.center().y),
                    &it.name,
                    label_right - row.min.x - 34.0,
                    Tokens::semibold(10.5),
                    t.text_dim,
                );
                let r = Rect::from_min_size(pos2(label_right - 26.0, row.min.y + 2.0), vec2(20.0, 18.0));
                let response = child.interact(r, egui::Id::new(("close-clip-animation", plan.clip.0)), Sense::click()).on_hover_text("Collapse clip animation");
                icons::paint(&p, r.shrink(4.0), Icon::Close, t.text_dim);
                app.auto.add_response(&format!("timeline.animation.clip.{}.close", plan.clip.0), &response, "Collapse clip animation");
                if response.clicked() {
                    command(app, &child, json!({"clip":plan.clip.0}));
                }
                let text = format!("{} effect groups · clip animation", it.effects.len());
                p.text(pos2(label_right + 8.0, row.center().y), Align2::LEFT_CENTER, text, Tokens::ui(10.0), t.text_faint);
            }
            Entry::Effect { slot, unique } => {
                let Some(e) = it.effects.get(*slot) else { continue };
                let name = e.def().map_or(e.effect.as_str(), |d| d.name);
                let open = app.tl.animation.clips.get(&plan.clip.0).is_some_and(|v| v.effects.contains(&e.effect));
                let id = format!("timeline.animation.clip.{}.effect.{}.toggle", plan.clip.0, e.effect);
                fold(
                    app,
                    &mut child,
                    Rect::from_min_size(row.min + vec2(8.0, 2.0), vec2(18.0, 18.0)),
                    open,
                    *unique,
                    &id,
                    json!({"clip":plan.clip.0,"effect":e.effect}),
                    name,
                );
                super::effect_controls::row_label(
                    &child,
                    pos2(row.min.x + 32.0, row.center().y),
                    name,
                    label_right - row.min.x - 56.0,
                    Tokens::semibold(11.0),
                    if e.enabled { t.text } else { t.text_dim },
                );
                let count = e.params.values().filter(|p| p.is_animated()).count();
                let summary = if !unique {
                    "Duplicate instances: independent disclosure awaits stable IDs".into()
                } else if count > 0 {
                    format!("{count} animated properties")
                } else {
                    "".into()
                };
                p.with_clip_rect(Rect::from_min_max(pos2(label_right, row.min.y), row.max)).text(
                    pos2(label_right + 8.0, row.center().y),
                    Align2::LEFT_CENTER,
                    summary,
                    Tokens::ui(10.0),
                    t.text_faint,
                );
                if *unique {
                    let r = Rect::from_min_size(pos2(label_right - 24.0, row.min.y + 2.0), vec2(20.0, 18.0));
                    let response =
                        child.interact(r, egui::Id::new(("inline-bypass", plan.clip.0, &e.effect)), Sense::click()).on_hover_text("Enable/bypass effect");
                    p.text(r.center(), Align2::CENTER_CENTER, "fx", Tokens::semibold(10.0), if e.enabled { t.text_dim } else { t.text_faint });
                    app.auto.add_response(&format!("timeline.animation.clip.{}.effect.{}.enabled", plan.clip.0, e.effect), &response, "Enable/bypass effect");
                    if response.clicked() {
                        property::execute(app, plan.clip, &e.effect, "effects.toggleEnabled", json!({}));
                    }
                }
            }
            Entry::Section { slot, name } => {
                let Some(e) = it.effects.get(*slot) else { continue };
                let open = app.tl.animation.clips.get(&plan.clip.0).is_some_and(|v| v.sections.contains(&(e.effect.clone(), name.clone())));
                fold(
                    app,
                    &mut child,
                    Rect::from_min_size(row.min + vec2(22.0, 2.0), vec2(18.0, 18.0)),
                    open,
                    true,
                    &format!("timeline.animation.clip.{}.effect.{}.section.{}", plan.clip.0, e.effect, name),
                    json!({"clip":plan.clip.0,"effect":e.effect,"section":name}),
                    name,
                );
                super::effect_controls::row_label(
                    &child,
                    pos2(row.min.x + 43.0, row.center().y),
                    name,
                    label_right - row.min.x - 47.0,
                    Tokens::ui(10.5),
                    t.text_dim,
                );
            }
            Entry::Property { slot, param } => {
                let Some(e) = it.effects.get(*slot) else { continue };
                let open = app.tl.animation.clips.get(&plan.clip.0).is_some_and(|v| v.properties.contains(&(e.effect.clone(), param.clone())));
                let def = e.def().and_then(|d| d.param(param));
                if def.is_some_and(|d| d.animatable) {
                    fold(
                        app,
                        &mut child,
                        Rect::from_min_size(row.min + vec2(24.0, 2.0), vec2(16.0, 18.0)),
                        open,
                        true,
                        &format!("timeline.animation.clip.{}.effect.{}.param.{}.toggle", plan.clip.0, e.effect, param),
                        json!({"clip":plan.clip.0,"effect":e.effect,"param":param}),
                        "Show/hide property keyframes",
                    );
                }
                property::row(app, &mut child, layout, row, plan.clip, it, e, param);
            }
            Entry::Keys { slot, param } => {
                let Some(e) = it.effects.get(*slot) else { continue };
                property::keys(app, &mut child, layout, row, plan.clip, it, e, param);
            }
        }
        p.line_segment([pos2(row.min.x, row.max.y - 0.5), pos2(row.max.x, row.max.y - 0.5)], Stroke::new(0.5, t.separator.gamma_multiply(0.4)));
    }
    p.line_segment([pos2(layout.content.min.x, area.min.y), pos2(layout.content.min.x, area.max.y)], Stroke::new(1.0, t.separator));
}
