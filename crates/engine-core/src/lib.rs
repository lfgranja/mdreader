//! Engine-core crate — closed-set vocabulary + engine contract.
//!
//! Library-first core (Principle I): every capability is
//! reachable as a Rust API, and the crate is CLI-drivable
//! headless (Principle II) through [`cli`], which mirrors the
//! [`trait_::Engine`] trait operations over text. The
//! modules: [`closed`] (T014, the closed-set vocabulary),
//! [`format`] (T015, the declared engine→shell
//! handoff format), [`trait_`] (T016, the engine trait and
//! capability declaration), [`cli`] (T017, the text-I/O
//! protocol over the trait operations), and the stand-in engines
//! [`standin_a`] / [`standin_b`] (T017b, the SC-051
//! failover fixtures).
//! Engine IDs are referenced from the constitution's TTS
//! Engine Registry by name and version only; the sole
//! exception is the `test_double` tier, restricted to the
//! `standin_a`/`standin_b` stand-ins (2026-10-05
//! clarification).

pub mod cli;
pub mod closed;
pub mod format;
pub mod standin_a;
pub mod standin_b;
pub mod trait_;
