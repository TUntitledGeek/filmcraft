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
fn duplicate_effects_never_get_index_based_disclosure_state() {
    let mut h = demo(894.0);
    let c = clip(&h);
    h.state_mut().session.execute("effects.apply", json!({"clips":[c.0],"effect":"gaussian_blur"})).unwrap();
    toggle(&mut h, json!({"clip":c.0,"effect":"gaussian_blur"}));
    h.state_mut().session.execute("effects.apply", json!({"clips":[c.0],"effect":"gaussian_blur"})).unwrap();
    step(&mut h);
    let ctx = h.ctx.clone();
    let before = h.state().session.project.clone();
    assert!(menus::invoke(h.state_mut(), &ctx, "timeline.animation.toggle", json!({"clip":c.0,"effect":"gaussian_blur"})).unwrap_err().contains("Duplicate"));
    assert!(Arc::ptr_eq(&before, &h.state().session.project));
    let p = plan(&mut h, c);
    assert!(!p.rows.iter().any(|r| matches!(r, Entry::Property { .. })));
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
