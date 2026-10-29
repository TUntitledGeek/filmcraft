//! Source loading uses the existing project payload and pending timeline gesture.
//! Timeline drops are consumed before its release can commit a move.
use egui::{LayerId, Rect, pos2, vec2};
use filmcraft_project::ItemId;
use filmcraft_time::Tick;
use serde_json::json;

use super::timeline::Drag;
use crate::{
    FilmcraftApp,
    dock::{Floating, Group, PanelKind},
};

#[derive(Clone, Copy)]
struct Target {
    rect: Rect,
    layer: LayerId,
}
#[derive(Clone, Copy)]
struct Request {
    item: ItemId,
    time: Option<Tick>,
}
fn id() -> egui::Id {
    egui::Id::new("source-media-drop-target")
}

pub(crate) fn prepare(ui: &egui::Ui, groups: &[Group], floating: &[Floating]) {
    if let Some(group) = groups.iter().find(|g| g.panels.get(g.active) == Some(&PanelKind::Source)) {
        ui.ctx().data_mut(|d| d.insert_temp(id(), Target { rect: group.content, layer: ui.layer_id() }));
    } else if !floating.iter().any(|g| g.panels.get(g.active) == Some(&PanelKind::Source)) {
        ui.ctx().data_mut(|d| d.remove::<Target>(id()));
    }
}

fn over_target(ui: &egui::Ui) -> bool {
    let Some(pos) = ui.ctx().pointer_hover_pos() else { return false };
    ui.ctx().data(|d| d.get_temp::<Target>(id())).is_some_and(|t| t.rect.contains(pos) && ui.ctx().layer_id_at(pos).is_none_or(|layer| layer == t.layer))
}

fn timeline_request(app: &FilmcraftApp) -> Option<Request> {
    let Drag::Move { clips, grab_tick, start_track, .. } = app.tl.drag.as_ref()? else { return None };
    let seq = app.session.active_sequence()?;
    let item = seq
        .track(*start_track)?
        .items
        .iter()
        .find(|item| clips.contains(&item.id) && *grab_tick >= item.start && *grab_tick < item.end())
        .or_else(|| clips.first().and_then(|id| seq.find_item(*id).map(|(_, item)| item)))?;
    Some(Request { item: item.item, time: Some(item.source_time_at((*grab_tick).clamp(item.start, item.end() - Tick(1)))) })
}

fn supported(app: &FilmcraftApp, request: Request) -> bool {
    filmcraft_engine::clip_ops::source_view(&app.session, request.item)
        .and_then(|view| app.session.project.item(view.media))
        .is_some_and(|item| item.as_media().is_some())
}

fn load(app: &mut FilmcraftApp, request: Option<Request>) {
    let Some(request) = request.filter(|request| supported(app, *request)) else {
        app.ui.status = "Drop a playable video, audio file or subclip into Source".into();
        return;
    };
    app.stop_source();
    let result = app.session.execute("source.open", json!({"item":request.item.0})).and_then(|_| match request.time {
        Some(time) => app.session.execute("source.setPlayhead", json!({"time":time.0})),
        None => Ok(serde_json::Value::Null),
    });
    match result {
        Ok(_) => {
            app.ui.source_closed = false;
            app.ui.focused = PanelKind::Source;
        }
        Err(error) => app.ui.status = error.to_string(),
    }
}

/// True means this release belongs to Source, including a safely rejected unsupported item.
pub(crate) fn take_timeline_release(app: &mut FilmcraftApp, ui: &egui::Ui) -> bool {
    if !matches!(app.tl.drag, Some(Drag::Move { .. })) || !over_target(ui) || super::drag_cancelled(ui) {
        return false;
    }
    let request = timeline_request(app);
    load(app, request);
    super::clear_drag(ui);
    true
}

pub(crate) fn show(app: &mut FilmcraftApp, ui: &egui::Ui, rect: Rect, picture: Rect) {
    ui.ctx().data_mut(|d| d.insert_temp(id(), Target { rect, layer: ui.layer_id() }));
    app.auto.add("source.dropTarget", picture, "Load media in Source");
    if !over_target(ui) || super::drag_cancelled(ui) {
        return;
    }
    let timeline = matches!(app.tl.drag, Some(Drag::Move { .. }));
    // Source-to-timeline payloads keep their existing semantics and are not reloaded here.
    let project = super::dragged_source(ui).is_none().then(|| super::dragged_project_item(ui)).flatten();
    let request = if timeline { timeline_request(app) } else { project.map(|item| Request { item, time: None }) };
    let dragging = timeline || (super::has_drag_payload(ui) && super::dragged_source(ui).is_none());
    if !dragging {
        return;
    }
    let valid = request.is_some_and(|request| supported(app, request));
    let t = app.tokens;
    let p = ui.ctx().layer_painter(LayerId::new(egui::Order::Tooltip, egui::Id::new("source-drop-feedback")));
    let color = if valid { t.accent } else { t.danger };
    p.rect_stroke(picture.shrink(2.0), 0.0, egui::Stroke::new(2.0, color), egui::StrokeKind::Inside);
    let label = Rect::from_center_size(picture.center(), vec2((picture.width() - 16.0).clamp(24.0, 210.0), 30.0));
    p.rect_filled(label, 4.0, egui::Color32::from_black_alpha(220));
    p.text(
        pos2(label.center().x, label.center().y),
        egui::Align2::CENTER_CENTER,
        if valid { "Drop to load in Source" } else { "Playable media only" },
        crate::theme::Tokens::ui(11.0),
        egui::Color32::WHITE,
    );
    if !timeline && ui.input(|i| i.pointer.any_released()) {
        load(app, request);
        super::clear_drag(ui);
    }
}
