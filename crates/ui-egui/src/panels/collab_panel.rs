//! Collaborative Editing (Co-op) Session Dialog, Progressive Proxy Ladder, and Extension Opt-in Modal.

use egui::{Color32, RichText, vec2};
use crate::FilmcraftApp;

/// Returns the base URL for generating invite links.
/// On web: dynamically uses the current browser origin + path (works seamlessly on localhost, GitHub Pages, or custom domains).
/// On desktop: defaults to the official web app URL.
pub fn get_base_invite_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Ok(loc) = w.location().href() {
                let clean = loc.split('?').next().unwrap_or(&loc);
                let clean = clean.split('#').next().unwrap_or(clean);
                return clean.to_string();
            }
        }
        "https://tuntitledgeek.github.io/filmcraft/".to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "https://tuntitledgeek.github.io/filmcraft/".to_string()
    }
}

/// Builds a 1-click invite URL that automatically joins the room upon browser load.
pub fn build_invite_url(room_code: &str, passcode: Option<&str>) -> String {
    let base = get_base_invite_url();
    let room = room_code.trim();
    let key_suffix = match passcode {
        Some(p) if !p.trim().is_empty() => format!("#key={}", p.trim()),
        _ => String::new(),
    };
    format!("{base}?room={room}{key_suffix}")
}

pub fn show_collab_dialog(app: &mut FilmcraftApp, ctx: &egui::Context) {
    if !app.collab_dialog_open {
        return;
    }

    let mut open = app.collab_dialog_open;
    egui::Window::new("Live Co-op Editing & Room Hub")
        .open(&mut open)
        .resizable(false)
        .default_size(vec2(420.0, 460.0))
        .show(ctx, |ui| {
            let collab_state = app.collab.as_ref().map(|c| (c.config.clone(), c.peers.clone()));
            if let Some((config, peers_lock)) = collab_state {
                // ==========================================
                // 1. CONNECTED STATE (Active Session)
                // ==========================================
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(0, 230, 110)).size(18.0));
                    ui.heading(format!("Room: {}", config.room_code));
                });
                ui.add_space(4.0);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let role_text = if app.collab_is_host {
                            RichText::new("👑 Host (Broadcaster)").color(Color32::from_rgb(255, 200, 80)).strong()
                        } else {
                            RichText::new("✏️ Collaborator (Synced)").color(Color32::from_rgb(100, 180, 255)).strong()
                        };
                        ui.label(role_text);

                        ui.separator();
                        let sec_text = if config.passcode.is_some() {
                            RichText::new("🔒 Passcode Protected").color(Color32::from_rgb(240, 180, 60))
                        } else {
                            RichText::new("🌐 Public Room").color(Color32::from_rgb(120, 210, 120))
                        };
                        ui.label(sec_text);
                    });
                    ui.label(RichText::new(format!("Editing as: {}", config.user_name)).weak());
                });

                ui.add_space(8.0);
                // Prominent 1-Click Invite Card
                ui.group(|ui| {
                    ui.label(RichText::new("Shareable 1-Click Invite Link:").strong());
                    let invite_url = build_invite_url(&config.room_code, config.passcode.as_deref());
                    let mut display_url = invite_url.clone();
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut display_url).desired_width(260.0));
                        let copy_btn = egui::Button::new(RichText::new("📋 Copy Link").strong());
                        if ui.add(copy_btn).clicked() {
                            ctx.copy_text(invite_url);
                            app.ui.status = "Copied 1-click invite link! Send it to your friend to join instantly.".into();
                        }
                    });
                    ui.label(RichText::new("Collaborators opening this link will immediately enter this room and load the project.").size(11.0).weak());
                });

                // Connected Editors
                ui.add_space(8.0);
                ui.label(RichText::new("Connected Editors:").strong());
                ui.group(|ui| {
                    // Local user indicator
                    let local_color = if app.collab_is_host {
                        Color32::from_rgb(60, 140, 255)
                    } else {
                        Color32::from_rgb(255, 120, 60)
                    };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("●").color(local_color));
                        let me_role = if app.collab_is_host { " (Host, You)" } else { " (You)" };
                        ui.label(format!("{}{}", config.user_name, me_role));
                    });

                    if let Ok(peers) = peers_lock.try_read() {
                        if peers.is_empty() {
                            ui.label(RichText::new("(Waiting for collaborators to join... Share the link above!)").weak().italics());
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
                                    ui.label(format!("{} (Peer #{})", peer.name, id));
                                });
                            }
                        }
                    }
                });

                // Google Drive Cloud Hub Card
                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("☁ Cloud Asset Hub:").strong());
                        if app.gdrive_config.is_authenticated() {
                            ui.colored_label(Color32::from_rgb(0, 200, 100), "Google Drive Connected");
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
                    ui.add_space(6.0);
                    ui.label(RichText::new("Progressive Media Streams:").strong());
                    for tier in &app.collab_media_tiers {
                        ui.horizontal(|ui| {
                            ui.label(&tier.filename);
                            ui.add(egui::ProgressBar::new(tier.progress_pct).show_percentage());
                            ui.label(&tier.status);
                        });
                    }
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let disc_btn = egui::Button::new(RichText::new("Leave & Disconnect").color(Color32::from_rgb(255, 90, 90)).strong());
                    if ui.add(disc_btn).clicked() {
                        app.disconnect_collab();
                        app.ui.status = "Disconnected from session".into();
                    }
                    if ui.button("Close Window").clicked() {
                        app.collab_dialog_open = false;
                    }
                });
            } else {
                // ==========================================
                // 2. NOT CONNECTED STATE (Create vs Join Tabs)
                // ==========================================
                let is_host = app.collab_is_host;
                let avail_width = ui.available_width();
                let tab_width = (avail_width - 8.0) / 2.0;

                ui.horizontal(|ui| {
                    let host_tab = egui::Button::new(
                        RichText::new("👑 Create & Host Room")
                            .size(13.0)
                            .strong()
                            .color(if is_host { Color32::WHITE } else { Color32::from_rgb(180, 180, 180) })
                    )
                    .fill(if is_host { Color32::from_rgb(35, 85, 160) } else { Color32::from_rgb(45, 45, 52) });

                    let join_tab = egui::Button::new(
                        RichText::new("🔗 Join Existing Room")
                            .size(13.0)
                            .strong()
                            .color(if !is_host { Color32::WHITE } else { Color32::from_rgb(180, 180, 180) })
                    )
                    .fill(if !is_host { Color32::from_rgb(35, 85, 160) } else { Color32::from_rgb(45, 45, 52) });

                    if ui.add_sized([tab_width, 32.0], host_tab).clicked() {
                        app.collab_is_host = true;
                    }
                    if ui.add_sized([tab_width, 32.0], join_tab).clicked() {
                        app.collab_is_host = false;
                    }
                });
                ui.add_space(8.0);

                if app.collab_is_host {
                    // ------------------------------------------
                    // TAB: CREATE & HOST A ROOM
                    // ------------------------------------------
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("ℹ").color(Color32::from_rgb(80, 160, 255)).strong());
                            ui.label(RichText::new("Host your current project. Anyone with your invite link can join and edit the timeline with you in real-time.").size(11.5).weak());
                        });
                    });
                    ui.add_space(6.0);

                    ui.label(RichText::new("Your Name (Host):").strong());
                    if ui.text_edit_singleline(&mut app.collab_name_input).changed() {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                            let _ = storage.set_item("filmcraft_collab_name", &app.collab_name_input);
                        }
                    }

                    ui.add_space(6.0);
                    ui.label(RichText::new("Room Code:").strong());
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut app.collab_room_input).desired_width(220.0));
                        if ui.button("🎲 New Code").clicked() {
                            app.collab_room_input = crate::generate_random_room_code();
                        }
                    });
                    ui.label(RichText::new("Unique room code for your session.").size(11.0).weak());

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Room Passcode (Optional):").strong());
                        let eye = if app.collab_passcode_visible { "👁 Show" } else { "🔒 Mask" };
                        if ui.small_button(eye).clicked() {
                            app.collab_passcode_visible = !app.collab_passcode_visible;
                        }
                    });
                    ui.add(egui::TextEdit::singleline(&mut app.collab_passcode_input).password(!app.collab_passcode_visible));
                    if app.collab_passcode_input.trim().is_empty() {
                        ui.label(RichText::new("🔓 Public Room — Anyone with your invite link can join directly.").size(11.0).weak());
                    } else {
                        ui.label(RichText::new("🔒 Passcode required to enter (embedded in invite link).").size(11.0).color(Color32::from_rgb(255, 200, 80)));
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("Listen Port:");
                            ui.add(egui::TextEdit::singleline(&mut app.collab_port_input).desired_width(70.0));
                        });
                    }

                    ui.add_space(10.0);
                    let start_btn = egui::Button::new(
                        RichText::new("🚀 Start Hosting Room").size(14.0).strong().color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgb(32, 140, 60));

                    if ui.add_sized([avail_width, 36.0], start_btn).clicked() {
                        let port: u16 = app.collab_port_input.trim().parse().unwrap_or(9090);
                        let name = if app.collab_name_input.trim().is_empty() { "Host".into() } else { app.collab_name_input.clone() };
                        let room = if app.collab_room_input.trim().is_empty() { crate::generate_random_room_code() } else { app.collab_room_input.clone() };
                        app.collab_room_input = room.clone();
                        let passcode = if app.collab_passcode_input.trim().is_empty() { None } else { Some(app.collab_passcode_input.clone()) };
                        match app.start_collab_host(port, &name, &room, passcode.as_deref()) {
                            Ok(()) => {
                                app.ui.status = format!("Hosting co-op room '{room}'. Copy the 1-click link to invite friends!");
                            }
                            Err(e) => {
                                app.ui.status = format!("Failed to host: {e}");
                            }
                        }
                    }

                    // Shareable Link Preview
                    ui.add_space(8.0);
                    ui.group(|ui| {
                        ui.label(RichText::new("1-Click Shareable Invite Link:").strong());
                        let invite_url = build_invite_url(&app.collab_room_input, Some(app.collab_passcode_input.trim()));
                        let mut display_url = invite_url.clone();
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut display_url).desired_width(260.0));
                            if ui.button("📋 Copy Link").clicked() {
                                ctx.copy_text(invite_url);
                                app.ui.status = "Copied 1-click invite link to clipboard!".into();
                            }
                        });
                        ui.label(RichText::new("Send this link to collaborators. When opened in their browser, they will automatically join.").size(11.0).weak());
                    });
                } else {
                    // ------------------------------------------
                    // TAB: JOIN AN EXISTING ROOM
                    // ------------------------------------------
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("💡").color(Color32::from_rgb(255, 200, 60)).strong());
                            ui.label(RichText::new("If your friend sent you an invite link, simply open it in your browser — you enter the room automatically!").size(11.5).strong());
                        });
                    });
                    ui.add_space(6.0);

                    ui.label(RichText::new("Your Display Name:").strong());
                    if ui.text_edit_singleline(&mut app.collab_name_input).changed() {
                        #[cfg(target_arch = "wasm32")]
                        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                            let _ = storage.set_item("filmcraft_collab_name", &app.collab_name_input);
                        }
                    }
                    ui.label(RichText::new("Your cursor and edits will appear under this name.").size(11.0).weak());

                    ui.add_space(6.0);
                    ui.label(RichText::new("Room Code:").strong());
                    ui.text_edit_singleline(&mut app.collab_room_input);
                    ui.label(RichText::new("Enter the 6-character room code from your host (e.g. FC-7890).").size(11.0).weak());

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Room Passcode (If required):").strong());
                        let eye = if app.collab_passcode_visible { "👁 Show" } else { "🔒 Mask" };
                        if ui.small_button(eye).clicked() {
                            app.collab_passcode_visible = !app.collab_passcode_visible;
                        }
                    });
                    ui.add(egui::TextEdit::singleline(&mut app.collab_passcode_input).password(!app.collab_passcode_visible));
                    ui.label(RichText::new("Leave empty if the host room is public.").size(11.0).weak());

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.add_space(4.0);
                        ui.label("Host Address (IP:Port):");
                        ui.text_edit_singleline(&mut app.collab_addr_input);
                    }

                    ui.add_space(12.0);
                    let join_btn = egui::Button::new(
                        RichText::new("🔗 Connect & Enter Room").size(14.0).strong().color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgb(35, 95, 180));

                    if ui.add_sized([avail_width, 36.0], join_btn).clicked() {
                        let addr = if cfg!(target_arch = "wasm32") { "127.0.0.1:9090".to_string() } else { app.collab_addr_input.trim().to_string() };
                        let name = if app.collab_name_input.trim().is_empty() { "Editor".into() } else { app.collab_name_input.clone() };
                        let room = if app.collab_room_input.trim().is_empty() { "ROOM-1".into() } else { app.collab_room_input.clone() };
                        let passcode = if app.collab_passcode_input.trim().is_empty() { None } else { Some(app.collab_passcode_input.clone()) };
                        match app.join_collab_room(&addr, &name, &room, passcode.as_deref()) {
                            Ok(()) => {
                                app.ui.status = format!("Connected to room '{room}'! Syncing timeline...");
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
