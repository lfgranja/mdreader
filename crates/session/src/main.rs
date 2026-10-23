//! The `session` binary: thin glue over the
//! library's text-I/O protocol entry point
//! (`session::cli::run`).
//!
//! All logic lives in the library so the CLI path
//! and any IPC path exercise identical code
//! (`constitution.md` one-code-path rule): this
//! binary only forwards the process's arguments
//! and streams — arguments after the program name,
//! and the locked standard streams — and maps the
//! result to a process exit status. The success
//! half of the protocol (payload on stdout, stderr
//! empty) exits `0`; the failure half (diagnostic
//! on stderr, stdout empty — already written by
//! `run`) exits `1`.

use std::io::{stderr, stdout};

use session::cli::run;

/// The exit status of a successful invocation.
const EXIT_SUCCESS: i32 = 0;
/// The exit status of a failed invocation.
const EXIT_FAILURE: i32 = 1;

/// The process entry point.
fn main() {
    let outcome = run(
        std::env::args().skip(1),
        &mut stdout().lock(),
        &mut stderr().lock(),
    );
    let exit_status = match outcome {
        Ok(()) => EXIT_SUCCESS,
        Err(_) => EXIT_FAILURE,
    };
    std::process::exit(exit_status);
}
