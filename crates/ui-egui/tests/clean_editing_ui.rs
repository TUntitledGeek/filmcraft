use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::{
    FilmcraftApp,
    dock::{self, PanelKind},
};
use serde_json::json;

fn demo() -> FilmcraftApp {
    let mut session = Session::default();
    session.execute("file.openDemoProject", json!({})).unwrap();
    session.execute("edit.deselectAll", json!({})).unwrap();
    let mut app = FilmcraftApp::new(session);
    app.set_workspace("Clean Editing");
    app
}

#[test]
fn empty_panes_reclaim_program_space_without_mutating_the_dock_tree() {
    let app = demo();
    let tree = app.ui.dock.clone();
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    for _ in 0..4 {
        h.step();
    }
    assert_eq!(h.state().ui.dock, tree);
    assert!(h.state().auto.query("panel.Source").is_empty());
    assert!(h.state().auto.query("panel.Properties").is_empty());
    let program = h.state().auto.query("panel.Program");
    assert!(program.first().unwrap().rect[2] > 1300.0);
    assert!(!h.state().auto.query("editing.pinSource").is_empty());
    h.state_mut().show_panel(PanelKind::Source);
    h.state_mut().show_panel(PanelKind::Properties);
    h.ctx.request_repaint();
    for _ in 0..3 {
        h.step();
    }
    assert!(!h.state().auto.query("panel.Source").is_empty());
    assert!(!h.state().auto.query("panel.Properties").is_empty());
    assert_eq!(h.state().ui.dock, tree);
}

#[test]
fn saved_clean_layout_is_not_contextually_collapsed() {
    let mut app = demo();
    app.workspaces.saved.push(dock::SavedWorkspace {
        name: "Clean Editing".into(),
        layout: serde_json::to_value(&app.ui.dock).unwrap(),
        floating: serde_json::Value::Null,
    });
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    for _ in 0..4 {
        h.step();
    }
    assert!(!h.state().auto.query("panel.Source").is_empty());
    assert!(!h.state().auto.query("panel.Properties").is_empty());
}

#[test]
fn all_new_presets_validate_and_classic_retains_the_previous_editing_layout() {
    for name in dock::WORKSPACES {
        dock::Layout { root: dock::workspace(name), floating: vec![] }.validate().unwrap();
    }
    assert_eq!(dock::workspace("Classic Editing"), dock::workspace("Editing"));
    assert!(dock::workspace("Fusion").is_visible(PanelKind::NodeGraph));
    assert!(!dock::workspace("Fusion").contains(PanelKind::Properties));
}
