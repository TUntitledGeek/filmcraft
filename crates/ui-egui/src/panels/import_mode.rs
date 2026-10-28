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

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().with_clip_rect(rect);

    // Deep panel surface
    p.rect_filled(rect, t.radius, t.panel_bg);

    // Header
    p.text(rect.min + vec2(28.0, 32.0), Align2::LEFT_CENTER, "Project Hub", Tokens::semibold(22.0), t.text);
    p.text(
        rect.min + vec2(28.0, 58.0),
        Align2::LEFT_CENTER,
        "Create a new project from presets, open recent work, or manage cloud-synced projects.",
        Tokens::ui(13.0),
        t.text_dim,
    );

    let ctx = ui.ctx().clone();

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

        // Cloud status badge (🟢 Synced vs 💾 Local)
        let is_bound_path = app.gdrive_config.active_folder_name.as_deref().map_or(false, |dir| path.starts_with(dir));
        let is_cloud = is_bound_path || path.contains("Google Drive") || path.contains("Cloud") || path.contains("gdrive");
        let badge_rect = Rect::from_min_size(pos2(row.max.x - 76.0, row.center().y - 10.0), vec2(68.0, 20.0));
        let (b_bg, b_fg, b_text) = if is_cloud {
            if app.gdrive_config.is_authenticated() || is_bound_path {
                (Color32::from_rgb(0x18, 0x38, 0x22), Color32::from_rgb(0x4a, 0xde, 0x80), "🟢 Synced")
            } else {
                (Color32::from_rgb(0x2d, 0x2e, 0x18), Color32::from_rgb(0xde, 0xcd, 0x4a), "☁ Cloud")
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

fn show_cloud_projects(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().with_clip_rect(rect);
    let squad = default_squad_folder();

    // Section title & Sync status
    let folder_title = if let Some(name) = &app.gdrive_config.active_folder_name {
        name.clone()
    } else if let Some(id) = &app.gdrive_config.active_folder_id {
        let short: String = id.chars().take(8).collect();
        format!("Google Drive ({short})")
    } else {
        "Google Drive & Cloud Storage".to_string()
    };
    p.text(rect.min + vec2(0.0, 12.0), Align2::LEFT_CENTER, &folder_title, Tokens::semibold(15.0), t.text);

    let (badge_bg, badge_fg, badge_text) = if app.gdrive_syncing {
        (Color32::from_rgb(0x18, 0x28, 0x48), Color32::from_rgb(0x60, 0xb0, 0xec), "● Syncing...")
    } else if app.gdrive_config.is_authenticated() {
        (Color32::from_rgb(0x18, 0x48, 0x28), Color32::from_rgb(0x60, 0xec, 0x90), "● Connected")
    } else if app.gdrive_config.active_folder_id.is_some() || app.gdrive_config.active_folder_name.is_some() {
        (Color32::from_rgb(0x38, 0x30, 0x18), Color32::from_rgb(0xec, 0xc0, 0x60), "● Directory Linked")
    } else {
        (Color32::from_rgb(0x28, 0x2b, 0x35), Color32::from_rgb(0x94, 0xa3, 0xb8), "○ Not Connected")
    };
    let status_badge = Rect::from_min_size(rect.min + vec2(240.0, 2.0), vec2(150.0, 22.0));
    p.rect_filled(status_badge, 11.0, badge_bg);
    p.text(status_badge.center(), Align2::CENTER_CENTER, badge_text, Tokens::ui(10.5), badge_fg);

    // Shared projects container
    let box_r = Rect::from_min_size(rect.min + vec2(0.0, 32.0), vec2(rect.width(), 240.0));
    p.rect_filled(box_r, 8.0, t.tl_header_bg);
    p.rect_stroke(box_r, 8.0, egui::Stroke::new(1.0, t.separator), egui::StrokeKind::Inside);

    let mut y = box_r.min.y + 8.0;
    for (idx, file) in squad.project_files.iter().enumerate() {
        let row = Rect::from_min_size(pos2(box_r.min.x + 8.0, y), vec2(box_r.width() - 16.0, 48.0));
        let resp = ui.interact(row, egui::Id::new(("cloud-file", idx)), Sense::click());

        if resp.hovered() {
            p.rect_filled(row, 6.0, t.hover);
        }

        // Icon
        let is_proj = file.name.ends_with(".fcproj") || file.name.ends_with(".filmcraft");
        let icon = if is_proj { Icon::Sequence } else { Icon::Film };
        icons::paint(&p, Rect::from_center_size(pos2(row.min.x + 20.0, row.center().y), vec2(18.0, 18.0)), icon, Color32::from_rgb(0x3a, 0x8a, 0xec));

        // Name & size info
        p.text(pos2(row.min.x + 42.0, row.min.y + 15.0), Align2::LEFT_CENTER, &file.name, Tokens::semibold(12.5), t.text);
        let kind_str = if is_proj { "Project File" } else { "Media Asset" };
        let meta_str = format!("{:.1} MB · {}", file.size_bytes as f64 / 1_000_000.0, kind_str);
        p.text(pos2(row.min.x + 42.0, row.min.y + 32.0), Align2::LEFT_CENTER, meta_str, Tokens::ui(10.5), t.text_dim);

        // Action button
        let btn_label = if is_proj { "Open" } else { "Import" };
        let btn_w = 75.0;
        let btn_r = Rect::from_min_size(pos2(row.max.x - btn_w - 8.0, row.center().y - 12.0), vec2(btn_w, 24.0));
        let bresp = ui.interact(btn_r, egui::Id::new(("open-cloud-file", idx)), Sense::click());
        let bcolor = if bresp.hovered() { t.accent_hover } else { t.accent };
        p.rect_filled(btn_r, 4.0, bcolor);
        p.text(btn_r.center(), Align2::CENTER_CENTER, btn_label, Tokens::semibold(11.0), Color32::WHITE);

        if bresp.clicked() || resp.clicked() {
            let ctx = ui.ctx().clone();
            let bound_candidate = app.gdrive_config.active_folder_name.as_deref().unwrap_or("").trim();
            let input_candidate = app.gdrive_local_path_input.trim();
            let candidate_dir = if !bound_candidate.is_empty() {
                bound_candidate
            } else {
                input_candidate
            };

            if is_proj {
                let candidate_path = std::path::Path::new(candidate_dir).join(&file.name);
                if !candidate_dir.is_empty() && candidate_path.exists() {
                    match crate::menus::invoke(app, &ctx, "file.open", json!({"path": candidate_path.to_string_lossy()})) {
                        Ok(_) => {
                            app.ui.mode = crate::state::Mode::Edit;
                            app.ui.status = format!("Opened project: {}", file.name);
                        }
                        Err(e) => {
                            app.ui.status = format!("Failed to open {}: {e}", file.name);
                        }
                    }
                } else if std::path::Path::new(&file.name).exists() {
                    match crate::menus::invoke(app, &ctx, "file.open", json!({"path": file.name})) {
                        Ok(_) => {
                            app.ui.mode = crate::state::Mode::Edit;
                            app.ui.status = format!("Opened project: {}", file.name);
                        }
                        Err(e) => {
                            app.ui.status = format!("Failed to open {}: {e}", file.name);
                        }
                    }
                } else {
                    app.ui.status = format!("Cannot open cloud project '{}': file is not cached locally. Bind a local directory or sync to open.", file.name);
                }
            } else {
                let candidate_path = std::path::Path::new(candidate_dir).join(&file.name);
                let target_path = if !candidate_dir.is_empty() && candidate_path.exists() {
                    Some(candidate_path.to_string_lossy().to_string())
                } else if std::path::Path::new(&file.name).exists() {
                    Some(file.name.clone())
                } else {
                    None
                };

                if let Some(path) = target_path {
                    match crate::menus::invoke(app, &ctx, "file.import", json!({"paths": [path]})) {
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
