//! Network transport layer for P2P collaborative editing over TCP.
//!
//! Provides framed asynchronous messaging, connection lifecycle management,
//! automatic heartbeat keep-alives, soft-lock lease pruning, and event dispatch
//! to the UI thread via non-blocking channels.

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::mpsc::{Receiver as StdReceiver, Sender as StdSender, channel as std_channel};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use log::{error, info};
use log::warn;
#[cfg(not(target_arch = "wasm32"))]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(not(target_arch = "wasm32"))]
use tokio::net::{TcpListener, TcpStream};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::{broadcast, mpsc, watch, RwLock};

#[cfg(not(target_arch = "wasm32"))]
use crate::protocol::{compute_auth_hmac, CollabMessage, PROTOCOL_VERSION};
use crate::protocol::PeerId;
use crate::session::CollabSession;

/// Maximum payload size allowed for framed network packets (64 MB).
pub const MAX_PACKET_BYTES: usize = 64 * 1024 * 1024;

/// Heartbeat interval (1 second).
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(1000);

/// Peer timeout threshold: peer is considered disconnected if no message/heartbeat in 5 seconds.
pub const PEER_TIMEOUT: Duration = Duration::from_millis(5000);

/// Events emitted by the transport layer to the main thread / UI loop.
#[derive(Clone, Debug)]
pub enum CollabEvent {
    PeerConnected {
        peer_id: PeerId,
        name: String,
        color_rgba: [u8; 4],
    },
    PeerDisconnected {
        peer_id: PeerId,
    },
    CommandReceived {
        peer_id: PeerId,
        rev: u64,
        id: String,
        params: serde_json::Value,
    },
    SnapshotRequested {
        peer_id: PeerId,
    },
    SnapshotReceived {
        project_json: String,
        revision: u64,
        active_sequence: Option<u64>,
    },
    RollbackReceived {
        peer_id: PeerId,
        revision: u64,
        label: String,
        project_json: String,
        active_sequence: Option<u64>,
    },
    AssetReceived {
        category: String,
        filename: String,
        sha256: String,
        data: Vec<u8>,
    },
    ExtensionOffered {
        extension_id: String,
        name: String,
        version: String,
        description: String,
        sha256: String,
    },
    MediaOffered {
        file_id: String,
        filename: String,
        sha256: String,
        size_bytes: u64,
        is_proxy: bool,
    },
    MediaChunkReceived {
        sha256: String,
        offset_bytes: u64,
        total_bytes: u64,
        data: Vec<u8>,
    },
    AuthFailed {
        reason: String,
    },
    MediaTierOffered {
        clip_id: u64,
        filename: String,
        tier: u8,
        sha256: String,
        size_bytes: u64,
        duration_ticks: u64,
        url: Option<String>,
    },
    MediaTierProgressed {
        clip_id: u64,
        tier: u8,
        received_bytes: u64,
        total_bytes: u64,
    },
    MediaTierPromoted {
        clip_id: u64,
        tier: u8,
        path_or_url: String,
    },
}

pub type WakeupCallback = Arc<dyn Fn() + Send + Sync + 'static>;

#[cfg(not(target_arch = "wasm32"))]
/// Send a framed message across an async writer: 4-byte big-endian length + JSON bytes.
pub async fn send_framed<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &CollabMessage) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let len = bytes.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
/// Read a framed message from an async reader: 4-byte big-endian length + JSON bytes.
pub async fn read_framed<R: AsyncReadExt + Unpin>(reader: &mut R) -> std::io::Result<CollabMessage> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_PACKET_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frame size {len} exceeds limit {MAX_PACKET_BYTES}"),
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let msg: CollabMessage = serde_json::from_slice(&buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(msg)
}

#[cfg(not(target_arch = "wasm32"))]
/// Represents an active network peer connection.
struct PeerConnection {
    sender: mpsc::UnboundedSender<CollabMessage>,
    last_seen: Arc<RwLock<Instant>>,
}

#[cfg(not(target_arch = "wasm32"))]
/// Collaborative Network Transport Controller.
pub struct CollabTransport {
    pub session: Arc<CollabSession>,
    runtime: Arc<tokio::runtime::Runtime>,
    event_rx: StdReceiver<CollabEvent>,
    event_tx: StdSender<CollabEvent>,
    peers: Arc<RwLock<HashMap<PeerId, PeerConnection>>>,
    shutdown_tx: watch::Sender<bool>,
    pub is_host: bool,
    wakeup: Arc<std::sync::Mutex<Option<WakeupCallback>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CollabTransport {
    /// Create a new transport handle and start background Tokio runtime.
    pub fn new(session: Arc<CollabSession>, is_host: bool) -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("collab-worker")
            .build()?;
        let (event_tx, event_rx) = std_channel();
        let (shutdown_tx, _) = watch::channel(false);

        let transport = Self {
            session,
            runtime: Arc::new(runtime),
            event_rx,
            event_tx,
            peers: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx,
            is_host,
            wakeup: Arc::new(std::sync::Mutex::new(None)),
        };

        transport.spawn_broadcast_forwarder();
        transport.spawn_heartbeat_worker();

        Ok(transport)
    }

    /// Set a callback invoked immediately when any network message arrives to wake up the UI.
    pub fn set_wakeup(&self, wakeup: WakeupCallback) {
        if let Ok(mut lock) = self.wakeup.lock() {
            *lock = Some(wakeup);
        }
    }

    /// Try to receive the next event queued for the UI thread without blocking.
    pub fn try_recv_event(&self) -> Option<CollabEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Start listening as Host on the configured bind address.
    pub fn start_host(&self, bind_addr: SocketAddr) -> Result<(), std::io::Error> {
        let session = self.session.clone();
        let event_tx = self.event_tx.clone();
        let peers = self.peers.clone();
        let wakeup = self.wakeup.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        self.runtime.spawn(async move {
            let listener = match TcpListener::bind(bind_addr).await {
                Ok(l) => {
                    info!("FilmCraft Collab host listening on {}", bind_addr);
                    l
                }
                Err(e) => {
                    error!("Failed to bind host socket on {}: {}", bind_addr, e);
                    return;
                }
            };

            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((stream, addr)) => {
                                let _ = stream.set_nodelay(true);
                                info!("Incoming peer connection from {}", addr);
                                let session = session.clone();
                                let event_tx = event_tx.clone();
                                let peers = peers.clone();
                                let wakeup = wakeup.clone();
                                tokio::spawn(async move {
                                    handle_peer_stream(stream, session, event_tx, peers, wakeup, false).await;
                                });
                            }
                            Err(e) => {
                                warn!("Error accepting connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        info!("Host listener shutting down.");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to a remote host as a Client.
    pub fn connect_to(&self, remote_addr: SocketAddr) {
        let session = self.session.clone();
        let event_tx = self.event_tx.clone();
        let peers = self.peers.clone();
        let wakeup = self.wakeup.clone();

        self.runtime.spawn(async move {
            info!("Connecting to collab host at {}...", remote_addr);
            match TcpStream::connect(remote_addr).await {
                Ok(stream) => {
                    let _ = stream.set_nodelay(true);
                    info!("Connected to collab host at {}", remote_addr);
                    handle_peer_stream(stream, session, event_tx, peers, wakeup, true).await;
                }
                Err(e) => {
                    error!("Failed to connect to host {}: {}", remote_addr, e);
                }
            }
        });
    }

    /// Forward messages published locally on `session.msg_tx` to all connected remote peers.
    fn spawn_broadcast_forwarder(&self) {
        let peers = self.peers.clone();
        let mut broadcast_rx = self.session.msg_tx.subscribe();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        self.runtime.spawn(async move {
            loop {
                tokio::select! {
                    msg_res = broadcast_rx.recv() => {
                        match msg_res {
                            Ok(msg) => {
                                let peer_map = peers.read().await;
                                for (_, peer) in peer_map.iter() {
                                    let _ = peer.sender.send(msg.clone());
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(missed)) => {
                                warn!("Collab broadcast forwarder lagged by {} messages", missed);
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    _ = shutdown_rx.changed() => break,
                }
            }
        });
    }

    /// Send regular heartbeats to peers and prune timed-out peers & expired locks.
    fn spawn_heartbeat_worker(&self) {
        let session = self.session.clone();
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        self.runtime.spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let now = Instant::now();
                        let now_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;

                        // 1. Broadcast heartbeat
                        let heartbeat = CollabMessage::Heartbeat {
                            peer_id: session.config.peer_id,
                            timestamp_ms: now_ms,
                        };
                        let peer_map = peers.read().await;
                        for (_, p) in peer_map.iter() {
                            let _ = p.sender.send(heartbeat.clone());
                        }
                        drop(peer_map);

                        // 2. Prune expired soft locks in LockManager
                        session.lock_manager.write().await.prune_expired(now_ms);

                        // 3. Detect timed out peers
                        let mut dead_peers = Vec::new();
                        {
                            let peer_map = peers.read().await;
                            for (id, p) in peer_map.iter() {
                                let last = *p.last_seen.read().await;
                                if now.duration_since(last) > PEER_TIMEOUT {
                                    dead_peers.push(*id);
                                }
                            }
                        }

                        if !dead_peers.is_empty() {
                            let mut peer_map = peers.write().await;
                            for id in dead_peers {
                                info!("Collab peer {} timed out; disconnecting", id);
                                peer_map.remove(&id);
                                session.peers.write().await.remove(&id);
                                session.lock_manager.write().await.release_all_for_peer(id);
                                let _ = event_tx.send(CollabEvent::PeerDisconnected { peer_id: id });
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => break,
                }
            }
        });
    }

    /// Shutdown the transport layer and all background tasks.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for CollabTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Manage the bidirectional message flow for a single TCP stream.
async fn handle_peer_stream(
    stream: TcpStream,
    session: Arc<CollabSession>,
    event_tx: StdSender<CollabEvent>,
    peers: Arc<RwLock<HashMap<PeerId, PeerConnection>>>,
    wakeup: Arc<std::sync::Mutex<Option<WakeupCallback>>>,
    is_client: bool,
) {
    let (mut reader, mut writer) = stream.into_split();
    let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<CollabMessage>();

    // Step 1: Send initial Hello handshake
    let hello = CollabMessage::Hello {
        peer_id: session.config.peer_id,
        name: session.config.user_name.clone(),
        color_rgba: session.config.user_color,
        protocol_version: PROTOCOL_VERSION,
        project_name: session.config.room_code.clone(),
    };
    if let Err(e) = send_framed(&mut writer, &hello).await {
        error!("Failed to send Hello handshake: {}", e);
        return;
    }

    let host_passcode = session.config.passcode.clone();
    let mut authenticated = is_client || host_passcode.is_none();
    let mut auth_nonce: Option<[u8; 32]> = None;

    // If host has a passcode set, challenge the incoming peer immediately
    if !is_client && host_passcode.is_some() {
        let nonce = rand::random::<[u8; 32]>();
        auth_nonce = Some(nonce);
        if let Err(e) = send_framed(&mut writer, &CollabMessage::AuthChallenge { nonce }).await {
            warn!("Failed to send AuthChallenge: {}", e);
            return;
        }
    }

    // Step 2: Spawn writer task for this connection
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if let Err(e) = send_framed(&mut writer, &msg).await {
                warn!("Write error to peer: {}", e);
                break;
            }
        }
    });

    let last_seen = Arc::new(RwLock::new(Instant::now()));
    let mut registered_peer_id: Option<PeerId> = None;
    let mut pending_peer_hello: Option<(PeerId, String, [u8; 4])> = None;

    // Step 3: Reader loop
    loop {
        match read_framed(&mut reader).await {
            Ok(msg) => {
                *last_seen.write().await = Instant::now();
                if let Ok(guard) = wakeup.lock() {
                    if let Some(w) = guard.as_ref() {
                        w();
                    }
                }

                match msg {
                    CollabMessage::Hello { peer_id, name, color_rgba, protocol_version, .. } => {
                        if protocol_version != PROTOCOL_VERSION {
                            warn!("Peer {} protocol version mismatch (theirs: {}, ours: {})", peer_id, protocol_version, PROTOCOL_VERSION);
                        }
                        if authenticated {
                            registered_peer_id = Some(peer_id);
                            peers.write().await.insert(
                                peer_id,
                                PeerConnection {
                                    sender: outgoing_tx.clone(),
                                    last_seen: last_seen.clone(),
                                },
                            );
                            let _ = event_tx.send(CollabEvent::PeerConnected {
                                peer_id,
                                name: name.clone(),
                                color_rgba,
                            });
                            info!("Peer {} ('{}') successfully registered", peer_id, name);
                        } else {
                            pending_peer_hello = Some((peer_id, name, color_rgba));
                        }
                    }
                    CollabMessage::Presence(presence) => {
                        let id = presence.peer_id;
                        session.peers.write().await.insert(id, presence.clone());
                        if !is_client {
                            let peer_map = peers.read().await;
                            for (pid, conn) in peer_map.iter() {
                                if *pid != id {
                                    let _ = conn.sender.send(CollabMessage::Presence(presence.clone()));
                                }
                            }
                        }
                    }
                    CollabMessage::ClaimLock(lock) => {
                        session.lock_manager.write().await.claim(
                            lock.peer_id,
                            lock.track_kind,
                            lock.track_index,
                            lock.expires_at_ms,
                            None,
                        );
                        if !is_client {
                            let peer_map = peers.read().await;
                            for (pid, conn) in peer_map.iter() {
                                if *pid != lock.peer_id {
                                    let _ = conn.sender.send(CollabMessage::ClaimLock(lock.clone()));
                                }
                            }
                        }
                    }
                    CollabMessage::ReleaseLock { peer_id, track_kind, track_index } => {
                        session.lock_manager.write().await.release(peer_id, track_kind, track_index);
                        if !is_client {
                            let peer_map = peers.read().await;
                            for (pid, conn) in peer_map.iter() {
                                if *pid != peer_id {
                                    let _ = conn.sender.send(CollabMessage::ReleaseLock { peer_id, track_kind, track_index });
                                }
                            }
                        }
                    }
                    CollabMessage::CommitCommand { peer_id, rev, id, params } => {
                        let mut cur = session.current_rev.write().await;
                        if rev > *cur {
                            *cur = rev;
                        }
                        let _ = event_tx.send(CollabEvent::CommandReceived {
                            peer_id,
                            rev,
                            id: id.clone(),
                            params: params.clone(),
                        });
                        if !is_client {
                            let peer_map = peers.read().await;
                            for (pid, conn) in peer_map.iter() {
                                if *pid != peer_id {
                                    let _ = conn.sender.send(CollabMessage::CommitCommand {
                                        peer_id,
                                        rev,
                                        id: id.clone(),
                                        params: params.clone(),
                                    });
                                }
                            }
                        }
                    }
                    CollabMessage::RequestSnapshot => {
                        if authenticated {
                            if let Some(peer_id) = registered_peer_id {
                                let _ = event_tx.send(CollabEvent::SnapshotRequested { peer_id });
                            }
                        } else {
                            warn!("Rejected snapshot request from unauthenticated peer");
                        }
                    }
                    CollabMessage::AuthChallenge { nonce } => {
                        let pass = session.config.passcode.as_deref().unwrap_or("");
                        let hmac = compute_auth_hmac(&nonce, pass);
                        let _ = outgoing_tx.send(CollabMessage::AuthResponse { hmac });
                    }
                    CollabMessage::AuthResponse { hmac } => {
                        if !is_client {
                            if let (Some(pass), Some(nonce)) = (&host_passcode, &auth_nonce) {
                                let expected = compute_auth_hmac(nonce, pass);
                                if hmac == expected {
                                    authenticated = true;
                                    let _ = outgoing_tx.send(CollabMessage::AuthResult { ok: true, reason: None });
                                    info!("Peer successfully authenticated with room passcode");
                                    if let Some((pid, name, color)) = pending_peer_hello.take() {
                                        registered_peer_id = Some(pid);
                                        peers.write().await.insert(
                                            pid,
                                            PeerConnection {
                                                sender: outgoing_tx.clone(),
                                                last_seen: last_seen.clone(),
                                            },
                                        );
                                        let _ = event_tx.send(CollabEvent::PeerConnected {
                                            peer_id: pid,
                                            name: name.clone(),
                                            color_rgba: color,
                                        });
                                        info!("Peer {} ('{}') successfully registered", pid, name);
                                    }
                                } else {
                                    warn!("Peer failed room passcode authentication");
                                    let _ = outgoing_tx.send(CollabMessage::AuthResult {
                                        ok: false,
                                        reason: Some("Invalid room passcode".into()),
                                    });
                                    let _ = event_tx.send(CollabEvent::AuthFailed {
                                        reason: "Peer entered invalid room passcode".into(),
                                    });
                                    tokio::time::sleep(Duration::from_millis(50)).await;
                                    break;
                                }
                            }
                        }
                    }
                    CollabMessage::AuthResult { ok, reason } => {
                        if !ok {
                            let r = reason.unwrap_or_else(|| "Invalid room passcode".into());
                            error!("Room authentication failed: {}", r);
                            let _ = event_tx.send(CollabEvent::AuthFailed { reason: r });
                            break;
                        } else {
                            authenticated = true;
                            info!("Room passcode verified; requesting snapshot");
                            if let Some((pid, name, color)) = pending_peer_hello.take() {
                                registered_peer_id = Some(pid);
                                peers.write().await.insert(
                                    pid,
                                    PeerConnection {
                                        sender: outgoing_tx.clone(),
                                        last_seen: last_seen.clone(),
                                    },
                                );
                                let _ = event_tx.send(CollabEvent::PeerConnected {
                                    peer_id: pid,
                                    name: name.clone(),
                                    color_rgba: color,
                                });
                                info!("Peer {} ('{}') successfully registered", pid, name);
                            }
                            if is_client {
                                let _ = outgoing_tx.send(CollabMessage::RequestSnapshot);
                            }
                        }
                    }
                    CollabMessage::MediaTierOffer { clip_id, filename, tier, sha256, size_bytes, duration_ticks, url } => {
                        let _ = event_tx.send(CollabEvent::MediaTierOffered {
                            clip_id,
                            filename,
                            tier,
                            sha256,
                            size_bytes,
                            duration_ticks,
                            url,
                        });
                    }
                    CollabMessage::MediaTierProgress { clip_id, tier, received_bytes, total_bytes } => {
                        let _ = event_tx.send(CollabEvent::MediaTierProgressed {
                            clip_id,
                            tier,
                            received_bytes,
                            total_bytes,
                        });
                    }
                    CollabMessage::MediaTierPromoted { clip_id, tier, path_or_url } => {
                        let _ = event_tx.send(CollabEvent::MediaTierPromoted {
                            clip_id,
                            tier,
                            path_or_url,
                        });
                    }
                    CollabMessage::SyncSnapshot { project_json, revision, active_sequence } => {
                        let _ = event_tx.send(CollabEvent::SnapshotReceived {
                            project_json,
                            revision,
                            active_sequence,
                        });
                    }
                    CollabMessage::RollbackToCheckpoint { peer_id, revision, label, project_json, active_sequence } => {
                        let _ = event_tx.send(CollabEvent::RollbackReceived {
                            peer_id,
                            revision,
                            label,
                            project_json,
                            active_sequence,
                        });
                    }
                    CollabMessage::AssetSync { category, filename, sha256, data } => {
                        let _ = event_tx.send(CollabEvent::AssetReceived {
                            category,
                            filename,
                            sha256,
                            data,
                        });
                    }
                    CollabMessage::ExtensionOffer { extension_id, name, version, description, sha256 } => {
                        let _ = event_tx.send(CollabEvent::ExtensionOffered {
                            extension_id,
                            name,
                            version,
                            description,
                            sha256,
                        });
                    }
                    CollabMessage::MediaOffer { file_id, filename, sha256, size_bytes, is_proxy } => {
                        let _ = event_tx.send(CollabEvent::MediaOffered {
                            file_id,
                            filename,
                            sha256,
                            size_bytes,
                            is_proxy,
                        });
                    }
                    CollabMessage::MediaChunk { sha256, offset_bytes, total_bytes, data } => {
                        let _ = event_tx.send(CollabEvent::MediaChunkReceived {
                            sha256,
                            offset_bytes,
                            total_bytes,
                            data,
                        });
                    }
                    CollabMessage::Heartbeat { .. } => {
                        // Timestamp refreshed above
                    }
                    _ => {}
                }
            }
            Err(e) => {
                info!("Peer stream disconnected: {}", e);
                if is_client && !authenticated {
                    let _ = event_tx.send(CollabEvent::AuthFailed {
                        reason: "Connection closed by host (authentication rejected or disconnected)".into(),
                    });
                }
                break;
            }
        }
    }

    writer_task.abort();

    if let Some(id) = registered_peer_id {
        peers.write().await.remove(&id);
        session.peers.write().await.remove(&id);
        session.lock_manager.write().await.release_all_for_peer(id);
        let _ = event_tx.send(CollabEvent::PeerDisconnected { peer_id: id });
        info!("Collab peer {} cleaned up after disconnect", id);
    }
}

#[cfg(target_arch = "wasm32")]
/// Collaborative Network Transport Controller (Web / WASM).
pub struct CollabTransport {
    pub session: Arc<CollabSession>,
    event_rx: StdReceiver<CollabEvent>,
    #[allow(dead_code)]
    event_tx: StdSender<CollabEvent>,
    pub is_host: bool,
    wakeup: Arc<std::sync::Mutex<Option<WakeupCallback>>>,
}

#[cfg(target_arch = "wasm32")]
impl CollabTransport {
    pub fn new(session: Arc<CollabSession>, is_host: bool) -> Result<Self, std::io::Error> {
        let (event_tx, event_rx) = std_channel();
        Ok(Self {
            session,
            event_rx,
            event_tx,
            is_host,
            wakeup: Arc::new(std::sync::Mutex::new(None)),
        })
    }

    pub fn set_wakeup(&self, wakeup: WakeupCallback) {
        if let Ok(mut lock) = self.wakeup.lock() {
            *lock = Some(wakeup);
        }
    }

    pub fn try_recv_event(&self) -> Option<CollabEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn start_host(&self, _bind_addr: SocketAddr) -> Result<(), std::io::Error> {
        warn!("Direct TCP host binding is unavailable in the browser environment.");
        Ok(())
    }

    pub fn connect_to(&self, _remote_addr: SocketAddr) {
        warn!("Direct TCP client connect is unavailable in the browser environment (use WebSocket bridge).");
    }

    pub fn shutdown(&self) {}
}
