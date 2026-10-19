//! The `adapter-synthesis` binary: thin glue over the
//! library's text-I/O protocol entry point
//! (`adapter_synthesis::cli::run`).
//!
//! All logic lives in the library so the CLI path and any
//! IPC path exercise identical code
//! (`constitution.md:255-257`): this binary only forwards
//! the process's arguments and streams — arguments after
//! the program name, and the locked standard streams —
//! and maps the result to a process exit status. The
//! success half of the protocol (payload on stdout,
//! stderr empty) exits `0`; the failure half (diagnostic
//! on stderr, stdout empty — already written by `run`)
//! exits `1`.

#![expect(
    clippy::multiple_crate_versions,
    reason = "the jni-sys duplicate is third-party-forced at newest-release pins: ndk 0.9.0 and ndk-sys 0.6.0 (the latest on crates.io) depend on jni-sys 0.3.1, whose published 0.3.1 release itself depends on jni-sys 0.4.1 — every ndk consumer therefore always carries both versions, cargo cannot unify a dependency with its own transitive re-export, and supply-chain governance for this duplicate belongs to cargo-deny, not this lint"
)]

use std::io::{stderr, stdin, stdout};

use adapter_synthesis::cli::run;

/// The exit status of a successful invocation.
const EXIT_SUCCESS: i32 = 0;
/// The exit status of a failed invocation.
const EXIT_FAILURE: i32 = 1;

/// The process entry point.
fn main() {
    let outcome = run(
        std::env::args().skip(1),
        &mut stdin().lock(),
        &mut stdout().lock(),
        &mut stderr().lock(),
    );
    let exit_status = match outcome {
        Ok(()) => EXIT_SUCCESS,
        Err(_) => EXIT_FAILURE,
    };
    std::process::exit(exit_status);
}
