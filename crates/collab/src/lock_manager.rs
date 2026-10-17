//! Leased soft-locking manager for collaborative tracks and items.
//!
//! To prevent simultaneous drag/razor edit collisions on the same track,
//! grabbing a clip claims a soft lock lease with a 500 ms Time-To-Live (TTL).
//!
//! If a peer's connection drops mid-drag, their lock automatically dissolves,
//! preventing permanent deadlocks.

use std::collections::HashMap;
use filmcraft_project::TrackKind;
use crate::protocol::{PeerId, SoftLock};

const DEFAULT_TTL_MS: u64 = 500;

#[derive(Debug, Default)]
pub struct LockManager {
    /// Active track locks keyed by `(TrackKind, track_index)`.
    track_locks: HashMap<(TrackKind, usize), SoftLock>,
}

impl LockManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Claim or refresh a lock on a track. Returns true if acquired or refreshed.
    pub fn claim(
        &mut self,
        peer_id: PeerId,
        kind: TrackKind,
        track_index: usize,
        now_ms: u64,
        ttl_ms: Option<u64>,
    ) -> bool {
        self.prune_expired(now_ms);

        let key = (kind, track_index);
        let ttl = ttl_ms.unwrap_or(DEFAULT_TTL_MS);
        let expires_at_ms = now_ms.saturating_add(ttl);

        if let Some(existing) = self.track_locks.get(&key) {
            if existing.peer_id != peer_id && existing.expires_at_ms > now_ms {
                return false; // Locked by another peer
            }
        }

        self.track_locks.insert(key, SoftLock { peer_id, track_kind: kind, track_index, expires_at_ms });
        true
    }

    /// Release an existing lock explicitly on mouse release.
    pub fn release(&mut self, peer_id: PeerId, kind: TrackKind, track_index: usize) -> bool {
        let key = (kind, track_index);
        if let Some(lock) = self.track_locks.get(&key) {
            if lock.peer_id == peer_id {
                self.track_locks.remove(&key);
                return true;
            }
        }
        false
    }

    /// Check if a track is currently locked by someone other than `for_peer`.
    pub fn is_locked_by_other(
        &self,
        for_peer: PeerId,
        kind: TrackKind,
        track_index: usize,
        now_ms: u64,
    ) -> Option<&SoftLock> {
        let key = (kind, track_index);
        self.track_locks.get(&key).filter(|l| l.peer_id != for_peer && l.expires_at_ms > now_ms)
    }

    /// Release all active locks held by a specific peer (e.g. on disconnect).
    pub fn release_all_for_peer(&mut self, peer_id: PeerId) {
        self.track_locks.retain(|_, lock| lock.peer_id != peer_id);
    }

    /// Remove expired leases.
    pub fn prune_expired(&mut self, now_ms: u64) {
        self.track_locks.retain(|_, lock| lock.expires_at_ms > now_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_lock_leases_and_expires() {
        let mut mgr = LockManager::new();

        // Peer 1 claims Video Track 0 at t=1000 with 500ms TTL
        assert!(mgr.claim(1, TrackKind::Video, 0, 1000, Some(500)));

        // Peer 2 attempts to claim at t=1200 (within TTL) -> rejected
        assert!(!mgr.claim(2, TrackKind::Video, 0, 1200, Some(500)));
        assert!(mgr.is_locked_by_other(2, TrackKind::Video, 0, 1200).is_some());

        // Peer 1 refreshes lease at t=1400
        assert!(mgr.claim(1, TrackKind::Video, 0, 1400, Some(500)));

        // At t=2000, the refreshed lease has expired
        assert!(mgr.is_locked_by_other(2, TrackKind::Video, 0, 2000).is_none());

        // Peer 2 can now claim it
        assert!(mgr.claim(2, TrackKind::Video, 0, 2000, Some(500)));
    }
}
