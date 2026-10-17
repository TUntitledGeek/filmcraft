//! FilmCraft Collaboration crate root (Layer L5).
//!
//! Provides zero-cloud P2P collaborative editing, transient presence (cursors,
//! ghost drag previews), leased soft-locking, and zero-collision ID partitioning.

pub mod id_allocator;
pub mod lock_manager;
pub mod protocol;
pub mod session;
pub mod transport;

pub use id_allocator::CollabIdAllocator;
pub use lock_manager::LockManager;
pub use protocol::{CollabMessage, DragGhost, PeerId, PeerPresence, SoftLock, PROTOCOL_VERSION};
pub use session::{CollabConfig, CollabSession};
pub use transport::{CollabEvent, CollabTransport, WakeupCallback};
