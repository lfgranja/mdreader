//! Text-I/O protocol for the diag crate (T074;
//! `constitution.md` Principle II and the one-code-path
//! rule).
//!
//! The scan capability is drivable over text: the record
//! path in, the scan report on stdout, the failure
//! diagnostic on stderr — never mixed. The
//! machine-readable JSON form is the default for
//! scripting (`--format json`, Principle II), with the
//! human-readable plain form selectable alongside it
//! (`--format human`). The same [`run`] entry point
//! serves the binary (`src/main.rs` forwards the
//! process's streams) and any in-process caller, so the
//! CLI path and the IPC path exercise identical code.
//!
//! The scan resolves through [`crate::scan::scan_record`]
//! — the single scanner implementation; the CLI adds no
//! second one. The report is tooling-facing only (SC-056):
//! there is no in-app surface (FR-039a), and the record
//! is never transmitted (FR-040).

use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::iter;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::scan::{ScanReport, ScanViolation, scan_record};
use crate::writer::DiagError;

/// The program name: the name the generated parser
/// reports under, and the argument [`run`] prepends
/// when it hands the caller's arguments — which
/// exclude the program name — to that parser.
const PROGRAM_NAME: &str = "diag";

/// Run one CLI invocation over the text-I/O protocol.
///
/// `args` are the arguments after the program name;
/// `stdout` and `stderr` are the injected streams, so
/// the protocol is drivable headless and testable
/// in-process with no spawned process.
///
/// Success writes the payload to `stdout`, leaves
/// `stderr` empty, and returns `Ok(())`. Failure
/// writes nothing to `stdout`, writes the diagnostic
/// to `stderr`, and returns the [`CliError`] for the
/// caller to map to a non-zero exit status. `--help`
/// and `--version` are payloads, not failures: the
/// generated parser renders them to `stdout` with an
/// `Ok` result.
///
/// # Errors
///
/// * [`CliError::Usage`] when the invocation is
///   rejected before any capability runs — an unknown,
///   missing, or malformed subcommand or option, or an
///   `--format` value outside the closed format set.
/// * [`CliError::Io`] when a payload cannot be written to
///   the injected streams.
/// * [`CliError::Scan`] when the record cannot be read
///   or a line is not a valid record event.
/// * [`CliError::Json`] when a machine-readable payload
///   cannot be rendered.
pub fn run(
    args: impl Iterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    // The generated parser treats the first argument
    // as the program name; `run` receives only the
    // arguments after it, so the fixed program name
    // is prepended.
    let invocation = iter::once(String::from(PROGRAM_NAME)).chain(args);
    let cli = match Cli::try_parse_from(invocation) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.render();
            if error.use_stderr() {
                // The failure half of the protocol:
                // the diagnostic goes to stderr through
                // the injected writer, and the error is
                // returned for the caller to map to a
                // non-zero exit status. Writing it here
                // rather than leaving it to the caller
                // is what keeps the streams unmixed for
                // a usage error: the caller only ever
                // maps the returned error to an exit
                // status, so a diagnostic left implicit
                // would reach stderr only if the caller
                // chose to print it — and `main` does
                // not, because `run` owns every write
                // to the injected streams.
                let _ignored = write!(stderr, "{rendered}");
                return Err(CliError::Usage(rendered.to_string()));
            }
            // `--help` and `--version` are payloads,
            // not failures: the generated parser
            // renders them onto stdout with a zero
            // exit status.
            write!(stdout, "{rendered}")?;
            return Ok(());
        }
    };
    match execute(cli, stdout) {
        Ok(()) => Ok(()),
        Err(error) => {
            // The failure half of the protocol: nothing
            // was written to stdout, so this diagnostic
            // on stderr completes it. A write failure
            // here is deliberately discarded — the
            // invocation has already failed, and the
            // returned error names the original failure.
            let _ignored = writeln!(stderr, "{error}");
            Err(error)
        }
    }
}

/// Why a [`run`] invocation failed: the failure half
/// of the text-I/O protocol (stdout empty, diagnostic
/// on stderr, non-zero exit status).
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The invocation was rejected before any
    /// capability ran: an unknown, missing, or
    /// malformed subcommand or option, or an
    /// `--format` value outside the closed format
    /// set. Carries the rendered diagnostic.
    #[error("{0}")]
    Usage(String),
    /// Writing the payload to the injected streams
    /// failed.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    /// The record could not be read, or a line of it
    /// is not a valid record event.
    #[error("diagnostic scan failed: {0}")]
    Scan(#[from] DiagError),
    /// A machine-readable payload could not be
    /// rendered.
    #[error("JSON rendering failed: {0}")]
    Json(#[from] serde_json::Error),
}

/// The diag command line: the arguments after the
/// program name.
///
/// The parser is generated by the `clap` derive
/// macro — the grammar below is the whole parser, and
/// the generated `try_parse_from` is the only parser
/// in the crate; no argument-parsing logic is
/// hand-written.
#[derive(Parser)]
#[command(name = PROGRAM_NAME, version)]
struct Cli {
    /// The output format: the machine-readable JSON
    /// form is the default for scripting, with the
    /// human-readable plain form selectable alongside
    /// it (`constitution.md`, Principle II).
    #[arg(long, global = true, value_enum, default_value = "json")]
    format: OutputFormat,

    /// The subcommand naming the capability to
    /// exercise.
    #[command(subcommand)]
    command: Command,
}

/// The closed output-format set: the machine-readable
/// JSON default and its human-readable plain sibling.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// The machine-readable JSON form (the default).
    Json,
    /// The human-readable plain form.
    Human,
}

/// The subcommands, each named for the capability it
/// exercises.
#[derive(Subcommand)]
enum Command {
    /// Scan a diagnostic record for conformance
    /// violations (SC-005, SC-056).
    Scan {
        /// The JSONL diagnostic record to scan, read
        /// only (FR-040).
        file: PathBuf,
    },
}

/// Execute one parsed invocation: the single code path
/// every caller exercises (`constitution.md`
/// one-code-path rule).
///
/// A success writes the payload to `stdout` and
/// returns `Ok`; a failure returns the diagnostic
/// having written nothing to `stdout`, so the caller
/// completes the failure half of the protocol.
fn execute(cli: Cli, stdout: &mut dyn Write) -> Result<(), CliError> {
    // `Command` is a closed set of one capability, so
    // the destructure is irrefutable; a second
    // capability makes it refutable and this site a
    // coordinated edit.
    let Command::Scan { file } = cli.command;
    let report = scan_record(&file)?;
    write_payload(stdout, &scan_payload(&report, cli.format)?)
}

/// Write one success payload to `stdout` with a
/// trailing newline: the success half of the protocol
/// (payload on stdout, nothing on stderr).
fn write_payload(stdout: &mut dyn Write, payload: &str) -> Result<(), CliError> {
    writeln!(stdout, "{payload}")?;
    Ok(())
}

/// The `scan` payload: the scan report — the JSON
/// object in the JSON form, the labeled lines in the
/// plain form.
fn scan_payload(report: &ScanReport, format: OutputFormat) -> Result<String, CliError> {
    Ok(match format {
        OutputFormat::Json => render_json(report)?,
        OutputFormat::Human => human_report(report),
    })
}

/// Render a payload as the machine-readable JSON
/// document — the default output format. The `Result`
/// return already carries `#[must_use]`.
fn render_json<T: Serialize + ?Sized>(payload: &T) -> Result<String, CliError> {
    Ok(serde_json::to_string(payload)?)
}

/// The human-readable plain form of the `scan`
/// payload: the capability name, the violation count,
/// and one line per violation, oldest-first.
#[must_use]
fn human_report(report: &ScanReport) -> String {
    let violation_count = report.violations.len();
    let mut text = format!("capability: scan\nviolation-count: {violation_count}\n");
    for violation in &report.violations {
        // The write cannot fail on a `String` sink;
        // the result is still handled because the
        // workspace denies unused results.
        let _ignored = writeln!(text, "{}", human_violation(violation));
    }
    text
}

/// One violation in the human-readable plain form: the
/// closed kind name, the entry timestamp, and the
/// `detail` length for a too-long `detail`.
#[must_use]
fn human_violation(violation: &ScanViolation) -> String {
    match violation {
        ScanViolation::ProseInDetail { ts_ms } => {
            format!("prose_in_detail: ts_ms={ts_ms}")
        }
        ScanViolation::DetailTooLong { ts_ms, len } => {
            format!("detail_too_long: ts_ms={ts_ms} len={len}")
        }
        ScanViolation::FailureWithoutError { ts_ms } => {
            format!("failure_without_error: ts_ms={ts_ms}")
        }
        ScanViolation::RefusalWithoutError { ts_ms } => {
            format!("refusal_without_error: ts_ms={ts_ms}")
        }
        ScanViolation::PlayingWithoutAdvance { ts_ms } => {
            format!("playing_without_advance: ts_ms={ts_ms}")
        }
        ScanViolation::SessionEndWithPlaying { ts_ms } => {
            format!("session_end_with_playing: ts_ms={ts_ms}")
        }
        ScanViolation::EngineWithoutTier { ts_ms } => {
            format!("engine_without_tier: ts_ms={ts_ms}")
        }
    }
}
