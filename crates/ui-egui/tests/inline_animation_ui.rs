use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_project::{ClipId, TrackKind};
use filmcraft_ui_egui::{
    FilmcraftApp, menus,
    panels::{
        inline_animation::{self, Entry},
        timeline,
    },
};
use serde_json::{Value, json};
use std::sync::Arc;

type App = Harness<'static, FilmcraftApp>;
fn demo(height: f32) -> App {
    let mut session = Session::default();
    session.execute("file.openDemoProject", json!({})).unwrap();
    session.execute("edit.deselectAll", json!({})).unwrap();
    let mut app = FilmcraftApp::new(session);
    app.set_workspace("Clean Editing");
    let mut h = Harness::builder().with_size(egui::vec2(1656.0, height)).build_eframe(move |_| app);
    step(&mut h);
    h
}
fn step(h: &mut App) {
    for _ in 0..3 {
        h.ctx.request_repaint();
        h.step();
    }
}
fn clip(h: &App) -> ClipId {
    h.state().session.active_sequence().unwrap().video_tracks[0].items[0].id
}
fn toggle(h: &mut App, args: Value) {
    let ctx = h.ctx.clone();
    menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", args).unwrap();
    step(h);
}
fn plan(h: &mut App, c: ClipId) -> inline_animation::Plan {
    let q = h.state().session.active_sequence().unwrap().clone();
    let id = h.state().session.state.active_sequence.unwrap();
    inline_animation::prepare(h.state_mut(), id, &q).into_iter().find(|p| p.clip == c).unwrap()
}
fn owner(h: &App, c: ClipId) -> egui::Rect {
    let track = h.state().session.active_sequence().unwrap().find_item(c).unwrap().0;
    h.state().tl.layout.as_ref().unwrap().rows.iter().find(|r| r.track == track).unwrap().rect
}

#[test]
fn default_hierarchy_keeps_owner_visible_and_split_fixed_without_editing_project() {
    let mut h = demo(894.0);
    let c = clip(&h);
    let before = h.state().tl.layout.as_ref().unwrap().clone();
    let original = h.state().session.project.clone();
    let revision = h.state().session.revision;
    toggle(&mut h, json!({"clip":c.0}));
    let l = h.state().tl.layout.as_ref().unwrap();
    assert_eq!(l.split_y, before.split_y);
    let owning = owner(&h, c);
    assert!(owning.min.y >= l.content.min.y);
    assert!(owning.max.y <= l.split_y);
    let a = l.animations.first().unwrap();
    assert_eq!(a.rect.min.y, owning.max.y);
    // Expanded rows may extend beyond their scroll viewport; only their visible hit/paint zone
    // must be disjoint from the other half's tracks.
    assert!(l.rows.iter().all(|r| r.rect.intersect(a.visible).height() <= 0.0));
    assert_eq!(timeline::hit_json(h.state(), a.visible.center(), Default::default())["hit"], "none");
    assert!(Arc::ptr_eq(&original, &h.state().session.project));
    assert_eq!(revision, h.state().session.revision);
    let p = plan(&mut h, c);
    assert!(p.rows.iter().all(|r| matches!(r, Entry::Clip | Entry::Effect { .. })));
    toggle(&mut h, json!({"clip":c.0}));
    let after = h.state().tl.layout.as_ref().unwrap();
    assert!(after.animations.is_empty());
    assert_eq!(after.split_y, before.split_y);
    for (a, b) in before.rows.iter().zip(&after.rows) {
        assert_eq!(a.rect, b.rect);
    }
}

#[test]
fn effect_and_property_disclosure_measure_exact_rows_and_keep_clip_anchor() {
    let mut h = demo(1100.0);
    let c = clip(&h);
    toggle(&mut h, json!({"clip":c.0}));
    let split = h.state().tl.layout.as_ref().unwrap().split_y;
    let start = owner(&h, c).min.y;
    let closed = plan(&mut h, c).height();
    toggle(&mut h, json!({"clip":c.0,"effect":"motion"}));
    let open = plan(&mut h, c);
    assert!(open.height() > closed);
    assert_eq!(owner(&h, c).min.y, start);
    assert_eq!(h.state().tl.layout.as_ref().unwrap().split_y, split);
    assert!(open.rows.iter().any(|r| matches!(r,Entry::Property{param,..}if param=="position")));
    assert!(!open.rows.iter().any(|r| matches!(r, Entry::Keys { .. })));
    toggle(&mut h, json!({"clip":c.0,"effect":"motion","param":"position"}));
    let keys = plan(&mut h, c);
    assert_eq!(keys.height(), open.height() + inline_animation::ROW_H);
    toggle(&mut h, json!({"clip":c.0,"effect":"motion","param":"position"}));
    assert_eq!(plan(&mut h, c).height(), open.height());
    toggle(&mut h, json!({"clip":c.0,"effect":"motion"}));
    assert_eq!(plan(&mut h, c).height(), closed);
}

#[test]
fn multiple_clips_same_track_and_same_filename_other_track_have_disjoint_rows() {
    let mut h = demo(894.0);
    let q = h.state().session.active_sequence().unwrap();
    let v2 = q.video_tracks[1].items[0].id;
    let name = q.video_tracks[1].items[0].name.clone();
    let same = q.video_tracks[0].items.iter().find(|it| it.name == name).unwrap().id;
    let first = q.video_tracks[0].items[0].id;
    for c in [first, same, v2] {
        toggle(&mut h, json!({"clip":c.0}));
    }
    let l = h.state().tl.layout.as_ref().unwrap();
    assert_eq!(l.animations.len(), 3);
    for (n, a) in l.animations.iter().enumerate() {
        for b in l.animations.iter().skip(n + 1) {
            assert!(a.rect.intersect(b.rect).height() <= 0.0);
        }
    }
    let r = owner(&h, v2);
    assert!(r.min.y >= l.content.min.y && r.max.y <= l.split_y);
}

#[test]
fn animation_uses_shared_timeline_wheel_and_no_inner_scroll_area() {
    let mut h = demo(894.0);
    let c = clip(&h);
    toggle(&mut h, json!({"clip":c.0}));
    toggle(&mut h, json!({"clip":c.0,"effect":"motion"}));
    let l = h.state().tl.layout.as_ref().unwrap();
    let a = &l.animations[0];
    let pos = egui::pos2(l.content.min.x + 100.0, a.visible.center().y);
    let before = h.state().ui.timeline.v_scroll;
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -50.0),
        modifiers: Default::default(),
        phase: egui::TouchPhase::Move,
    });
    step(&mut h);
    assert!(h.state().ui.timeline.v_scroll < before);
}

#[test]
fn duplicate_effects_have_independent_identity_based_disclosure_state() {
    let mut h = demo(894.0);
    let c = clip(&h);
    h.state_mut().session.execute("effects.apply", json!({"clips":[c.0],"effect":"gaussian_blur"})).unwrap();
    toggle(&mut h, json!({"clip":c.0,"effect":"gaussian_blur"}));
    h.state_mut().session.execute("effects.apply", json!({"clips":[c.0],"effect":"gaussian_blur"})).unwrap();
    step(&mut h);
    let ctx = h.ctx.clone();
    assert!(menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", json!({"clip":c.0,"effect":"gaussian_blur"})).is_err());
    let before = plan(&mut h, c);
    let effects: Vec<_> = h
        .state()
        .session
        .active_sequence()
        .unwrap()
        .find_item(c)
        .unwrap()
        .1
        .effects
        .iter()
        .filter(|e| e.effect == "gaussian_blur")
        .map(|e| e.instance_id)
        .collect();
    toggle(&mut h, json!({"clip":c.0,"effectId":effects[1]}));
    let after = plan(&mut h, c);
    assert!(after.height() > before.height());
    h.state_mut().session.execute("effects.reorder", json!({"clip":c.0,"effect":{"instanceId":effects[1]},"to":0})).unwrap();
    assert_eq!(plan(&mut h, c).height(), after.height());
    toggle(&mut h, json!({"clip":c.0,"effectId":effects[0]}));
    assert_eq!(plan(&mut h, c).height(), before.height());
}

#[test]
fn document_replacement_clears_disclosure_even_with_reused_clip_ids() {
    let mut h = demo(894.0);
    let c = clip(&h);
    toggle(&mut h, json!({"clip":c.0,"effect":"motion"}));
    assert!(!h.state().tl.animation.is_empty());
    h.state_mut().session.execute("file.openDemoProject", json!({})).unwrap();
    step(&mut h);
    assert!(h.state().tl.animation.is_empty());
}

#[test]
fn audio_disclosure_preserves_split_and_deleted_clips_are_pruned() {
    let mut h = demo(894.0);
    let c = h.state().session.active_sequence().unwrap().audio_tracks[0].items[0].id;
    let split = h.state().tl.layout.as_ref().unwrap().split_y;
    toggle(&mut h, json!({"clip":c.0}));
    assert_eq!(h.state().tl.layout.as_ref().unwrap().split_y, split);
    let r = owner(&h, c);
    assert!(r.min.y >= split);
    assert!(h.state().tl.layout.as_ref().unwrap().rows.iter().any(|r| r.kind == TrackKind::Video));
    h.state_mut().session.execute("timeline.select", json!({"clips":[c.0],"linked":false})).unwrap();
    h.state_mut().session.execute("edit.clear", json!({})).unwrap();
    step(&mut h);
    assert!(!h.state().tl.animation.contains(c));
    let v = clip(&h);
    toggle(&mut h, json!({"clip":v.0}));
    h.state_mut().tl.reset_media_caches();
    assert!(h.state().tl.animation.is_empty());
}

#[test]
fn malformed_view_arguments_are_actionable_and_non_destructive() {
    let mut h = demo(894.0);
    let ctx = h.ctx.clone();
    let before = h.state().session.project.clone();
    for args in [json!(null), json!({"clip":-1}), json!({"effect":12}), json!({"clip":u64::MAX}), json!({"clip":clip(&h).0,"effect":"missing"})] {
        assert!(menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", args).is_err());
    }
    assert!(h.state().tl.animation.is_empty());
    assert!(Arc::ptr_eq(&before, &h.state().session.project));
}

#[test]
fn shortening_the_video_viewport_reveals_the_previously_visible_owner_without_overriding_split() {
    let mut h = demo(894.0);
    let c = clip(&h);
    toggle(&mut h, json!({"clip":c.0}));
    h.state_mut().ui.timeline.split = 0.25;
    step(&mut h);
    assert_eq!(h.state().ui.timeline.split, 0.25);
    let l = h.state().tl.layout.as_ref().unwrap();
    let r = owner(&h, c);
    assert!(r.min.y >= l.content.min.y);
    assert!(r.max.y <= l.split_y);
}

#[test]
fn fixed_controls_precede_added_effects_without_changing_project_order() {
    let mut h = demo(894.0);
    let c = clip(&h);
    for effect in ["lumetri", "gaussian_blur"] {
        h.state_mut().session.execute("effects.apply", json!({"clips":[c.0],"effect":effect})).unwrap();
    }
    toggle(&mut h, json!({"clip":c.0}));
    let before = h.state().session.project.clone();
    let p = plan(&mut h, c);
    let it = h.state().session.active_sequence().unwrap().find_item(c).unwrap().1;
    let displayed: Vec<_> =
        p.rows.iter().filter_map(|r| if let Entry::Effect { slot, .. } = r { Some(it.effects[*slot].effect.as_str()) } else { None }).collect();
    assert_eq!(&displayed[..3], ["motion", "opacity", "time_remap"]);
    let expected: Vec<_> = it.effects.iter().filter(|e| !e.def().is_some_and(|d| d.intrinsic)).map(|e| e.effect.as_str()).collect();
    assert_eq!(&displayed[3..], expected.as_slice());
    assert!(Arc::ptr_eq(&before, &h.state().session.project));
}

#[test]
fn canonical_inline_parameter_and_key_commands_survive_real_project_save_and_reopen() {
    let mut h = demo(894.0);
    let c = clip(&h);
    let qid = h.state().session.state.active_sequence.unwrap();
    let rate = h.state().session.sequence_rate();
    for (frame, value) in [(15, 40.0), (45, 75.0)] {
        h.state_mut().session.execute("playhead.set", json!({"frame":frame})).unwrap();
        if frame == 15 {
            h.state_mut().session.execute("effects.setParam", json!({"clip":c.0,"effect":"opacity","param":"opacity","value":value})).unwrap();
            h.state_mut().session.execute("effects.addKeyframe", json!({"clip":c.0,"effect":"opacity","param":"opacity"})).unwrap();
        } else {
            h.state_mut().session.execute("effects.setParam", json!({"clip":c.0,"effect":"opacity","param":"opacity","value":value})).unwrap();
        }
    }
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("fc-v7-inline-{}-{nonce}.fcproj", std::process::id()));
    h.state_mut().session.execute("file.saveAs", json!({"path":path.to_string_lossy()})).unwrap();
    h.state_mut().session.execute("file.newProject", json!({"name":"Roundtrip"})).unwrap();
    h.state_mut().session.execute("file.open", json!({"path":path.to_string_lossy()})).unwrap();
    let it = h.state().session.project.sequence(qid).unwrap().find_item(c).unwrap().1;
    let param = it.effect("opacity").unwrap().param("opacity").unwrap();
    assert_eq!(param.keyframes.len(), 2);
    assert_eq!(param.f64_at(it.effect_time_at(rate.tick_of(15))), 40.0);
    assert_eq!(param.f64_at(it.effect_time_at(rate.tick_of(45))), 75.0);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn unsupported_retime_domains_display_values_but_disable_inline_mutation() {
    for (command, args) in [
        ("clip.speedDuration", json!({"speed":200,"ripple":false})),
        ("clip.speedDuration", json!({"speed":100,"reverse":true,"ripple":false})),
        ("clip.frameHoldOptions", json!({"holdOn":"in","enabled":true})),
    ] {
        let mut h = demo(1100.0);
        let c = clip(&h);
        h.state_mut().session.execute("timeline.select", json!({"clips":[c.0],"linked":false})).unwrap();
        h.state_mut().session.execute(command, args).unwrap();
        toggle(&mut h, json!({"clip":c.0,"effect":"opacity"}));
        toggle(&mut h, json!({"clip":c.0,"effect":"opacity","param":"opacity"}));
        // A disclosed unsupported domain still shows its actual canonical property, but
        // neither its value editor nor a key mutation control can silently write a wrong time.
        let id = format!("timeline.animation.clip.{}.effect.opacity.param.opacity", c.0);
        let value = h.state().auto.find(&format!("{id}.value")).expect("visible Opacity value");
        assert_eq!(value.enabled, Some(false), "{command}");
        for suffix in ["animation", "addKey", "previous", "next"] {
            assert!(h.state().auto.find(&format!("{id}.{suffix}")).is_none(), "{command}: {suffix}");
        }
    }
}

fn click_widget(h: &mut App, id: &str, button: egui::PointerButton) {
    let [x, y, w, height] = h.state().auto.find(id).unwrap_or_else(|| panic!("Missing {id}")).rect;
    let pos = egui::pos2(x + w / 2.0, y + height / 2.0);
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: true, modifiers: Default::default() });
    step(h);
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: false, modifiers: Default::default() });
    step(h);
}

fn key_fixture() -> (App, ClipId, Vec<filmcraft_time::Tick>) {
    let mut h = demo(1100.0);
    let c = clip(&h);
    h.state_mut().ui.timeline.split = 0.8;
    for frame in [15, 45, 75] {
        h.state_mut().session.execute("playhead.set", json!({"frame":frame})).unwrap();
        h.state_mut().session.execute("effects.addKeyframe", json!({"clip":c.0,"effect":"opacity","param":"opacity"})).unwrap();
    }
    toggle(&mut h, json!({"clip":c.0,"effect":"opacity","param":"opacity"}));
    let times = h
        .state()
        .session
        .active_sequence()
        .unwrap()
        .find_item(c)
        .unwrap()
        .1
        .effect("opacity")
        .unwrap()
        .param("opacity")
        .unwrap()
        .keyframes
        .iter()
        .map(|k| k.time)
        .collect();
    (h, c, times)
}

fn key_point(h: &App, c: ClipId, time: filmcraft_time::Tick) -> egui::Pos2 {
    let id = format!("timeline.animation.clip.{}.effect.opacity.param.opacity.key.{}", c.0, time.0);
    let [x, y, w, height] = h.state().auto.find(&id).unwrap_or_else(|| panic!("Missing {id}")).rect;
    egui::pos2(x + w / 2.0, y + height / 2.0)
}

fn pointer(h: &mut App, pos: egui::Pos2, down: bool, modifiers: egui::Modifiers) {
    h.input_mut().events.push(egui::Event::ModifiersChanged(modifiers));
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: down, modifiers });
    step(h);
}

fn preview(h: &mut App, from: egui::Pos2, to: egui::Pos2) {
    pointer(h, from, true, Default::default());
    for n in 1..=8 {
        h.input_mut().events.push(egui::Event::PointerMoved(from + (to - from) * (n as f32 / 8.0)));
        step(h);
    }
}

#[test]
fn inline_shift_ctrl_selection_group_drag_preview_cancel_and_history_use_canonical_keys() {
    let (mut h, c, times) = key_fixture();
    for (n, modifiers) in [(0, egui::Modifiers::NONE), (1, egui::Modifiers::SHIFT), (2, egui::Modifiers::CTRL)] {
        let at = key_point(&h, c, times[n]);
        pointer(&mut h, at, true, modifiers);
        pointer(&mut h, at, false, modifiers);
    }
    assert_eq!(h.state().ui.keyframe_selection.len(), 3);
    let project = h.state().session.project.clone();
    let undo = h.state().session.history.undo.len();
    let start = key_point(&h, c, times[0]);
    let delta = h.state().session.sequence_rate().tick_of(15);
    let end = start + egui::vec2(h.state().tl.layout.as_ref().unwrap().pps as f32 * delta.seconds() as f32, 0.0);
    preview(&mut h, start, end);
    assert!(Arc::ptr_eq(&project, &h.state().session.project), "preview edited the project");
    pointer(&mut h, end, false, Default::default());
    assert_eq!(h.state().session.history.undo.len(), undo + 1);
    let moved: Vec<_> = h.state().ui.keyframe_selection.iter().map(|k| k.time).collect();
    assert_eq!(moved, times.iter().map(|t| *t + delta).collect::<Vec<_>>());
    let after = h.state().session.project.clone();
    h.state_mut().session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(*h.state().session.project, *project);
    h.state_mut().session.execute("edit.redo", json!({})).unwrap();
    assert_eq!(*h.state().session.project, *after);
    step(&mut h);
    let start = key_point(&h, c, moved[0]);
    preview(&mut h, start, start + egui::vec2(22.0, 0.0));
    h.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
    step(&mut h);
    pointer(&mut h, start + egui::vec2(22.0, 0.0), false, Default::default());
    assert_eq!(*h.state().session.project, *after, "Escape committed a preview");
}

#[test]
fn inline_marquee_and_collision_rejection_preserve_unselected_keys() {
    let (mut h, c, times) = key_fixture();
    let a = key_point(&h, c, times[0]);
    let b = key_point(&h, c, times[1]);
    let from = a - egui::vec2(14.0, 8.0);
    let to = b + egui::vec2(8.0, 8.0);
    preview(&mut h, from, to);
    pointer(&mut h, to, false, Default::default());
    assert_eq!(h.state().ui.keyframe_selection.len(), 2);
    pointer(&mut h, a, true, Default::default());
    pointer(&mut h, a, false, Default::default());
    assert_eq!(h.state().ui.keyframe_selection.len(), 1);
    let before = h.state().session.project.clone();
    preview(&mut h, a, b);
    pointer(&mut h, b, false, Default::default());
    assert_eq!(*h.state().session.project, *before);
    assert!(h.state().ui.status.contains("occupies"), "{}", h.state().ui.status);
}

#[test]
fn inline_filters_only_change_view_and_shared_selection_survives_inspector_open() {
    let (mut h, c, times) = key_fixture();
    let at = key_point(&h, c, times[0]);
    pointer(&mut h, at, true, Default::default());
    pointer(&mut h, at, false, Default::default());
    let selected = h.state().ui.keyframe_selection.clone();
    let before = h.state().session.project.clone();
    for filter in ["Animated", "Modified", "Selected", "All"] {
        click_widget(&mut h, &format!("timeline.animation.clip.{}.filter.{filter}", c.0), egui::PointerButton::Primary);
        assert_eq!(h.state().ui.keyframe_selection, selected);
        assert!(Arc::ptr_eq(&before, &h.state().session.project));
    }
    h.state_mut().show_panel(filmcraft_ui_egui::dock::PanelKind::EffectControls);
    step(&mut h);
    assert_eq!(h.state().ui.keyframe_selection, selected);
    assert_eq!(h.state().session.state.selected_effect.as_ref().unwrap().instance_id, selected[0].effect_id);
}

#[test]
fn real_inline_key_menu_edits_interpolation_and_exact_time_with_undo_and_collision_guards() {
    use filmcraft_project::Interpolation;
    let mut h = demo(1100.0);
    let c = clip(&h);
    h.state_mut().ui.timeline.split = 0.8;
    let rate = h.state().session.sequence_rate();
    for (frame, value) in [(15, 40.0), (45, 75.0)] {
        h.state_mut().session.execute("playhead.set", json!({"frame":frame})).unwrap();
        h.state_mut().session.execute("effects.setParam", json!({"clip":c.0,"effect":"opacity","param":"opacity","value":value})).unwrap();
        if frame == 15 {
            h.state_mut().session.execute("effects.addKeyframe", json!({"clip":c.0,"effect":"opacity","param":"opacity"})).unwrap();
        }
    }
    toggle(&mut h, json!({"clip":c.0,"effect":"opacity"}));
    toggle(&mut h, json!({"clip":c.0,"effect":"opacity","param":"opacity"}));
    let key = |h: &App, frame| {
        let it = h.state().session.active_sequence().unwrap().find_item(c).unwrap().1;
        it.effect("opacity").unwrap().param("opacity").unwrap().keyframes.iter().find(|k| k.time == it.effect_time_at(rate.tick_of(frame))).cloned()
    };
    let source = key(&h, 15).unwrap().time;
    let id = format!("timeline.animation.clip.{}.effect.opacity.param.opacity.key.{}", c.0, source.0);
    for (kind, wire) in [
        (Interpolation::Hold, "hold"),
        (Interpolation::Bezier, "bezier"),
        (Interpolation::AutoBezier, "autoBezier"),
        (Interpolation::ContinuousBezier, "continuousBezier"),
        (Interpolation::EaseIn, "easeIn"),
        (Interpolation::EaseOut, "easeOut"),
        (Interpolation::Linear, "linear"),
    ] {
        let old = key(&h, 15).unwrap();
        let project = h.state().session.project.clone();
        click_widget(&mut h, &id, egui::PointerButton::Secondary);
        assert!(Arc::ptr_eq(&project, &h.state().session.project));
        click_widget(&mut h, &format!("{id}.menu.{wire}"), egui::PointerButton::Primary);
        assert_eq!(key(&h, 15).unwrap().interp, kind);
        h.state_mut().session.execute("edit.undo", json!({})).unwrap();
        assert_eq!(key(&h, 15).unwrap(), old);
        h.state_mut().session.execute("edit.redo", json!({})).unwrap();
        assert_eq!(key(&h, 15).unwrap().interp, kind);
        step(&mut h);
    }
    // The second key already occupies the playhead, so a move must not overwrite it.
    click_widget(&mut h, &id, egui::PointerButton::Secondary);
    assert_eq!(h.state().auto.find(&format!("{id}.menu.moveToPlayhead")).unwrap().enabled, Some(false));
    h.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
    step(&mut h);
    let outside = h.state().session.active_sequence().unwrap().find_item(c).unwrap().1.end() + rate.frame_duration();
    h.state_mut().session.execute("playhead.set", json!({"time":outside.0})).unwrap();
    step(&mut h);
    click_widget(&mut h, &id, egui::PointerButton::Secondary);
    assert_eq!(h.state().auto.find(&format!("{id}.menu.moveToPlayhead")).unwrap().enabled, Some(false));
    h.input_mut().events.push(egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
    step(&mut h);
    h.state_mut().session.execute("playhead.set", json!({"frame":30})).unwrap();
    step(&mut h);
    let old = key(&h, 15).unwrap();
    let expected_source = h.state().session.active_sequence().unwrap().find_item(c).unwrap().1.effect_time_at(rate.tick_of(30));
    click_widget(&mut h, &id, egui::PointerButton::Secondary);
    click_widget(&mut h, &format!("{id}.menu.moveToPlayhead"), egui::PointerButton::Primary);
    let moved = key(&h, 30).unwrap();
    assert!(key(&h, 15).is_none());
    assert_eq!(moved.time, expected_source);
    assert_eq!(moved.value, old.value);
    assert_eq!(moved.interp, old.interp);
    h.state_mut().session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(key(&h, 15).unwrap(), old);
    h.state_mut().session.execute("edit.redo", json!({})).unwrap();
    step(&mut h);
    let moved_id = format!("timeline.animation.clip.{}.effect.opacity.param.opacity.key.{}", c.0, moved.time.0);
    click_widget(&mut h, &moved_id, egui::PointerButton::Secondary);
    click_widget(&mut h, &format!("{moved_id}.menu.delete"), egui::PointerButton::Primary);
    assert!(key(&h, 30).is_none());
    assert!(key(&h, 45).is_some());
    h.state_mut().session.execute("edit.undo", json!({})).unwrap();
    assert_eq!(key(&h, 30).unwrap(), moved);
}
