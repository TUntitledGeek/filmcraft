//! Collaborative Editing (Co-op) Session Dialog, Progressive Proxy Ladder, and Extension Opt-in Modal.

use egui::{Color32, RichText, vec2};
use crate::FilmcraftApp;

pub fn show_collab_dialog(app: &mut FilmcraftApp, ctx: &egui::Context) {
    if !app.collab_dialog_open {
        return;
    }

    let mut open = app.collab_dialog_open;
    egui::Window::new("Live Co-op Editing & Cloud Hub")
        .open(&mut open)
        .resizable(false)
        .default_size(vec2(380.0, 360.0))
        .show(ctx, |ui| {
            let collab_state = app.collab.as_ref().map(|c| (c.config.clone(), c.peers.clone()));
            if let Some((config, peers_lock)) = collab_state {
                // Connected state
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(0, 220, 100)));
                    ui.heading("Connected to Session");
                });
                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.label(format!("Room Code: {}", config.room_code));
                    ui.label(format!("Your Name: {}", config.user_name));
                    let role = if app.collab_is_host { "Host" } else { "Client" };
                    ui.label(format!("Role: {}", role));
                    let protected = if config.passcode.is_some() { "🔒 Passcode Protected" } else { "Public Room" };
                    ui.label(format!("Security: {}", protected));
                });

                // Google Drive Cloud Hub Card
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("☁ Google Drive Cloud Hub:").strong());
                        if app.gdrive_config.is_authenticated() {
                            ui.colored_label(Color32::from_rgb(0, 200, 100), "Connected");
                        } else if ui.small_button("Connect Google Drive").clicked() {
                            let url = app.gdrive_config.authorization_url("http://127.0.0.1:8765/oauth/callback", "collab_auth");
                            crate::links::open(app, ctx, &url);
                            app.ui.status = "Opened Google Drive authorization in browser".into();
                        }
                    });
                    ui.label(RichText::new(format!("Sync Folder: {}", app.gdrive_config.root_folder_name)).weak());
                });

                // Progressive Media Stream Ladder
                if !app.collab_media_tiers.is_empty() {
                    ui.add_space(8.0);
                    ui.label(RichText::new("Progressive Media Streams:").strong());
                    for tier in &app.collab_media_tiers {
                        ui.horizontal(|ui| {
                            ui.label(&tier.filename);
                            ui.add(egui::ProgressBar::new(tier.progress_pct).show_percentage());
                            ui.label(&tier.status);
                        });
                    }
                }

                ui.add_space(10.0);
                ui.label(RichText::new("Connected Collaborators:").strong());

                if let Ok(peers) = peers_lock.try_read() {
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

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Room Passcode (Optional):");
                    let eye = if app.collab_passcode_visible { "👁 Show" } else { "🔒 Mask" };
                    if ui.small_button(eye).clicked() {
                        app.collab_passcode_visible = !app.collab_passcode_visible;
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut app.collab_passcode_input).password(!app.collab_passcode_visible));

                if app.collab_is_host {
                    ui.add_space(4.0);
                    ui.label("Listen Port:");
                    ui.text_edit_singleline(&mut app.collab_port_input);

                    ui.add_space(8.0);
                    let invite_url = format!(
                        "https://filmcraft.online/editor?room={}&key={}",
                        app.collab_room_input.trim(),
                        app.collab_passcode_input.trim()
                    );
                    if ui.button("📋 Copy 1-Click Protected Invite Link").clicked() {
                        ctx.copy_text(invite_url);
                        app.ui.status = "Copied 1-click invite link to clipboard".into();
                    }

                    ui.add_space(12.0);
                    if ui.button(RichText::new("Start Hosting Session").strong()).clicked() {
                        let port: u16 = app.collab_port_input.trim().parse().unwrap_or(9090);
                        let name = if app.collab_name_input.trim().is_empty() { "Host".into() } else { app.collab_name_input.clone() };
                        let room = if app.collab_room_input.trim().is_empty() { "ROOM-1".into() } else { app.collab_room_input.clone() };
                        let passcode = if app.collab_passcode_input.trim().is_empty() { None } else { Some(app.collab_passcode_input.clone()) };
                        match app.start_collab_host(port, &name, &room, passcode.as_deref()) {
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
                        let passcode = if app.collab_passcode_input.trim().is_empty() { None } else { Some(app.collab_passcode_input.clone()) };
                        match app.join_collab_room(&addr, &name, &room, passcode.as_deref()) {
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
