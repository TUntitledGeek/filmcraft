//! DaVinci Fusion Node Compositor Panel for FilmCraft.
//!
//! Interactive immediate-mode DAG editor for clip effects and compositing.
//! Features:
//! - Canvas navigation: Pan (Middle-drag or Space+drag), Zoom (wheel/pinch 0.25x-2.5x focused on cursor)
//! - Toolbar navigation controls: [⟲ 100%], [⛶ Fit], [+ Zoom In], [- Zoom Out], Zoom % indicator
//! - Interactive Socket Wire Dragging: Pull bezier curves from Output socket to Input socket with snapping
//! - Wire / Socket Disconnection via Alt-click or Right-click
//! - DaVinci Resolve color-coded sockets (Yellow: Background, Green: Foreground, Blue: Mask, White: Out)
//! - Node selection with glowing amber border (#ec9324)
//! - Parameter inspector drawer with live sliders for Blur, ColorGrade, Glow, Sharpen, Merge, Keyer, Transform
//! - Dual monitor soloing: Key `1` solos selected node to Monitor 1; Key `2` routes output to Program
//! - Delete key removes selected node (protecting MediaIn and MediaOut)
//! - Immediate synchronization into item.effects via graph.to_linear_effects()

use std::collections::HashMap;
use std::sync::Arc;

use egui::{Align2, Color32, Pos2, Rect, RichText, Sense, Stroke, pos2, vec2};
use filmcraft_project::keyframe::{Param, ParamValue};
use filmcraft_project::node_graph::{NodeGraph, NodeId, NodeType};
use filmcraft_project::{ClipId, TrackItem};

use crate::FilmcraftApp;
use crate::theme::Tokens;

#[derive(Clone, Copy, Debug)]
struct CanvasState {
    pan: egui::Vec2,
    zoom: f32,
    selected_node: Option<NodeId>,
    dragging_from: Option<(NodeId, &'static str)>,
}

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().with_clip_rect(rect);

    // Get selected clip from timeline
    let (clip_id, mut item) = match get_selected_clip(app) {
        Some((id, it)) => (id, it),
        None => {
            p.rect_filled(rect, 0.0, Color32::from_rgb(0x16, 0x16, 0x18));
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Select a video clip in the Timeline to open Fusion Nodes",
                Tokens::ui(14.0),
                t.text_dim,
            );
            return;
        }
    };

    // Ensure clip has a valid NodeGraph
    if item.node_graph.is_none() {
        item.node_graph = Some(NodeGraph::from_effects(&item.name, &item.effects));
    }
    let mut graph = item.node_graph.clone().unwrap_or_default();

    // Load persistent canvas state
    let state_id = egui::Id::new("fusion_canvas_state");
    let mut canvas_state = ui.ctx().data(|d| {
        d.get_temp::<CanvasState>(state_id).unwrap_or(CanvasState {
            pan: vec2(20.0, 20.0),
            zoom: 1.0,
            selected_node: None,
            dragging_from: None,
        })
    });

    let mut modified = false;

    // =========================================================================
    // 1. TOP TOOLBAR
    // =========================================================================
    let toolbar_h = 36.0;
    let toolbar_r = Rect::from_min_size(rect.min, vec2(rect.width(), toolbar_h));
    let drawer_h = if canvas_state.selected_node.is_some() { 160.0 } else { 0.0 };
    let canvas_rect = Rect::from_min_max(pos2(rect.min.x, toolbar_r.max.y), pos2(rect.max.x, rect.max.y - drawer_h));
    p.rect_filled(toolbar_r, 0.0, Color32::from_rgb(0x1e, 0x1e, 0x22));
    p.line_segment([toolbar_r.left_bottom(), toolbar_r.right_bottom()], Stroke::new(1.0, Color32::from_rgb(0x2c, 0x2c, 0x32)));

    // Title & Clip pill
    p.text(toolbar_r.min + vec2(12.0, toolbar_h / 2.0), Align2::LEFT_CENTER, "☊ Fusion Nodes", Tokens::semibold(13.0), Color32::from_rgb(0xec, 0x93, 0x24));
    let clip_label = format!("Clip: {}", item.name);
    p.text(toolbar_r.min + vec2(130.0, toolbar_h / 2.0), Align2::LEFT_CENTER, clip_label, Tokens::ui(12.0), t.text);

    // Zoom and Fit navigation toolbar buttons
    let mut rx = toolbar_r.max.x - 10.0;

    // Zoom In [+]
    let btn_zin_r = Rect::from_min_size(pos2(rx - 28.0, toolbar_r.min.y + 6.0), vec2(28.0, 24.0));
    let resp_zin = ui.interact(btn_zin_r, egui::Id::new("fusion_zoom_in"), Sense::click()).on_hover_text("Zoom In (+15%)");
    p.rect_filled(btn_zin_r, 4.0, if resp_zin.hovered() { Color32::from_rgb(0x3a, 0x3a, 0x44) } else { Color32::from_rgb(0x25, 0x25, 0x2d) });
    p.text(btn_zin_r.center(), Align2::CENTER_CENTER, "+", Tokens::semibold(14.0), Color32::WHITE);
    if resp_zin.clicked() {
        canvas_state.zoom = (canvas_state.zoom * 1.15).clamp(0.25, 2.5);
    }
    rx = btn_zin_r.min.x - 4.0;

    // Zoom percentage readout
    let zoom_pct_str = format!("{}%", (canvas_state.zoom * 100.0).round() as i32);
    let zg = p.layout_no_wrap(zoom_pct_str.clone(), Tokens::ui(11.0), Color32::from_rgb(0xaa, 0xaa, 0xb8));
    let zreadout_r = Rect::from_min_size(pos2(rx - 48.0, toolbar_r.min.y + 6.0), vec2(48.0, 24.0));
    p.rect_filled(zreadout_r, 4.0, Color32::from_rgb(0x18, 0x18, 0x1c));
    p.galley(pos2(zreadout_r.center().x - zg.size().x / 2.0, zreadout_r.center().y - zg.size().y / 2.0), zg, Color32::from_rgb(0xaa, 0xaa, 0xb8));
    rx = zreadout_r.min.x - 4.0;

    // Zoom Out [-]
    let btn_zout_r = Rect::from_min_size(pos2(rx - 28.0, toolbar_r.min.y + 6.0), vec2(28.0, 24.0));
    let resp_zout = ui.interact(btn_zout_r, egui::Id::new("fusion_zoom_out"), Sense::click()).on_hover_text("Zoom Out (-15%)");
    p.rect_filled(btn_zout_r, 4.0, if resp_zout.hovered() { Color32::from_rgb(0x3a, 0x3a, 0x44) } else { Color32::from_rgb(0x25, 0x25, 0x2d) });
    p.text(btn_zout_r.center(), Align2::CENTER_CENTER, "−", Tokens::semibold(14.0), Color32::WHITE);
    if resp_zout.clicked() {
        canvas_state.zoom = (canvas_state.zoom * 0.85).clamp(0.25, 2.5);
    }
    rx = btn_zout_r.min.x - 6.0;

    // Reset 100% [⟲ 100%]
    let btn_100_r = Rect::from_min_size(pos2(rx - 56.0, toolbar_r.min.y + 6.0), vec2(56.0, 24.0));
    let resp_100 = ui.interact(btn_100_r, egui::Id::new("fusion_zoom_100"), Sense::click()).on_hover_text("Reset to 100% zoom");
    p.rect_filled(btn_100_r, 4.0, if resp_100.hovered() { Color32::from_rgb(0x3a, 0x3a, 0x44) } else { Color32::from_rgb(0x25, 0x25, 0x2d) });
    p.text(btn_100_r.center(), Align2::CENTER_CENTER, "⟲ 100%", Tokens::ui(11.0), Color32::WHITE);
    if resp_100.clicked() {
        canvas_state.zoom = 1.0;
        canvas_state.pan = vec2(20.0, 20.0);
    }
    rx = btn_100_r.min.x - 4.0;

    // Fit [⛶ Fit]
    let btn_fit_r = Rect::from_min_size(pos2(rx - 52.0, toolbar_r.min.y + 6.0), vec2(52.0, 24.0));
    let resp_fit = ui.interact(btn_fit_r, egui::Id::new("fusion_zoom_fit"), Sense::click()).on_hover_text("Fit all nodes to view");
    p.rect_filled(btn_fit_r, 4.0, if resp_fit.hovered() { Color32::from_rgb(0x3a, 0x3a, 0x44) } else { Color32::from_rgb(0x25, 0x25, 0x2d) });
    p.text(btn_fit_r.center(), Align2::CENTER_CENTER, "⛶ Fit", Tokens::ui(11.0), Color32::WHITE);
    rx = btn_fit_r.min.x - 14.0;

    // Quick add node buttons
    let node_types = [
        ("+ Blur", NodeType::Blur),
        ("+ Glow", NodeType::Glow),
        ("+ Sharpen", NodeType::Sharpen),
        ("+ Grade", NodeType::ColorGrade),
        ("+ Merge", NodeType::Merge),
        ("+ Keyer", NodeType::Keyer),
    ];

    for (label, ntype) in node_types.into_iter().rev() {
        let bw = 68.0;
        rx -= bw + 4.0;
        let br = Rect::from_min_size(pos2(rx, toolbar_r.min.y + 6.0), vec2(bw, 24.0));
        let bresp = ui.interact(br, egui::Id::new(("add_node", label)), Sense::click());
        let bg = if bresp.hovered() { Color32::from_rgb(0x3a, 0x3a, 0x44) } else { Color32::from_rgb(0x28, 0x28, 0x30) };
        p.rect_filled(br, 4.0, bg);
        p.rect_stroke(br, 4.0, Stroke::new(1.0, Color32::from_rgb(0x40, 0x40, 0x4c)), egui::StrokeKind::Inside);
        p.text(br.center(), Align2::CENTER_CENTER, label, Tokens::ui(11.0), Color32::WHITE);

        if bresp.clicked() {
            let center_canvas_x = (canvas_rect.center().x - canvas_rect.min.x - canvas_state.pan.x) / canvas_state.zoom;
            let center_canvas_y = (canvas_rect.center().y - canvas_rect.min.y - canvas_state.pan.y) / canvas_state.zoom;
            let snapped_x = (center_canvas_x / 16.0).round() * 16.0;
            let snapped_y = (center_canvas_y / 16.0).round() * 16.0;
            let new_name = format!("{}{}", ntype.label(), graph.nodes.len() + 1);
            let nid = graph.add_node(&new_name, ntype, [snapped_x - 64.0, snapped_y - 40.0]);
            canvas_state.selected_node = Some(nid);
            modified = true;
        }
    }

    // =========================================================================
    // 2. CANVAS VIEW & NAVIGATION
    // =========================================================================
    // Canvas background
    p.rect_filled(canvas_rect, 0.0, Color32::from_rgb(0x15, 0x15, 0x18));

    // Handle Fit button logic if clicked
    if resp_fit.clicked() && !graph.nodes.is_empty() {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for n in &graph.nodes {
            min_x = min_x.min(n.pos[0]);
            min_y = min_y.min(n.pos[1]);
            max_x = max_x.max(n.pos[0] + 130.0);
            max_y = max_y.max(n.pos[1] + 80.0);
        }
        let bw = (max_x - min_x).max(120.0);
        let bh = (max_y - min_y).max(100.0);
        let tz = ((canvas_rect.width() * 0.8) / bw).min((canvas_rect.height() * 0.8) / bh).clamp(0.35, 1.8);
        canvas_state.zoom = tz;
        canvas_state.pan = vec2(
            (canvas_rect.width() - bw * tz) / 2.0 - min_x * tz,
            (canvas_rect.height() - bh * tz) / 2.0 - min_y * tz,
        );
    }

    // Canvas interaction (Pan & Zoom)
    let canvas_id = egui::Id::new("fusion_canvas_view");
    let canvas_resp = ui.interact(canvas_rect, canvas_id, Sense::click_and_drag());
    if canvas_resp.clicked() || canvas_resp.drag_started() {
        app.ui.focused = crate::dock::PanelKind::NodeGraph;
    }

    let space_down = ui.input(|i| i.key_down(egui::Key::Space));
    if canvas_resp.dragged_by(egui::PointerButton::Middle) || (space_down && canvas_resp.dragged_by(egui::PointerButton::Primary)) {
        canvas_state.pan += canvas_resp.drag_delta();
        if app.playback.playing {
            let _ = crate::menus::invoke(app, ui.ctx(), "playback.stop", serde_json::json!({}));
        }
    }

    let mouse_hover = ui.input(|i| i.pointer.hover_pos()).unwrap_or(pos2(-1000.0, -1000.0));
    let over_any_node = graph.nodes.iter().any(|n| {
        let nr = Rect::from_min_size(canvas_rect.min + canvas_state.pan + vec2(n.pos[0] * canvas_state.zoom, n.pos[1] * canvas_state.zoom), vec2(130.0 * canvas_state.zoom, 80.0 * canvas_state.zoom));
        nr.contains(mouse_hover)
    });
    if canvas_resp.clicked() && !space_down && !over_any_node {
        canvas_state.selected_node = None;
    }

    // Zoom with scroll wheel focused on cursor
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
    if canvas_resp.hovered() && scroll_delta.abs() > 0.05 {
        let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(canvas_rect.center());
        let old_zoom = canvas_state.zoom;
        let factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
        canvas_state.zoom = (canvas_state.zoom * factor).clamp(0.25, 2.5);
        canvas_state.pan = mouse_pos - canvas_rect.min - (mouse_pos - canvas_rect.min - canvas_state.pan) * (canvas_state.zoom / old_zoom);
    }

    // Grid rendering scaled with zoom and pan
    draw_canvas_grid(&p, canvas_rect, canvas_state.pan, canvas_state.zoom);

    // =========================================================================
    // 3. DRAW CONNECTION BEZIER PIPES
    // =========================================================================
    let zoom = canvas_state.zoom;
    let pan = canvas_state.pan;

    let node_map: HashMap<NodeId, &filmcraft_project::node_graph::NodeInstance> = graph.nodes.iter().map(|n| (n.id, n)).collect();
    let mut disconnect_input: Option<(NodeId, String)> = None;

    for conn in &graph.connections {
        let from_n = node_map.get(&conn.from_node);
        let to_n = node_map.get(&conn.to_node);
        if let (Some(fn_node), Some(tn_node)) = (from_n, to_n) {
            let start = canvas_rect.min + pan + vec2((fn_node.pos[0] + 130.0) * zoom, (fn_node.pos[1] + 36.0) * zoom);
            
            // Calculate destination socket position
            let inputs = tn_node.node_type.default_inputs();
            let sock_idx = inputs.iter().position(|&s| s == conn.to_socket).unwrap_or(0);
            let end_y = tn_node.pos[1] + 24.0 + 16.0 * (sock_idx as f32 + 1.0);
            let end = canvas_rect.min + pan + vec2(tn_node.pos[0] * zoom, end_y * zoom);

            let cp_dist = 45.0 * zoom;
            let cp1 = start + vec2(cp_dist, 0.0);
            let cp2 = end - vec2(cp_dist, 0.0);

            let base_color = match conn.to_socket.as_str() {
                "Foreground" => Color32::from_rgb(0x3a, 0xcc, 0x5a), // Green
                "Background" => Color32::from_rgb(0xf2, 0xa9, 0x24), // Yellow
                "Mask" => Color32::from_rgb(0x3a, 0x8a, 0xf2),       // Blue
                _ => Color32::from_rgb(0xd8, 0xd8, 0xd8),             // Standard signal
            };

            // Wire hover hit-testing (Alt-click or Right-click to disconnect)
            let wire_dist = bezier_distance(start, cp1, cp2, end, mouse_hover);
            let is_wire_hovered = canvas_rect.contains(mouse_hover) && wire_dist <= 7.0 * zoom.clamp(0.8, 1.4);

            let (glow_col, core_col, core_w) = if is_wire_hovered {
                (Color32::from_rgba_unmultiplied(255, 230, 100, 150), Color32::from_rgb(255, 240, 140), 3.0 * zoom.clamp(0.8, 1.4))
            } else {
                (Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), 60), base_color, 2.0 * zoom.clamp(0.6, 1.2))
            };

            draw_bezier(&p, start, cp1, cp2, end, Stroke::new(5.0 * zoom.clamp(0.6, 1.4), glow_col));
            draw_bezier(&p, start, cp1, cp2, end, Stroke::new(core_w, core_col));

            if is_wire_hovered {
                let alt_click = ui.input(|i| i.pointer.primary_clicked() && i.modifiers.alt);
                let right_click = ui.input(|i| i.pointer.secondary_clicked());
                if alt_click || right_click {
                    disconnect_input = Some((conn.to_node, conn.to_socket.clone()));
                }
            }
        }
    }

    // Find snap target if dragging wire
    let mut snap_target_pos = None;
    if let Some((from_id, _)) = canvas_state.dragging_from {
        let snap_radius = 24.0 * zoom.clamp(0.8, 1.5);
        for n in &graph.nodes {
            if n.id == from_id { continue; }
            let inputs = n.node_type.default_inputs();
            for (idx, _) in inputs.iter().enumerate() {
                let sy = (n.pos[1] + 24.0 + 16.0 * (idx as f32 + 1.0)) * zoom;
                let spos = canvas_rect.min + pan + vec2(n.pos[0] * zoom, sy);
                if spos.distance(mouse_hover) <= snap_radius {
                    snap_target_pos = Some(spos);
                    break;
                }
            }
            if snap_target_pos.is_some() { break; }
        }
    }

    // Active wire drag preview
    if let Some((from_id, _)) = canvas_state.dragging_from {
        if let Some(from_n) = node_map.get(&from_id) {
            let start = canvas_rect.min + pan + vec2((from_n.pos[0] + 130.0) * zoom, (from_n.pos[1] + 36.0) * zoom);
            let wire_end = snap_target_pos.unwrap_or(mouse_hover);
            let cp1 = start + vec2(40.0 * zoom, 0.0);
            let cp2 = wire_end - vec2(40.0 * zoom, 0.0);
            draw_bezier(&p, start, cp1, cp2, wire_end, Stroke::new(4.0, Color32::from_rgba_unmultiplied(236, 147, 36, 80)));
            draw_bezier(&p, start, cp1, cp2, wire_end, Stroke::new(2.0, Color32::from_rgb(0xec, 0x93, 0x24)));
        }
    }

    // =========================================================================
    // 4. RENDER NODE BLOCKS & SOCKETS
    // =========================================================================
    let mut moved: Option<(usize, egui::Vec2)> = None;
    let mut new_drag_start: Option<(NodeId, &'static str)> = None;
    let mut dropped_connection: Option<(NodeId, &'static str, NodeId, &'static str)> = None;
    let mut toggle_solo: Option<usize> = None;
    let mut disconnect_output: Option<NodeId> = None;
    let mut snap_grid_node: Option<usize> = None;

    for i in 0..graph.nodes.len() {
        let node = &graph.nodes[i];
        let node_id = node.id;
        let is_selected = canvas_state.selected_node == Some(node_id);

        let node_screen_min = canvas_rect.min + pan + vec2(node.pos[0] * zoom, node.pos[1] * zoom);
        let node_rect = Rect::from_min_size(node_screen_min, vec2(130.0 * zoom, 80.0 * zoom));

        // Node card interaction
        let card_id = egui::Id::new(("fusion_node", node_id.0));
        let card_resp = ui.interact(node_rect, card_id, Sense::click_and_drag());

        if card_resp.clicked() {
            canvas_state.selected_node = Some(node_id);
        }
        if card_resp.dragged() && !space_down {
            moved = Some((i, card_resp.drag_delta() / zoom));
        }
        if card_resp.drag_stopped() {
            snap_grid_node = Some(i);
        }

        // Card background & glowing amber border if selected
        let card_bg = if is_selected {
            Color32::from_rgb(0x28, 0x28, 0x32)
        } else if card_resp.hovered() {
            Color32::from_rgb(0x24, 0x24, 0x2c)
        } else {
            Color32::from_rgb(0x1c, 0x1c, 0x22)
        };

        p.rect_filled(node_rect, 6.0 * zoom.clamp(0.6, 1.2), card_bg);

        let border_stroke = if is_selected {
            Stroke::new(2.5, Color32::from_rgb(0xec, 0x93, 0x24)) // Glowing Amber selection
        } else if node.solo {
            Stroke::new(2.0, Color32::from_rgb(0x26, 0x80, 0xeb)) // Solo active blue
        } else {
            Stroke::new(1.0, Color32::from_rgb(0x38, 0x38, 0x44))
        };
        p.rect_stroke(node_rect, 6.0 * zoom.clamp(0.6, 1.2), border_stroke, egui::StrokeKind::Inside);

        // Header color band
        let header_h = 24.0 * zoom;
        let header_r = Rect::from_min_size(node_rect.min, vec2(node_rect.width(), header_h));
        let header_color = match node.node_type {
            NodeType::MediaIn => Color32::from_rgb(0x24, 0x5c, 0x9e),
            NodeType::MediaOut => Color32::from_rgb(0x9e, 0x3a, 0x24),
            NodeType::Merge => Color32::from_rgb(0x24, 0x7a, 0x48),
            NodeType::ColorGrade => Color32::from_rgb(0x8a, 0x5a, 0x1e),
            NodeType::Blur | NodeType::Glow | NodeType::Sharpen => Color32::from_rgb(0x5e, 0x3a, 0x82),
            NodeType::Keyer => Color32::from_rgb(0x24, 0x6e, 0x7a),
            NodeType::Transform => Color32::from_rgb(0x4a, 0x4a, 0x6a),
        };
        p.rect_filled(header_r, 6.0 * zoom.clamp(0.6, 1.2), header_color);
        p.text(header_r.min + vec2(8.0 * zoom, header_h / 2.0), Align2::LEFT_CENTER, &node.name, Tokens::semibold(11.0 * zoom.clamp(0.7, 1.2)), Color32::WHITE);

        // Solo indicator dot
        let solo_r = Rect::from_center_size(node_rect.left_bottom() + vec2(14.0 * zoom, -12.0 * zoom), vec2(16.0 * zoom, 16.0 * zoom));
        let solo_resp = ui.interact(solo_r, egui::Id::new(("solo_dot", node_id.0)), Sense::click()).on_hover_text("Monitor 1 Solo");
        p.circle_filled(solo_r.center(), 5.0 * zoom.clamp(0.6, 1.2), if node.solo { Color32::from_rgb(0x26, 0x80, 0xeb) } else { Color32::from_rgb(0x44, 0x44, 0x50) });
        p.text(solo_r.center(), Align2::CENTER_CENTER, "1", Tokens::ui(8.0 * zoom.clamp(0.7, 1.2)), Color32::WHITE);
        if solo_resp.clicked() {
            toggle_solo = Some(i);
        }

        // Input sockets (Left edge)
        let inputs = node.node_type.default_inputs();
        for (idx, in_name) in inputs.iter().enumerate() {
            let sy = node_rect.min.y + (24.0 + 16.0 * (idx as f32 + 1.0)) * zoom;
            let sock_pos = pos2(node_rect.min.x, sy);
            let sock_r = Rect::from_center_size(sock_pos, vec2(14.0 * zoom, 14.0 * zoom));
            let sresp = ui.interact(sock_r, egui::Id::new(("in_sock", node_id.0, idx)), Sense::click_and_drag());

            let sock_color = match *in_name {
                "Foreground" => Color32::from_rgb(0x3a, 0xcc, 0x5a),
                "Background" => Color32::from_rgb(0xf2, 0xa9, 0x24),
                "Mask" => Color32::from_rgb(0x3a, 0x8a, 0xf2),
                _ => Color32::from_rgb(0xd8, 0xd8, 0xd8),
            };

            // Alt-click or Right-click on socket to disconnect
            if sresp.secondary_clicked() || (sresp.clicked() && ui.input(|i| i.modifiers.alt)) {
                disconnect_input = Some((node_id, in_name.to_string()));
            }

            // Snap feedback during wire dragging
            let snap_radius = 24.0 * zoom.clamp(0.8, 1.5);
            let in_snap = sock_pos.distance(mouse_hover) <= snap_radius;
            if let Some((from_id, from_sock)) = canvas_state.dragging_from {
                if from_id != node_id && in_snap {
                    p.circle_stroke(sock_pos, 9.0 * zoom.clamp(0.8, 1.4), Stroke::new(2.5, Color32::from_rgb(0xec, 0x93, 0x24)));
                    if ui.input(|i| i.pointer.any_released()) {
                        dropped_connection = Some((from_id, from_sock, node_id, in_name));
                    }
                }
            }

            p.circle_filled(sock_pos, 4.5 * zoom.clamp(0.6, 1.2), sock_color);
        }

        // Output socket (Right edge)
        let outputs = node.node_type.default_outputs();
        if !outputs.is_empty() {
            let sock_pos = pos2(node_rect.max.x, node_rect.min.y + 36.0 * zoom);
            let sock_r = Rect::from_center_size(sock_pos, vec2(14.0 * zoom, 14.0 * zoom));
            let out_resp = ui.interact(sock_r, egui::Id::new(("out_sock", node_id.0)), Sense::click_and_drag());

            if out_resp.drag_started() {
                new_drag_start = Some((node_id, "Out"));
            }

            // Alt-click or Right-click on output socket to disconnect outgoing wires
            if out_resp.secondary_clicked() || (out_resp.clicked() && ui.input(|i| i.modifiers.alt)) {
                disconnect_output = Some(node_id);
            }

            p.circle_filled(sock_pos, 4.5 * zoom.clamp(0.6, 1.2), Color32::from_rgb(0xec, 0xec, 0xec));
        }
    }

    // Apply deferred graph mutations
    if let Some(idx) = toggle_solo {
        let was_solo = graph.nodes[idx].solo;
        for n in &mut graph.nodes {
            n.solo = false;
        }
        graph.nodes[idx].solo = !was_solo;
        modified = true;
    }

    if let Some((nid, sock)) = disconnect_input {
        graph.disconnect(nid, &sock);
        modified = true;
    }

    if let Some(nid) = disconnect_output {
        graph.connections.retain(|c| c.from_node != nid);
        modified = true;
    }

    // Apply wire connection if dropped on input socket
    if let Some((from_id, from_sock, to_id, to_sock)) = dropped_connection {
        graph.connect(from_id, from_sock, to_id, to_sock);
        canvas_state.dragging_from = None;
        modified = true;
    } else if ui.input(|i| i.pointer.any_released()) {
        canvas_state.dragging_from = None;
    }

    if let Some(ds) = new_drag_start {
        canvas_state.dragging_from = Some(ds);
    }

    // Apply node drag delta
    if let Some((idx, delta)) = moved {
        graph.nodes[idx].pos[0] += delta.x;
        graph.nodes[idx].pos[1] += delta.y;
        modified = true;
    }

    if let Some(idx) = snap_grid_node {
        graph.nodes[idx].pos[0] = (graph.nodes[idx].pos[0] / 16.0).round() * 16.0;
        graph.nodes[idx].pos[1] = (graph.nodes[idx].pos[1] / 16.0).round() * 16.0;
        modified = true;
    }

    // =========================================================================
    // 5. KEYBOARD SHORTCUTS: 1 / 2 (Solo) & Delete (Remove Node)
    // =========================================================================
    let keyboard_focused = ui.ctx().egui_wants_keyboard_input();
    if let Some(sel_id) = canvas_state.selected_node {
        if !keyboard_focused {
            if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
                let was_solo = graph.nodes.iter().find(|n| n.id == sel_id).map(|n| n.solo).unwrap_or(false);
                for n in &mut graph.nodes {
                    n.solo = if n.id == sel_id { !was_solo } else { false };
                }
                modified = true;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
                for n in &mut graph.nodes {
                    n.solo = false;
                }
                modified = true;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                if let Some(pos) = graph.nodes.iter().position(|n| n.id == sel_id) {
                    let ntype = &graph.nodes[pos].node_type;
                    if *ntype != NodeType::MediaIn && *ntype != NodeType::MediaOut {
                        graph.nodes.remove(pos);
                        graph.connections.retain(|c| c.from_node != sel_id && c.to_node != sel_id);
                        canvas_state.selected_node = None;
                        modified = true;
                    }
                }
            }
        }
    }

    // =========================================================================
    // 6. PARAMETER INSPECTOR DRAWER
    // =========================================================================
    if let Some(sel_id) = canvas_state.selected_node {
        let drawer_rect = Rect::from_min_max(pos2(rect.min.x, canvas_rect.max.y), rect.max);
        p.rect_filled(drawer_rect, 0.0, Color32::from_rgb(0x1a, 0x1a, 0x1e));
        p.line_segment([drawer_rect.left_top(), drawer_rect.right_top()], Stroke::new(1.0, Color32::from_rgb(0x32, 0x32, 0x3c)));

        if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == sel_id) {
            let mut du = ui.new_child(egui::UiBuilder::new().max_rect(drawer_rect.shrink(8.0)));
            du.horizontal(|ui| {
                ui.heading(RichText::new(&node.name).color(Color32::from_rgb(0xec, 0x93, 0x24)).size(14.0));
                ui.label(RichText::new(format!("({})", node.node_type.label())).size(12.0).weak());
                ui.separator();

                let mut enabled = node.enabled;
                if ui.checkbox(&mut enabled, "Enabled").changed() {
                    node.enabled = enabled;
                    modified = true;
                }
            });
            du.add_space(6.0);

            // Live parameter controls according to NodeType
            egui::Grid::new("fusion_param_grid").num_columns(2).spacing([16.0, 6.0]).show(&mut du, |ui| {
                match node.node_type {
                    NodeType::Blur => {
                        ui.label("Blur Radius:");
                        let mut radius = get_float_param(node, "blurriness", get_float_param(node, "radius", 10.0));
                        if ui.add(egui::Slider::new(&mut radius, 0.0..=100.0).suffix(" px")).changed() {
                            set_float_param(node, "radius", radius);
                            set_float_param(node, "blurriness", radius);
                            modified = true;
                        }
                        ui.end_row();
                    }
                    NodeType::ColorGrade => {
                        ui.label("Exposure:");
                        let mut exp = get_float_param(node, "exposure", 0.0);
                        if ui.add(egui::Slider::new(&mut exp, -5.0..=5.0).suffix(" EV")).changed() {
                            set_float_param(node, "exposure", exp);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Contrast:");
                        let mut contrast = get_float_param(node, "contrast", 0.0);
                        if ui.add(egui::Slider::new(&mut contrast, -100.0..=100.0)).changed() {
                            set_float_param(node, "contrast", contrast);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Saturation:");
                        let mut sat = get_float_param(node, "saturation", 100.0);
                        if ui.add(egui::Slider::new(&mut sat, 0.0..=200.0).suffix("%")).changed() {
                            set_float_param(node, "saturation", sat);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Temperature:");
                        let mut temp = get_float_param(node, "temperature", 0.0);
                        if ui.add(egui::Slider::new(&mut temp, -100.0..=100.0)).changed() {
                            set_float_param(node, "temperature", temp);
                            modified = true;
                        }
                        ui.end_row();
                    }
                    NodeType::Glow => {
                        ui.label("Threshold:");
                        let mut thres = get_float_param(node, "threshold", 0.7);
                        if ui.add(egui::Slider::new(&mut thres, 0.0..=1.0)).changed() {
                            set_float_param(node, "threshold", thres);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Intensity:");
                        let mut inten = get_float_param(node, "intensity", 1.5);
                        if ui.add(egui::Slider::new(&mut inten, 0.0..=5.0)).changed() {
                            set_float_param(node, "intensity", inten);
                            modified = true;
                        }
                        ui.end_row();
                    }
                    NodeType::Sharpen => {
                        ui.label("Amount:");
                        let mut amt = get_float_param(node, "amount", 25.0);
                        if ui.add(egui::Slider::new(&mut amt, 0.0..=100.0)).changed() {
                            set_float_param(node, "amount", amt);
                            modified = true;
                        }
                        ui.end_row();
                    }
                    NodeType::Merge => {
                        ui.label("Opacity:");
                        let mut opac = get_float_param(node, "opacity", 100.0);
                        if ui.add(egui::Slider::new(&mut opac, 0.0..=100.0).suffix("%")).changed() {
                            set_float_param(node, "opacity", opac);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Blend Mode:");
                        let mut mode_idx = get_choice_param(node, "blend_mode", get_choice_param(node, "blend", 0));
                        let modes = ["Normal", "Add", "Multiply", "Screen", "Overlay"];
                        let current_label = modes.get(mode_idx as usize).unwrap_or(&"Normal");
                        egui::ComboBox::from_id_salt("merge_blend_combo")
                            .selected_text(*current_label)
                            .show_ui(ui, |ui| {
                                for (i, m) in modes.iter().enumerate() {
                                    if ui.selectable_value(&mut mode_idx, i as u32, *m).clicked() {
                                        set_choice_param(node, "blend_mode", mode_idx);
                                        set_choice_param(node, "blend", mode_idx);
                                        modified = true;
                                    }
                                }
                            });
                        ui.end_row();
                    }
                    NodeType::Keyer => {
                        ui.label("Threshold:");
                        let mut thres = get_float_param(node, "threshold", 0.15);
                        if ui.add(egui::Slider::new(&mut thres, 0.0..=1.0)).changed() {
                            set_float_param(node, "threshold", thres);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Softness:");
                        let mut soft = get_float_param(node, "softness", 0.05);
                        if ui.add(egui::Slider::new(&mut soft, 0.0..=0.5)).changed() {
                            set_float_param(node, "softness", soft);
                            modified = true;
                        }
                        ui.end_row();
                    }
                    NodeType::Transform => {
                        ui.label("Scale:");
                        let mut scale = get_float_param(node, "scale", 100.0);
                        if ui.add(egui::Slider::new(&mut scale, 0.0..=300.0).suffix("%")).changed() {
                            set_float_param(node, "scale", scale);
                            modified = true;
                        }
                        ui.end_row();

                        ui.label("Rotation:");
                        let mut rot = get_float_param(node, "rotation", 0.0);
                        if ui.add(egui::Slider::new(&mut rot, -180.0..=180.0).suffix("°")).changed() {
                            set_float_param(node, "rotation", rot);
                            modified = true;
                        }
                        ui.end_row();
                    }
                    _ => {
                        ui.label(RichText::new("Terminal node (outputs directly to timeline program stream).").weak());
                        ui.end_row();
                    }
                }
            });
        }
    }

    // Save updated canvas state
    ui.ctx().data_mut(|d| d.insert_temp(state_id, canvas_state));

    // Save modifications to timeline clip & linear effect pipeline
    if modified {
        item.effects = graph.to_linear_effects();
        item.node_graph = Some(graph);
        save_selected_clip(app, clip_id, item);
    }
}

fn draw_canvas_grid(p: &egui::Painter, rect: Rect, pan: egui::Vec2, zoom: f32) {
    let step = 32.0 * zoom;
    if step < 8.0 {
        return;
    }
    let stroke = Stroke::new(1.0, Color32::from_rgb(0x20, 0x20, 0x24));
    let offset_x = (pan.x % step + step) % step;
    let mut x = rect.min.x + offset_x;
    while x <= rect.max.x {
        p.line_segment([pos2(x, rect.min.y), pos2(x, rect.max.y)], stroke);
        x += step;
    }
    let offset_y = (pan.y % step + step) % step;
    let mut y = rect.min.y + offset_y;
    while y <= rect.max.y {
        p.line_segment([pos2(rect.min.x, y), pos2(rect.max.x, y)], stroke);
        y += step;
    }
}

fn draw_bezier(p: &egui::Painter, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, stroke: Stroke) {
    let steps = 24;
    let mut prev = p0;
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x + t * t * t * p3.x;
        let y = u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y + t * t * t * p3.y;
        let curr = pos2(x, y);
        p.line_segment([prev, curr], stroke);
        prev = curr;
    }
}

fn get_float_param(node: &filmcraft_project::node_graph::NodeInstance, key: &str, default: f64) -> f64 {
    match node.params.get(key).map(|p| &p.value) {
        Some(ParamValue::Float(v)) => *v,
        _ => default,
    }
}

fn set_float_param(node: &mut filmcraft_project::node_graph::NodeInstance, key: &str, val: f64) {
    if let Some(p) = node.params.get_mut(key) {
        p.value = ParamValue::Float(val);
    } else {
        node.params.insert(key.to_string(), Param::new(ParamValue::Float(val)));
    }
}

fn get_choice_param(node: &filmcraft_project::node_graph::NodeInstance, key: &str, default: u32) -> u32 {
    if let Some(p) = node.params.get(key) {
        if let ParamValue::Choice(c) = p.value {
            return c;
        }
    }
    default
}

fn set_choice_param(node: &mut filmcraft_project::node_graph::NodeInstance, key: &str, val: u32) {
    if let Some(p) = node.params.get_mut(key) {
        p.value = ParamValue::Choice(val);
    } else {
        node.params.insert(key.to_string(), Param::new(ParamValue::Choice(val)));
    }
}

fn get_selected_clip(app: &FilmcraftApp) -> Option<(ClipId, TrackItem)> {
    let seq = app.session.active_sequence()?;
    for track in &seq.video_tracks {
        for it in &track.items {
            if app.session.state.selection.contains(&it.id) {
                return Some((it.id, it.clone()));
            }
        }
    }
    // Fallback: first clip if none explicitly selected
    for track in &seq.video_tracks {
        if let Some(it) = track.items.first() {
            return Some((it.id, it.clone()));
        }
    }
    None
}

fn save_selected_clip(app: &mut FilmcraftApp, clip_id: ClipId, item: TrackItem) {
    let Some(seq_id) = app.session.state.active_sequence else { return };
    let project = Arc::make_mut(&mut app.session.project);
    if let Some(seq) = project.sequence_mut(seq_id) {
        if let Some((_, existing)) = seq.find_item_mut(clip_id) {
            *existing = item;
            app.session.revision += 1;
            app.session.events.push(filmcraft_engine::Event::ProjectChanged { revision: app.session.revision });
        }
    }
}

fn bezier_distance(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, pt: Pos2) -> f32 {
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

