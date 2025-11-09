pub mod genes;
pub mod protocol;

pub use genes::*;
pub use protocol::*;

/// The protocol version - clients must match this exactly
pub const PROTOCOL_VERSION: u32 = 1;
