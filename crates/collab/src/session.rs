//! Peer-to-peer room session coordinator and TCP/WebSocket relay driver.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::id_allocator::CollabIdAllocator;
use crate::lock_manager::LockManager;
use crate::protocol::{CollabMessage, PeerId, PeerPresence};

/// Configuration for a collaborative session.
#[derive(Clone, Debug)]
pub struct CollabConfig {
    pub peer_id: PeerId,
    pub user_name: String,
    pub user_color: [u8; 4],
    pub bind_addr: SocketAddr,
    pub room_code: String,
}

/// State of an active collaborative session.
pub struct CollabSession {
    pub config: CollabConfig,
    pub id_allocator: Arc<RwLock<CollabIdAllocator>>,
    pub lock_manager: Arc<RwLock<LockManager>>,
    pub peers: Arc<RwLock<HashMap<PeerId, PeerPresence>>>,
    pub current_rev: Arc<RwLock<u64>>,
    pub msg_tx: broadcast::Sender<CollabMessage>,
}

impl CollabSession {
    pub fn new(config: CollabConfig) -> Self {
        let (msg_tx, _) = broadcast::channel(1024);
        let id_allocator = Arc::new(RwLock::new(CollabIdAllocator::new(config.peer_id)));

        Self {
            config,
            id_allocator,
            lock_manager: Arc::new(RwLock::new(LockManager::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            current_rev: Arc::new(RwLock::new(0)),
            msg_tx,
        }
    }

    /// Process an incoming message from a network peer.
    pub async fn handle_message(&self, msg: CollabMessage) -> Option<CollabMessage> {
        match msg {
            CollabMessage::Presence(presence) => {
                let peer_id = presence.peer_id;
                self.peers.write().await.insert(peer_id, presence);
                None
            }
            CollabMessage::ClaimLock(lock) => {
                self.lock_manager.write().await.claim(
                    lock.peer_id,
                    lock.track_kind,
                    lock.track_index,
                    lock.expires_at_ms,
                    None,
                );
                None
            }
            CollabMessage::ReleaseLock { peer_id, track_kind, track_index } => {
                self.lock_manager.write().await.release(peer_id, track_kind, track_index);
                None
            }
            CollabMessage::CommitCommand { peer_id, rev, id, params } => {
                let mut current = self.current_rev.write().await;
                if rev > *current {
                    *current = rev;
                }
                Some(CollabMessage::CommitCommand { peer_id, rev, id, params })
            }
            _ => None,
        }
    }

    /// Generate an ephemeral presence message for the local user.
    pub fn create_presence(
        &self,
        panel: Option<String>,
        playhead: Option<filmcraft_time::Tick>,
        cursor_norm: Option<[f32; 2]>,
        active_drag: Option<crate::protocol::DragGhost>,
        selected_clip: Option<u64>,
        in_hand: Option<String>,
    ) -> CollabMessage {
        CollabMessage::Presence(PeerPresence {
            peer_id: self.config.peer_id,
            name: self.config.user_name.clone(),
            color_rgba: self.config.user_color,
            panel,
            playhead_tick: playhead,
            cursor_norm,
            active_drag,
            selected_clip,
            in_hand,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        })
    }
}
