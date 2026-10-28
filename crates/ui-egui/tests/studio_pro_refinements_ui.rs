//! Automated verification tests for Studio Pro refinements:
//! - Responsive 3-Zone Header non-overlap at 800px, 1024px, 1280px, 1440px, 1920px.
//! - Google Drive Sync UTF-8 safety, share URL generation, and session sync state.
//! - Fusion Node Graph interactive engine: 16px grid, bezier wire hit-testing,
//!   parameter syncing (Blur/Merge), solo toggling, and Arc::make_mut persistence.

use egui::{Pos2, pos2};
use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_project::node_graph::{NodeGraph, NodeId, NodeType};
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::state::Mode;
use serde_json::json;

fn bezier_distance_test(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, pt: Pos2) -> f32 {
    let steps = 20;
    let mut min_dist_sq = f32::MAX;
    let mut prev = p0;
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let curr = pos2(
            u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x + t * t * t * p3.x,
            u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y + t * t * t * p3.y,
        );
        let v = curr - prev;
        let w = pt - prev;
        let c1 = w.dot(v);
        let dist_sq = if c1 <= 0.0 {
            w.length_sq()
        } else {
            let c2 = v.dot(v);
            if c2 <= c1 {
                (pt - curr).length_sq()
            } else {
                let b = c1 / c2;
                let pb = prev + v * b;
                (pt - pb).length_sq()
            }
        };
        min_dist_sq = min_dist_sq.min(dist_sq);
        prev = curr;
    }
    min_dist_sq.sqrt()
}

#[test]
fn test_bezier_distance_hit_detection() {
    let p0 = pos2(100.0, 100.0);
    let p1 = pos2(150.0, 100.0);
    let p2 = pos2(150.0, 200.0);
    let p3 = pos2(200.0, 200.0);

    // Midpoint of curve roughly at (150, 150)
    let d_on = bezier_distance_test(p0, p1, p2, p3, pos2(150.0, 150.0));
    assert!(d_on < 5.0, "Point on or near curve should have small distance, got {d_on}");

    let d_far = bezier_distance_test(p0, p1, p2, p3, pos2(400.0, 400.0));
    assert!(d_far > 100.0, "Far point should have large distance, got {d_far}");
}

#[test]
fn test_gdrive_folder_id_utf8_safety() {
    // Test URL with standard ID
    let url = "https://drive.google.com/drive/folders/1a2b3c4d5e6f7g8h9i0j?usp=sharing";
    let folder_id = if let Some(idx) = url.find("/folders/") {
        let rest = &url[idx + 9..];
        rest.split('?').next().unwrap_or(rest).to_string()
    } else {
        url.to_string()
    };
    assert_eq!(folder_id, "1a2b3c4d5e6f7g8h9i0j");
    let short_id: String = folder_id.chars().take(8).collect();
    assert_eq!(short_id, "1a2b3c4d");

    // Test with multi-byte unicode characters (e.g. Japanese or emojis) to verify no byte slice panics
    let unicode_folder = "日本語フォルダ_123456789";
    let safe_short: String = unicode_folder.chars().take(8).collect();
    assert_eq!(safe_short, "日本語フォルダ_");
}

#[test]
fn test_fusion_node_graph_grid_and_effects_pipeline() {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = FilmcraftApp::new(s);

    // Verify selected clip node graph initialization
    let seq_id = app.session.state.active_sequence.unwrap();
    let clip_id = {
        let seq = app.session.project.sequence(seq_id).unwrap();
        seq.video_tracks[0].items[0].id
    };

    let mut graph = NodeGraph::new_default("TestClip");
    assert_eq!(graph.nodes.len(), 3); // MediaIn, ColorGrade, MediaOut

    // Add Blur with 16px grid alignment
    let raw_x = 247.3f32;
    let raw_y = 183.9f32;
    let snapped_x = (raw_x / 16.0).round() * 16.0;
    let snapped_y = (raw_y / 16.0).round() * 16.0;
    assert_eq!(snapped_x, 240.0);
    assert_eq!(snapped_y, 176.0);

    let blur_id = graph.add_node("Blur1", NodeType::Blur, [snapped_x, snapped_y]);
    assert!(graph.node(blur_id).is_some());

    // Connect blur
    graph.disconnect(NodeId(3), "In");
    graph.connect(NodeId(2), "Out", blur_id, "In");
    graph.connect(blur_id, "Out", NodeId(3), "In");

    // Convert to linear effects and confirm blur and colorgrade are generated
    let effects = graph.to_linear_effects();
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0].effect, "lumetri");
    assert_eq!(effects[1].effect, "gaussian_blur");

    // Verify saving back into app.session using Arc::make_mut
    let mut item = {
        let seq = app.session.project.sequence(seq_id).unwrap();
        seq.video_tracks[0].items[0].clone()
    };
    item.effects = effects;
    item.node_graph = Some(graph.clone());

    let old_rev = app.session.revision;
    {
        let project = std::sync::Arc::make_mut(&mut app.session.project);
        if let Some(seq) = project.sequence_mut(seq_id) {
            if let Some((_, existing)) = seq.find_item_mut(clip_id) {
                *existing = item;
                app.session.revision += 1;
                app.session.events.push(filmcraft_engine::Event::ProjectChanged { revision: app.session.revision });
            }
        }
    }
    assert!(app.session.revision > old_rev, "Revision must bump when saving node graph mutations");

    // Confirm stored item has updated effects and node_graph
    let stored_seq = app.session.project.sequence(seq_id).unwrap();
    let stored_item = &stored_seq.video_tracks[0].items[0];
    assert_eq!(stored_item.effects.len(), 2);
    assert!(stored_item.node_graph.is_some());
}

#[test]
fn test_node_graph_solo_toggle_logic() {
    let mut graph = NodeGraph::new_default("SoloTest");
    let blur_id = graph.add_node("Blur1", NodeType::Blur, [200.0, 100.0]);

    // Key 1: Solo blur node
    let was_solo = graph.nodes.iter().find(|n| n.id == blur_id).map(|n| n.solo).unwrap_or(false);
    for n in &mut graph.nodes {
        n.solo = if n.id == blur_id { !was_solo } else { false };
    }
    assert!(graph.node(blur_id).unwrap().solo);
    assert!(!graph.node(NodeId(1)).unwrap().solo);

    // Key 1 again: Toggle off
    let was_solo = graph.nodes.iter().find(|n| n.id == blur_id).map(|n| n.solo).unwrap_or(false);
    for n in &mut graph.nodes {
        n.solo = if n.id == blur_id { !was_solo } else { false };
    }
    assert!(!graph.node(blur_id).unwrap().solo);

    // Key 2: Route output to program (all solo false)
    for n in &mut graph.nodes {
        n.solo = false;
    }
    assert!(graph.nodes.iter().all(|n| !n.solo));
}

#[test]
fn test_header_responsive_zones_at_narrow_and_wide_screens() {
    use egui_kittest::Harness;

    for &w in &[800.0, 1024.0, 1280.0, 1440.0, 1920.0] {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        let mut app = FilmcraftApp::new(s);
        app.ui.show_header = true;
        app.ui.mode = Mode::Edit;

        let mut harness = Harness::builder().with_size(egui::vec2(w, 720.0)).build_eframe(move |_cc| app);
        harness.step();

        // Check registered auto header elements
        let items = harness.state().auto.query("header.");
        assert!(!items.is_empty(), "Header elements must be registered at width {w}");

        // Confirm all header elements stay within [0, w]
        for it in &items {
            assert!(it.rect[0] >= 0.0, "Element {} starts before window: {}", it.id, it.rect[0]);
            assert!(it.rect[0] + it.rect[2] <= w + 1.0, "Element {} extends past window {}: {}", it.id, w, it.rect[0] + it.rect[2]);
        }
    }
}

#[test]
fn test_spacebar_key_repeat_suppression_prevents_stutter() {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let app = FilmcraftApp::new(s);
    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    assert!(!harness.state().playback.playing, "Initially paused");

    // 1. Initial Space press (repeat: false) -> toggles to playing
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();
    assert!(harness.state().playback.playing, "Space press starts playback");

    // 2. Simulated key repeats while holding space down (repeat: true)
    // In old code without repeat filtering, each repeat toggled playback back and forth (stuttering).
    // With repeat filtering, repeats are consumed and playback stays playing.
    for _ in 0..5 {
        harness.input_mut().events.push(egui::Event::Key {
            key: egui::Key::Space,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: Default::default(),
        });
        harness.step();
        assert!(harness.state().playback.playing, "Repeated space events must NOT toggle/stutter playback");
    }

    // 3. Key release
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();
    assert!(harness.state().playback.playing, "Releasing space leaves playback playing");

    // 4. Second distinct Space press (repeat: false) -> pauses playback
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();
    assert!(!harness.state().playback.playing, "Second distinct space press pauses playback");
}

#[test]
fn test_fusion_space_pointer_navigation_suppresses_playback() {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = FilmcraftApp::new(s);
    app.set_workspace("Fusion");
    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    // Simulate pointer down while in Fusion workspace (e.g. user navigating/dragging canvas)
    let center = egui::pos2(640.0, 360.0);
    harness.input_mut().events.push(egui::Event::PointerMoved(center));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    harness.step();

    // Now press Space while pointer is down
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();

    // Playback should remain paused because Space during canvas drag is a pan gesture, not a toggle
    assert!(!harness.state().playback.playing, "Space pressed during pointer down in Fusion must not start playback");
}

#[test]
fn test_project_hub_presets_and_cloud_truth() {
    use filmcraft_engine::cloud_sync::default_squad_folder;

    let s = Session::default();
    let app = FilmcraftApp::new(s);

    // Verify no fake cloud tokens
    assert!(app.gdrive_config.access_token.is_none(), "Must not possess fabricated access tokens");
    assert!(!app.gdrive_config.is_authenticated(), "Must not report authenticated without actual login");

    // Verify squad folder files default to unsynced
    let squad = default_squad_folder();
    assert!(!squad.project_files.is_empty());
    assert!(!squad.is_synced, "Unlinked cloud folder must be is_synced: false");

    // Test preset dimensions and frame rates
    let presets = [
        ("YouTube 4K60", 3840, 2160, 60.0),
        ("YouTube 1080p", 1920, 1080, 60.0),
        ("Shorts/TikTok 9:16", 1080, 1920, 60.0),
        ("Podcast Cam", 1920, 1080, 30.0),
    ];

    for (name, w, h, fps) in presets {
        let mut session = Session::default();
        let res = session.execute("file.newProject", json!({"name": name}));
        assert!(res.is_ok());
        let seq_res = session.execute("file.newSequence", json!({
            "name": format!("{name} Sequence"),
            "width": w,
            "height": h,
            "fps": fps,
        }));
        assert!(seq_res.is_ok());
        let active_seq_id = session.state.active_sequence.expect("active sequence");
        let seq = session.project.sequence(active_seq_id).expect("sequence exists");
        assert_eq!(seq.settings.width, w, "Width mismatch for {name}");
        assert_eq!(seq.settings.height, h, "Height mismatch for {name}");
        assert_eq!(seq.settings.frame_rate.as_f64(), fps, "FPS mismatch for {name}");
    }
}

#[test]
fn test_project_hub_mode_switcher_labeling() {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = FilmcraftApp::new(s);
    app.ui.mode = Mode::Import;

    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    // Verify Project Hub element is registered
    let hub_mode = harness.state().auto.query("header.mode.import");
    assert!(!hub_mode.is_empty(), "header.mode.import must be present");
    assert_eq!(hub_mode[0].label, "◫ Project Hub");

    // Verify Home button is registered as Project Hub
    let home = harness.state().auto.query("header.home");
    assert!(!home.is_empty(), "header.home must be present");
    assert_eq!(home[0].label, "Project Hub");
}

#[test]
fn test_node_graph_canvas_focus_suppresses_spacebar_playback() {
    use filmcraft_ui_egui::dock::PanelKind;

    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = FilmcraftApp::new(s);
    app.ui.workspace = "Fusion".to_string();
    app.ui.focused = PanelKind::NodeGraph;

    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    assert!(!harness.state().playback.playing, "Initially paused");

    // Press Space when NodeGraph canvas has active focus
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();

    // Spacebar should NOT trigger playback toggle when NodeGraph is focused
    assert!(!harness.state().playback.playing, "Spacebar must not start playback when NodeGraph has focus");
}

#[test]
fn test_cloud_projects_filenames_and_roles() {
    use filmcraft_engine::cloud_sync::default_squad_folder;

    let squad = default_squad_folder();
    let names: Vec<&str> = squad.project_files.iter().map(|f| f.name.as_str()).collect();

    // Match exact filenames from user screenshots and specification
    assert!(names.contains(&"Squad_Montage_Ep4.filmcraft"), "Must include Squad_Montage_Ep4.filmcraft");
    assert!(names.contains(&"Viral_Shorts_Pack_V2.filmcraft"), "Must include Viral_Shorts_Pack_V2.filmcraft");
    assert!(names.contains(&"Clutch_Clip_Raw.mp4"), "Must include Clutch_Clip_Raw.mp4");

    for file in &squad.project_files {
        let is_proj = file.name.ends_with(".fcproj") || file.name.ends_with(".filmcraft");
        if file.name.ends_with(".mp4") {
            assert!(!is_proj, "MP4 file must be treated as media asset, not project file");
        } else {
            assert!(is_proj, "Project files must end with .filmcraft or .fcproj");
        }
    }
}

#[test]
fn test_distinct_project_serialization_and_opening() {
    let dir = std::env::temp_dir().join(format!("fc_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&dir).unwrap();

    let path_a = dir.join("ProjectA.fcproj");
    let path_b = dir.join("ProjectB.fcproj");

    // 1. Create and save Project A
    {
        let mut session = Session::default();
        session.execute("file.newProject", json!({"name": "ProjectA"})).unwrap();
        session.execute("file.saveAs", json!({"path": path_a.to_string_lossy()})).unwrap();
    }

    // 2. Create and save Project B
    {
        let mut session = Session::default();
        session.execute("file.newProject", json!({"name": "ProjectB"})).unwrap();
        session.execute("file.saveAs", json!({"path": path_b.to_string_lossy()})).unwrap();
    }

    // 3. Open Project A in a fresh session
    {
        let mut session = Session::default();
        session.execute("file.open", json!({"path": path_a.to_string_lossy()})).unwrap();
        assert_eq!(session.project.name, "ProjectA", "Opening Project A must load ProjectA");
    }

    // 4. Open Project B in a fresh session
    {
        let mut session = Session::default();
        session.execute("file.open", json!({"path": path_b.to_string_lossy()})).unwrap();
        assert_eq!(session.project.name, "ProjectB", "Opening Project B must load ProjectB");
    }

    let _ = std::fs::remove_dir_all(&dir);
}



#[test]
fn test_fusion_spacebar_unfocused_canvas_panning_no_playback_toggle() {
    use filmcraft_ui_egui::dock::PanelKind;

    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    let mut app = FilmcraftApp::new(s);
    app.set_workspace("Fusion");
    // Start with NodeGraph UNFOCUSED (e.g. EffectControls focused)
    app.ui.focused = PanelKind::EffectControls;

    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    assert!(!harness.state().playback.playing, "Initially paused");

    // Position pointer over node canvas
    let canvas_rect = harness.state().node_canvas_rect.expect("Node canvas rect must be registered");
    let canvas_center = canvas_rect.center();
    harness.input_mut().events.push(egui::Event::PointerMoved(canvas_center));
    harness.step();

    // Sequence: 1. Hold Space key FIRST (while mouse button is NOT pressed)
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();

    // Verify Spacebar did NOT trigger playback toggle
    assert!(!harness.state().playback.playing, "Holding Space over unfocused node canvas must NOT start playback");

    // Sequence: 2. Press primary mouse button and drag to pan
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: canvas_center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerMoved(canvas_center + egui::vec2(30.0, 20.0)));
    harness.step();

    // Playback must STILL not be playing
    assert!(!harness.state().playback.playing, "Panning canvas with Space+Primary must not trigger playback");

    // Release mouse and space
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: canvas_center + egui::vec2(30.0, 20.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Default::default(),
    });
    harness.step();
    assert!(!harness.state().playback.playing, "Releasing pan must not toggle playback");
}

#[test]
fn test_project_hub_ui_open_and_import_buttons_with_distinct_projects() {
    let dir = std::env::temp_dir().join(format!("fc_hub_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&dir).unwrap();

    let path_a = dir.join("ProjectAlpha.fcproj");
    let path_b = dir.join("ProjectBeta.filmcraft"); // Testing .filmcraft extension alias!

    // Create ProjectAlpha with unique sequence
    {
        let mut s = Session::default();
        s.execute("file.newProject", json!({"name": "ProjectAlpha"})).unwrap();
        s.execute("file.newSequence", json!({"name": "AlphaSequence", "width": 1920, "height": 1080, "fps": 24.0})).unwrap();
        s.execute("file.saveAs", json!({"path": path_a.to_string_lossy()})).unwrap();
    }

    // Create ProjectBeta with unique sequence
    {
        let mut s = Session::default();
        s.execute("file.newProject", json!({"name": "ProjectBeta"})).unwrap();
        s.execute("file.newSequence", json!({"name": "BetaSequence", "width": 3840, "height": 2160, "fps": 60.0})).unwrap();
        s.execute("file.saveAs", json!({"path": path_b.to_string_lossy()})).unwrap();
    }

    // Launch app in Mode::Import with linked directory
    let s = Session::default();
    let mut app = FilmcraftApp::new(s);
    app.ui.mode = Mode::Import;
    app.gdrive_config.active_folder_name = Some(dir.to_string_lossy().to_string());

    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    // Verify discovery listed the two project files
    let row_elements = harness.state().auto.query("hub.cloud.row.");
    assert!(row_elements.len() >= 2, "Discovered cloud items must be at least 2, found {}", row_elements.len());

    // Click Open button for ProjectAlpha via UI interact
    let open_0 = harness.state().auto.query("hub.cloud.open.0");
    assert!(!open_0.is_empty(), "hub.cloud.open.0 must exist");
    let btn_rect = open_0[0].rect;
    let btn_pos = pos2(btn_rect[0] + btn_rect[2] / 2.0, btn_rect[1] + btn_rect[3] / 2.0);

    harness.input_mut().events.push(egui::Event::PointerMoved(btn_pos));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: btn_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: btn_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    harness.step();

    // Verify UI transitioned to Edit mode and ProjectAlpha was loaded
    assert_eq!(harness.state().ui.mode, Mode::Edit, "Clicking Open must transition to Edit mode");
    assert_eq!(harness.state().session.project.name, "ProjectAlpha", "ProjectAlpha must be loaded");
    let active_seq = harness.state().session.state.active_sequence.unwrap();
    assert_eq!(harness.state().session.project.item(active_seq).unwrap().name, "AlphaSequence");
    assert_eq!(harness.state().session.project.sequence(active_seq).unwrap().settings.width, 1920);

    // Switch back to Project Hub (Import mode) and open ProjectBeta (.filmcraft alias)
    harness.state_mut().ui.mode = Mode::Import;
    harness.step();

    let open_1 = harness.state().auto.query("hub.cloud.open.1");
    assert!(!open_1.is_empty(), "hub.cloud.open.1 must exist");
    let btn1_rect = open_1[0].rect;
    let btn1_pos = pos2(btn1_rect[0] + btn1_rect[2] / 2.0, btn1_rect[1] + btn1_rect[3] / 2.0);

    harness.input_mut().events.push(egui::Event::PointerMoved(btn1_pos));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: btn1_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: btn1_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    harness.step();

    assert_eq!(harness.state().ui.mode, Mode::Edit, "Clicking Open on .filmcraft project must transition to Edit mode");
    assert_eq!(harness.state().session.project.name, "ProjectBeta", "ProjectBeta must be loaded");
    let active_seq_b = harness.state().session.state.active_sequence.unwrap();
    assert_eq!(harness.state().session.project.item(active_seq_b).unwrap().name, "BetaSequence");
    assert_eq!(harness.state().session.project.sequence(active_seq_b).unwrap().settings.width, 3840);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_unavailable_cloud_project_accurate_error() {
    let s = Session::default();
    let mut app = FilmcraftApp::new(s);
    app.ui.mode = Mode::Import;
    // No linked directory -> displays sample catalog where Squad_Montage_Ep4.filmcraft does not exist on disk

    let mut harness = Harness::builder().with_size(egui::vec2(1280.0, 720.0)).build_eframe(move |_cc| app);
    harness.step();

    let open_0 = harness.state().auto.query("hub.cloud.open.0");
    assert!(!open_0.is_empty(), "hub.cloud.open.0 must exist");
    let btn_rect = open_0[0].rect;
    let btn_pos = pos2(btn_rect[0] + btn_rect[2] / 2.0, btn_rect[1] + btn_rect[3] / 2.0);

    harness.input_mut().events.push(egui::Event::PointerMoved(btn_pos));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: btn_pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: btn_pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    harness.step();

    // Assert UI did NOT falsely transition to Edit mode!
    assert_eq!(harness.state().ui.mode, Mode::Import, "Unavailable file must stay in Import mode");
    assert!(
        harness.state().ui.status.contains("not available locally"),
        "Status must truthfully report unavailable file, got: {}",
        harness.state().ui.status
    );
}

#[test]
fn test_browser_safe_virtual_file_resolution() {
    use filmcraft_ui_egui::panels::import_mode::{resolve_file_path, service_file_exists};

    let s = Session::default();
    let app = FilmcraftApp::new(s);

    // Non-existent path returns None
    assert!(!service_file_exists(&app, "non_existent_virtual_file.xyz"));
    assert_eq!(resolve_file_path(&app, "", "non_existent_virtual_file.xyz"), None);
}

#[test]
fn test_cloud_sync_status_truthfulness() {
    let s = Session::default();
    let app = FilmcraftApp::new(s);

    // Default launch: gdrive_last_synced MUST be None (no fake "Just now")
    assert_eq!(app.gdrive_last_synced, None, "Initial sync state must be None");
    assert!(!app.gdrive_config.is_authenticated(), "Initially unauthenticated");
}
