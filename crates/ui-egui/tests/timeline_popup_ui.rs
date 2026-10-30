//! Popup ownership must outlive the canvas hit zone currently below the pointer.
use std::sync::Arc;

use egui::{PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::{FilmcraftApp, menus};
use serde_json::json;

type App = Harness<'static, FilmcraftApp>;
fn step(h: &mut App) {
    for _ in 0..3 {
        h.ctx.request_repaint();
        h.step();
    }
}
fn fixture(expanded: bool) -> App {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = FilmcraftApp::new(s);
    app.set_workspace("Clean Editing");
    let mut h = Harness::builder().with_size(vec2(1656.0, 894.0)).build_eframe(move |_| app);
    step(&mut h);
    if expanded {
        let clip = h.state().session.active_sequence().unwrap().video_tracks[0].items[0].id;
        let ctx = h.ctx.clone();
        menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", json!({"clip":clip.0})).unwrap();
        step(&mut h);
    }
    h
}
fn click(h: &mut App, pos: Pos2, button: PointerButton) {
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: true, modifiers: Default::default() });
    step(h);
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: false, modifiers: Default::default() });
    step(h);
}
fn widget(h: &App, id: &str) -> Rect {
    let [x, y, w, height] = h.state().auto.find(id).unwrap_or_else(|| panic!("Missing {id}")).rect;
    Rect::from_min_size(pos2(x, y), vec2(w, height))
}
fn first_clip(h: &App) -> String {
    format!("timeline.clip.{}", h.state().session.active_sequence().unwrap().video_tracks[0].items[0].id.0)
}
fn move_pointer(h: &mut App, pos: Pos2) {
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    step(h);
}
fn assert_clip_menu(h: &App) {
    assert!(h.state().auto.find("timeline.clipMenu.clip.speedDuration").is_some(), "The clip menu vanished");
    assert!(h.state().tl.drag.is_none(), "The canvas started a drag behind the menu");
}

#[test]
fn clip_menu_survives_drawer_disclosure_and_trim_edge_hover_and_dispatches_settings() {
    let mut h = fixture(true);
    let body = widget(&h, &first_clip(&h));
    let layout = h.state().tl.layout.as_ref().unwrap().clone();
    let drawer = layout.animations[0].visible;
    let chevron = layout.disclosures.iter().find(|(_, r)| r.min.x > body.max.x).unwrap().1;
    let before = h.state().session.project.clone();
    click(&mut h, body.center(), PointerButton::Secondary);
    assert_clip_menu(&h);
    let menu = widget(&h, "timeline.clipMenu.clip.speedDuration");
    // The menu may cover the canvas point, or the pointer may leave the menu without clicking.
    // Neither changes the menu's owner or dismisses it.
    for point in [pos2(menu.center().x, drawer.center().y), chevron.center(), pos2(body.max.x - 2.0, body.center().y)] {
        move_pointer(&mut h, point);
        assert_clip_menu(&h);
        assert_ne!(h.ctx.output(|o| o.cursor_icon), egui::CursorIcon::ResizeColumn);
        assert!(Arc::ptr_eq(&before, &h.state().session.project));
    }
    let settings = widget(&h, "timeline.clipMenu.clip.speedDuration").center();
    click(&mut h, settings, PointerButton::Primary);
    assert_eq!(h.state().ui.clip_dialog.as_ref().map(|d| d.command.as_str()), Some("clip.speedDuration"));
    assert!(Arc::ptr_eq(&before, &h.state().session.project));
}

#[test]
fn collapsed_clip_menu_keeps_pointer_ownership_over_disclosure_and_clip_edges() {
    let mut h = fixture(false);
    let body = widget(&h, &first_clip(&h));
    let arrow = h.state().tl.layout.as_ref().unwrap().disclosures.iter().find(|(_, r)| r.min.x > body.max.x).unwrap().1;
    click(&mut h, body.center(), PointerButton::Secondary);
    for point in [arrow.center(), pos2(body.max.x - 2.0, body.center().y)] {
        move_pointer(&mut h, point);
        assert_clip_menu(&h);
    }
}

#[test]
fn popup_still_dismisses_and_edit_point_settings_keep_their_original_target() {
    let mut h = fixture(true);
    let body = widget(&h, &first_clip(&h));
    let before = h.state().session.project.clone();
    click(&mut h, body.center(), PointerButton::Secondary);
    for pressed in [true, false] {
        h.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed, repeat: false, modifiers: Default::default() });
        step(&mut h);
    }
    assert!(h.state().auto.find("timeline.clipMenu.clip.speedDuration").is_none());
    click(&mut h, body.center(), PointerButton::Secondary);
    click(&mut h, pos2(2.0, 2.0), PointerButton::Primary);
    assert!(h.state().auto.find("timeline.clipMenu.clip.speedDuration").is_none());
    let clip = h.state().session.active_sequence().unwrap().video_tracks[0].items[1].id;
    let second = widget(&h, &format!("timeline.clip.{}", clip.0));
    click(&mut h, pos2(second.max.x - 2.0, second.center().y), PointerButton::Secondary);
    let trim = widget(&h, "timeline.editPointMenu.trimOut");
    let drawer = h.state().tl.layout.as_ref().unwrap().animations[0].visible;
    move_pointer(&mut h, pos2(trim.center().x, drawer.center().y));
    assert!(h.state().auto.find("timeline.editPointMenu.trimOut").is_some());
    let target = widget(&h, "timeline.editPointMenu.trimOut").center();
    click(&mut h, target, PointerButton::Primary);
    let selected = h.state().session.state.edit_points.first().unwrap();
    assert_eq!(selected.clip, clip);
    assert!(selected.out);
    assert_eq!(serde_json::to_value(selected.kind).unwrap(), json!("trim"));
    assert!(Arc::ptr_eq(&before, &h.state().session.project));
}
