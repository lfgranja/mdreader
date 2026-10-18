//! Player crate — transport, buffering, rewind/snap math.
//!
//! Single authority for playback position and transport state
//! (FR-002, FR-003). Every surface routes through the same
//! Rust code path (Principle II).

#![expect(
    clippy::multiple_crate_versions,
    reason = "every duplicate is third-party-forced at newest-release pins: ndk 0.9.0 (directly and via cpal 0.18.2) carries thiserror 1.0.69 and jni-sys 0.3.1 — whose published 0.3.1 release itself depends on jni-sys 0.4.1 — while cpal's jni 0.22.4 carries thiserror 2.0.21 and jni-sys 0.4.1; cargo cannot unify across semver majors or a crate with its own re-export, crates.io shows no newer versions, and supply-chain governance for these transitive duplicates belongs to cargo-deny, not this lint"
)]

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
