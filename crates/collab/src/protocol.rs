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
    pub active_drag: Option<DragGhost>,
    #[serde(default)]
    pub selected_clip: Option<u64>,
    #[serde(default)]
    pub in_hand: Option<String>,
    pub timestamp_ms: u64,
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
}

pub const PROTOCOL_VERSION: u32 = 1;

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
            active_drag: Some(DragGhost {
                clip_id: 42,
                track_kind: TrackKind::Video,
                track_index: 1,
                start_tick: Tick(100_000),
                duration_ticks: Tick(500_000),
                label: "Interview.mp4".into(),
            }),
            selected_clip: Some(42),
            in_hand: Some("Interview.mp4".into()),
            timestamp_ms: 1_700_000_000,
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
}
