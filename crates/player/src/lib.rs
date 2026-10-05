//! Player crate — transport, buffering, rewind/snap math.
//!
//! Single authority for playback position and transport state
//! (FR-002, FR-003). Every surface routes through the same
//! Rust code path (Principle II).

pub mod cli;
pub mod position;
pub mod transport;
pub mod skip;
pub mod focus;
pub mod resume;
