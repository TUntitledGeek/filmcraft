use crate::Session;
use filmcraft_project::MediaRef;
use filmcraft_time::Tick;
use std::sync::Arc;

#[test]
fn retiring_worker_sources_keeps_recent_sources_and_live_handles_usable() {
    let mut session = Session::default();
    session.execute("file.openDemoProject", serde_json::json!({})).unwrap();
    session.media.set_idle_source_limit(2);
    let items: Vec<_> = session.project.items.values().filter(|i| i.as_media().is_some()).map(|i| i.id).take(4).collect();
    assert_eq!(items.len(), 4);
    let sources: Vec<_> = items.iter().map(|id| session.media.source_for(&session.project, *id, &*session.services).unwrap()).collect();
    // Reusing the first item makes it recent; arbitrary map iteration order must
    // not evict it in favor of sources the worker is no longer showing.
    let recent = session.media.source_for(&session.project, items[0], &*session.services).unwrap();
    session.media.trim_idle_sources();
    assert!(Arc::ptr_eq(&recent, &session.media.cached(items[0]).unwrap()));
    assert!(session.media.cached(items[3]).is_some());
    assert!(session.media.cached(items[1]).is_none());
    assert!(sources[1].video_frame(filmcraft_media::FrameRequest { time: Tick::ZERO, scale: 0.01 }).is_ok());
    let reopened = session.media.source_for(&session.project, items[1], &*session.services).unwrap();
    assert!(!Arc::ptr_eq(&sources[1], &reopened), "retired decoder/source was recreated rather than retaining its entire cache");
}

#[test]
fn preview_manifest_accepts_only_preview_names_and_rejects_traversal_without_changes() {
    let store = crate::previews::PreviewStore::temp();
    let names = vec!["0123456789abcdef0123456789abcdef.mov".into(), "0123456789abcdef0123456789abcdef-24-48.mov".into()];
    store.install_manifest(Some("/previews".into()), &names).unwrap();
    let original = store.manifest();
    assert_eq!(original.1.len(), 2);
    for bad in ["../secret.mov", "invalid.mov", "0123456789abcdef0123456789abcdef01-1-2.mov/../other"] {
        assert!(store.install_manifest(Some("/different".into()), &[bad.into()]).is_err());
        assert_eq!(store.manifest(), original);
    }
}

#[test]
fn idle_file_media_check_matches_enumeration_for_file_generator_and_offline_items() {
    let mut session = Session::default();
    session.execute("file.openDemoProject", serde_json::json!({})).unwrap();
    assert!(!crate::media_watch::has_file_media(&session.project));
    let project = Arc::make_mut(&mut session.project);
    let media = project.items.values_mut().find_map(|i| i.as_media_mut()).unwrap();
    media.media = MediaRef::File { path: "/files/worker-check.mp4".into() };
    media.offline = false;
    assert!(crate::media_watch::has_file_media(project));
    assert!(!crate::media_watch::media_files(project, &session.media).is_empty());
    let media = project.items.values_mut().find_map(|i| i.as_media_mut()).unwrap();
    media.offline = true;
    assert!(!crate::media_watch::has_file_media(project));
    assert!(crate::media_watch::media_files(project, &session.media).is_empty());
}
