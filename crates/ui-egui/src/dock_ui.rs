//! FilmCraft's panel chrome and document tabs over the shared docking transactions.

use craft_ui::docking::{Action, DropTarget, Placement, Zone, resolve_drop};
use egui::{Color32, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use serde_json::{Value, json};

use crate::dock::{self, DockAction, DockNode, Group, PanelKind};
use crate::icons::{self, Icon};
use crate::{FilmcraftApp, panels};

/// Single source for rendering and hit testing: saved offscreen positions remain
/// reachable, and every panel has enough room for its chrome.
fn floating_rect(group: &dock::Floating, screen: Rect) -> Rect {
    let rect = Rect::from_min_size(pos2(group.rect[0], group.rect[1]), vec2(group.rect[2].max(160.0), group.rect[3].max(100.0)));
    rect.translate(vec2(
        (screen.min.x - rect.min.x).max(0.0) + (screen.max.x - 40.0 - rect.min.x).min(0.0),
        (screen.min.y - rect.min.y).max(0.0) + (screen.max.y - 30.0 - rect.min.y).min(0.0),
    ))
}

fn drag_id() -> egui::Id {
    egui::Id::new("dock-panel-drag")
}

pub(crate) fn route(app: &mut FilmcraftApp, ctx: &egui::Context, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if id == "window.dockAction" {
        return Some((|| {
            let value = params.get("action").cloned().ok_or("missing docking action")?;
            let action = serde_json::from_value(value).map_err(|error| format!("invalid docking action: {error}"))?;
            app.apply_dock(action)?;
            Ok(Value::Null)
        })());
    }
    if !matches!(id, "window.dockPanel" | "window.floatPanel" | "window.closePanel") {
        return None;
    }
    Some((|| {
        let panel = match params.get("panel") {
            None => app.ui.focused,
            Some(value) => value.as_str().and_then(PanelKind::from_name).ok_or("unknown panel")?,
        };
        let action = match id {
            "window.closePanel" => Action::Close { panel },
            "window.floatPanel" => {
                let rect = match params.get("rect") {
                    None => default_float_rect(ctx.content_rect()),
                    Some(value) => serde_json::from_value::<[f32; 4]>(value.clone()).map_err(|_| "rect must contain four finite numbers")?,
                };
                Action::Float { panel, rect }
            }
            _ => {
                let zone_name = match params.get("zone") {
                    None => "center",
                    Some(value) => value.as_str().ok_or("zone must be a string")?,
                };
                let zone = match zone_name.to_ascii_lowercase().as_str() {
                    "left" => Zone::Left,
                    "right" => Zone::Right,
                    "top" => Zone::Top,
                    "bottom" => Zone::Bottom,
                    "center" | "tab" => Zone::Center,
                    _ => return Err("unknown docking zone".into()),
                };
                let Some(anchor) = params.get("anchor") else {
                    if params.get("before").is_some() {
                        return Err("before needs an anchor panel".into());
                    }
                    app.dock_to_edge(panel, zone)?;
                    return Ok(json!({"panel": panel.id()}));
                };
                let anchor = anchor.as_str().and_then(PanelKind::from_name).ok_or("a known anchor panel is required")?;
                let placement = match params.get("before") {
                    Some(value) => Placement::Tab { before: Some(value.as_str().and_then(PanelKind::from_name).ok_or("unknown before panel")?) },
                    None => Placement::Split(zone),
                };
                Action::Move { panel, anchor, placement }
            }
        };
        app.apply_dock(action)?;
        Ok(json!({"panel": panel.id()}))
    })())
}

pub(crate) fn default_float_rect(screen: Rect) -> [f32; 4] {
    let center = screen.center();
    [center.x - 210.0, center.y - 160.0, 420.0, 320.0]
}

impl FilmcraftApp {
    fn dock_command(&mut self, ctx: &egui::Context, action: Action<PanelKind>) -> Result<(), String> {
        let value = serde_json::to_value(action).map_err(|error| error.to_string())?;
        crate::menus::invoke(self, ctx, "window.dockAction", json!({"action": value})).map(|_| ())
    }

    pub fn apply_dock(&mut self, action: Action<PanelKind>) -> Result<(), String> {
        let relocating = matches!(&action, Action::Move { .. } | Action::Float { .. });
        let reveal = match &action {
            Action::Activate { panel } => Some(*panel),
            _ => None,
        };
        // a panel closed while floating remembers where; closed while docked, it forgets (#1985)
        let closing = match &action {
            Action::Close { panel } => Some((*panel, self.ui.floating.iter().find(|g| g.panels.contains(panel)).map(|g| g.rect))),
            _ => None,
        };
        self.commit_dock(|shared| {
            crate::dock_shared::check_last_docked(shared, &action)?;
            shared.apply(action).map_err(|error| error.to_string())
        })?;
        if let Some(panel) = reveal {
            self.pending_panel_raise = Some(panel);
        }
        if let Some((panel, rect)) = closing {
            self.ui.closed_floating.retain(|(p, _)| *p != panel);
            if let Some(rect) = rect {
                self.ui.closed_floating.push((panel, rect));
            }
        }
        if relocating
            || self.ui.keys.maximized.is_some_and(|panel| !self.ui.dock.contains(panel) && !self.ui.floating.iter().any(|group| group.panels.contains(&panel)))
        {
            self.ui.keys.maximized = None;
        }
        Ok(())
    }

    /// Dock a panel along an edge of the whole workspace rather than beside one group.
    pub fn dock_to_edge(&mut self, panel: PanelKind, zone: Zone) -> Result<(), String> {
        self.commit_dock(|shared| crate::dock_shared::dock_at_edge(shared, panel, zone))?;
        self.ui.keys.maximized = None;
        Ok(())
    }

    fn commit_dock(&mut self, change: impl FnOnce(&mut craft_ui::docking::Layout<PanelKind>) -> Result<(), String>) -> Result<(), String> {
        let mut shared = crate::dock_shared::encode(&self.ui.dock, &self.ui.floating)?;
        shared.validate().map_err(|error| error.to_string())?;
        change(&mut shared)?;
        let layout = crate::dock_shared::decode(shared)?;
        crate::dock_shared::encode(&layout.root, &layout.floating)?;
        self.ui.dock = layout.root;
        self.ui.floating = layout.floating;
        Ok(())
    }

    fn panel_body(&mut self, ui: &mut egui::Ui, panel: PanelKind, content: Rect) {
        self.visible_panels.push((panel.id(), content));
        self.auto.add(&format!("panel.{}", panel.id()), content, panel.title());
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content).id_salt(("panel", panel.id())));
        child.set_clip_rect(content.intersect(ui.clip_rect()));
        panels::show(self, &mut child, panel, content);

        // Collaborative presence: render remote collaborator cursors in this active panel!
        if let Some(collab) = &self.collab
            && let Ok(peers) = collab.peers.try_read()
        {
            let panel_name = panel.id();
            // Timeline has dedicated tracks overlay; ProgramPic / SourcePic are handled on monitor pic
            if panel != PanelKind::Timeline {
                for peer in peers.values() {
                    if peer.panel.as_deref().is_some_and(|p| p.eq_ignore_ascii_case(&panel_name))
                        && let Some([nx, ny]) = peer.cursor_norm
                    {
                        let cursor_pos = pos2(content.min.x + nx * content.width(), content.min.y + ny * content.height());
                        let col = Color32::from_rgba_unmultiplied(peer.color_rgba[0], peer.color_rgba[1], peer.color_rgba[2], peer.color_rgba[3]);
                        crate::collab_overlay::paint_pointer(child.painter(), cursor_pos, col, &peer.name, peer.in_hand.as_deref());
                    }
                }
            }
        }
    }

    /// OS drag positions come from the host cursor hook, so egui pointer capture alone
    /// cannot decide whether a Timeline is covered. Use the same geometry and layer
    /// ordering as our renderer before any panel can dispatch an edit.
    pub(crate) fn timeline_drop_target(&self, ctx: &egui::Context, pos: egui::Pos2) -> bool {
        if !ctx.content_rect().contains(pos) {
            return false;
        }
        if let Some(panel) =
            self.ui.keys.maximized.filter(|panel| self.ui.dock.contains(*panel) || self.ui.floating.iter().any(|group| group.panels.contains(panel)))
        {
            return panel == PanelKind::Timeline;
        }
        for index in self.floating_order(ctx).into_iter().rev() {
            if let Some(group) = self.ui.floating.get(index)
                && floating_rect(group, ctx.content_rect()).contains(pos)
            {
                return group.panels.get(group.active) == Some(&PanelKind::Timeline);
            }
        }
        true
    }

    fn floating_order(&self, ctx: &egui::Context) -> Vec<usize> {
        let layers = ctx.memory(|memory| memory.layer_ids().collect::<Vec<_>>());
        let mut order: Vec<_> = (0..self.ui.floating.len()).collect();
        order.sort_by_key(|index| {
            self.ui
                .floating
                .get(*index)
                .and_then(|group| group.panels.first())
                .and_then(|panel| {
                    let id = egui::Id::new(("floating-panel", panel.id()));
                    layers.iter().position(|layer| layer.id == id)
                })
                .unwrap_or(usize::MAX)
        });
        order
    }

    pub(crate) fn dock_area(&mut self, ui: &mut egui::Ui, body: Rect) {
        self.visible_panels.clear();
        self.program_pic_rect = None;
        self.source_pic_rect = None;
        self.collab_drop_ghost = None;
        let t = self.tokens;
        let ctx = ui.ctx().clone();
        if let Some(panel) = self.pending_panel_raise.take()
            && let Some(owner) = self.ui.floating.iter().find(|group| group.panels.contains(&panel)).and_then(|group| group.panels.first())
        {
            ctx.move_to_top(egui::LayerId::new(egui::Order::Middle, egui::Id::new(("floating-panel", owner.id()))));
        }
        let maximized =
            self.ui.keys.maximized.filter(|panel| self.ui.dock.contains(*panel) || self.ui.floating.iter().any(|group| group.panels.contains(panel)));
        let mut dock = match maximized {
            Some(panel) => DockNode::Tabs { panels: vec![panel], active: 0 },
            None => std::mem::replace(&mut self.ui.dock, DockNode::Tabs { panels: vec![], active: 0 }),
        };
        let mut groups = vec![];
        let mut layout_actions = Vec::new();
        let mut hidden = Vec::new();
        if maximized.is_none()
            && self.ui.workspace == "Clean Editing"
            && self.ui.source_closed
            && !self.ui.pin_source
            && self.session.state.source_item.is_none()
        {
            hidden.push(PanelKind::Source);
        }
        // A saved/custom layout always keeps the user's geometry. Contextual hiding is a
        // rendering policy of the new built-in preset, never a migration of their dock tree.
        if maximized.is_none()
            && self.ui.workspace == "Clean Editing"
            && (self.ui.workspace_preview.is_some() || !self.workspaces.saved.iter().any(|s| s.name == self.ui.workspace))
        {
            if self.session.state.source_item.is_none() && !self.ui.pin_source {
                hidden.push(PanelKind::Source);
            }
            if self.session.state.selection.is_empty() && !self.ui.pin_inspector {
                hidden.extend([PanelKind::Properties, PanelKind::EffectControls]);
            }
        }
        // Built-in workspaces share the horizontal toolbar. Saved/custom layouts retain a
        // docked Tools panel; their timeline then omits the duplicate horizontal controls.
        if maximized.is_none() && !self.workspaces.saved.iter().any(|s| s.name == self.ui.workspace) && dock.contains(PanelKind::Timeline) {
            hidden.push(PanelKind::Tools);
        }
        dock::layout_contextual(ui, &mut dock, body, &t, "", &mut groups, &mut self.auto, &mut layout_actions, &hidden);
        let seqs = dock::SeqTabs {
            open: self
                .session
                .state
                .open_sequences
                .iter()
                .filter_map(|id| {
                    self.session
                        .project
                        .item(*id)
                        .filter(|item| matches!(item.kind, filmcraft_project::ItemKind::Sequence(_)))
                        .map(|item| (id.0, item.name.clone()))
                })
                .collect(),
            active: self.session.state.active_sequence.map(|id| id.0),
            tool: (!groups.iter().any(|g| g.panels.get(g.active) == Some(&PanelKind::Tools))
                && !self.ui.floating.iter().any(|g| g.panels.get(g.active) == Some(&PanelKind::Tools)))
            .then_some(self.ui.tool),
        };
        let mut actions = vec![];
        let mut targets = vec![];
        for group in &groups {
            let chrome = dock::draw_group_chrome_details(ui, group, &seqs, &t, &mut self.auto);
            if let Some(rect) = chrome.tools {
                panels::tools::horizontal(self, ui, rect);
            }
            if let Some(collab) = &self.collab
                && let Ok(peers) = collab.peers.try_read()
            {
                for (panel_kind, tab_rect) in &chrome.tabs {
                    let panel_id = panel_kind.id();
                    for peer in peers.values() {
                        let is_on_panel = peer.panel.as_deref().is_some_and(|p| {
                            p.eq_ignore_ascii_case(&panel_id)
                                || (panel_id == "Program" && p.eq_ignore_ascii_case("ProgramPic"))
                                || (panel_id == "Source" && p.eq_ignore_ascii_case("SourcePic"))
                        });
                        if is_on_panel && tab_rect.width() > 0.0 {
                            let dot_col = Color32::from_rgba_unmultiplied(peer.color_rgba[0], peer.color_rgba[1], peer.color_rgba[2], peer.color_rgba[3]);
                            let dot_pos = pos2(tab_rect.max.x - 6.0, tab_rect.min.y + 6.0);
                            ui.painter().circle_filled(dot_pos, 3.0, dot_col);
                        }
                    }
                }
            }
            if let Some(target) = target(group, chrome.tabs, true) {
                targets.push(target);
            }
            actions.extend(chrome.actions);
        }
        if self.ui.workspace == "Clean Editing" {
            for group in &groups {
                let strip_y = group.rect.min.y;
                let p = ui.painter();
                // Source group header: subtle pin and close buttons
                if group.panels.get(group.active) == Some(&PanelKind::Source) {
                    let pin_active = self.ui.pin_source;
                    let pin_r = Rect::from_min_size(pos2(group.rect.max.x - 52.0, strip_y + (t.tab_h - 22.0) / 2.0), vec2(22.0, 22.0));
                    let pin_resp = ui.interact(pin_r, egui::Id::new("viewer-source-pin"), Sense::click()).on_hover_text(if pin_active {
                        "Unpin Source viewer"
                    } else {
                        "Pin Source viewer (keep dual layout)"
                    });
                    self.auto.add_response("editing.sourcePin", &pin_resp, "Pin Source viewer");
                    if pin_active {
                        p.rect_filled(
                            pin_r,
                            4.0,
                            if t.kind == crate::theme::ThemeKind::Light { t.pill_active_bg } else { Color32::from_rgb(0x27, 0x39, 0x54) },
                        );
                        p.rect_stroke(pin_r, 4.0, Stroke::new(1.0, t.accent), StrokeKind::Inside);
                    } else if pin_resp.hovered() {
                        p.rect_filled(pin_r, 4.0, t.hover);
                    }
                    let pin_col = if pin_active {
                        t.accent
                    } else if pin_resp.hovered() {
                        t.tab_text_active
                    } else {
                        t.icon
                    };
                    let ir = pin_r.shrink(5.0);
                    p.line_segment([pos2(ir.center().x, ir.min.y + 2.0), pos2(ir.center().x, ir.max.y - 2.0)], Stroke::new(1.3, pin_col));
                    p.line_segment([pos2(ir.min.x + 2.0, ir.min.y + 4.0), pos2(ir.max.x - 2.0, ir.min.y + 4.0)], Stroke::new(1.3, pin_col));
                    p.circle_filled(pos2(ir.center().x, ir.min.y + 2.0), 2.0, pin_col);
                    if pin_resp.clicked() {
                        self.ui.pin_source = !self.ui.pin_source;
                    }
                    // Close button (returns to single Program viewer)
                    let close_r = Rect::from_min_size(pos2(group.rect.max.x - 26.0, strip_y + (t.tab_h - 22.0) / 2.0), vec2(22.0, 22.0));
                    let close_resp =
                        ui.interact(close_r, egui::Id::new("viewer-source-close"), Sense::click()).on_hover_text("Close Source viewer (single Program)");
                    self.auto.add_response("editing.singleProgram", &close_resp, "Single Program");
                    if close_resp.hovered() {
                        p.rect_filled(close_r, 4.0, t.hover);
                    }
                    let close_col = if close_resp.hovered() { t.tab_text_active } else { t.text_dim };
                    icons::paint(p, Rect::from_center_size(close_r.center(), vec2(11.0, 11.0)), Icon::Close, close_col);
                    if close_resp.clicked() {
                        self.stop_source();
                        self.ui.source_closed = true;
                        self.ui.pin_source = false;
                        self.session.state.source_item = None;
                    }
                }
                // Program group header: subtle split-view toggle button
                if group.panels.get(group.active) == Some(&PanelKind::Program) {
                    let dual_active = groups.iter().any(|g| g.panels.get(g.active) == Some(&PanelKind::Source));
                    let split_r = Rect::from_min_size(pos2(group.rect.max.x - 30.0, strip_y + (t.tab_h - 22.0) / 2.0), vec2(24.0, 22.0));
                    let split_resp = ui.interact(split_r, egui::Id::new("viewer-split-toggle"), Sense::click()).on_hover_text(if dual_active {
                        "Hide Source viewer (Single Program)"
                    } else {
                        "Split View: Show Source beside Program"
                    });
                    self.auto.add_response("editing.pinSource", &split_resp, "Keep Source open");
                    if dual_active {
                        p.rect_filled(
                            split_r,
                            4.0,
                            if t.kind == crate::theme::ThemeKind::Light { t.pill_active_bg } else { Color32::from_rgb(0x27, 0x39, 0x54) },
                        );
                        p.rect_stroke(split_r, 4.0, Stroke::new(1.0, t.accent), StrokeKind::Inside);
                    } else if split_resp.hovered() {
                        p.rect_filled(split_r, 4.0, t.hover);
                    }
                    let split_col = if dual_active {
                        t.accent
                    } else if split_resp.hovered() {
                        t.tab_text_active
                    } else {
                        t.icon
                    };
                    let ir = split_r.shrink(4.0);
                    p.rect_stroke(ir, 2.0, Stroke::new(1.2, split_col), StrokeKind::Inside);
                    let div_x = ir.center().x;
                    p.line_segment([pos2(div_x, ir.min.y), pos2(div_x, ir.max.y)], Stroke::new(1.0, split_col));
                    if dual_active {
                        p.rect_filled(Rect::from_min_max(ir.min, pos2(div_x, ir.max.y)), 1.0, split_col.gamma_multiply(0.3));
                    }
                    if split_resp.clicked() {
                        if dual_active {
                            self.stop_source();
                            self.ui.source_closed = true;
                            self.ui.pin_source = false;
                            self.session.state.source_item = None;
                        } else {
                            self.ui.source_closed = false;
                            self.ui.pin_source = true;
                        }
                    }
                }
            }
            self.auto.add("editing.pinInspector", Rect::ZERO, "Keep Inspector open");
        }
        if maximized.is_none() {
            self.ui.dock = dock;
        }
        // Apply editing gestures before monitors snapshot the project. Otherwise a Lumetri
        // drag queued an obsolete revision first, spending a whole cooperative render job
        // on the old value before the next UI pass could request the new one.
        // This changes paint order only; docking geometry and persisted order stay intact.
        groups.sort_by_key(|group| match group.panels.get(group.active) {
            Some(PanelKind::Timeline) => 0,
            Some(PanelKind::LumetriColor | PanelKind::EffectControls | PanelKind::Properties | PanelKind::NodeGraph) => 1,
            Some(PanelKind::Project | PanelKind::MediaBrowser) => 2,
            Some(PanelKind::Program | PanelKind::Source) => 3,
            _ => 4,
        });
        for group in &groups {
            if let Some(panel) = group.panels.get(group.active).copied() {
                self.panel_body(ui, panel, group.content);
            }
        }
        for group in &groups {
            dock::paint_group_focus(ui, group, self.ui.focused, &t);
        }
        let floating_order = self.floating_order(&ctx);
        for index in floating_order.into_iter().filter(|_| maximized.is_none()) {
            if let Some(target) = self.floating_group(&ctx, index, &seqs, &mut actions) {
                targets.push(target);
            }
        }
        for action in layout_actions {
            let _ = self.dock_command(&ctx, action);
        }
        for action in actions {
            match action {
                DockAction::Activate(panel) => {
                    let _ = self.dock_command(&ctx, Action::Activate { panel });
                }
                DockAction::Close(panel) => {
                    let _ = self.dock_command(&ctx, Action::Close { panel });
                }
                DockAction::Focus(panel) => self.ui.focused = panel,
                DockAction::BeginDrag(panel) => {
                    ctx.data_mut(|data| data.insert_temp(drag_id(), panel));
                }
                DockAction::OpenSequence(id) => {
                    let _ = self.dock_command(&ctx, Action::Activate { panel: PanelKind::Timeline });
                    if self.session.state.active_sequence.map(|item| item.0) != Some(id)
                        && let Err(error) = self.session.execute("sequence.open", json!({"item": id}))
                    {
                        self.ui.status = error.to_string();
                    }
                }
                DockAction::CloseSequence(id) => {
                    if let Err(error) = self.session.execute("sequence.close", json!({"item": id})) {
                        self.ui.status = error.to_string();
                    }
                }
                DockAction::MoveSequence(id, index) => {
                    if let Err(error) = self.session.execute("sequence.moveTab", json!({"item": id, "index": index})) {
                        self.ui.status = error.to_string();
                    }
                }
                DockAction::PanelMenu(panel, pos) => {
                    let frame = ctx.cumulative_frame_nr();
                    ctx.data_mut(|data| {
                        data.insert_temp(egui::Id::new("panel-menu"), (panel, pos));
                        data.insert_temp(egui::Id::new("panel-menu-opened"), frame);
                    });
                }
                DockAction::SelectTool(tool) => self.ui.tool = tool,
            }
        }
        self.panel_drag(&ctx, body, &targets);
        panels::panel_menu_popup(self, ui);
        self.cancel_value_drag(ui.ctx());
    }

    fn floating_group(&mut self, ctx: &egui::Context, index: usize, seqs: &dock::SeqTabs, actions: &mut Vec<DockAction>) -> Option<DropTarget<PanelKind>> {
        let floating = self.ui.floating.get(index)?.clone();
        let panel = *floating.panels.first()?;
        let rect = floating_rect(&floating, ctx.content_rect());
        let t = self.tokens;
        let id = egui::Id::new(("floating-panel", panel.id()));
        let mut output = None;
        egui::Area::new(id).order(egui::Order::Middle).fixed_pos(rect.min).constrain(false).interactable(true).show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(rect.size(), Sense::hover());
            let compact = floating.panels.len() == 1 && panel.compact();
            let header = if compact { 8.0 } else { t.tab_h };
            let strip = Rect::from_min_size(rect.min, vec2(rect.width(), header));
            let response = ui.interact(strip, id.with("move"), Sense::drag());
            self.auto.add_response(&format!("panel.float.{index}.move"), &response, "Move floating panel");
            let mut next = if response.dragged() { rect.translate(response.drag_delta()) } else { rect };
            let group = Group {
                path: format!("f{index}"),
                rect,
                content: Rect::from_min_max(pos2(rect.min.x, rect.min.y + header), rect.max),
                panels: floating.panels.clone(),
                active: floating.active,
            };
            let chrome = dock::draw_group_chrome_details(ui, &group, seqs, &t, &mut self.auto);
            if let Some(rect) = chrome.tools {
                panels::tools::horizontal(self, ui, rect);
            }
            if let Some(collab) = &self.collab
                && let Ok(peers) = collab.peers.try_read()
            {
                for (panel_kind, tab_rect) in &chrome.tabs {
                    let panel_id = panel_kind.id();
                    for peer in peers.values() {
                        let is_on_panel = peer.panel.as_deref().is_some_and(|p| {
                            p.eq_ignore_ascii_case(&panel_id)
                                || (panel_id == "Program" && p.eq_ignore_ascii_case("ProgramPic"))
                                || (panel_id == "Source" && p.eq_ignore_ascii_case("SourcePic"))
                        });
                        if is_on_panel && tab_rect.width() > 0.0 {
                            let dot_col = Color32::from_rgba_unmultiplied(peer.color_rgba[0], peer.color_rgba[1], peer.color_rgba[2], peer.color_rgba[3]);
                            let dot_pos = pos2(tab_rect.max.x - 6.0, tab_rect.min.y + 6.0);
                            ui.painter().circle_filled(dot_pos, 3.0, dot_col);
                        }
                    }
                }
            }
            actions.extend(chrome.actions);
            if let Some(active) = group.panels.get(group.active).copied() {
                self.panel_body(ui, active, group.content);
            }
            ui.painter().rect_stroke(rect, t.radius, Stroke::new(1.0, t.focus), StrokeKind::Outside);
            let grip = Rect::from_min_max(rect.max - vec2(14.0, 14.0), rect.max);
            let response = ui.interact(grip, id.with("resize"), Sense::click_and_drag());
            response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, response.enabled(), tl!("Resize floating panel")));
            self.auto.add_response(&format!("panel.float.{index}.resize"), &response, "Resize floating panel");
            for step in 0..3 {
                let offset = 4.0 + step as f32 * 4.0;
                ui.painter()
                    .line_segment([pos2(rect.max.x - offset, rect.max.y - 2.0), pos2(rect.max.x - 2.0, rect.max.y - offset)], Stroke::new(1.0, t.text_faint));
            }
            if response.hovered() || response.dragged() {
                ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
            }
            let delta = floating_resize_delta(ui, &response);
            if response.has_focus() {
                ui.painter().rect_stroke(grip, 2.0, Stroke::new(1.5, t.focus), StrokeKind::Inside);
            }
            if delta != egui::Vec2::ZERO {
                next = Rect::from_min_size(next.min, (next.size() + delta).max(vec2(160.0, 100.0)));
            }
            if next != rect {
                let _ = self.dock_command(ctx, Action::MoveFloating { panel, rect: [next.min.x, next.min.y, next.width(), next.height()] });
            }
            dock::paint_group_focus(ui, &group, self.ui.focused, &t);
            output = target(&group, chrome.tabs, false);
        });
        output
    }

    fn panel_drag(&mut self, ctx: &egui::Context, body: Rect, targets: &[DropTarget<PanelKind>]) {
        let Some(panel) = ctx.data(|data| data.get_temp::<PanelKind>(drag_id())) else { return };
        let canceled =
            ctx.input(|input| input.key_pressed(egui::Key::Escape) || !input.focused || (!input.pointer.primary_down() && !input.pointer.primary_released()));
        if canceled || (!self.ui.dock.contains(panel) && !self.ui.floating.iter().any(|group| group.panels.contains(&panel))) {
            ctx.data_mut(|data| data.remove::<PanelKind>(drag_id()));
            return;
        }
        let Some(pos) = ctx.pointer_latest_pos() else { return };
        let force_float = ctx.input(|input| input.modifiers.command);
        let edge = (!force_float).then(|| workspace_edge(body, pos, targets)).flatten();
        let proposal = (!force_float && edge.is_none()).then(|| resolve_drop(&panel, pos, targets)).flatten();
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("dock-drop")));
        let t = self.tokens;
        if let Some((_, _, preview)) = edge {
            painter.rect_filled(preview.shrink(2.0), t.radius, t.focus.gamma_multiply(0.2));
            painter.rect_stroke(preview.shrink(2.0), t.radius, Stroke::new(2.0, t.focus), StrokeKind::Inside);
        }
        if let Some(proposal) = &proposal {
            painter.rect_filled(proposal.preview.shrink(2.0), t.radius, t.focus.gamma_multiply(0.2));
            painter.rect_stroke(proposal.preview.shrink(2.0), t.radius, Stroke::new(2.0, t.focus), StrokeKind::Inside);
            if let Some(mark) = proposal.insertion {
                painter.rect_filled(mark, 0.0, t.focus);
            }
        }
        let title = painter.layout_no_wrap(panel.title().to_owned(), crate::theme::Tokens::ui(12.0), Color32::WHITE);
        let ghost = Rect::from_min_size(pos + vec2(12.0, 10.0), title.size() + vec2(16.0, 8.0));
        painter.rect_filled(ghost, 4.0, t.focus.gamma_multiply(0.85));
        painter.galley(ghost.min + vec2(8.0, 4.0), title, Color32::WHITE);
        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        ctx.request_repaint();
        if ctx.input(|input| input.pointer.primary_released()) {
            ctx.data_mut(|data| data.remove::<PanelKind>(drag_id()));
            if let Some((_, zone, _)) = edge {
                if crate::menus::invoke(self, ctx, "window.dockPanel", json!({"panel": panel.id(), "zone": zone})).is_ok() {
                    self.ui.focused = panel;
                }
                return;
            }
            let action = match proposal {
                Some(proposal) => Some(Action::Move { panel, anchor: proposal.anchor, placement: proposal.placement }),
                None if force_float || !targets.iter().any(|target| target.rect.contains(pos)) => {
                    Some(Action::Float { panel, rect: [pos.x - 40.0, pos.y - 12.0, 420.0, 320.0] })
                }
                None => None,
            };
            if let Some(action) = action
                && self.dock_command(ctx, action).is_ok()
            {
                self.ui.focused = panel;
            }
        }
    }
}

/// Width of the band along the workspace border that docks beside the whole layout.
const EDGE_BAND: f32 = 8.0;

/// The outermost drop zone: near the workspace border a drop splits the root, not the group
/// under the pointer. A floating group over the border (it rejects splits) keeps the drop.
fn workspace_edge(body: Rect, pos: egui::Pos2, targets: &[DropTarget<PanelKind>]) -> Option<(Zone, &'static str, Rect)> {
    if !body.is_finite() || !body.contains(pos) || targets.iter().any(|target| !target.allow_splits && target.rect.contains(pos)) {
        return None;
    }
    let band = EDGE_BAND.min(body.width() / 4.0).min(body.height() / 4.0);
    let edges = [
        (pos.x - body.left(), Zone::Left, "left"),
        (body.right() - pos.x, Zone::Right, "right"),
        (pos.y - body.top(), Zone::Top, "top"),
        (body.bottom() - pos.y, Zone::Bottom, "bottom"),
    ];
    let (distance, zone, name) = edges.into_iter().min_by(|a, b| a.0.total_cmp(&b.0))?;
    if distance > band {
        return None;
    }
    let size = body.size() * 0.25;
    let preview = match zone {
        Zone::Left => Rect::from_min_size(body.min, vec2(size.x, body.height())),
        Zone::Right => Rect::from_min_max(pos2(body.right() - size.x, body.top()), body.max),
        Zone::Top => Rect::from_min_size(body.min, vec2(body.width(), size.y)),
        Zone::Bottom | Zone::Center => Rect::from_min_max(pos2(body.left(), body.bottom() - size.y), body.max),
    };
    Some((zone, name, preview))
}

fn target(group: &Group, mut tabs: Vec<(PanelKind, Rect)>, allow_splits: bool) -> Option<DropTarget<PanelKind>> {
    let strip = Rect::from_min_max(group.rect.min, pos2(group.rect.max.x, group.content.min.y));
    // A compact grip still identifies the source group to the shared resolver.
    for panel in &group.panels {
        if !tabs.iter().any(|(candidate, _)| candidate == panel) {
            tabs.push((*panel, Rect::NOTHING));
        }
    }
    Some(DropTarget { anchor: *group.panels.get(group.active)?, rect: group.rect, tab_strip: Some(strip), tabs, allow_splits, accepts_tabs: true })
}

/// A corner is a two-axis resize control; focused arrows belong to it and Tab moves on.
fn floating_resize_delta(ui: &mut egui::Ui, response: &egui::Response) -> egui::Vec2 {
    if response.drag_started() {
        response.request_focus();
    }
    let mut delta = if response.dragged() { response.drag_delta() } else { egui::Vec2::ZERO };
    if response.has_focus() {
        ui.memory_mut(|memory| {
            memory.set_focus_lock_filter(response.id, egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() })
        });
        ui.input_mut(|input| {
            for (key, amount) in [
                (egui::Key::ArrowLeft, vec2(-8.0, 0.0)),
                (egui::Key::ArrowRight, vec2(8.0, 0.0)),
                (egui::Key::ArrowUp, vec2(0.0, -8.0)),
                (egui::Key::ArrowDown, vec2(0.0, 8.0)),
            ] {
                if input.consume_key(egui::Modifiers::NONE, key) {
                    delta += amount;
                }
            }
        });
    }
    if response.enabled() { delta } else { egui::Vec2::ZERO }
}
