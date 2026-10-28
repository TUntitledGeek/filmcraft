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

#[test]
fn viewer_toggle_switches_between_single_and_dual_monitors() {
    let app = demo();
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    for _ in 0..4 {
        h.step();
    }
    // Starts in single Program viewer mode
    assert!(!h.state().ui.pin_source);
    assert!(h.state().auto.query("panel.Source").is_empty());
    assert!(!h.state().auto.query("editing.pinSource").is_empty());

    // Toggle to dual Source + Program viewing
    h.state_mut().ui.pin_source = true;
    h.ctx.request_repaint();
    for _ in 0..3 {
        h.step();
    }
    assert!(!h.state().auto.query("panel.Source").is_empty());
    assert!(!h.state().auto.query("panel.Program").is_empty());

    // Toggle back to single Program
    h.state_mut().ui.pin_source = false;
    h.state_mut().session.state.source_item = None;
    h.ctx.request_repaint();
    for _ in 0..3 {
        h.step();
    }
    assert!(h.state().auto.query("panel.Source").is_empty());
    let prog = h.state().auto.query("panel.Program");
    assert!(prog.first().unwrap().rect[2] > 1300.0);
}

fn frames(h: &mut Harness<'_, FilmcraftApp>) {
    h.ctx.request_repaint();
    for _ in 0..4 {
        h.step();
        assert!(h.state().ui_error.is_none(), "{:?}", h.state().ui_error);
    }
}

fn at(h: &Harness<'_, FilmcraftApp>, id: &str) -> egui::Pos2 {
    let r = h.state().auto.elements.iter().find(|e| e.id == id).unwrap_or_else(|| panic!("Missing {id}"));
    egui::pos2(r.rect[0] + r.rect[2] / 2.0, r.rect[1] + r.rect[3] / 2.0)
}

fn button(h: &mut Harness<'_, FilmcraftApp>, id: &str, button: egui::PointerButton) {
    let pos = at(h, id);
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: true, modifiers: Default::default() });
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: false, modifiers: Default::default() });
    frames(h);
}

#[test]
fn actual_source_toggle_closes_loaded_footage_and_preserves_the_resized_split() {
    let app = demo();
    let media = *app.session.project.items.iter().find(|(_, i)| i.as_media().is_some()).unwrap().0;
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    frames(&mut h);
    button(&mut h, "editing.pinSource", egui::PointerButton::Primary);
    assert!(h.state().ui.pin_source);
    assert!(h.state().session.state.source_item.is_none(), "A viewer toggle must not pick arbitrary footage");
    let pos = at(&h, "dock.gutter.aba");
    let before = h.state().ui.dock.clone();
    h.input_mut().events.extend([
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() },
    ]);
    h.step();
    h.input_mut().events.push(egui::Event::PointerMoved(pos + egui::vec2(70.0, 0.0)));
    h.step();
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: pos + egui::vec2(70.0, 0.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    frames(&mut h);
    let resized = h.state().ui.dock.clone();
    assert_ne!(resized, before, "The Source/Program divider must resize");
    h.state_mut().set_workspace("Color");
    h.state_mut().set_workspace("Clean Editing");
    frames(&mut h);
    assert_eq!(h.state().ui.dock, resized, "Unsaved divider changes survive workspace visits");
    button(&mut h, "editing.singleProgram", egui::PointerButton::Primary);
    assert!(h.state().auto.query("panel.Source").is_empty());
    h.state_mut().session.execute("source.open", json!({"item":media.0})).unwrap();
    frames(&mut h);
    assert!(!h.state().auto.query("panel.Source").is_empty());
    assert_eq!(h.state().ui.dock, resized);
    button(&mut h, "editing.pinSource", egui::PointerButton::Primary);
    assert!(h.state().auto.query("panel.Source").is_empty(), "The toggle closes a loaded, unpinned Source too");
    assert_eq!(h.state().ui.dock, resized);
}

#[test]
fn explicit_source_close_does_not_rewrite_a_saved_clean_layout() {
    let mut app = demo();
    let tree = app.ui.dock.clone();
    app.workspaces.saved.push(dock::SavedWorkspace {
        name: "Clean Editing".into(),
        layout: serde_json::to_value(&tree).unwrap(),
        floating: serde_json::Value::Null,
    });
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    frames(&mut h);
    button(&mut h, "editing.singleProgram", egui::PointerButton::Primary);
    assert!(h.state().auto.query("panel.Source").is_empty());
    assert_eq!(h.state().ui.dock, tree);
    assert_eq!(h.state().workspaces.saved[0].layout, serde_json::to_value(&tree).unwrap());
    button(&mut h, "editing.pinSource", egui::PointerButton::Primary);
    assert!(!h.state().auto.query("panel.Source").is_empty());
    assert_eq!(h.state().ui.dock, tree);
}

#[test]
fn horizontal_tools_keep_group_flyouts_and_narrow_overflow_accessible() {
    let app = demo();
    let mut h = Harness::builder().with_size(egui::vec2(1024.0, 894.0)).build_eframe(move |_| app);
    frames(&mut h);
    button(&mut h, "tools.Ripple", egui::PointerButton::Secondary);
    button(&mut h, "tools.select.Rolling", egui::PointerButton::Primary);
    assert_eq!(h.state().ui.tool, filmcraft_ui_egui::state::Tool::Rolling);
    h.state_mut().set_workspace("Fusion");
    frames(&mut h);
    assert!(h.state().auto.query("panel.Tools").is_empty(), "No duplicate vertical toolbar in factory layouts");
    button(&mut h, "tools.more", egui::PointerButton::Primary);
    button(&mut h, "tools.select.Zoom", egui::PointerButton::Primary);
    assert_eq!(h.state().ui.tool, filmcraft_ui_egui::state::Tool::Zoom);
}

#[test]
fn new_sequence_defaults_show_the_first_two_video_and_audio_track_controls() {
    let app = demo();
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    frames(&mut h);
    assert_eq!(h.state().ui.timeline.video_track_h, 40.0);
    assert_eq!(h.state().ui.timeline.audio_track_h, 36.0);
    let timeline = h.state().auto.query("panel.Timeline")[0].rect;
    for id in ["V1", "V2", "A1", "A2"] {
        let p = at(&h, &format!("timeline.track.{id}.target"));
        assert!(p.y > timeline[1] + 30.0 && p.y < timeline[1] + timeline[3] - 40.0, "{id} clipped: {p:?}");
    }
}

#[test]
fn caption_options_create_an_undoable_subtitle_track() {
    let app = demo();
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, 894.0)).build_eframe(move |_| app);
    frames(&mut h);
    let count = h.state().session.active_sequence().unwrap().caption_tracks.len();
    button(&mut h, "timeline.toggle.captions", egui::PointerButton::Primary);
    button(&mut h, "timeline.captions.newTrack", egui::PointerButton::Primary);
    assert_eq!(h.state().session.active_sequence().unwrap().caption_tracks.len(), count + 1);
    h.state_mut().session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(h.state().session.active_sequence().unwrap().caption_tracks.len(), count);
}

#[test]
fn inspector_axes_keep_separate_value_and_keyframe_columns() {
    for width in [1656.0, 1024.0] {
        let mut app = demo();
        let clip = app.session.active_sequence().unwrap().video_tracks.iter().flat_map(|t| &t.items).next().unwrap().id;
        app.session.execute("timeline.select", json!({"clips":[clip.0],"linked":false})).unwrap();
        let mut h = Harness::builder().with_size(egui::vec2(width, 894.0)).build_eframe(move |_| app);
        frames(&mut h);
        let p = h.state().auto.query("panel.Properties")[0].rect;
        let x = h.state().auto.query("properties.motion.position.x")[0].rect;
        let y = h.state().auto.query("properties.motion.position.y")[0].rect;
        assert!(x[0] + x[2] <= y[0], "Axes overlap at {width}");
        assert!(y[0] + y[2] <= p[0] + p[2] - 50.0, "Values overlap keyframe navigation at {width}");
    }
}
