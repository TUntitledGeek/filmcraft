use std::{cell::Cell, rc::Rc, sync::Arc};

use filmcraft_ui_egui::{FilmcraftApp, menus};
use serde_json::json;

#[test]
fn fullscreen_routes_to_the_host_without_editing_or_using_native_viewports() {
    let mut app = FilmcraftApp::new(filmcraft_engine::Session::default());
    let original = app.session.project.clone();
    let revision = app.session.revision;
    let state = Rc::new(Cell::new(false));
    let toggle = state.clone();
    app.hooks.toggle_fullscreen = Some(Box::new(move || {
        toggle.set(!toggle.get());
        Ok(())
    }));
    app.hooks.fullscreen_state = Some(Box::new(move || state.get()));
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        menus::invoke(&mut app, ui.ctx(), "app.fullscreen", json!({})).unwrap();
    });
    assert!(app.hooks.fullscreen_state.as_ref().unwrap()());
    assert!(output.viewport_output.values().all(|v| v.commands.iter().all(|c| !matches!(c, egui::ViewportCommand::Fullscreen(_)))));
    output.textures_delta.clear();
    menus::invoke(&mut app, &ctx, "app.fullscreen", json!({})).unwrap();
    assert!(!app.hooks.fullscreen_state.as_ref().unwrap()());
    assert!(Arc::ptr_eq(&app.session.project, &original));
    assert_eq!(app.session.revision, revision);
}

#[test]
fn denied_host_fullscreen_is_actionable_and_native_fallback_is_preserved() {
    let mut app = FilmcraftApp::new(filmcraft_engine::Session::default());
    let ctx = egui::Context::default();
    app.hooks.toggle_fullscreen = Some(Box::new(|| Err("Fullscreen denied by host".into())));
    assert_eq!(menus::invoke(&mut app, &ctx, "app.fullscreen", json!({})).unwrap_err(), "Fullscreen denied by host");
    assert_eq!(app.ui.status, "Fullscreen denied by host");
    app.hooks.toggle_fullscreen = None;
    let mut output = ctx.run_ui(Default::default(), |ui| {
        menus::invoke(&mut app, ui.ctx(), "app.fullscreen", json!({})).unwrap();
    });
    assert!(output.viewport_output.values().any(|v| v.commands.contains(&egui::ViewportCommand::Fullscreen(true))));
    output.textures_delta.clear();
}
