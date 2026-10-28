//! Project Hub (Header "Project Hub", also Home).
//!
//! Provides project templates (YouTube 4K60, YouTube 1080p, Shorts/TikTok 9:16, Podcast Cam),
//! recent project timelines with cloud status badges, and Google Drive cloud-synced / shared projects.

use egui::{Align2, Color32, Rect, Sense, pos2, vec2};
use filmcraft_engine::cloud_sync::{default_squad_folder, generate_share_url, ShareParams};
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
    p.text(
        rect.min + vec2(28.0, 58.0),
        Align2::LEFT_CENTER,
        "Create high-performance projects, manage synced directories, and launch studio templates.",
        Tokens::ui(13.0),
        t.text_dim,
    );

    // Quick Action Header Buttons
    let mut btn_x = rect.max.x - 28.0;

    // Button 3: Link Folder
    let link_w = 120.0;
    btn_x -= link_w;
    let link_r = Rect::from_min_size(pos2(btn_x, rect.min.y + 26.0), vec2(link_w, 32.0));
    let lresp = ui.interact(link_r, egui::Id::new("hub-hdr-link-folder"), Sense::click());
    app.auto.add("hub.action.link_folder", link_r, "Link Folder");
    let lbg = if lresp.hovered() { t.hover } else { t.tl_header_bg };
    p.rect_filled(link_r, 6.0, lbg);
    p.rect_stroke(link_r, 6.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    p.text(link_r.center(), Align2::CENTER_CENTER, "📁 Link Folder", Tokens::semibold(12.0), t.text);
    if lresp.clicked() {
        app.collab_dialog_open = true;
    }

    // Button 2: Load Demo
    let demo_w = 140.0;
    btn_x -= demo_w + 10.0;
    let demo_r = Rect::from_min_size(pos2(btn_x, rect.min.y + 26.0), vec2(demo_w, 32.0));
    let dresp = ui.interact(demo_r, egui::Id::new("hub-hdr-load-demo"), Sense::click());
    app.auto.add("hub.action.load_demo", demo_r, "Load Demo Edit");
    let dbg = if dresp.hovered() { t.accent_hover } else { t.accent };
    p.rect_filled(demo_r, 6.0, dbg);
    p.text(demo_r.center(), Align2::CENTER_CENTER, "🎬 Load Demo Edit", Tokens::semibold(12.0), Color32::WHITE);
    if dresp.clicked() {
        if crate::menus::invoke(app, &ctx, "file.openDemoProject", json!({})).is_ok() {
            app.ui.mode = crate::state::Mode::Edit;
            app.ui.status = "Opened FilmCraft Studio Pro Demo Project.".into();
        }
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
    if nresp.clicked() {
        if crate::menus::invoke(app, &ctx, "file.newProject", json!({"name": "Untitled Studio Project"})).is_ok() {
            let _ = crate::menus::invoke(app, &ctx, "file.newSequence", json!({
                "name": "Sequence 01",
                "width": 1920,
                "height": 1080,
                "fps": 23.976
            }));
            app.ui.mode = crate::state::Mode::Edit;
            app.ui.status = "Created new project.".into();
        }
    }

    // SECTION 1: New Project Creator Presets
    let presets: [(&str, u32, u32, f64, &str, Icon, Color32); 4] = [
        ("YouTube 4K60", 3840, 2160, 60.0, "3840 × 2160 @ 60fps · 16:9 Cinematic Pro", Icon::Film, Color32::from_rgb(0xff, 0x4e, 0x4e)),
        ("YouTube 1080p", 1920, 1080, 60.0, "1920 × 1080 @ 60fps · High Efficiency", Icon::Sequence, Color32::from_rgb(0x3a, 0x8a, 0xec)),
        ("Shorts/TikTok 9:16", 1080, 1920, 60.0, "1080 × 1920 @ 60fps · Vertical Reel", Icon::Film, Color32::from_rgb(0xec, 0x3a, 0x8a)),
        ("Podcast Cam", 1920, 1080, 30.0, "1920 × 1080 @ 30fps · Multi-Cam Studio", Icon::Folder, Color32::from_rgb(0x3a, 0xcc, 0x8a)),
    ];

    let cw = ((rect.width() - 56.0 - 3.0 * 16.0) / 4.0).clamp(180.0, 280.0);
    for (i, (title, width, height, fps, sub, icon, accent_col)) in presets.iter().enumerate() {
        let r = Rect::from_min_size(rect.min + vec2(28.0 + i as f32 * (cw + 16.0), 88.0), vec2(cw, 120.0));
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
                let _ = crate::menus::invoke(app, &ctx, "file.newSequence", json!({
                    "name": format!("{title} Sequence"),
                    "width": *width,
                    "height": *height,
                    "fps": *fps
                }));
            }
            app.ui.mode = crate::state::Mode::Edit;
        }
    }

    let y_split = 88.0 + 120.0 + 32.0;
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
        let is_bound_path = app.gdrive_config.active_folder_name.as_deref().map_or(false, |dir| path.starts_with(dir));
        let badge_rect = Rect::from_min_size(pos2(row.max.x - 76.0, row.center().y - 10.0), vec2(68.0, 20.0));
        let (b_bg, b_fg, b_text) = if is_bound_path {
            if app.gdrive_last_synced.is_some() {
                (Color32::from_rgb(0x18, 0x38, 0x22), Color32::from_rgb(0x4a, 0xde, 0x80), "🟢 Synced")
            } else {
                (Color32::from_rgb(0x18, 0x28, 0x48), Color32::from_rgb(0x60, 0xb0, 0xec), "📁 Linked")
            }
        } else {
            (Color32::from_rgb(0x25, 0x28, 0x34), Color32::from_rgb(0x94, 0xa3, 0xb8), "💾 Local")
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
    let p = ui.painter().with_clip_rect(rect);

    let candidate_dir = app.gdrive_config.active_folder_name.as_deref().unwrap_or("").trim().to_string();
    let local_input = app.gdrive_local_path_input.trim().to_string();
    let active_dir = if !candidate_dir.is_empty() { candidate_dir } else { local_input };

    // Section title & refresh button
    let folder_title = if !active_dir.is_empty() {
        active_dir.clone()
    } else if let Some(id) = &app.gdrive_config.active_folder_id {
        let short: String = id.chars().take(8).collect();
        format!("Google Drive ({short})")
    } else {
        "Sample Projects & Templates".to_string()
    };
    p.text(rect.min + vec2(0.0, 12.0), Align2::LEFT_CENTER, &folder_title, Tokens::semibold(15.0), t.text);

    let refresh_w = 72.0;
    let refresh_r = Rect::from_min_size(pos2(rect.max.x - refresh_w, rect.min.y + 2.0), vec2(refresh_w, 22.0));
    let refresh_resp = ui.interact(refresh_r, egui::Id::new("hub-cloud-refresh"), Sense::click());
    app.auto.add("hub.cloud.refresh", refresh_r, "Refresh Directory");
    let rbg = if refresh_resp.hovered() { t.hover } else { t.tl_header_bg };
    p.rect_filled(refresh_r, 4.0, rbg);
    p.rect_stroke(refresh_r, 4.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
    p.text(refresh_r.center(), Align2::CENTER_CENTER, "⟳ Refresh", Tokens::ui(10.5), t.text);

    let force_refresh = refresh_resp.clicked();
    let cache_expired = app.hub_cache.as_ref().map_or(true, |c| {
        c.path != active_dir || c.scanned_at.elapsed() > std::time::Duration::from_secs(5)
    });

    if force_refresh || cache_expired {
        let mut discovered = Vec::new();
        let mut err = None;

        if !active_dir.is_empty() {
            match app.session.services.list_entries(&active_dir) {
                Some(Ok(entries)) => {
                    for entry in entries {
                        let name = entry.name;
                        let is_proj = name.ends_with(".fcproj") || name.ends_with(".filmcraft");
                        let is_media = name.ends_with(".mp4") || name.ends_with(".mov") || name.ends_with(".m2ts")
                            || name.ends_with(".wav") || name.ends_with(".mp3") || name.ends_with(".png") || name.ends_with(".jpg");
                        if is_proj || is_media {
                            let sz = if let Some(s) = entry.size {
                                s
                            } else {
                                let full = if active_dir.ends_with('/') || active_dir.ends_with('\\') {
                                    format!("{active_dir}{name}")
                                } else {
                                    format!("{active_dir}/{name}")
                                };
                                app.session.services.file_size(&full).unwrap_or(0)
                            };
                            discovered.push(DiscoveredEntry {
                                name,
                                size_bytes: sz,
                                is_project: is_proj,
                                is_sample: false,
                            });
                        }
                    }
                }
                Some(Err(e)) => {
                    err = Some(e.to_string());
                }
                None => {
                    err = Some("Directory listing is not supported on this platform".to_string());
                }
            }
        }

        app.hub_cache = Some(HubDirectoryCache {
            path: active_dir.clone(),
            scanned_at: web_time::Instant::now(),
            entries: discovered,
            error: err,
        });
    }

    let cache = app.hub_cache.as_ref().unwrap();
    let has_error = cache.error.is_some();
    let is_dir_empty = !active_dir.is_empty() && !has_error && cache.entries.is_empty();
    let is_sample_catalog = active_dir.is_empty();

    let (badge_bg, badge_fg, badge_text) = if has_error {
        (Color32::from_rgb(0x4a, 0x18, 0x18), Color32::from_rgb(0xf8, 0x71, 0x71), "⚠️ Inaccessible")
    } else if is_dir_empty {
        (Color32::from_rgb(0x38, 0x30, 0x18), Color32::from_rgb(0xec, 0xc0, 0x60), "📁 Empty Folder")
    } else if is_sample_catalog && !app.gdrive_config.is_authenticated() {
        (Color32::from_rgb(0x28, 0x2b, 0x35), Color32::from_rgb(0x94, 0xa3, 0xb8), "○ Sample Catalog")
    } else if app.gdrive_syncing {
        (Color32::from_rgb(0x18, 0x28, 0x48), Color32::from_rgb(0x60, 0xb0, 0xec), "● Syncing...")
    } else if app.gdrive_last_synced.is_some() {
        (Color32::from_rgb(0x18, 0x48, 0x28), Color32::from_rgb(0x60, 0xec, 0x90), "● Remote Synced")
    } else if app.gdrive_config.is_authenticated() {
        (Color32::from_rgb(0x18, 0x48, 0x28), Color32::from_rgb(0x60, 0xec, 0x90), "● Connected")
    } else if !active_dir.is_empty() {
        (Color32::from_rgb(0x18, 0x28, 0x48), Color32::from_rgb(0x60, 0xb0, 0xec), "● Directory Linked")
    } else {
        (Color32::from_rgb(0x28, 0x2b, 0x35), Color32::from_rgb(0x94, 0xa3, 0xb8), "○ Not Connected")
    };

    let status_badge = Rect::from_min_size(rect.min + vec2(220.0, 2.0), vec2(130.0, 22.0));
    p.rect_filled(status_badge, 11.0, badge_bg);
    p.text(status_badge.center(), Align2::CENTER_CENTER, badge_text, Tokens::ui(10.5), badge_fg);

    // Shared projects container
    let box_r = Rect::from_min_size(rect.min + vec2(0.0, 32.0), vec2(rect.width(), 240.0));
    p.rect_filled(box_r, 8.0, t.tl_header_bg);
    p.rect_stroke(box_r, 8.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);

    if let Some(err_msg) = &cache.error {
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 40.0), Align2::LEFT_CENTER, "⚠️ Cannot Access Directory", Tokens::semibold(14.0), Color32::from_rgb(0xf8, 0x71, 0x71));
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 70.0), Align2::LEFT_CENTER, format!("Path: {active_dir}"), Tokens::mono(11.5), t.text);
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 96.0), Align2::LEFT_CENTER, format!("Error: {err_msg}"), Tokens::ui(11.0), t.text_dim);
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 124.0), Align2::LEFT_CENTER, "Please verify folder permissions or path, or unlink this directory in the Collab panel.", Tokens::ui(11.0), t.text_dim);

        let unlink_r = Rect::from_min_size(pos2(box_r.min.x + 24.0, box_r.min.y + 160.0), vec2(140.0, 28.0));
        let unresp = ui.interact(unlink_r, egui::Id::new("unlink-inaccessible-dir"), Sense::click());
        let ubg = if unresp.hovered() { t.hover } else { t.panel_bg };
        p.rect_filled(unlink_r, 4.0, ubg);
        p.rect_stroke(unlink_r, 4.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
        p.text(unlink_r.center(), Align2::CENTER_CENTER, "Unlink Directory", Tokens::ui(11.0), t.text);
        if unresp.clicked() {
            app.gdrive_config.active_folder_name = None;
            app.gdrive_local_path_input.clear();
            app.hub_cache = None;
            app.ui.status = "Unlinked directory path.".into();
        }
    } else if is_dir_empty {
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 40.0), Align2::LEFT_CENTER, "📁 Directory is Empty", Tokens::semibold(14.0), Color32::from_rgb(0xec, 0xc0, 0x60));
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 70.0), Align2::LEFT_CENTER, "No .fcproj, .filmcraft, or supported media files found in:", Tokens::ui(12.0), t.text);
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 96.0), Align2::LEFT_CENTER, active_dir.clone(), Tokens::mono(11.5), t.text_dim);
        p.text(pos2(box_r.min.x + 24.0, box_r.min.y + 124.0), Align2::LEFT_CENTER, "Save your active project here or place video/audio assets into this directory to access them.", Tokens::ui(11.0), t.text_dim);

        let unlink_r = Rect::from_min_size(pos2(box_r.min.x + 24.0, box_r.min.y + 160.0), vec2(140.0, 28.0));
        let unresp = ui.interact(unlink_r, egui::Id::new("unlink-empty-dir"), Sense::click());
        let ubg = if unresp.hovered() { t.hover } else { t.panel_bg };
        p.rect_filled(unlink_r, 4.0, ubg);
        p.rect_stroke(unlink_r, 4.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);
        p.text(unlink_r.center(), Align2::CENTER_CENTER, "Unlink Directory", Tokens::ui(11.0), t.text);
        if unresp.clicked() {
            app.gdrive_config.active_folder_name = None;
            app.gdrive_local_path_input.clear();
            app.hub_cache = None;
            app.ui.status = "Unlinked directory path.".into();
        }
    } else {
        let entries: Vec<DiscoveredEntry> = if is_sample_catalog {
            let squad = default_squad_folder();
            squad.project_files.into_iter().map(|f| {
                let is_proj = f.name.ends_with(".fcproj") || f.name.ends_with(".filmcraft");
                DiscoveredEntry {
                    name: f.name,
                    size_bytes: f.size_bytes,
                    is_project: is_proj,
                    is_sample: true,
                }
            }).collect()
        } else {
            cache.entries.clone()
        };

        let mut y = box_r.min.y + 8.0;
        for (idx, file) in entries.iter().enumerate() {
            let row = Rect::from_min_size(pos2(box_r.min.x + 8.0, y), vec2(box_r.width() - 16.0, 48.0));
            let resp = ui.interact(row, egui::Id::new(("cloud-file", idx)), Sense::click());
            app.auto.add(&format!("hub.cloud.row.{idx}"), row, &file.name);

            if resp.hovered() {
                p.rect_filled(row, 6.0, t.hover);
            }

            // Icon
            let icon = if file.is_project { Icon::Sequence } else { Icon::Film };
            icons::paint(&p, Rect::from_center_size(pos2(row.min.x + 20.0, row.center().y), vec2(18.0, 18.0)), icon, Color32::from_rgb(0x3a, 0x8a, 0xec));

            // Name & size info
            p.text(pos2(row.min.x + 42.0, row.min.y + 15.0), Align2::LEFT_CENTER, &file.name, Tokens::semibold(12.5), t.text);
            let kind_str = if file.is_sample {
                if file.is_project { "Sample Project Template (Demo)" } else { "Sample Media Asset (Demo)" }
            } else if app.gdrive_last_synced.is_some() {
                if file.is_project { "Cloud Synced Project" } else { "Cloud Synced Media" }
            } else {
                if file.is_project { "Linked Directory Project" } else { "Linked Directory Media" }
            };
            let meta_str = format!("{:.1} MB · {}", file.size_bytes as f64 / 1_000_000.0, kind_str);
            p.text(pos2(row.min.x + 42.0, row.min.y + 32.0), Align2::LEFT_CENTER, meta_str, Tokens::ui(10.5), t.text_dim);

            // Action button
            let btn_label = if file.is_project { "Open" } else { "Import" };
            let btn_w = 75.0;
            let btn_r = Rect::from_min_size(pos2(row.max.x - btn_w - 8.0, row.center().y - 12.0), vec2(btn_w, 24.0));
            let bresp = ui.interact(btn_r, egui::Id::new(("open-cloud-file", idx)), Sense::click());
            app.auto.add(&format!("hub.cloud.action.{idx}"), btn_r, btn_label);
            app.auto.add(&format!("hub.cloud.open.{idx}"), btn_r, btn_label);

            let bcolor = if bresp.hovered() { t.accent_hover } else { t.accent };
            p.rect_filled(btn_r, 4.0, bcolor);
            p.text(btn_r.center(), Align2::CENTER_CENTER, btn_label, Tokens::semibold(11.0), Color32::WHITE);

            if bresp.clicked() || resp.clicked() {
                let ctx = ui.ctx().clone();
                if file.is_project {
                    if let Some(target_path) = resolve_file_path(app, &active_dir, &file.name) {
                        match crate::menus::invoke(app, &ctx, "file.open", json!({"path": target_path})) {
                            Ok(_) => {
                                app.ui.mode = crate::state::Mode::Edit;
                                app.ui.status = format!("Opened project: {}", file.name);
                            }
                            Err(e) => {
                                app.ui.status = format!("Failed to open {}: {e}", file.name);
                            }
                        }
                    } else {
                        app.ui.status = format!("Cannot open project '{}': file is not available locally. Bind a directory or sync to open.", file.name);
                    }
                } else {
                    if let Some(target_path) = resolve_file_path(app, &active_dir, &file.name) {
                        match crate::menus::invoke(app, &ctx, "file.import", json!({"paths": [target_path]})) {
                            Ok(_) => {
                                app.ui.mode = crate::state::Mode::Edit;
                                app.ui.status = format!("Imported media into project: {}", file.name);
                            }
                            Err(e) => {
                                app.ui.status = format!("Failed to import {}: {e}", file.name);
                            }
                        }
                    } else {
                        app.ui.status = format!("Cannot import media '{}': file is not available locally.", file.name);
                    }
                }
            }

            y += 54.0;
        }
    }

    // 1-Click Share Link button
    let link_btn_r = Rect::from_min_size(pos2(rect.min.x, box_r.max.y + 12.0), vec2(190.0, 32.0));
    let link_resp = ui.interact(link_btn_r, egui::Id::new("copy-coop-link"), Sense::click()).on_hover_text("Copy 1-click URL to invite collaborators into this project");
    let lbg = if link_resp.hovered() { t.accent_hover } else { t.accent };
    p.rect_filled(link_btn_r, 6.0, lbg);
    icons::paint(&p, Rect::from_center_size(pos2(link_btn_r.min.x + 18.0, link_btn_r.center().y), vec2(16.0, 16.0)), Icon::Globe, Color32::WHITE);
    p.text(link_btn_r.min + vec2(36.0, 16.0), Align2::LEFT_CENTER, "Copy 1-Click Share Link", Tokens::semibold(11.5), Color32::WHITE);

    if link_resp.clicked() {
        let room_id = if app.collab_room_input.trim().is_empty() { "FC-LIVE".to_string() } else { app.collab_room_input.trim().to_string() };
        let host_name = if app.collab_name_input.trim().is_empty() { "Editor".to_string() } else { app.collab_name_input.trim().to_string() };
        let squad = default_squad_folder();
        let params = ShareParams {
            room_id,
            drive_id: squad.folder_id,
            project_name: app.session.project.name.clone(),
            host_name,
        };
        let share_url = generate_share_url(crate::panels::collab_panel::get_base_invite_url().as_str(), &params);
        ui.ctx().copy_text(share_url);
        app.ui.status = "Copied 1-Click Share Link to clipboard!".into();
    }
}
