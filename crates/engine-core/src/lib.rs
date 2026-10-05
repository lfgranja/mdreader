//! Engine-core crate — closed-set vocabulary + stand-in contracts.
//!
//! Library-first core (Principle I): every capability is
//! reachable as a Rust API, and the crate is CLI-drivable
//! headless (Principle II) once the CLI lands (T017). The
//! modules: [`closed`] (T014, the closed-set vocabulary),
//! [`crate::format`] (T015, the declared engine→shell
//! handoff format), and the stand-in engines
//! [`standin_a`] / [`standin_b`] (T017b, the SC-051
//! failover fixtures).
//! Engine IDs are referenced from the constitution's TTS
//! Engine Registry by name and version only; the sole
//! exception is the `test_double` tier, restricted to the
//! `standin_a`/`standin_b` stand-ins (2026-10-05
//! clarification).

pub mod closed;
pub mod format;
pub mod standin_a;
pub mod standin_b;
