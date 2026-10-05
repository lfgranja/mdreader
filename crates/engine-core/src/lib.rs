//! Engine-core crate — closed-set vocabulary + stand-in contracts.
//!
//! Library-first core (Principle I): every capability is
//! reachable as a Rust API, and the crate is CLI-drivable
//! headless (Principle II) once the CLI lands (T017). This
//! module currently exposes the closed-set vocabulary
//! (T014, [`closed`]); the declared handoff format (T015,
//! [`format`]) and the stand-in engines (T017b,
//! [`standin_a`], [`standin_b`]) follow.

pub mod closed;
