use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::{FilmcraftApp, menus};
use serde_json::json;
use std::sync::Arc;

#[test]
fn drawer_reserves_space_excludes_clip_hits_and_collapses_without_project_changes() {
    let mut session = Session::default();
    session.execute("file.openDemoProject", json!({})).unwrap();
    let q = session.active_sequence().unwrap();
    let clip = q.video_tracks[0].items[0].id;
    session.execute("timeline.select", json!({"clips":[clip.0], "linked":false})).unwrap();
    let mut app = FilmcraftApp::new(session);
    app.set_workspace("Clean Editing");
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    for _ in 0..3 {
        h.step();
    }
    let original = h.state().session.project.clone();
    let revision = h.state().session.revision;
    let rows = h.state().tl.layout.as_ref().unwrap().rows.clone();
    let ctx = h.ctx.clone();
    menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", json!({})).unwrap();
    for _ in 0..3 {
        h.step();
    }
    let layout = h.state().tl.layout.as_ref().unwrap();
    let drawer = layout.animation.unwrap();
    assert!(layout.rows.iter().all(|r| r.rect.intersect(drawer).height() <= 0.0));
    assert_eq!(filmcraft_ui_egui::panels::timeline::hit_json(h.state(), egui::pos2(drawer.center().x, drawer.center().y), Default::default())["hit"], "none");
    assert!(Arc::ptr_eq(&original, &h.state().session.project));
    assert_eq!(revision, h.state().session.revision);
    assert!(!h.state().auto.query("timeline.animation.opacity.addKey").is_empty());
    menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", json!({})).unwrap();
    for _ in 0..3 {
        h.step();
    }
    assert!(h.state().tl.layout.as_ref().unwrap().animation.is_none());
    for (before, after) in rows.iter().zip(&h.state().tl.layout.as_ref().unwrap().rows) {
        assert_eq!(before.rect, after.rect);
    }
    assert!(Arc::ptr_eq(&original, &h.state().session.project));
}

#[test]
fn drawer_closes_on_clip_deletion_and_project_replacement() {
    let mut session = Session::default();
    session.execute("file.openDemoProject", json!({})).unwrap();
    let clip = session.active_sequence().unwrap().video_tracks[0].items[0].id;
    session.execute("timeline.select", json!({"clips":[clip.0],"linked":false})).unwrap();
    let mut app = FilmcraftApp::new(session);
    app.set_workspace("Clean Editing");
    let ctx = egui::Context::default();
    menus::invoke(&mut app, &ctx, "timeline.animation.toggle", json!({})).unwrap();
    app.session.execute("edit.clear", json!({})).unwrap();
    let seq = app.session.active_sequence().unwrap().clone();
    let id = app.session.state.active_sequence.unwrap();
    assert!(filmcraft_ui_egui::panels::inline_animation::clip(&mut app, id, &seq).is_none());
    app.tl.animation = Some((id, clip, "Clean Editing".into()));
    app.tl.reset_media_caches();
    assert!(app.tl.animation.is_none());
}
