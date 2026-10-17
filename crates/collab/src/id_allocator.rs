//! Zero-collision ID partition allocator for distributed multiplayer editing.
//!
//! To prevent ID collisions between Editor A (Host) and Editor B (Peer) when
//! allocating [`filmcraft_project::ClipId`] or [`filmcraft_project::ItemId`],
//! the allocator partitions the 64-bit ID space by encoding the PeerId in the
//! upper 16 bits:
//!
//! `id = ((peer_id as u64) << 48) | (sequence_counter & 0x0000_FFFF_FFFF_FFFF)`
//!
//! This ensures mathematical impossibility of ID collisions, even if both peers
//! edit completely offline or simultaneously with 0 network latency.

use crate::protocol::PeerId;

#[derive(Debug)]
pub struct CollabIdAllocator {
    peer_id: PeerId,
    counter: u64,
}

impl CollabIdAllocator {
    pub fn new(peer_id: PeerId) -> Self {
        Self { peer_id, counter: 1 }
    }

    /// Allocate the next collision-free 64-bit ID for this peer.
    pub fn alloc_id(&mut self) -> u64 {
        let seq = self.counter & 0x0000_FFFF_FFFF_FFFF;
        self.counter += 1;
        ((self.peer_id as u64) << 48) | seq
    }

    /// Extract the origin peer from any partitioned ID.
    pub fn peer_of(id: u64) -> PeerId {
        (id >> 48) as PeerId
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn partitioned_ids_never_collide() {
        let mut host_alloc = CollabIdAllocator::new(1);
        let mut guest_alloc = CollabIdAllocator::new(2);

        let mut seen = HashSet::new();

        for _ in 0..10_000 {
            let id_a = host_alloc.alloc_id();
            let id_b = guest_alloc.alloc_id();

            assert!(seen.insert(id_a), "Host ID collision: {id_a}");
            assert!(seen.insert(id_b), "Guest ID collision: {id_b}");

            assert_eq!(CollabIdAllocator::peer_of(id_a), 1);
            assert_eq!(CollabIdAllocator::peer_of(id_b), 2);
        }
    }
}
