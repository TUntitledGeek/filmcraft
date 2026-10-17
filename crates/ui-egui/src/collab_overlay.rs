//! Rendering helpers for remote collaborative peers (presence cursors, ghost drags, and soft locks).

use egui::{pos2, vec2, Align2, Color32, Painter, Pos2, Rect, Stroke};
use filmcraft_collab::{PeerPresence, SoftLock};
use filmcraft_project::TrackKind;
use filmcraft_time::Tick;

pub struct CollabRenderCtx<'a> {
    pub peers: &'a [PeerPresence],
    pub locks: &'a [SoftLock],
}

/// Paint remote peers' playheads, cursors, and ghost clip drag boxes on the timeline panel.
pub fn paint_timeline_presence(
    painter: &Painter,
    peers: &[PeerPresence],
    locks: &[SoftLock],
    content_rect: Rect,
    ruler_rect: Rect,
    time_to_x: impl Fn(Tick) -> f32,
    track_y_range: impl Fn(TrackKind, usize) -> Option<(f32, f32)>,
) {
    // 1. Soft track locks (tinting locked tracks with diagonal hash / banner)
    for lock in locks {
        if let Some((top_y, bot_y)) = track_y_range(lock.track_kind, lock.track_index) {
            let lock_rect = Rect::from_min_max(
                pos2(content_rect.min.x, top_y),
                pos2(content_rect.max.x, bot_y),
            );
            // Translucent caution background
            painter.rect_filled(lock_rect, 0.0, Color32::from_rgba_unmultiplied(220, 80, 0, 30));
            // Top border
            painter.line_segment(
                [pos2(lock_rect.min.x, top_y), pos2(lock_rect.max.x, top_y)],
                Stroke::new(1.5, Color32::from_rgb(220, 100, 0)),
            );
        }
    }

    // 2. Peer presence: remote playheads & ghost clip dragging
    for peer in peers {
        let col = Color32::from_rgba_unmultiplied(
            peer.color_rgba[0],
            peer.color_rgba[1],
            peer.color_rgba[2],
            peer.color_rgba[3],
        );

        // Remote Playhead
        if let Some(tick) = peer.playhead_tick {
            let px = time_to_x(tick);
            if px >= content_rect.min.x && px <= content_rect.max.x {
                // Secondary playhead line
                painter.line_segment(
                    [pos2(px, ruler_rect.min.y), pos2(px, content_rect.max.y)],
                    Stroke::new(1.5, col),
                );

                // Remote flag label on the ruler
                let badge_rect = Rect::from_min_size(pos2(px - 14.0, ruler_rect.min.y + 2.0), vec2(28.0, 14.0));
                painter.rect_filled(badge_rect, 3.0, col);
                painter.text(
                    badge_rect.center(),
                    Align2::CENTER_CENTER,
                    &peer.name,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        // Ghost clip drag representation
        if let Some(ghost) = &peer.active_drag {
            if let Some((top_y, bot_y)) = track_y_range(ghost.track_kind, ghost.track_index) {
                let x0 = time_to_x(ghost.start_tick);
                let x1 = time_to_x(ghost.start_tick + ghost.duration_ticks);

                let min_x = x0.min(x1);
                let max_x = x0.max(x1).max(min_x + 12.0);
                let raw_rect = Rect::from_min_max(pos2(min_x, top_y + 2.0), pos2(max_x, bot_y - 2.0));

                if raw_rect.intersects(content_rect) {
                    let ghost_rect = raw_rect.intersect(content_rect);
                    let p = painter.with_clip_rect(content_rect);

                    // 1. Semi-translucent glowing backdrop (vibrant 130 alpha)
                    p.rect_filled(
                        ghost_rect,
                        4.0,
                        Color32::from_rgba_unmultiplied(peer.color_rgba[0], peer.color_rgba[1], peer.color_rgba[2], 130),
                    );

                    // 2. High-contrast border (2.5px solid peer stroke)
                    p.rect_stroke(
                        ghost_rect,
                        4.0,
                        Stroke::new(2.5, col),
                        egui::StrokeKind::Inside,
                    );

                    // 3. Diagonal animated-feel hatch stripes
                    let mut stripe_x = ghost_rect.min.x;
                    let stripe_col = Color32::from_rgba_unmultiplied(255, 255, 255, 40);
                    while stripe_x < ghost_rect.max.x {
                        p.line_segment(
                            [pos2(stripe_x, ghost_rect.max.y), pos2((stripe_x + ghost_rect.height() * 0.7).min(ghost_rect.max.x), ghost_rect.min.y)],
                            Stroke::new(1.5, stripe_col),
                        );
                        stripe_x += 18.0;
                    }

                    // 4. Prominent high-contrast floating tag
                    let badge_text = format!("{} • {}", peer.name, ghost.label);
                    let font_id = egui::FontId::proportional(11.0);
                    let badge_pos = pos2(ghost_rect.min.x + 6.0, ghost_rect.min.y + 4.0);

                    let galley = p.layout_no_wrap(badge_text, font_id, Color32::WHITE);
                    let badge_bg = Rect::from_min_size(badge_pos - vec2(3.0, 2.0), galley.size() + vec2(6.0, 4.0));
                    p.rect_filled(badge_bg, 3.0, Color32::from_black_alpha(200));
                    p.rect_stroke(badge_bg, 3.0, Stroke::new(1.5, col), egui::StrokeKind::Inside);
                    p.galley(badge_pos, galley, Color32::WHITE);
                }
            }
        }

        // Remote Cursor in Timeline
        if let (Some("timeline"), Some([nx, ny])) = (peer.panel.as_deref(), peer.cursor_norm) {
            let cursor_pos = pos2(
                content_rect.min.x + nx * content_rect.width(),
                content_rect.min.y + ny * content_rect.height(),
            );
            paint_pointer(painter, cursor_pos, col, &peer.name);
        }
    }
}

/// Paint a sleek collaborative pointer with an attached name tag (Figma-style).
pub fn paint_pointer(painter: &Painter, pos: Pos2, color: Color32, name: &str) {
    // Arrow polygon
    let tip = pos;
    let arrow = vec![
        tip,
        tip + vec2(0.0, 16.0),
        tip + vec2(4.5, 12.0),
        tip + vec2(12.0, 12.0),
    ];
    painter.add(egui::Shape::convex_polygon(arrow, color, Stroke::new(1.0, Color32::BLACK)));

    // Name tag badge
    let tag_pos = tip + vec2(14.0, 10.0);
    let galley = painter.layout_no_wrap(
        name.to_string(),
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );
    let tag_rect = Rect::from_min_size(tag_pos - vec2(3.0, 2.0), galley.size() + vec2(6.0, 4.0));
    painter.rect_filled(tag_rect, 3.0, color);
    painter.galley(tag_pos, galley, Color32::WHITE);
}

/// Paint remote peers' cursors and active transform / drag indicators over the monitor picture.
pub fn paint_monitor_presence(painter: &Painter, peers: &[PeerPresence], pic: Rect) {
    for peer in peers {
        let col = Color32::from_rgba_unmultiplied(
            peer.color_rgba[0],
            peer.color_rgba[1],
            peer.color_rgba[2],
            peer.color_rgba[3],
        );

        // Remote cursor on monitor
        if let (Some("program" | "monitor"), Some([nx, ny])) = (peer.panel.as_deref(), peer.cursor_norm) {
            let cursor_pos = pos2(
                pic.min.x + nx * pic.width(),
                pic.min.y + ny * pic.height(),
            );
            if pic.contains(cursor_pos) {
                paint_pointer(painter, cursor_pos, col, &peer.name);
            }
        }

        // Remote active drag or transform indicator
        if let Some(ghost) = &peer.active_drag {
            if peer.panel.as_deref() == Some("program") || peer.panel.as_deref() == Some("monitor") {
                let ghost_rect = Rect::from_center_size(pic.center(), pic.size() * 0.5);
                painter.rect_stroke(ghost_rect, 2.0, Stroke::new(1.5, col), egui::StrokeKind::Outside);
                let badge_pos = ghost_rect.left_top() - vec2(0.0, 18.0);
                let text = format!("{}: {}", peer.name, ghost.label);
                let galley = painter.layout_no_wrap(text, egui::FontId::proportional(11.0), Color32::WHITE);
                let badge_rect = Rect::from_min_size(badge_pos, galley.size() + vec2(6.0, 4.0));
                painter.rect_filled(badge_rect, 3.0, col);
                painter.galley(badge_pos + vec2(3.0, 2.0), galley, Color32::WHITE);
            }
        }
    }
}
