//! Project Hub (Header "Project Hub", also Home).
//!
//! Provides project templates (YouTube 4K60, YouTube 1080p, Shorts/TikTok 9:16, Podcast Cam),
//! recent projects, and real files in a user-selected local folder.

use egui::{Align2, Color32, Rect, Sense, pos2, vec2};
use serde_json::json;

use crate::FilmcraftApp;
use crate::icons::{self, Icon};
use crate::theme::Tokens;

/// Check if a path exists using the engine's host services (safe on desktop and browser virtual files).
pub fn service_file_exists(app: &FilmcraftApp, path: &str) -> bool {
    app.session.services.file_size(path).is_ok()
}

/// Resolve candidate file location across bound directory, web virtual files (/files/...), and local path.
pub fn resolve_file_path(app: &FilmcraftApp, candidate_dir: &str, file_name: &str) -> Option<String> {
    if !candidate_dir.is_empty() {
        let combined = if candidate_dir.ends_with('/') || candidate_dir.ends_with('\\') {
            format!("{candidate_dir}{file_name}")
        } else {
            format!("{candidate_dir}/{file_name}")
        };
        if service_file_exists(app, &combined) {
            return Some(combined);
        }
    }
    let web_path = format!("/files/{file_name}");
    if service_file_exists(app, &web_path) {
        return Some(web_path);
    }
    if service_file_exists(app, file_name) {
        return Some(file_name.to_string());
    }
    None
}

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().with_clip_rect(rect);

    // Deep panel surface
    p.rect_filled(rect, t.radius, t.panel_bg);

    let ctx = ui.ctx().clone();

    // Header Title & Subtitle
    p.text(rect.min + vec2(28.0, 32.0), Align2::LEFT_CENTER, "Project Hub", Tokens::semibold(24.0), t.text);
    p.text(rect.min + vec2(28.0, 58.0), Align2::LEFT_CENTER, "Create a project, pick up an edit, or explore the demo.", Tokens::ui(13.0), t.text_dim);

    // Quick Action Header Buttons
    let mut btn_x = rect.max.x - 28.0;

    // Button 3: Link Folder
    let link_w = 120.0;
    btn_x -= link_w;
    let link_r = Rect::from_min_size(pos2(btn_x, rect.min.y + 26.0), vec2(link_w, 32.0));
    let can_link = crate::native_dialogs::FileDialog::Folder { at: None }.available(&app.hooks);
    let lresp = ui.interact(link_r, egui::Id::new("hub-hdr-link-folder"), if can_link { Sense::click() } else { Sense::hover() }).on_hover_text(if can_link {
        "Choose a local folder"
    } else {
        "This browser host has no folder picker. Use Open Project to access local files."
    });
    app.auto.add("hub.action.link_folder", link_r, "Link Folder");
    let lbg = if lresp.hovered() { t.hover } else { t.tl_header_bg };
    p.rect_filled(link_r, 6.0, lbg);
    p.rect_stroke(link_r, 6.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    p.text(link_r.center(), Align2::CENTER_CENTER, "Link Folder", Tokens::semibold(12.0), if can_link { t.text } else { t.text_faint });
    if lresp.clicked() {
        link_local_folder(app);
    }

    // Button 2: Load Demo
    let demo_w = 140.0;
    btn_x -= demo_w + 10.0;
    let demo_r = Rect::from_min_size(pos2(btn_x, rect.min.y + 26.0), vec2(demo_w, 32.0));
    let dresp = ui.interact(demo_r, egui::Id::new("hub-hdr-load-demo"), Sense::click());
    app.auto.add("hub.action.load_demo", demo_r, "Load Demo Edit");
    let dbg = if dresp.hovered() { t.accent_hover } else { t.accent };
    p.rect_filled(demo_r, 6.0, dbg);
    p.text(demo_r.center(), Align2::CENTER_CENTER, "Load Demo", Tokens::semibold(12.0), Color32::WHITE);
    if dresp.clicked() && crate::menus::invoke(app, &ctx, "file.openDemoProject", json!({})).is_ok() {
        app.ui.mode = crate::state::Mode::Edit;
        app.ui.status = "Opened FilmCraft Studio Pro Demo Project.".into();
    }

    // Button 1: New Project
    let new_w = 120.0;
    btn_x -= new_w + 10.0;
    let new_r = Rect::from_min_size(pos2(btn_x, rect.min.y + 26.0), vec2(new_w, 32.0));
    let nresp = ui.interact(new_r, egui::Id::new("hub-hdr-new-project"), Sense::click());
    app.auto.add("hub.action.new_project", new_r, "New Project");
    let nbg = if nresp.hovered() { t.hover } else { t.tl_header_bg };
    p.rect_filled(new_r, 6.0, nbg);
    p.rect_stroke(new_r, 6.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    p.text(new_r.center(), Align2::CENTER_CENTER, "+ New Project", Tokens::semibold(12.0), t.text);
    if nresp.clicked() && crate::menus::invoke(app, &ctx, "file.newProject", json!({"name": "Untitled Studio Project"})).is_ok() {
        let _ = crate::menus::invoke(
            app,
            &ctx,
            "file.newSequence",
            json!({
                "name": "Sequence 01",
                "width": 1920,
                "height": 1080,
                "fps": 23.976
            }),
        );
        app.ui.mode = crate::state::Mode::Edit;
        app.ui.status = "Created new project.".into();
    }

    let open_r = Rect::from_min_size(pos2(new_r.min.x - 130.0, new_r.min.y), vec2(120.0, 32.0));
    let open = ui.interact(open_r, egui::Id::new("hub-open-project"), Sense::click());
    app.auto.add("hub.action.open_project", open_r, "Open Project");
    p.rect_filled(open_r, 6.0, if open.hovered() { t.hover } else { t.tl_header_bg });
    p.rect_stroke(open_r, 6.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    icons::paint(&p, Rect::from_center_size(pos2(open_r.min.x + 17.0, open_r.center().y), vec2(16.0, 16.0)), Icon::Folder, t.icon);
    p.text(open_r.min + vec2(31.0, 16.0), Align2::LEFT_CENTER, "Open Project", Tokens::semibold(12.0), t.text);
    if open.clicked() {
        let _ = crate::menus::invoke(app, &ctx, "file.open", json!({}));
    }
    p.text(rect.min + vec2(28.0, 92.0), Align2::LEFT_CENTER, "NEW SEQUENCE PRESETS", Tokens::semibold(10.0), t.text_dim);

    // SECTION 1: New Project Creator Presets
    let presets: [(&str, u32, u32, f64, &str, Icon, Color32); 4] = [
        ("YouTube 4K60", 3840, 2160, 60.0, "3840 × 2160 · 60 fps", Icon::Film, Color32::from_rgb(0xff, 0x4e, 0x4e)),
        ("YouTube 1080p", 1920, 1080, 60.0, "1920 × 1080 · 60 fps", Icon::Sequence, Color32::from_rgb(0x3a, 0x8a, 0xec)),
        ("Shorts/TikTok 9:16", 1080, 1920, 60.0, "1080 × 1920 · 60 fps", Icon::Film, Color32::from_rgb(0xec, 0x3a, 0x8a)),
        ("Podcast 1080p", 1920, 1080, 30.0, "1920 × 1080 · 30 fps", Icon::Folder, Color32::from_rgb(0x3a, 0xcc, 0x8a)),
    ];

    let cw = ((rect.width() - 56.0 - 3.0 * 16.0) / 4.0).max(100.0);
    for (i, (title, width, height, fps, sub, icon, accent_col)) in presets.iter().enumerate() {
        let r = Rect::from_min_size(rect.min + vec2(28.0 + i as f32 * (cw + 16.0), 112.0), vec2(cw, 120.0));
        let resp = ui.interact(r, egui::Id::new(("hub-preset", i)), Sense::click());
        app.auto.add(&format!("hub.preset.{i}"), r, title);

        let bg = if resp.hovered() { t.hover } else { t.tl_header_bg };
        p.rect_filled(r, 8.0, bg);
        p.rect_stroke(r, 8.0, egui::Stroke::new(1.0, if resp.hovered() { *accent_col } else { t.separator }), egui::StrokeKind::Inside);

        // Icon with color badge
        let icon_r = Rect::from_min_size(r.min + vec2(16.0, 16.0), vec2(32.0, 32.0));
        p.rect_filled(icon_r, 6.0, Color32::from_rgba_unmultiplied(accent_col.r(), accent_col.g(), accent_col.b(), 40));
        icons::paint(&p, icon_r.shrink(6.0), *icon, *accent_col);

        p.text(r.min + vec2(16.0, 68.0), Align2::LEFT_CENTER, *title, Tokens::semibold(14.0), t.text);
        p.text(r.min + vec2(16.0, 92.0), Align2::LEFT_CENTER, *sub, Tokens::ui(11.0), t.text_dim);

        if resp.clicked() {
            let p_name = title.to_string();
            if crate::menus::invoke(app, &ctx, "file.newProject", json!({"name": p_name})).is_ok() {
                let _ = crate::menus::invoke(
                    app,
                    &ctx,
                    "file.newSequence",
                    json!({
                        "name": format!("{title} Sequence"),
                        "width": *width,
                        "height": *height,
                        "fps": *fps
                    }),
                );
            }
            app.ui.mode = crate::state::Mode::Edit;
        }
    }

    let y_split = 112.0 + 120.0 + 32.0;
    let col_w = ((rect.width() - 56.0 - 24.0) / 2.0).max(300.0);

    // SECTION 2 (Left): Recent Projects
    let recent_r = Rect::from_min_size(rect.min + vec2(28.0, y_split), vec2(col_w, rect.height() - y_split - 28.0));
    p.text(recent_r.min + vec2(0.0, 12.0), Align2::LEFT_CENTER, "Recent Projects", Tokens::semibold(15.0), t.text);
    show_recent_projects(app, ui, recent_r);

    // SECTION 3 (Right): Google Drive Cloud & Shared Projects
    let gdrive_r = Rect::from_min_size(rect.min + vec2(28.0 + col_w + 24.0, y_split), vec2(col_w, rect.height() - y_split - 28.0));
    show_cloud_projects(app, ui, gdrive_r);
}

fn show_recent_projects(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let paths: Vec<String> = app.session.prefs.general.recent_projects.iter().take(6).cloned().collect();

    if paths.is_empty() {
        ui.painter().text(rect.min + vec2(0.0, 48.0), Align2::LEFT_CENTER, "(No recent projects yet)", Tokens::ui(12.0), t.text_dim);
        return;
    }

    let row_h = 44.0;
    let mut open = None;
    for (i, path) in paths.iter().enumerate() {
        let row = Rect::from_min_size(rect.min + vec2(0.0, 32.0 + i as f32 * (row_h + 8.0)), vec2(rect.width(), row_h));
        let resp = ui.interact(row, egui::Id::new(("hub-recent", i)), Sense::click()).on_hover_text(path.as_str());
        let name = crate::menus::recent_name(path);
        app.auto.add(&format!("hub.recent.{i}"), row, &name);

        let bg = if resp.hovered() { t.hover } else { t.tl_header_bg };
        ui.painter().rect_filled(row, 6.0, bg);
        ui.painter().rect_stroke(row, 6.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);

        // Project badge icon
        icons::paint(ui.painter(), Rect::from_center_size(pos2(row.min.x + 22.0, row.center().y), vec2(18.0, 18.0)), Icon::Sequence, t.accent);

        // Title and folder
        let title_pos = pos2(row.min.x + 42.0, row.min.y + 14.0);
        ui.painter().text(title_pos, Align2::LEFT_CENTER, &name, Tokens::semibold(13.0), t.text);

        if let Some(parent) = std::path::Path::new(path).parent() {
            let path_pos = pos2(row.min.x + 42.0, row.min.y + 30.0);
            ui.painter().text(path_pos, Align2::LEFT_CENTER, parent.to_string_lossy(), Tokens::ui(11.0), t.text_dim);
        }

        // Truthful cloud status badge (🟢 Synced vs 📁 Linked vs 💾 Local)
        let is_bound_path = app.gdrive_config.active_folder_name.as_deref().is_some_and(|dir| path.starts_with(dir));
        let badge_rect = Rect::from_min_size(pos2(row.max.x - 76.0, row.center().y - 10.0), vec2(68.0, 20.0));
        let (b_bg, b_fg, b_text) = if is_bound_path {
            if app.gdrive_last_synced.is_some() {
                (Color32::from_rgb(0x18, 0x38, 0x22), Color32::from_rgb(0x4a, 0xde, 0x80), "Synced")
            } else {
                (Color32::from_rgb(0x18, 0x28, 0x48), Color32::from_rgb(0x60, 0xb0, 0xec), "Linked")
            }
        } else {
            (Color32::from_rgb(0x25, 0x28, 0x34), Color32::from_rgb(0x94, 0xa3, 0xb8), "Local")
        };
        ui.painter().rect_filled(badge_rect, 4.0, b_bg);
        ui.painter().text(badge_rect.center(), Align2::CENTER_CENTER, b_text, Tokens::ui(10.5), b_fg);

        if resp.clicked() {
            open = Some(path.clone());
        }
    }

    if let Some(path) = open {
        let ctx = ui.ctx().clone();
        match crate::menus::invoke(app, &ctx, "file.open", json!({"path": path})) {
            Ok(_) => {
                app.ui.mode = crate::state::Mode::Edit;
            }
            Err(e) => {
                app.ui.status = format!("Failed to open recent project '{path}': {e}");
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiscoveredEntry {
    pub name: String,
    pub size_bytes: u64,
    pub is_project: bool,
    pub is_sample: bool,
}

#[derive(Clone, Debug)]
pub struct HubDirectoryCache {
    pub path: String,
    pub scanned_at: web_time::Instant,
    pub entries: Vec<DiscoveredEntry>,
    pub error: Option<String>,
}

fn show_cloud_projects(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let active_dir = app.gdrive_config.active_folder_name.clone().unwrap_or_default();
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).id_salt("hub-local-files"));
    child.horizontal(|ui| {
        ui.label(egui::RichText::new("Linked folder").strong().size(15.0));
        let refresh = ui.small_button("Refresh");
        app.auto.add_response("hub.folder.refresh", &refresh, "Refresh folder");
        if refresh.clicked() {
            app.hub_cache = None;
        }
    });
    if active_dir.is_empty() {
        child.add_space(18.0);
        child.label("Your files, where you keep them.");
        child.label(
            egui::RichText::new(
                "Link a local folder to browse real projects and media.
Try Load Demo for a complete editing project.",
            )
            .color(t.text_dim),
        );
        return;
    }
    child.add(egui::Label::new(egui::RichText::new(&active_dir).color(t.text_dim)).truncate());
    let stale = app.hub_cache.as_ref().is_none_or(|c| c.path != active_dir || c.scanned_at.elapsed() > std::time::Duration::from_secs(5));
    if stale {
        let mut entries = Vec::new();
        let mut error = None;
        match app.session.services.list_entries(&active_dir) {
            Some(Ok(files)) => {
                for file in files.into_iter().take(4096) {
                    let ext = std::path::Path::new(&file.name).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
                    let is_project = matches!(ext.as_str(), "fcproj" | "filmcraft");
                    if is_project || matches!(ext.as_str(), "mp4" | "mov" | "m2ts" | "wav" | "mp3" | "png" | "jpg") {
                        entries.push(DiscoveredEntry { name: file.name, size_bytes: file.size.unwrap_or(0), is_project, is_sample: false });
                    }
                }
                entries.sort_by_key(|e| (!e.is_project, e.name.to_lowercase()));
            }
            Some(Err(e)) => error = Some(e.to_string()),
            None => error = Some("Folder browsing is unavailable in this host. Use Open Project or Import instead.".into()),
        }
        app.hub_cache = Some(HubDirectoryCache { path: active_dir.clone(), scanned_at: web_time::Instant::now(), entries, error });
    }
    let Some(cache) = &app.hub_cache else { return };
    if let Some(error) = &cache.error {
        child.colored_label(t.danger, "Cannot access this folder");
        child.label(error);
        return;
    }
    if cache.entries.is_empty() {
        child.add_space(18.0);
        child.label("No projects or media in this folder yet.");
        child.label(egui::RichText::new("Save a project here or choose another folder.").color(t.text_dim));
        return;
    }
    let entries = cache.entries.clone();
    let mut open = None;
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(&mut child, |ui| {
        for (i, entry) in entries.iter().enumerate() {
            let row = ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(20.0, 30.0), Sense::hover());
                icons::paint(ui.painter(), r.shrink2(vec2(2.0, 7.0)), if entry.is_project { Icon::Sequence } else { Icon::Film }, t.icon);
                ui.add(egui::Label::new(&entry.name).truncate());
                let label = if entry.is_project { "Open" } else { "Import" };
                let action = ui.button(label);
                app.auto.add_response(&format!("hub.cloud.action.{i}"), &action, label);
                app.auto.add_response(&format!("hub.cloud.open.{i}"), &action, label);
                if action.clicked() {
                    open = Some(entry.clone());
                }
            });
            app.auto.add(&format!("hub.cloud.row.{i}"), row.response.rect, &entry.name);
            ui.separator();
        }
    });
    if let Some(entry) = open {
        let Some(path) = resolve_file_path(app, &active_dir, &entry.name) else {
            app.ui.status = format!("{} is not available locally. Refresh the folder.", entry.name);
            return;
        };
        let (command, params) = if entry.is_project { ("file.open", json!({"path": path})) } else { ("file.import", json!({"paths": [path]})) };
        match crate::menus::invoke(app, ui.ctx(), command, params) {
            Ok(_) => app.ui.mode = crate::state::Mode::Edit,
            Err(e) => app.ui.status = e,
        }
    }
}

fn link_local_folder(app: &mut FilmcraftApp) {
    let dialog = crate::native_dialogs::FileDialog::Folder { at: app.gdrive_config.active_folder_name.clone() };
    if !dialog.available(&app.hooks) {
        app.ui.status = "Folder linking is unavailable in this host. Use Open Project or Import.".into();
        return;
    }
    app.pick_ui(dialog, |app, paths| {
        if let Some(path) = paths.into_iter().next() {
            app.gdrive_config.active_folder_name = Some(path.clone());
            app.gdrive_local_path_input = path.clone();
            app.hub_cache = None;
            app.ui.status = format!("Linked local folder: {path}");
        }
    });
}
