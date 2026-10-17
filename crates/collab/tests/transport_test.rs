use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use filmcraft_collab::protocol::{CollabMessage, PeerPresence};
use filmcraft_collab::session::{CollabConfig, CollabSession};
use filmcraft_collab::transport::{CollabEvent, CollabTransport};
use serde_json::json;

#[test]
fn test_transport_host_and_client_connect_and_sync() {
    let host_addr: SocketAddr = "127.0.0.1:18942".parse().unwrap();

    let host_config = CollabConfig {
        peer_id: 1,
        user_name: "Alice Host".into(),
        user_color: [255, 60, 60, 255],
        bind_addr: host_addr,
        room_code: "ROOM-123".into(),
        passcode: None,
    };
    let host_session = Arc::new(CollabSession::new(host_config));
    let host_transport = CollabTransport::new(host_session.clone(), true).unwrap();
    host_transport.start_host(host_addr).unwrap();

    // Give host listener 50ms to bind
    std::thread::sleep(Duration::from_millis(50));

    let client_config = CollabConfig {
        peer_id: 2,
        user_name: "Bob Client".into(),
        user_color: [60, 180, 255, 255],
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        room_code: "ROOM-123".into(),
        passcode: None,
    };
    let client_session = Arc::new(CollabSession::new(client_config));
    let client_transport = CollabTransport::new(client_session.clone(), false).unwrap();
    client_transport.connect_to(host_addr);

    // Wait up to 2 seconds for connection handshake
    let mut client_connected_on_host = false;
    let mut host_connected_on_client = false;

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        while let Some(evt) = host_transport.try_recv_event() {
            if let CollabEvent::PeerConnected { peer_id, name, .. } = evt {
                if peer_id == 2 && name == "Bob Client" {
                    client_connected_on_host = true;
                }
            }
        }
        while let Some(evt) = client_transport.try_recv_event() {
            if let CollabEvent::PeerConnected { peer_id, name, .. } = evt {
                if peer_id == 1 && name == "Alice Host" {
                    host_connected_on_client = true;
                }
            }
        }
        if client_connected_on_host && host_connected_on_client {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(client_connected_on_host, "Host did not receive PeerConnected for Client");
    assert!(host_connected_on_client, "Client did not receive PeerConnected for Host");

    // Client broadcasts a command
    let cmd = CollabMessage::CommitCommand {
        peer_id: 2,
        rev: 1,
        id: "timeline.split".into(),
        params: json!({"time": 1000}),
    };
    client_session.msg_tx.send(cmd).unwrap();

    // Host should receive CommandReceived
    let mut command_received = false;
    let cmd_start = std::time::Instant::now();
    while cmd_start.elapsed() < Duration::from_secs(2) {
        while let Some(evt) = host_transport.try_recv_event() {
            if let CollabEvent::CommandReceived { peer_id, id, params, .. } = evt {
                if peer_id == 2 && id == "timeline.split" && params["time"] == 1000 {
                    command_received = true;
                    break;
                }
            }
        }
        if command_received {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(command_received, "Host did not receive CommandReceived event");

    // Host broadcasts presence
    let presence_msg = CollabMessage::Presence(PeerPresence {
        peer_id: 1,
        name: "Alice Host".into(),
        color_rgba: [255, 60, 60, 255],
        panel: Some("timeline".into()),
        playhead_tick: Some(filmcraft_time::Tick(500)),
        cursor_norm: Some([0.25, 0.5]),
        cursor_tick: Some(filmcraft_time::Tick(500)),
        cursor_track: Some((filmcraft_project::TrackKind::Video, 0)),
        cursor_track_y_norm: Some(0.5),
        active_drag: None,
        selected_clip: None,
        selected_clips: Vec::new(),
        in_hand: None,
        timestamp_ms: 12345,
    });
    host_session.msg_tx.send(presence_msg).unwrap();

    // Client session peers map should update
    let mut presence_updated = false;
    let pres_start = std::time::Instant::now();
    while pres_start.elapsed() < Duration::from_secs(2) {
        if let Ok(peers) = client_session.peers.try_read() {
            if let Some(p) = peers.get(&1) {
                if p.playhead_tick == Some(filmcraft_time::Tick(500)) {
                    presence_updated = true;
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(presence_updated, "Client did not receive Presence update in peers map");

    host_transport.shutdown();
    client_transport.shutdown();
}

#[test]
fn test_passcode_protected_room_connect_success() {
    let host_addr: SocketAddr = "127.0.0.1:18943".parse().unwrap();

    let host_config = CollabConfig {
        peer_id: 1,
        user_name: "Host Secure".into(),
        user_color: [255, 60, 60, 255],
        bind_addr: host_addr,
        room_code: "SECRET-ROOM".into(),
        passcode: Some("passcode987".into()),
    };
    let host_session = Arc::new(CollabSession::new(host_config));
    let host_transport = CollabTransport::new(host_session.clone(), true).unwrap();
    host_transport.start_host(host_addr).unwrap();
    std::thread::sleep(Duration::from_millis(50));

    let client_config = CollabConfig {
        peer_id: 2,
        user_name: "Guest Secure".into(),
        user_color: [60, 180, 255, 255],
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        room_code: "SECRET-ROOM".into(),
        passcode: Some("passcode987".into()),
    };
    let client_session = Arc::new(CollabSession::new(client_config));
    let client_transport = CollabTransport::new(client_session.clone(), false).unwrap();
    client_transport.connect_to(host_addr);

    let mut authenticated = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        while let Some(evt) = host_transport.try_recv_event() {
            if let CollabEvent::PeerConnected { peer_id, name, .. } = evt {
                if peer_id == 2 && name == "Guest Secure" {
                    authenticated = true;
                    break;
                }
            }
        }
        if authenticated {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(authenticated, "Peer with correct passcode should be authenticated and registered");
    host_transport.shutdown();
    client_transport.shutdown();
}

#[test]
fn test_passcode_protected_room_wrong_passcode_rejected() {
    let host_addr: SocketAddr = "127.0.0.1:18944".parse().unwrap();

    let host_config = CollabConfig {
        peer_id: 1,
        user_name: "Host Secure".into(),
        user_color: [255, 60, 60, 255],
        bind_addr: host_addr,
        room_code: "SECRET-ROOM".into(),
        passcode: Some("correct_passcode".into()),
    };
    let host_session = Arc::new(CollabSession::new(host_config));
    let host_transport = CollabTransport::new(host_session.clone(), true).unwrap();
    host_transport.start_host(host_addr).unwrap();
    std::thread::sleep(Duration::from_millis(50));

    let client_config = CollabConfig {
        peer_id: 2,
        user_name: "Guest Attacker".into(),
        user_color: [60, 180, 255, 255],
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        room_code: "SECRET-ROOM".into(),
        passcode: Some("wrong_passcode".into()),
    };
    let client_session = Arc::new(CollabSession::new(client_config));
    let client_transport = CollabTransport::new(client_session.clone(), false).unwrap();
    client_transport.connect_to(host_addr);

    let mut client_auth_failed = false;
    let mut host_auth_failed = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        while let Some(evt) = client_transport.try_recv_event() {
            if let CollabEvent::AuthFailed { .. } = evt {
                client_auth_failed = true;
            }
        }
        while let Some(evt) = host_transport.try_recv_event() {
            if let CollabEvent::AuthFailed { .. } = evt {
                host_auth_failed = true;
            }
        }
        if client_auth_failed && host_auth_failed {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(client_auth_failed, "Client should receive AuthFailed on incorrect passcode");
    assert!(host_auth_failed, "Host should record AuthFailed on incorrect passcode");
    host_transport.shutdown();
    client_transport.shutdown();
}
