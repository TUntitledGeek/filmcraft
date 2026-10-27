//! Rendering helpers for remote collaborative peers (presence cursors, ghost drags, and soft locks).

use std::collections::HashMap;
use std::sync::Mutex;
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke};
use filmcraft_collab::{PeerPresence, SoftLock};
use filmcraft_project::TrackKind;
use filmcraft_time::Tick;

static PEER_VISUAL_CURSORS: Mutex<Option<HashMap<(filmcraft_collab::PeerId, &'static str), Pos2>>> = Mutex::new(None);
static PEER_VISUAL_PLAYHEADS: Mutex<Option<HashMap<filmcraft_collab::PeerId, f32>>> = Mutex::new(None);

/// Critically damped spring / exponential smoothing for remote peer positions (Figma-style).
/// Snaps immediately if distance > 500px (initial join or large jump).
/// Interpolates smoothly when distance > 0.5px and calls request_repaint().
/// Settles to target when distance <= 0.5px (0% idle CPU).
pub fn interpolate_cursor(ctx: &egui::Context, peer_id: filmcraft_collab::PeerId, scope: &'static str, target_pos: Pos2) -> Pos2 {
    let mut guard = PEER_VISUAL_CURSORS.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let key = (peer_id, scope);
    let visual_pos = if let Some(vis) = map.get_mut(&key) {
        let dist_sq = (target_pos - *vis).length_sq();
        if dist_sq > 250_000.0 {
            *vis = target_pos;
        } else if dist_sq > 0.25 {
            let dt = ctx.input(|i| i.stable_dt).clamp(0.001, 0.1);
            let alpha = 1.0 - (-dt * 18.0).exp();
            *vis += (target_pos - *vis) * alpha;
            ctx.request_repaint();
        } else {
            *vis = target_pos;
        }
        *vis
    } else {
        map.insert(key, target_pos);
        target_pos
    };
    visual_pos
}

/// Critically damped spring interpolation for remote peer playhead X coordinate.
pub fn interpolate_playhead_x(ctx: &egui::Context, peer_id: filmcraft_collab::PeerId, target_x: f32) -> f32 {
    let mut guard = PEER_VISUAL_PLAYHEADS.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let visual_x = if let Some(vis) = map.get_mut(&peer_id) {
        let diff = (target_x - *vis).abs();
        if diff > 500.0 {
            *vis = target_x;
        } else if diff > 0.5 {
            let dt = ctx.input(|i| i.stable_dt).clamp(0.001, 0.1);
            let alpha = 1.0 - (-dt * 18.0).exp();
            *vis += (target_x - *vis) * alpha;
            ctx.request_repaint();
        } else {
            *vis = target_x;
        }
        *vis
    } else {
        map.insert(peer_id, target_x);
        target_x
    };
    visual_x
}

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
            let target_px = time_to_x(tick);
            let px = interpolate_playhead_x(painter.ctx(), peer.peer_id, target_px);
            if px >= content_rect.min.x && px <= content_rect.max.x {
                // Secondary playhead line
                painter.line_segment(
                    [pos2(px, ruler_rect.min.y), pos2(px, content_rect.max.y)],
                    Stroke::new(1.5, col),
                );

                // Remote flag label on the ruler
                let font_id = egui::FontId::proportional(10.0);
                let galley = painter.layout_no_wrap(peer.name.clone(), font_id, Color32::WHITE);
                let badge_w = galley.size().x + 10.0;
                let badge_h = 16.0;
                let badge_rect = Rect::from_center_size(pos2(px, ruler_rect.min.y + 10.0), vec2(badge_w, badge_h));
                painter.rect_filled(badge_rect, 3.0, col);
                painter.galley(badge_rect.min + vec2(5.0, 3.0), galley, Color32::WHITE);
            }
        }

        // Ghost clip drag representation (supports single and multi-clip drags)
        if let Some(ghost) = &peer.active_drag {
            for item in ghost.all_items() {
                if let Some((top_y, bot_y)) = track_y_range(item.track_kind, item.track_index) {
                    let x0 = time_to_x(item.start_tick);
                    let x1 = time_to_x(item.start_tick + item.duration_ticks);

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
                        let badge_text = format!("{} • {}", peer.name, item.label);
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
        }

        // Remote Cursor & Off-Screen Presence in Timeline
        if let Some(panel) = peer.panel.as_deref() {
            if panel.eq_ignore_ascii_case("timeline") {
                // Determine canonical project tick and vertical track position
                let tick_opt = peer.cursor_tick.or_else(|| peer.active_drag.as_ref().map(|g| g.start_tick));
                let cy = if let Some((kind, idx)) = peer.cursor_track.or_else(|| peer.active_drag.as_ref().map(|g| (g.track_kind, g.track_index))) {
                    if let Some((top_y, bot_y)) = track_y_range(kind, idx) {
                        let ratio = peer.cursor_track_y_norm.unwrap_or(0.5);
                        top_y + ratio * (bot_y - top_y)
                    } else {
                        content_rect.center().y
                    }
                } else {
                    ruler_rect.center().y
                };

                if let Some(tick) = tick_opt {
                    let cx = time_to_x(tick);
                    if cx < content_rect.min.x {
                        // Off-screen to the LEFT (earlier in timeline)
                        paint_offscreen_indicator(
                            painter,
                            OffscreenDirection::Left,
                            content_rect.min.x + 4.0,
                            cy,
                            content_rect,
                            col,
                            &peer.name,
                            tick,
                            peer.active_drag.as_ref(),
                        );
                    } else if cx > content_rect.max.x {
                        // Off-screen to the RIGHT (later in timeline)
                        paint_offscreen_indicator(
                            painter,
                            OffscreenDirection::Right,
                            content_rect.max.x - 4.0,
                            cy,
                            content_rect,
                            col,
                            &peer.name,
                            tick,
                            peer.active_drag.as_ref(),
                        );
                    } else {
                        // Exactly ON SCREEN: Draw cursor arrow and name tag
                        let clamped_y = cy.clamp(ruler_rect.min.y, content_rect.max.y - 12.0);
                        let target_pos = pos2(cx, clamped_y);
                        let visual_pos = interpolate_cursor(painter.ctx(), peer.peer_id, "timeline", target_pos);
                        paint_pointer(painter, visual_pos, col, &peer.name, peer.in_hand.as_deref());
                    }
                } else if let Some([nx, ny]) = peer.cursor_norm {
                    // Fallback only if no tick is known (e.g. over timeline panel headers)
                    let target_pos = pos2(
                        content_rect.min.x + nx * content_rect.width(),
                        content_rect.min.y + ny * content_rect.height(),
                    );
                    let visual_pos = interpolate_cursor(painter.ctx(), peer.peer_id, "timeline", target_pos);
                    if content_rect.contains(visual_pos) {
                        paint_pointer(painter, visual_pos, col, &peer.name, peer.in_hand.as_deref());
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OffscreenDirection {
    Left,
    Right,
}

pub fn format_tick_time(t: Tick) -> String {
    let secs = t.seconds();
    let total_s = secs.floor() as u64;
    let h = total_s / 3600;
    let m = (total_s % 3600) / 60;
    let s = total_s % 60;
    let f = ((secs % 1.0) * 24.0).floor() as u32;
    if h > 0 {
        format!("{:02}:{:02}:{:02}:{:02}", h, m, s, f)
    } else {
        format!("{:02}:{:02}:{:02}", m, s, f)
    }
}

pub fn paint_offscreen_indicator(
    painter: &Painter,
    direction: OffscreenDirection,
    dock_x: f32,
    y: f32,
    clamped_bounds: Rect,
    color: Color32,
    name: &str,
    tick: Tick,
    active_drag: Option<&filmcraft_collab::DragGhost>,
) {
    let time_str = format_tick_time(tick);
    let text = match (direction, active_drag) {
        (OffscreenDirection::Left, Some(g)) => {
            let n = g.items.len().max(1);
            if n > 1 {
                format!("◀ {} [dragging +{} clips] ({})", name, n, time_str)
            } else {
                format!("◀ {} [dragging] ({})", name, time_str)
            }
        }
        (OffscreenDirection::Left, None) => {
            format!("◀ {} ({})", name, time_str)
        }
        (OffscreenDirection::Right, Some(g)) => {
            let n = g.items.len().max(1);
            if n > 1 {
                format!("{} [dragging +{} clips] ({}) ▶", name, n, time_str)
            } else {
                format!("{} [dragging] ({}) ▶", name, time_str)
            }
        }
        (OffscreenDirection::Right, None) => {
            format!("{} ({}) ▶", name, time_str)
        }
    };

    let font_id = egui::FontId::proportional(11.0);
    let galley = painter.layout_no_wrap(text, font_id, Color32::WHITE);
    let badge_size = galley.size() + vec2(14.0, 6.0);
    let clamped_y = y.clamp(
        clamped_bounds.min.y + badge_size.y * 0.5 + 2.0,
        clamped_bounds.max.y - badge_size.y * 0.5 - 2.0,
    );

    let badge_rect = match direction {
        OffscreenDirection::Left => {
            Rect::from_min_size(pos2(dock_x, clamped_y - badge_size.y * 0.5), badge_size)
        }
        OffscreenDirection::Right => {
            Rect::from_min_size(pos2(dock_x - badge_size.x, clamped_y - badge_size.y * 0.5), badge_size)
        }
    };

    // Dark sleek pill with peer outline
    painter.rect_filled(badge_rect, 4.0, Color32::from_black_alpha(225));
    painter.rect_stroke(badge_rect, 4.0, Stroke::new(1.5, color), egui::StrokeKind::Inside);
    painter.galley(badge_rect.min + vec2(7.0, 3.0), galley, Color32::WHITE);
}

/// Paint a sleek collaborative pointer with an attached name tag and optional in-hand item chip (Figma-style).
pub fn paint_pointer(painter: &Painter, pos: Pos2, color: Color32, name: &str, in_hand: Option<&str>) {
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

    // In-hand item chip (when dragging footage / media / effects)
    if let Some(item_name) = in_hand {
        let chip_text = format!("📄 {item_name}");
        let chip_galley = painter.layout_no_wrap(
            chip_text,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
        let chip_pos = pos2(tag_rect.min.x, tag_rect.max.y + 3.0);
        let chip_rect = Rect::from_min_size(chip_pos, chip_galley.size() + vec2(8.0, 4.0));
        painter.rect_filled(chip_rect, 3.0, Color32::from_black_alpha(220));
        painter.rect_stroke(chip_rect, 3.0, Stroke::new(1.5, color), egui::StrokeKind::Inside);
        painter.galley(chip_pos + vec2(4.0, 2.0), chip_galley, Color32::WHITE);
    }
}

/// Paint remote peers' cursors and active transform / drag indicators over the monitor picture.
pub fn paint_monitor_presence(painter: &Painter, peers: &[PeerPresence], pic: Rect, target_panel: &str) {
    for peer in peers {
        let col = Color32::from_rgba_unmultiplied(
            peer.color_rgba[0],
            peer.color_rgba[1],
            peer.color_rgba[2],
            peer.color_rgba[3],
        );

        // Remote cursor on monitor picture
        if let (Some(panel), Some([nx, ny])) = (peer.panel.as_deref(), peer.cursor_norm) {
            let is_match = if target_panel.eq_ignore_ascii_case("program") {
                panel.eq_ignore_ascii_case("programpic")
            } else {
                panel.eq_ignore_ascii_case("sourcepic")
            };
            if is_match {
                let target_pos = pos2(
                    pic.min.x + nx * pic.width(),
                    pic.min.y + ny * pic.height(),
                );
                let visual_pos = interpolate_cursor(painter.ctx(), peer.peer_id, "monitor", target_pos);
                if pic.contains(visual_pos) {
                    paint_pointer(painter, visual_pos, col, &peer.name, peer.in_hand.as_deref());
                }
            }
        }

        // Remote active drag or transform indicator
        if let Some(ghost) = &peer.active_drag {
            let is_match = if target_panel.eq_ignore_ascii_case("program") {
                peer.panel.as_deref().is_some_and(|p| {
                    p.eq_ignore_ascii_case("programpic")
                        || p.eq_ignore_ascii_case("program")
                        || p.eq_ignore_ascii_case("monitor")
                })
            } else {
                peer.panel.as_deref().is_some_and(|p| {
                    p.eq_ignore_ascii_case("sourcepic") || p.eq_ignore_ascii_case("source")
                })
            };
            if is_match {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tick_time_formats_hours_minutes_seconds_frames() {
        let t1 = Tick::from_seconds_f64(0.0);
        assert_eq!(format_tick_time(t1), "00:00:00");

        let t2 = Tick::from_seconds_f64(12.5);
        assert_eq!(format_tick_time(t2), "00:12:12");

        let t3 = Tick::from_seconds_f64(3665.25);
        assert_eq!(format_tick_time(t3), "01:01:05:06");
    }

    #[test]
    fn offscreen_direction_enum_works() {
        assert_ne!(OffscreenDirection::Left, OffscreenDirection::Right);
    }

    #[test]
    fn peer_spring_interpolation_tracks_and_settles() {
        let ctx = egui::Context::default();
        let p1 = pos2(100.0, 100.0);
        let res1 = interpolate_cursor(&ctx, 42, "test", p1);
        assert_eq!(res1, p1);

        // Teleport snap (>500px)
        let p_teleport = pos2(1000.0, 1000.0);
        let res_teleport = interpolate_cursor(&ctx, 42, "test", p_teleport);
        assert_eq!(res_teleport, p_teleport);

        // Smooth move (small step)
        let p_step = pos2(1010.0, 1010.0);
        let res_step = interpolate_cursor(&ctx, 42, "test", p_step);
        assert!(res_step.x > 1000.0 && res_step.x <= 1010.0);
    }
}
