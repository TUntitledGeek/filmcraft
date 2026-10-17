//! Collaborative Editing (Co-op) Session Dialog and Extension Opt-in Modal.

use egui::{Color32, RichText, vec2};
use crate::FilmcraftApp;

pub fn show_collab_dialog(app: &mut FilmcraftApp, ctx: &egui::Context) {
    if !app.collab_dialog_open {
        return;
    }

    let mut open = app.collab_dialog_open;
    egui::Window::new("Live Co-op Editing")
        .open(&mut open)
        .resizable(false)
        .default_size(vec2(360.0, 300.0))
        .show(ctx, |ui| {
            if let Some(collab) = &app.collab {
                // Connected state
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(0, 220, 100)));
                    ui.heading("Connected");
                });
                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.label(format!("Room Code: {}", collab.config.room_code));
                    ui.label(format!("Your Name: {}", collab.config.user_name));
                    let role = if app.collab_is_host { "Host" } else { "Client" };
                    ui.label(format!("Role: {}", role));
                });

                ui.add_space(10.0);
                ui.label(RichText::new("Connected Collaborators:").strong());

                if let Ok(peers) = collab.peers.try_read() {
                    if peers.is_empty() {
                        ui.label(RichText::new("(No other editors connected)").weak());
                    } else {
                        for (id, peer) in peers.iter() {
                            let col = Color32::from_rgba_unmultiplied(
                                peer.color_rgba[0],
                                peer.color_rgba[1],
                                peer.color_rgba[2],
                                peer.color_rgba[3],
                            );
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("●").color(col));
                                ui.label(format!("{} (Peer {})", peer.name, id));
                            });
                        }
                    }
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Disconnect").color(Color32::from_rgb(255, 80, 80))).clicked() {
                        app.disconnect_collab();
                    }
                    if ui.button("Close").clicked() {
                        app.collab_dialog_open = false;
                    }
                });
            } else {
                // Not connected state
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut app.collab_is_host, true, "Host Session");
                    ui.selectable_value(&mut app.collab_is_host, false, "Join Session");
                });
                ui.add_space(8.0);

                ui.label("Your Display Name:");
                ui.text_edit_singleline(&mut app.collab_name_input);

                ui.add_space(4.0);
                ui.label("Room Code:");
                ui.text_edit_singleline(&mut app.collab_room_input);

                if app.collab_is_host {
                    ui.add_space(4.0);
                    ui.label("Listen Port:");
                    ui.text_edit_singleline(&mut app.collab_port_input);

                    ui.add_space(12.0);
                    if ui.button(RichText::new("Start Hosting Session").strong()).clicked() {
                        let port: u16 = app.collab_port_input.trim().parse().unwrap_or(9090);
                        let name = if app.collab_name_input.trim().is_empty() { "Host".into() } else { app.collab_name_input.clone() };
                        let room = if app.collab_room_input.trim().is_empty() { "ROOM-1".into() } else { app.collab_room_input.clone() };
                        match app.start_collab_host(port, &name, &room) {
                            Ok(()) => {
                                app.ui.status = format!("Hosting co-op room '{room}' on port {port}");
                            }
                            Err(e) => {
                                app.ui.status = format!("Failed to host: {e}");
                            }
                        }
                    }
                } else {
                    ui.add_space(4.0);
                    ui.label("Host Address (IP:Port):");
                    ui.text_edit_singleline(&mut app.collab_addr_input);

                    ui.add_space(12.0);
                    if ui.button(RichText::new("Connect to Host").strong()).clicked() {
                        let addr = app.collab_addr_input.trim().to_string();
                        let name = if app.collab_name_input.trim().is_empty() { "Editor".into() } else { app.collab_name_input.clone() };
                        let room = if app.collab_room_input.trim().is_empty() { "ROOM-1".into() } else { app.collab_room_input.clone() };
                        match app.join_collab_room(&addr, &name, &room) {
                            Ok(()) => {
                                app.ui.status = format!("Connecting to '{addr}'...");
                            }
                            Err(e) => {
                                app.ui.status = format!("Failed to connect: {e}");
                            }
                        }
                    }
                }
            }
        });
    app.collab_dialog_open = open;
}

pub fn show_extension_offer_modal(app: &mut FilmcraftApp, ctx: &egui::Context) {
    let Some(offer) = app.pending_extension_offer.clone() else { return };
    let mut close = false;

    egui::Window::new("Collaborator Extension Offer")
        .collapsible(false)
        .resizable(false)
        .default_size(vec2(380.0, 220.0))
        .show(ctx, |ui| {
            ui.heading(&offer.name);
            ui.label(format!("Version: {}", offer.version));
            ui.label(format!("ID: {}", offer.extension_id));
            ui.add_space(4.0);
            ui.label(&offer.description);
            ui.add_space(8.0);

            ui.colored_label(Color32::from_rgb(220, 160, 0), "Note: Only install extensions from people you trust.");

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Install Extension").strong()).clicked() {
                    app.ui.status = format!("Installed extension '{}'", offer.name);
                    close = true;
                }
                if ui.button("Decline").clicked() {
                    close = true;
                }
            });
        });

    if close {
        app.pending_extension_offer = None;
    }
}
