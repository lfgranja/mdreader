//! Text-I/O protocol for the doc-structure crate
//! (T061c; `constitution.md` Principle II and the
//! one-code-path rule).
//!
//! Every capability the crate exposes is drivable over
//! text: arguments in, the success payload on stdout,
//! the failure diagnostic on stderr — never mixed.
//! The machine-readable JSON form is the default for
//! scripting (`--format json`), with the
//! human-readable plain form selectable alongside it
//! (`--format human`). The same [`run`] entry point
//! serves the binary (`src/main.rs` forwards the
//! process's streams) and any in-process caller, so
//! the CLI path and the IPC path exercise identical
//! code.
//!
//! The headings file format is JSON — a JSON array of
//! `{position_secs, title}` objects — so the CLI
//! round-trips the resolver's own input: `headings`
//! prints the ordered list in exactly the form
//! [`Document::new`](crate::Document::new) accepts,
//! and `reanchor` reports the FR-038 re-anchor point
//! the resolver computes for a stored position.

use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::iter;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::{Document, Heading};

/// The program name: the name the generated parser
/// reports under, and the argument [`run`] prepends
/// when it hands the caller's arguments — which
/// exclude the program name — to that parser.
const PROGRAM_NAME: &str = "doc-structure";

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
///   rejected before any capability runs — an
///   unknown, missing, or malformed subcommand or
///   option, or an `--format` value outside the
///   closed format set.
/// * [`CliError::Io`] when the headings file cannot
///   be read.
/// * [`CliError::Json`] when the headings file is not
///   a JSON array of headings, or a machine-readable
///   payload cannot be rendered.
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
    /// The headings file could not be read.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    /// The headings file is not a JSON array of
    /// headings, or a machine-readable payload could
    /// not be rendered.
    #[error("JSON failure: {0}")]
    Json(#[from] serde_json::Error),
}

/// The doc-structure command line: the arguments after
/// the program name.
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
    /// Print the document's ordered headings.
    Headings {
        /// The headings file: a JSON array of
        /// `{position_secs, title}` objects.
        file: PathBuf,
    },
    /// Print the FR-038 re-anchor point for a position:
    /// the nearest boundary at or before it, else
    /// exactly `0.0`.
    Reanchor {
        /// The headings file: a JSON array of
        /// `{position_secs, title}` objects.
        file: PathBuf,
        /// The position to re-anchor, in seconds.
        #[arg(long)]
        at: f64,
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
    match cli.command {
        Command::Headings { file } => {
            let document = Document::new(load_headings(&file)?);
            write_payload(stdout, &headings_payload(&document, cli.format)?)
        }
        Command::Reanchor { file, at } => {
            let document = Document::new(load_headings(&file)?);
            write_payload(
                stdout,
                &reanchor_payload(document.reanchor(at), cli.format)?,
            )
        }
    }
}

/// Read the headings to operate on: the JSON array of
/// `{position_secs, title}` objects that is both the
/// resolver's own input format and the `headings`
/// command's output form.
fn load_headings(file: &Path) -> Result<Vec<Heading>, CliError> {
    let text = std::fs::read_to_string(file)?;
    Ok(serde_json::from_str(&text)?)
}

/// Write one success payload to `stdout` with a
/// trailing newline: the success half of the protocol
/// (payload on stdout, nothing on stderr).
fn write_payload(stdout: &mut dyn Write, payload: &str) -> Result<(), CliError> {
    writeln!(stdout, "{payload}")?;
    Ok(())
}

/// Render a payload as the machine-readable JSON
/// document — the default output format. The
/// `Result` return already carries `#[must_use]`.
fn render_json<T: Serialize + ?Sized>(payload: &T) -> Result<String, CliError> {
    Ok(serde_json::to_string(payload)?)
}

/// The `headings` payload: the document's ordered
/// headings — the JSON array of
/// `{position_secs, title}` objects in the JSON form,
/// one line per heading in the plain form.
fn headings_payload(document: &Document, format: OutputFormat) -> Result<String, CliError> {
    Ok(match format {
        OutputFormat::Json => render_json(document.headings())?,
        OutputFormat::Human => human_headings(document),
    })
}

/// The `reanchor` payload: the FR-038 re-anchor
/// point — the nearest boundary at or before the
/// position, else exactly `0.0` — as the JSON number
/// it is in the JSON form, labeled in the plain form.
fn reanchor_payload(anchor: f64, format: OutputFormat) -> Result<String, CliError> {
    Ok(match format {
        OutputFormat::Json => render_json(&anchor)?,
        OutputFormat::Human => human_reanchor(anchor),
    })
}

/// The human-readable plain form of the `headings`
/// payload: one line per ordered heading, position
/// first.
#[must_use]
fn human_headings(document: &Document) -> String {
    let mut text = format!(
        "capability: headings\nheading-count: {}\n",
        document.headings().len(),
    );
    for heading in document.headings() {
        // The write cannot fail on a `String` sink;
        // the result is still handled because the
        // workspace denies unused results.
        let _ignored = writeln!(text, "{}: {}", heading.position_secs, heading.title);
    }
    text
}

/// The human-readable plain form of the `reanchor`
/// payload: the labeled re-anchor point.
#[must_use]
fn human_reanchor(anchor: f64) -> String {
    format!("capability: reanchor\nreanchor-point-secs: {anchor}\n")
}
