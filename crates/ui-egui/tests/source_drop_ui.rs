//! Real pointer drags into Source must load media without committing a timeline edit.
use egui::{Event, PointerButton, Pos2, pos2, vec2};
use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::FilmcraftApp;
use serde_json::json;

fn frames(h: &mut Harness<'_, FilmcraftApp>, count: usize) {
    h.ctx.request_repaint();
    for _ in 0..count {
        h.step();
        assert!(h.state().ui_error.is_none(), "{:?}", h.state().ui_error);
    }
}
fn fixture() -> Harness<'static, FilmcraftApp> {
    let mut session = Session::default();
    session.execute("file.openDemoProject", json!({})).unwrap();
    session.execute("edit.deselectAll", json!({})).unwrap();
    let mut app = FilmcraftApp::new(session);
    app.ui.pin_source = true;
    // Use interactive frame cadence for pointer gestures.
    let mut h = Harness::builder().with_size(vec2(1656.0, 894.0)).with_step_dt(1.0 / 60.0).with_max_steps(1000).build_eframe(move |_| app);
    frames(&mut h, 5);
    h
}
fn center(h: &Harness<'_, FilmcraftApp>, id: &str) -> Pos2 {
    let e = h.state().auto.elements.iter().find(|e| e.id == id).unwrap_or_else(|| panic!("Missing {id}"));
    pos2(e.rect[0] + e.rect[2] / 2.0, e.rect[1] + e.rect[3] / 2.0)
}
fn pointer(h: &mut Harness<'_, FilmcraftApp>, pos: Pos2, pressed: bool) {
    h.event(Event::PointerMoved(pos));
    h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Default::default() });
    h.step();
}
fn drag(h: &mut Harness<'_, FilmcraftApp>, from: Pos2, to: Pos2) {
    pointer(h, from, true);
    h.event(Event::PointerMoved(from + vec2(12.0, 0.0)));
    h.step();
    h.event(Event::PointerMoved(to));
    frames(h, 2);
    pointer(h, to, false);
    frames(h, 4);
}

#[test]
fn project_drags_load_and_replace_source_without_edits() {
    let mut h = fixture();
    let bin = h.state().auto.elements.iter().find(|e| e.id.starts_with("project.bin.") && e.label == "Footage").unwrap().id.clone();
    // Bin navigation/double-click is covered by the browser workflow; isolate media drops here.
    h.state_mut().ui.project_panel.bin = Some(bin.rsplit('.').next().unwrap().parse().unwrap());
    frames(&mut h, 5);
    let media: Vec<_> = h
        .state()
        .auto
        .elements
        .iter()
        .filter_map(|element| element.id.strip_prefix("project.item.")?.parse::<u64>().ok())
        .map(filmcraft_project::ItemId)
        .filter(|id| h.state().session.project.item(*id).is_some_and(|i| i.as_media().is_some_and(|m| m.info.video.is_some())))
        .collect();
    assert!(media.len() >= 2, "The Footage bin must show media cards");
    let first = media[0];
    let second = media[1];
    let before = h.state().session.project.clone();
    let undo = h.state().session.undo_mark();
    let from = center(&h, &format!("project.item.{}", first.0));
    let to = center(&h, "source.dropTarget");
    drag(&mut h, from, to);
    assert_eq!(h.state().session.state.source_item, Some(first));
    let from = center(&h, &format!("project.item.{}", second.0));
    let to = center(&h, "source.dropTarget");
    drag(&mut h, from, to);
    assert_eq!(h.state().session.state.source_item, Some(second));
    assert_eq!(h.state().session.project.as_ref(), before.as_ref());
    assert_eq!(h.state().session.undo_mark(), undo);
}

#[test]
fn timeline_drag_to_source_uses_grabbed_source_time_and_never_moves_the_clip() {
    let mut h = fixture();
    let seq = h.state().session.active_sequence().unwrap();
    let (track, item) = seq.video_tracks.iter().find_map(|t| t.items.first().map(|item| (t.id, item.clone()))).unwrap();
    let layout = h.state().tl.layout.as_ref().unwrap();
    let row = layout.rows.iter().find(|r| r.track == track).unwrap();
    let from = pos2(layout.x_of(item.start + filmcraft_time::Tick(item.duration.0 / 3)), row.rect.center().y);
    let grabbed = layout.tick_at(from.x);
    let expected = h.state().session.project.item(item.item).unwrap().frame_rate().snap(item.source_time_at(grabbed));
    let before = h.state().session.project.clone();
    let undo = h.state().session.undo_mark();
    let to = center(&h, "source.dropTarget");
    drag(&mut h, from, to);
    assert_eq!(h.state().session.state.source_item, Some(item.item));
    assert_eq!(h.state().session.state.source_playhead, expected);
    assert_eq!(h.state().session.project.as_ref(), before.as_ref());
    assert_eq!(h.state().session.undo_mark(), undo);
}

#[test]
fn dropping_a_bin_into_source_is_safely_rejected() {
    let mut h = fixture();
    let bin = h.state().auto.elements.iter().find(|e| e.id.starts_with("project.bin.")).unwrap().id.clone();
    let before = h.state().session.project.clone();
    let from = center(&h, &bin);
    let to = center(&h, "source.dropTarget");
    drag(&mut h, from, to);
    assert!(h.state().session.state.source_item.is_none());
    assert_eq!(h.state().session.project.as_ref(), before.as_ref());
    assert!(h.state().ui.status.contains("playable"));
}
