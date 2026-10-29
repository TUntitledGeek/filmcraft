//! Responsive 3-Zone Header Bar (Premiere 26 layout, 38 pt):
//! - Zone Left: Home icon + in-window menus (`File`, `Edit`, `Clip`, etc.), auto-collapsing to `[ ☰ Menu ▾ ]` if width < 1400px.
//! - Zone Center: Centered Mode Switcher Pills (`[◫ Media Hub]`, `[▶ Edit Timeline]`, `[☊ Fusion Nodes]`, `[🎨 Color]`, `[⇗ Export]`).
//! - Zone Right: Combined Project Title & Google Drive capsule, Workspaces dropdown, Quick Export, Appearance toggle, Fullscreen toggle.

use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

use crate::FilmcraftApp;
use crate::icons::{self, Icon};
use crate::state::Mode;
use crate::theme::Tokens;

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, t.header_bg);
    p.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.separator));
    let drag = ui.interact(rect, egui::Id::new("header-drag"), Sense::click_and_drag());
    if drag.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
    }

    let total_w = rect.width();

    // =========================================================================
    // ZONE LEFT BOUNDS & MENU SETUP
    // =========================================================================
    let left_start_x = rect.min.x + if app.integrated_titlebar { 104.0 } else { 14.0 };
    let home = Rect::from_center_size(pos2(left_start_x + 13.0, rect.center().y), vec2(26.0, 26.0));
    let hresp = ui.interact(home, egui::Id::new("hdr-home"), Sense::click()).on_hover_text(format!("Project Hub ({})", tl!("Home")));
    app.auto.add_response("header.home", &hresp, "Project Hub");
    if hresp.hovered() {
        p.rect_filled(home, 4.0, t.hover);
    }
    icons::paint(&p, home.shrink(5.0), Icon::Home, if hresp.hovered() { t.text } else { t.text_dim });
    crate::widgets::paint_focus(&p, &hresp, &t);
    if hresp.clicked() {
        app.ui.mode = Mode::Import;
    }

    let menus_start_x = home.max.x + 10.0;
    let min_center_start_x =
        if app.ui.show_menu_bar { if total_w < 1400.0 { menus_start_x + 84.0 + 12.0 } else { menus_start_x + 460.0 } } else { home.max.x + 14.0 };

    // =========================================================================
    // ZONE RIGHT FIXED CONTROLS WIDTH ESTIMATION
    // =========================================================================
    // Right buttons: Fullscreen (36) + Appearance (36) + Quick Export (36) + Workspaces (36) = 144
    let ws_str = match app.ui.mode {
        Mode::Import => "Project Hub".to_string(),
        Mode::Export => "Export".to_string(),
        Mode::Edit if matches!(app.ui.workspace.as_str(), "Fusion" | "Fusion Nodes") => crate::i18n::t("Fusion (Experimental)").into(),
        Mode::Edit => crate::i18n::t(&app.ui.workspace).to_owned(),
    };
    let show_ws_name = total_w >= 1100.0;
    let ws_text_w = if show_ws_name {
        let wg = p.layout_no_wrap(ws_str.clone(), Tokens::semibold(11.0), t.text_dim);
        wg.size().x + 28.0
    } else {
        0.0
    };
    let right_fixed_w = (if show_ws_name { 108.0 } else { 144.0 }) + ws_text_w + 14.0;
    let right_fixed_start_x = rect.max.x - right_fixed_w;

    // =========================================================================
    // ZONE CENTER: Responsive Centered Mode Switcher Pills
    // =========================================================================
    // Dynamically choose label density based on available screen width
    let mode_tabs: Vec<(Mode, &'static str, &str, Option<&'static str>, Color32)> = if total_w >= 1200.0 {
        vec![
            (Mode::Import, "import", "Project Hub", None, t.accent),
            (Mode::Edit, "edit", "Edit", Some("Clean Editing"), t.accent),
            (Mode::Edit, "color", "Color", Some("Color"), t.accent),
            (Mode::Export, "export", "Export", None, t.accent),
        ]
    } else if total_w >= 900.0 {
        vec![
            (Mode::Import, "import", "Hub", None, t.accent),
            (Mode::Edit, "edit", "Edit", Some("Clean Editing"), t.accent),
            (Mode::Edit, "color", "Color", Some("Color"), t.accent),
            (Mode::Export, "export", "Export", None, t.accent),
        ]
    } else {
        vec![
            (Mode::Import, "import", "", None, t.accent),
            (Mode::Edit, "edit", "", Some("Clean Editing"), t.accent),
            (Mode::Edit, "color", "", Some("Color"), t.accent),
            (Mode::Export, "export", "", None, t.accent),
        ]
    };

    let pill_widths: Vec<f32> = mode_tabs
        .iter()
        .map(|(_, _, label, _, _)| {
            let galley = p.layout_no_wrap(label.to_string(), Tokens::ui(12.5), Color32::WHITE);
            galley.size().x + if total_w >= 900.0 { 42.0 } else { 32.0 }
        })
        .collect();
    let center_spacing = if total_w >= 900.0 { 6.0 } else { 4.0 };
    let center_total_w: f32 = pill_widths.iter().sum::<f32>() + (mode_tabs.len() as f32 - 1.0) * center_spacing;

    let ideal_center_start = rect.center().x - center_total_w / 2.0;
    let mut center_start_x = ideal_center_start.max(min_center_start_x);
    let mut center_end_x = center_start_x + center_total_w;

    // Shift left if encroaching on right controls
    if center_end_x > right_fixed_start_x - 10.0 {
        center_start_x = (right_fixed_start_x - 10.0 - center_total_w).max(min_center_start_x);
        center_end_x = center_start_x + center_total_w;
    }

    // Render in-window menus in Zone Left (clipped cleanly before center pills)
    let left_avail_w = (center_start_x - 12.0 - menus_start_x).max(0.0);
    if app.ui.show_menu_bar && left_avail_w > 40.0 {
        if total_w < 1400.0 || left_avail_w < 440.0 {
            let bw = 84.0f32.min(left_avail_w);
            let btn_rect = Rect::from_min_size(pos2(menus_start_x, rect.min.y + 7.0), vec2(bw, rect.height() - 14.0));
            let mut mu = ui.new_child(egui::UiBuilder::new().max_rect(btn_rect));
            mu.style_mut().visuals.widgets.inactive.weak_bg_fill = t.control_surface;
            crate::menus::collapsed_menu_button(app, &mut mu);
        } else {
            let menu_rect = Rect::from_min_max(pos2(menus_start_x, rect.min.y + 7.0), pos2(center_start_x - 12.0, rect.max.y - 7.0));
            let mut mu = ui.new_child(egui::UiBuilder::new().max_rect(menu_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
            mu.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            mu.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
            mu.style_mut().visuals.override_text_color = Some(t.text_dim);
            crate::menus::menu_bar(app, &mut mu);
        }
    }

    // Draw the mode tabs in Zone Center
    let mut cx = center_start_x;
    for (i, (m, id_tag, label_text, target_ws, _underline_col)) in mode_tabs.into_iter().enumerate() {
        let is_active = match (m, target_ws) {
            (Mode::Import, _) => app.ui.mode == Mode::Import,
            (Mode::Export, _) => app.ui.mode == Mode::Export,
            (Mode::Edit, Some("Color")) => app.ui.mode == Mode::Edit && app.ui.workspace == "Color",
            (Mode::Edit, _) => app.ui.mode == Mode::Edit && app.ui.workspace != "Color",
        };
        let pw = pill_widths[i];
        let r = Rect::from_min_size(pos2(cx, rect.min.y + 5.0), vec2(pw, rect.height() - 10.0));
        let resp = ui.interact(r, egui::Id::new(("studio-hdr-mode", id_tag)), Sense::click());
        app.auto.add_response(&format!("header.mode.{}", id_tag), &resp, label_text);

        let pill_radius = r.height() / 2.0;
        if is_active {
            p.rect_filled(r, pill_radius, t.pill_active_bg);
            if t.kind == crate::theme::ThemeKind::Light {
                p.rect_stroke(r, pill_radius, Stroke::new(1.0, t.accent.gamma_multiply(0.25)), StrokeKind::Inside);
            } else {
                p.rect_stroke(r, pill_radius, Stroke::new(1.0, t.separator), StrokeKind::Inside);
            }
        } else if resp.hovered() {
            p.rect_filled(r, pill_radius, t.hover);
        }

        let col = if is_active {
            if t.kind == crate::theme::ThemeKind::Light { t.accent } else { t.tab_text_active }
        } else if resp.hovered() {
            t.text
        } else {
            t.tab_text
        };
        let icon = match id_tag {
            "import" => Icon::Home,
            "fusion" => Icon::Freeform,
            "color" => Icon::Adjust,
            "export" => Icon::Export,
            _ => Icon::Sequence,
        };
        icons::paint(&p, Rect::from_center_size(pos2(r.min.x + 17.0, r.center().y), vec2(16.0, 16.0)), icon, col);
        p.text(pos2(r.min.x + 31.0, r.center().y), Align2::LEFT_CENTER, label_text, Tokens::ui(12.5), col);
        crate::widgets::paint_focus(&p, &resp, &t);

        if resp.clicked() {
            app.ui.mode = m;
            if let Some(ws) = target_ws
                && app.ui.workspace != ws
            {
                app.set_workspace(ws);
            }
        }
        cx = r.max.x + center_spacing;
    }

    // =========================================================================
    // ZONE RIGHT: Capsule, Workspaces, Quick Export, Appearance, Fullscreen
    // =========================================================================
    let mut rx = rect.max.x - 14.0;
    let btn = |ui: &mut egui::Ui, icon: Icon, id: &str, tip: &str, app: &mut FilmcraftApp, rx_pos: &mut f32| -> egui::Response {
        let r = Rect::from_center_size(pos2(*rx_pos - 14.0, rect.center().y), vec2(28.0, 28.0));
        *rx_pos -= 36.0;
        let resp = ui.interact(r, egui::Id::new(("hdr", id)), Sense::click()).on_hover_text(tip);
        app.auto.add_response(&format!("header.{id}"), &resp, tip);
        if resp.hovered() {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        icons::paint(ui.painter(), r.shrink(6.0), icon, if resp.hovered() { t.text } else { t.text_dim });
        crate::widgets::paint_focus(ui.painter(), &resp, &t);
        resp
    };

    // Fullscreen toggle
    if btn(ui, Icon::Fullscreen, "fullscreen", tl!("Full screen"), app, &mut rx).clicked() {
        let fs = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fs));
    }

    // Appearance toggle
    let (icon, mode) = match app.session.prefs.appearance.appearance_mode.as_str() {
        "auto" => (Icon::Monitor, tl!("Sync with system")),
        "light" => (Icon::Sun, tl!("Light")),
        _ => (Icon::Moon, tl!("Dark")),
    };
    let tip = format!("{}: {mode}", tl!("Appearance Mode"));
    let appearance = btn(ui, icon, "appearance", &tip, app, &mut rx);
    appearance.context_menu(|ui| crate::panels::settings::appearance_context_menu(app, ui));
    if appearance.clicked() {
        crate::panels::settings::cycle_appearance(app, ui.ctx());
    }

    // Quick Export
    let qx = rx - 14.0;
    if btn(ui, Icon::Export, "quickExport", tl!("Quick Export"), app, &mut rx).clicked() {
        app.ui.export.quick_open = !app.ui.export.quick_open;
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("quick-export-toggled"), true));
    }
    crate::panels::export_mode::quick_export(app, ui.ctx(), pos2(qx - 340.0, rect.max.y + 4.0));

    // Workspace switcher button & label
    let mut wr = Rect::ZERO;
    let ws_resp = if show_ws_name {
        let wg = p.layout_no_wrap(ws_str.clone(), Tokens::semibold(11.0), if t.kind == crate::theme::ThemeKind::Light { t.text } else { t.tab_text_active });
        let pill_w = wg.size().x + 28.0;
        wr = Rect::from_min_size(pos2(rx - pill_w + 4.0, rect.center().y - 12.0), vec2(pill_w, 24.0));
        rx = wr.min.x - 10.0;
        let wresp = ui.interact(wr, egui::Id::new("hdr-ws-name"), Sense::click());
        app.auto.add_response("header.workspaceName", &wresp, &ws_str);
        app.auto.add("header.workspaces", wr, tl!("Workspaces"));
        let bg = if wresp.hovered() { t.hover } else { t.field_bg };
        p.rect_filled(wr, 12.0, bg);
        p.rect_stroke(wr, 12.0, Stroke::new(1.0, t.separator), StrokeKind::Inside);
        p.galley_with_override_text_color(pos2(wr.min.x + 9.0, rect.center().y - wg.size().y / 2.0), wg, if wresp.hovered() { t.accent } else { t.text });
        icons::paint(
            &p,
            Rect::from_center_size(pos2(wr.max.x - 10.0, rect.center().y), vec2(8.0, 8.0)),
            Icon::ChevronDown,
            if wresp.hovered() { t.accent } else { t.text_dim },
        );
        crate::widgets::paint_focus(&p, &wresp, &t);
        wresp
    } else {
        btn(ui, Icon::Workspaces, "workspaces", tl!("Workspaces"), app, &mut rx)
    };

    // Interactive Combined Project Title & Google Drive Capsule
    let title_raw = if app.session.is_dirty() { format!("{}*", app.session.project.name) } else { app.session.project.name.clone() };
    let (status_text, short_status, dot_color) = if app.collab.is_some() {
        ("Collab: Live", "Collab", Color32::from_rgb(0x10, 0xb9, 0x81))
    } else if app.gdrive_last_synced.is_some() {
        ("Google Drive: Synced", "Synced", Color32::from_rgb(0x10, 0xb9, 0x81))
    } else if app.gdrive_config.is_authenticated() {
        ("Google Drive: Connected", "Online", Color32::from_rgb(0x10, 0xb9, 0x81))
    } else if app.gdrive_config.active_folder_name.is_some() || app.gdrive_config.active_folder_id.is_some() {
        ("Linked Folder", "Linked", Color32::from_rgb(0x38, 0x82, 0xf6))
    } else {
        ("Local storage", "Local", t.text_dim)
    };

    let avail_capsule_space = (rx - (center_end_x + 14.0)).max(0.0);
    if avail_capsule_space >= 90.0 {
        let capsule_label = if avail_capsule_space >= 220.0 {
            format!("{title_raw} · {status_text}")
        } else if avail_capsule_space >= 140.0 {
            format!("{title_raw} · {short_status}")
        } else {
            status_text.to_string()
        };

        let cg = p.layout_no_wrap(capsule_label.clone(), Tokens::ui(11.5), t.text);
        let desired_capsule_w = cg.size().x + 30.0;
        let capsule_w = desired_capsule_w.min(avail_capsule_space);

        let cr = Rect::from_min_size(pos2(rx - capsule_w, rect.center().y - 12.0), vec2(capsule_w, 24.0));
        let cresp = ui.interact(cr, egui::Id::new("hdr-gdrive-capsule"), Sense::click()).on_hover_text("Project storage and collaboration");
        app.auto.add_response("header.gdrive_capsule", &cresp, "Project storage and collaboration");

        let capsule_bg = if cresp.hovered() { t.hover } else { t.tl_header_bg };
        p.rect_filled(cr, 12.0, capsule_bg);
        p.rect_stroke(cr, 12.0, Stroke::new(1.0, t.separator), StrokeKind::Inside);

        // Status dot
        p.circle_filled(pos2(cr.min.x + 11.0, cr.center().y), 3.5, dot_color);

        // Clipped text to prevent any overflow
        let text_clip = Rect::from_min_max(pos2(cr.min.x + 20.0, cr.min.y), pos2(cr.max.x - 6.0, cr.max.y));
        p.with_clip_rect(text_clip).text(pos2(cr.min.x + 20.0, cr.center().y), Align2::LEFT_CENTER, capsule_label, Tokens::ui(11.5), t.text);

        if cresp.clicked() {
            app.collab_dialog_open = true;
        }
    }

    // Workspace popup menu
    let popup_id = egui::Id::new("workspaces-popup");
    if ws_resp.clicked() {
        ui.ctx().data_mut(|d| d.insert_temp(popup_id, true));
    }
    let open = ui.ctx().data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
    if open {
        let anchor = pos2((if show_ws_name { wr.min.x } else { rx }).min((rect.max.x - 244.0).max(rect.min.x + 4.0)), rect.max.y + 4.0);
        let area = egui::Area::new(popup_id.with("area")).order(egui::Order::Foreground).fixed_pos(anchor).show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(220.0);
                for w in crate::dock::names(&app.workspaces) {
                    let sel = app.ui.workspace == w;
                    let label = crate::i18n::t(if matches!(w.as_str(), "Fusion" | "Fusion Nodes") { "Fusion (Experimental)" } else { &w });
                    let resp = ui.selectable_label(sel, label);
                    if w == "Classic Editing" {
                        app.auto.add_response("editing.classic", &resp, "Classic Editing");
                    }
                    if resp.clicked() {
                        app.set_workspace(&w);
                        ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                    }
                }
                ui.separator();
                if crate::dock::is_builtin(&app.ui.workspace) {
                    let label = if app.ui.workspace_preview.is_some() { "Return to Previous Layout" } else { "Preview Factory Layout" };
                    let preview = ui.button(label);
                    app.auto.add_response("workspace.previewDefault", &preview, label);
                    if preview.clicked() {
                        if let Some(previous) = app.ui.workspace_preview.take() {
                            app.ui.dock = previous.root;
                            app.ui.floating = previous.floating;
                        } else {
                            app.ui.workspace_preview = Some(crate::dock::Layout { root: app.ui.dock.clone(), floating: app.ui.floating.clone() });
                            app.ui.dock = crate::dock::workspace(&app.ui.workspace);
                            app.ui.floating.clear();
                        }
                        app.ui.keys.maximized = None;
                        ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                    }
                }
                if ui.button(tl!("Reset to Saved Layout")).clicked() {
                    app.reset_workspace();
                    ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                }
                let save = ui.button("Save Layout As…");
                app.auto.add_response("workspace.saveAs", &save, "Save Layout As");
                if save.clicked() {
                    let _ = crate::menus::invoke(app, ui.ctx(), "window.workspace.saveAs", serde_json::json!({}));
                    ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                }
            });
        });
        if (area.response.clicked_elsewhere() && !ws_resp.clicked()) || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
        }
    }
}
