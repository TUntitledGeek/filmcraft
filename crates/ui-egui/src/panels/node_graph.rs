//! DaVinci Fusion Node Compositor Panel for FilmCraft.
//!
//! Interactive immediate-mode DAG editor for clip effects and compositing.
//! Features:
//! - Visual node canvas with pan & drag
//! - DaVinci Resolve Fusion color-coded sockets (Yellow: Background, Green: Foreground, Blue: Mask/Effect, White: Output)
//! - Cubic bezier connection pipes with signal glow
//! - Real-time node parameter sliders
//! - Solo monitor toggle (`1` key: preview node buffer in Monitor 1, e.g. Alpha Matte or Raw Source)
//! - Top toolbar with quick-add buttons (+Blur, +Glow, +Sharpen, +Grade, +Merge, +Keyer)

use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, pos2, vec2};
use filmcraft_project::node_graph::{NodeGraph, NodeType};
use filmcraft_project::{ClipId, TrackItem};

use crate::FilmcraftApp;
use crate::theme::Tokens;

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().with_clip_rect(rect);

    // Dark grid canvas background (DaVinci Resolve style)
    p.rect_filled(rect, 0.0, Color32::from_rgb(0x16, 0x16, 0x18));
    draw_grid(&p, rect, 32.0, Color32::from_rgb(0x22, 0x22, 0x26));

    // Get selected clip
    let (clip_id, mut item) = match get_selected_clip(app) {
        Some((id, it)) => (id, it),
        None => {
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

    // Top toolbar
    let toolbar_h = 34.0;
    let toolbar_r = Rect::from_min_size(rect.min, vec2(rect.width(), toolbar_h));
    p.rect_filled(toolbar_r, 0.0, Color32::from_rgb(0x1e, 0x1e, 0x22));
    p.line_segment([toolbar_r.left_bottom(), toolbar_r.right_bottom()], Stroke::new(1.0, Color32::from_rgb(0x2c, 0x2c, 0x32)));

    // Title & Clip pill
    p.text(toolbar_r.min + vec2(12.0, toolbar_h / 2.0), Align2::LEFT_CENTER, "☊ Fusion Nodes", Tokens::semibold(13.0), Color32::from_rgb(0xec, 0x93, 0x24));
    let clip_label = format!("Clip: {}", item.name);
    p.text(toolbar_r.min + vec2(130.0, toolbar_h / 2.0), Align2::LEFT_CENTER, clip_label, Tokens::ui(12.0), t.text);

    // Quick add node buttons
    let mut bx = rect.max.x - 12.0;
    let node_types = [
        ("+ Blur", NodeType::Blur),
        ("+ Glow", NodeType::Glow),
        ("+ Sharpen", NodeType::Sharpen),
        ("+ Grade", NodeType::ColorGrade),
        ("+ Merge", NodeType::Merge),
        ("+ Keyer", NodeType::Keyer),
    ];

    for (label, ntype) in node_types.into_iter().rev() {
        let bw = 70.0;
        bx -= bw + 6.0;
        let br = Rect::from_min_size(pos2(bx, toolbar_r.min.y + 5.0), vec2(bw, 24.0));
        let bresp = ui.interact(br, egui::Id::new(("add_node", label)), Sense::click());
        let bg = if bresp.hovered() { Color32::from_rgb(0x3a, 0x3a, 0x44) } else { Color32::from_rgb(0x28, 0x28, 0x30) };
        p.rect_filled(br, 4.0, bg);
        p.rect_stroke(br, 4.0, Stroke::new(1.0, Color32::from_rgb(0x40, 0x40, 0x4c)), egui::StrokeKind::Inside);
        p.text(br.center(), Align2::CENTER_CENTER, label, Tokens::ui(11.0), Color32::WHITE);

        if bresp.clicked() {
            let center_pos = [rect.center().x - rect.min.x - 60.0, rect.center().y - rect.min.y - 40.0];
            let new_name = format!("{}{}", ntype.label(), graph.nodes.len() + 1);
            graph.add_node(&new_name, ntype, center_pos);
        }
    }

    // Canvas region
    let canvas_rect = Rect::from_min_max(pos2(rect.min.x, toolbar_r.max.y), rect.max);
    let mut node_positions = std::collections::HashMap::new();

    // Draw connection bezier curves
    for conn in &graph.connections {
        let from_node = graph.node(conn.from_node);
        let to_node = graph.node(conn.to_node);
        if let (Some(fn_node), Some(tn_node)) = (from_node, to_node) {
            let start = canvas_rect.min + vec2(fn_node.pos[0] + 130.0, fn_node.pos[1] + 35.0);
            let end = canvas_rect.min + vec2(tn_node.pos[0], tn_node.pos[1] + 35.0);

            let cp1 = start + vec2(40.0, 0.0);
            let cp2 = end - vec2(40.0, 0.0);

            // DaVinci pipe glow & wire
            let color = match conn.to_socket.as_str() {
                "Foreground" => Color32::from_rgb(0x3a, 0xcc, 0x5a), // Green
                "Background" => Color32::from_rgb(0xf2, 0xa9, 0x24), // Yellow
                "Mask" => Color32::from_rgb(0x3a, 0x8a, 0xf2),       // Blue
                _ => Color32::from_rgb(0xd8, 0xd8, 0xd8),             // Standard signal
            };

            draw_bezier(&p, start, cp1, cp2, end, Stroke::new(3.0, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60)));
            draw_bezier(&p, start, cp1, cp2, end, Stroke::new(1.8, color));
        }
    }

    // Render node blocks
    let mut modified = false;
    let mut moved: Option<(usize, egui::Vec2)> = None;
    let mut toggle_solo: Option<usize> = None;

    for i in 0..graph.nodes.len() {
        let node = &graph.nodes[i];
        let node_id = node.id;
        let node_rect = Rect::from_min_size(canvas_rect.min + vec2(node.pos[0], node.pos[1]), vec2(130.0, 80.0));

        let drag_id = egui::Id::new(("node_drag", node_id.0));
        let drag_resp = ui.interact(node_rect, drag_id, Sense::click_and_drag());

        if drag_resp.dragged() {
            moved = Some((i, drag_resp.drag_delta()));
        }

        node_positions.insert(node_id, node_rect);

        // Node card background
        let is_active = drag_resp.hovered() || node.solo;
        let card_bg = if is_active { Color32::from_rgb(0x28, 0x28, 0x30) } else { Color32::from_rgb(0x1f, 0x1f, 0x24) };
        let border_stroke = if node.solo {
            Stroke::new(1.5, Color32::from_rgb(0x26, 0x80, 0xeb)) // Solo active blue ring
        } else if is_active {
            Stroke::new(1.0, Color32::from_rgb(0x5a, 0x5a, 0x6e))
        } else {
            Stroke::new(1.0, Color32::from_rgb(0x32, 0x32, 0x3a))
        };

        p.rect_filled(node_rect, 6.0, card_bg);
        p.rect_stroke(node_rect, 6.0, border_stroke, egui::StrokeKind::Inside);

        // Node header band
        let header_r = Rect::from_min_size(node_rect.min, vec2(node_rect.width(), 24.0));
        let header_color = match node.node_type {
            NodeType::MediaIn => Color32::from_rgb(0x24, 0x5c, 0x9e),
            NodeType::MediaOut => Color32::from_rgb(0x9e, 0x3a, 0x24),
            NodeType::Merge => Color32::from_rgb(0x24, 0x7a, 0x48),
            NodeType::ColorGrade => Color32::from_rgb(0x8a, 0x5a, 0x1e),
            NodeType::Blur | NodeType::Glow | NodeType::Sharpen => Color32::from_rgb(0x5e, 0x3a, 0x82),
            NodeType::Keyer => Color32::from_rgb(0x24, 0x6e, 0x7a),
            NodeType::Transform => Color32::from_rgb(0x4a, 0x4a, 0x6a),
        };
        p.rect_filled(header_r, 6.0, header_color);
        p.text(header_r.min + vec2(8.0, 12.0), Align2::LEFT_CENTER, &node.name, Tokens::semibold(11.0), Color32::WHITE);

        // Node body info
        p.text(node_rect.min + vec2(8.0, 36.0), Align2::LEFT_CENTER, node.node_type.label(), Tokens::ui(10.0), Color32::from_rgb(0xaa, 0xaa, 0xb8));

        // DaVinci Solo 1 / 2 monitor dots
        let dot1_r = Rect::from_center_size(node_rect.left_bottom() + vec2(16.0, -12.0), vec2(14.0, 14.0));
        let dot1_resp = ui.interact(dot1_r, egui::Id::new(("solo1", node_id.0)), Sense::click());
        let dot1_col = if node.solo { Color32::from_rgb(0x26, 0x80, 0xeb) } else { Color32::from_rgb(0x44, 0x44, 0x52) };
        p.circle_filled(dot1_r.center(), 5.0, dot1_col);
        p.text(dot1_r.center(), Align2::CENTER_CENTER, "1", Tokens::ui(8.0), Color32::WHITE);

        if dot1_resp.clicked() {
            toggle_solo = Some(i);
        }

        // Input sockets (Left edge)
        let inputs = node.node_type.default_inputs();
        for (idx, in_name) in inputs.iter().enumerate() {
            let sy = node_rect.min.y + 24.0 + 16.0 * (idx as f32 + 1.0);
            let sock_pos = pos2(node_rect.min.x, sy);
            let sock_color = match *in_name {
                "Foreground" => Color32::from_rgb(0x3a, 0xcc, 0x5a),
                "Background" => Color32::from_rgb(0xf2, 0xa9, 0x24),
                "Mask" => Color32::from_rgb(0x3a, 0x8a, 0xf2),
                _ => Color32::from_rgb(0xd8, 0xd8, 0xd8),
            };
            p.circle_filled(sock_pos, 4.0, sock_color);
        }

        // Output socket (Right edge)
        let outputs = node.node_type.default_outputs();
        if !outputs.is_empty() {
            let sock_pos = pos2(node_rect.max.x, node_rect.min.y + 35.0);
            p.circle_filled(sock_pos, 4.0, Color32::from_rgb(0xec, 0xec, 0xec));
        }
    }

    if let Some((idx, delta)) = moved {
        graph.nodes[idx].pos[0] += delta.x;
        graph.nodes[idx].pos[1] += delta.y;
        modified = true;
    }

    if let Some(idx) = toggle_solo {
        let was_solo = graph.nodes[idx].solo;
        for n in &mut graph.nodes {
            n.solo = false;
        }
        graph.nodes[idx].solo = !was_solo;
        modified = true;
    }

    // Save changes back to clip and linear effects pipeline
    if modified {
        item.effects = graph.to_linear_effects();
        item.node_graph = Some(graph);
        save_selected_clip(app, clip_id, item);
    }
}

fn draw_grid(p: &egui::Painter, rect: Rect, step: f32, color: Color32) {
    let stroke = Stroke::new(1.0, color);
    let mut x = rect.min.x;
    while x <= rect.max.x {
        p.line_segment([pos2(x, rect.min.y), pos2(x, rect.max.y)], stroke);
        x += step;
    }
    let mut y = rect.min.y;
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
    if let Some(project) = Arc::get_mut(&mut app.session.project) {
        if let Some(seq) = project.sequence_mut(seq_id) {
            for track in &mut seq.video_tracks {
                if let Some(existing) = track.items.iter_mut().find(|it| it.id == clip_id) {
                    *existing = item;
                    return;
                }
            }
        }
    }
}

use std::sync::Arc;
