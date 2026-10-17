//! Protocol definitions, packet types, and serialization for FilmCraft P2P collaboration.

use filmcraft_project::TrackKind;
use filmcraft_time::Tick;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Unique identifier for each peer within a co-op room.
pub type PeerId = u16;

/// High-frequency transient presence packet sent across peers (30-60 Hz).
/// These are ephemeral and never touch the undo journal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerPresence {
    pub peer_id: PeerId,
    pub name: String,
    pub color_rgba: [u8; 4],
    pub panel: Option<String>,
    pub playhead_tick: Option<Tick>,
    pub cursor_norm: Option<[f32; 2]>,
    #[serde(default)]
    pub cursor_tick: Option<Tick>,
    #[serde(default)]
    pub cursor_track: Option<(TrackKind, usize)>,
    #[serde(default)]
    pub cursor_track_y_norm: Option<f32>,
    pub active_drag: Option<DragGhost>,
    #[serde(default)]
    pub selected_clip: Option<u64>,
    #[serde(default)]
    pub selected_clips: Vec<u64>,
    #[serde(default)]
    pub in_hand: Option<String>,
    pub timestamp_ms: u64,
}

/// An individual clip item within a multi-clip ghost drag.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GhostClipItem {
    pub clip_id: u64,
    pub track_kind: TrackKind,
    pub track_index: usize,
    pub start_tick: Tick,
    pub duration_ticks: Tick,
    pub label: String,
}

/// A ghost representation of a clip or item being dragged across the timeline
/// before the mouse is released.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DragGhost {
    pub clip_id: u64,
    pub track_kind: TrackKind,
    pub track_index: usize,
    pub start_tick: Tick,
    pub duration_ticks: Tick,
    pub label: String,
    #[serde(default)]
    pub items: Vec<GhostClipItem>,
}

impl DragGhost {
    pub fn single(
        clip_id: u64,
        track_kind: TrackKind,
        track_index: usize,
        start_tick: Tick,
        duration_ticks: Tick,
        label: String,
    ) -> Self {
        let item = GhostClipItem {
            clip_id,
            track_kind,
            track_index,
            start_tick,
            duration_ticks,
            label: label.clone(),
        };
        Self {
            clip_id,
            track_kind,
            track_index,
            start_tick,
            duration_ticks,
            label,
            items: vec![item],
        }
    }

    pub fn multiple(items: Vec<GhostClipItem>) -> Option<Self> {
        let first = items.first()?;
        let label = if items.len() > 1 {
            format!("{} (+{} clips)", first.label, items.len() - 1)
        } else {
            first.label.clone()
        };
        Some(Self {
            clip_id: first.clip_id,
            track_kind: first.track_kind,
            track_index: first.track_index,
            start_tick: first.start_tick,
            duration_ticks: first.duration_ticks,
            label,
            items,
        })
    }

    pub fn all_items(&self) -> Vec<GhostClipItem> {
        if !self.items.is_empty() {
            self.items.clone()
        } else {
            vec![GhostClipItem {
                clip_id: self.clip_id,
                track_kind: self.track_kind,
                track_index: self.track_index,
                start_tick: self.start_tick,
                duration_ticks: self.duration_ticks,
                label: self.label.clone(),
            }]
        }
    }
}

/// A soft lock leased by a peer while performing an interactive gesture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoftLock {
    pub peer_id: PeerId,
    pub track_kind: TrackKind,
    pub track_index: usize,
    pub expires_at_ms: u64,
}

/// Messages exchanged across the collaboration network.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum CollabMessage {
    /// Initial handshake when entering a room.
    Hello {
        peer_id: PeerId,
        name: String,
        color_rgba: [u8; 4],
        protocol_version: u32,
        project_name: String,
    },

    /// Transient presence update (cursor, ghost drag, playhead).
    Presence(PeerPresence),

    /// A soft lock claim on a track while dragging.
    ClaimLock(SoftLock),

    /// Release of a soft lock.
    ReleaseLock {
        peer_id: PeerId,
        track_kind: TrackKind,
        track_index: usize,
    },

    /// An atomic committed command executed on the project.
    CommitCommand {
        peer_id: PeerId,
        rev: u64,
        id: String,
        params: Value,
    },

    /// Heartbeat ping to keep P2P connection and leases alive.
    Heartbeat {
        peer_id: PeerId,
        timestamp_ms: u64,
    },

    /// Media availability offer.
    MediaOffer {
        file_id: String,
        filename: String,
        sha256: String,
        size_bytes: u64,
        is_proxy: bool,
    },

    /// Request to stream a missing media file or proxy.
    MediaRequest {
        sha256: String,
        offset_bytes: u64,
        chunk_size: u32,
    },

    /// Chunk of media payload.
    MediaChunk {
        sha256: String,
        offset_bytes: u64,
        total_bytes: u64,
        data: Vec<u8>,
    },

    /// Project-scoped asset (custom WGSL shader, FCGT template, LUT).
    AssetSync {
        category: String, // "effects", "templates", "luts"
        filename: String,
        sha256: String,
        data: Vec<u8>,
    },

    /// Offer to install an external system extension.
    ExtensionOffer {
        extension_id: String,
        name: String,
        version: String,
        description: String,
        sha256: String,
    },

    /// Request the latest project snapshot from the host.
    RequestSnapshot,

    /// Full snapshot of the project serialized to JSON.
    SyncSnapshot {
        project_json: String,
        revision: u64,
        active_sequence: Option<u64>,
    },

    /// Rollback or time-travel to a specific checkpoint snapshot.
    RollbackToCheckpoint {
        peer_id: PeerId,
        revision: u64,
        label: String,
        project_json: String,
        active_sequence: Option<u64>,
    },

    /// Authentication challenge sent from host to connecting peer when passcode is set.
    AuthChallenge {
        nonce: [u8; 32],
    },

    /// Authentication response containing HMAC-SHA256 proof of room passcode.
    AuthResponse {
        hmac: [u8; 32],
    },

    /// Result of authentication challenge.
    AuthResult {
        ok: bool,
        reason: Option<String>,
    },

    /// Multi-tier progressive proxy notification (Tier 0: Waveforms -> Tier 1: 480p -> Tier 2: 1080p -> Tier 3: 4K Master).
    MediaTierOffer {
        clip_id: u64,
        filename: String,
        tier: u8,
        sha256: String,
        size_bytes: u64,
        duration_ticks: u64,
        url: Option<String>,
    },

    /// Progress report for an actively streaming proxy or media tier.
    MediaTierProgress {
        clip_id: u64,
        tier: u8,
        received_bytes: u64,
        total_bytes: u64,
    },

    /// Notification that a higher quality tier has finished downloading and is promoted in the timeline.
    MediaTierPromoted {
        clip_id: u64,
        tier: u8,
        path_or_url: String,
    },
}

pub const PROTOCOL_VERSION: u32 = 1;

/// Computes standard HMAC-SHA256 for room passcode verification without external dependencies.
pub fn compute_auth_hmac(nonce: &[u8; 32], passcode: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut key = [0u8; 64];
    let pass_bytes = passcode.as_bytes();
    if pass_bytes.len() > 64 {
        let hash = Sha256::digest(pass_bytes);
        key[..32].copy_from_slice(&hash);
    } else {
        key[..pass_bytes.len()].copy_from_slice(pass_bytes);
    }
    let mut o_key_pad = [0x5cu8; 64];
    let mut i_key_pad = [0x36u8; 64];
    for i in 0..64 {
        o_key_pad[i] ^= key[i];
        i_key_pad[i] ^= key[i];
    }
    let mut inner = Sha256::new();
    inner.update(&i_key_pad);
    inner.update(nonce);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&o_key_pad);
    outer.update(&inner_hash);
    outer.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_packet_roundtrips_cleanly() {
        let presence = PeerPresence {
            peer_id: 1,
            name: "Alice".into(),
            color_rgba: [255, 120, 0, 255],
            panel: Some("timeline".into()),
            playhead_tick: Some(Tick(127_008_000_000)),
            cursor_norm: Some([0.45, 0.62]),
            cursor_tick: Some(Tick(100_000)),
            cursor_track: Some((TrackKind::Video, 1)),
            cursor_track_y_norm: Some(0.5),
            active_drag: Some(DragGhost::single(
                42,
                TrackKind::Video,
                1,
                Tick(100_000),
                Tick(500_000),
                "Interview.mp4".into(),
            )),
            selected_clip: Some(42),
            selected_clips: vec![42],
            in_hand: Some("Interview.mp4".into()),
            timestamp_ms: 1_700_000_000,
        };

        let msg = CollabMessage::Presence(presence.clone());
        let json = serde_json::to_string(&msg).expect("serialize");
        let decoded: CollabMessage = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(msg, decoded);
    }

    #[test]
    fn multi_clip_drag_ghost_roundtrips() {
        let items = vec![
            GhostClipItem {
                clip_id: 10,
                track_kind: TrackKind::Video,
                track_index: 0,
                start_tick: Tick(100_000),
                duration_ticks: Tick(50_000),
                label: "Clip A".into(),
            },
            GhostClipItem {
                clip_id: 11,
                track_kind: TrackKind::Video,
                track_index: 1,
                start_tick: Tick(120_000),
                duration_ticks: Tick(40_000),
                label: "Clip B".into(),
            },
        ];
        let ghost = DragGhost::multiple(items).expect("multiple ghost");
        assert_eq!(ghost.items.len(), 2);
        assert_eq!(ghost.all_items().len(), 2);
        assert_eq!(ghost.label, "Clip A (+1 clips)");

        let presence = PeerPresence {
            peer_id: 2,
            name: "Bob".into(),
            color_rgba: [0, 200, 255, 255],
            panel: Some("timeline".into()),
            playhead_tick: Some(Tick(200_000)),
            cursor_norm: None,
            cursor_tick: Some(Tick(110_000)),
            cursor_track: Some((TrackKind::Video, 0)),
            cursor_track_y_norm: Some(0.3),
            active_drag: Some(ghost),
            selected_clip: Some(10),
            selected_clips: vec![10, 11],
            in_hand: None,
            timestamp_ms: 1_700_000_001,
        };

        let msg = CollabMessage::Presence(presence.clone());
        let json = serde_json::to_string(&msg).expect("serialize");
        let decoded: CollabMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(msg, decoded);
    }

    #[test]
    fn commit_command_packet_roundtrips() {
        let msg = CollabMessage::CommitCommand {
            peer_id: 2,
            rev: 88,
            id: "timeline.place".into(),
            params: serde_json::json!({
                "clip": 42,
                "track": 1,
                "time": 254_016_000_000i64
            }),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let decoded: CollabMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn auth_hmac_computes_deterministically() {
        let nonce = [42u8; 32];
        let hmac1 = compute_auth_hmac(&nonce, "secret123");
        let hmac2 = compute_auth_hmac(&nonce, "secret123");
        let hmac_wrong = compute_auth_hmac(&nonce, "wrong_pass");

        assert_eq!(hmac1, hmac2, "HMAC must be deterministic");
        assert_ne!(hmac1, hmac_wrong, "Different passcodes must yield different HMACs");
    }

    #[test]
    fn auth_and_media_tier_packets_roundtrip() {
        let nonce = [7u8; 32];
        let challenge = CollabMessage::AuthChallenge { nonce };
        let json = serde_json::to_string(&challenge).unwrap();
        assert_eq!(challenge, serde_json::from_str(&json).unwrap());

        let hmac = compute_auth_hmac(&nonce, "room_pin_99");
        let response = CollabMessage::AuthResponse { hmac };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(response, serde_json::from_str(&json).unwrap());

        let result = CollabMessage::AuthResult { ok: true, reason: None };
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(result, serde_json::from_str(&json).unwrap());

        let tier_offer = CollabMessage::MediaTierOffer {
            clip_id: 101,
            filename: "drone_4k.mp4".into(),
            tier: 1, // 480p proxy
            sha256: "abc123sha".into(),
            size_bytes: 5_242_880,
            duration_ticks: 360_000_000_000,
            url: Some("https://storage.googleapis.com/filmcraft/drone_480p.mp4".into()),
        };
        let json = serde_json::to_string(&tier_offer).unwrap();
        assert_eq!(tier_offer, serde_json::from_str(&json).unwrap());
    }
}
