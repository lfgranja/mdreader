//! Player crate — transport, buffering, rewind/snap math.
//!
//! Single authority for playback position and transport state
//! (FR-002, FR-003). Every surface routes through the same
//! Rust code path (Principle II).

pub mod backend;
pub mod buffer;
pub mod cli;
pub mod earcon;
pub mod focus;
pub mod interruption;
pub mod position;
pub mod rate;
pub mod recovery;
pub mod resume;
pub mod rewind;
pub mod routing;
pub mod skip;
pub mod surface;
pub mod transport;
