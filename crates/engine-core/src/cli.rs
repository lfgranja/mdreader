//! Text-I/O protocol for the engine core (T017).
//!
//! Every trait operation is drivable over text: arguments and
//! stdin in, the success payload on stdout, the failure
//! diagnostic on stderr — never mixed. The machine-readable
//! JSON payload is the default for scripting (`--format json`),
//! with a human-readable sibling selectable alongside it
//! (`--format human`). The same [`run`] entry point serves the
//! binary (`src/main.rs` forwards the process's streams) and
//! any in-process caller (the unit tests below inject their own
//! readers and writers), so the CLI path and any IPC path
//! exercise identical code (Principle II).
//!
//! The subcommands mirror the [`crate::trait_::Engine`] trait
//! operations as contract checks against the closed vocabulary:
//! `capabilities` projects the closed sets
//! [`crate::trait_::Engine::declared_capabilities`] declares
//! from; `set-rate` validates one rung of the closed rate
//! ladder [`crate::trait_::Engine::set_rate`] stretches over;
//! `stream` validates the pre-roll bound
//! [`crate::trait_::Engine::stream`] serves under. No payload
//! carries audio or document text — only metadata — and every
//! failure is a named [`CliError`] carried through the failure
//! half of the protocol (stdout empty, diagnostic on stderr,
//! non-zero exit status).

use std::io::{Read, Write};
use std::iter;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::closed::PlaybackRate;
use crate::format::{DeclaredFormat, check_handoff};
use crate::trait_::{AlignmentGranularity, MAX_PREROLL_SECS, SsmlLevel};

/// The program name: the name the generated parser reports
/// under, and the argument [`run`] prepends when it hands the
/// caller's arguments — which exclude the program name — to
/// that parser.
const PROGRAM_NAME: &str = "engine-core";

/// Run one CLI invocation over the text-I/O protocol.
///
/// `args` are the arguments after the program name; `stdin`,
/// `stdout` and `stderr` are the injected streams, so the
/// protocol is drivable headless and testable in-process with
/// no spawned process.
///
/// Success writes the payload to `stdout`, leaves `stderr`
/// empty, and returns `Ok(())`. Failure writes nothing to
/// `stdout`, writes the diagnostic to `stderr`, and returns
/// the [`CliError`] for the caller to map to a non-zero exit
/// status. `--help` and `--version` are payloads, not
/// failures: the generated parser renders them to `stdout`
/// with an `Ok` result.
///
/// # Errors
///
/// * [`CliError::Usage`] when the invocation is rejected
///   before any contract check runs — an unknown, missing, or
///   malformed subcommand or option, an `--format` value
///   outside the closed format set, an off-ladder `--rate`, or
///   a `stream` position or budget outside its bound.
/// * [`CliError::Verification`] when the handoff-shape gate
///   names a closed-set
///   [`crate::closed::EngineError`] deviation.
/// * [`CliError::Io`] when reading stdin or writing a stream
///   fails.
/// * [`CliError::Rendering`] when a machine-readable payload
///   cannot be rendered.
pub fn run(
    args: impl Iterator<Item = String>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    // The generated parser treats the first argument as the
    // program name; `run` receives only the arguments after
    // it, so the fixed program name is prepended.
    let invocation = iter::once(String::from(PROGRAM_NAME)).chain(args);
    let cli = match Cli::try_parse_from(invocation) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.render();
            if error.use_stderr() {
                // The failure half of the protocol: the
                // diagnostic goes to stderr through the
                // injected writer, and the error is returned
                // for the caller to map to a non-zero exit
                // status. Nothing was written to stdout.
                let _ignored = write!(stderr, "{rendered}");
                return Err(CliError::Usage(rendered.to_string()));
            }
            // `--help` and `--version` are payloads, not
            // failures: the generated parser renders them
            // onto stdout with a zero exit status.
            write!(stdout, "{rendered}")?;
            return Ok(());
        }
    };
    match execute(&cli, stdin, stdout) {
        Ok(()) => Ok(()),
        Err(error) => {
            // The failure half of the protocol: nothing was
            // written to stdout, so this diagnostic on
            // stderr completes it. A write failure here is
            // deliberately discarded — the invocation has
            // already failed, and the returned error names
            // the original failure.
            let _ignored = writeln!(stderr, "{error}");
            Err(error)
        }
    }
}

/// Why a [`run`] invocation failed: the failure half of the
/// text-I/O protocol (stdout empty, diagnostic on stderr,
/// non-zero exit status).
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The invocation was rejected before any contract check
    /// ran: an unknown, missing, or malformed subcommand or
    /// option, an `--format` value outside the closed format
    /// set, an off-ladder `--rate`, or a `stream` position or
    /// budget outside its bound. Carries the rendered
    /// diagnostic.
    #[error("{0}")]
    Usage(String),
    /// The handoff-shape gate named a closed-set
    /// [`crate::closed::EngineError`] deviation (FR-019,
    /// FR-035) (FR-033, superseded by FR-019).
    #[error("verification failed: {0:?}")]
    Verification(crate::closed::EngineError),
    /// Reading stdin or writing a stream failed.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    /// Rendering the machine-readable payload failed.
    #[error("payload rendering failed: {0}")]
    Rendering(#[from] serde_json::Error),
}

impl From<crate::closed::EngineError> for CliError {
    /// Routes a closed-set handoff deviation through the
    /// failure half of the protocol.
    fn from(error: crate::closed::EngineError) -> Self {
        Self::Verification(error)
    }
}

/// The engine-core command line: the arguments after the
/// program name.
///
/// The parser is generated by the `clap` derive macro — the
/// grammar below is the whole parser, and the generated
/// `try_parse_from` is the only parser in the crate; no
/// argument-parsing logic is hand-written.
#[derive(Parser, Debug)]
#[command(name = PROGRAM_NAME, version)]
struct Cli {
    /// The output format: the machine-readable JSON payload
    /// is the default for scripting, with the human-readable
    /// sibling selectable alongside it (Principle II).
    #[arg(long, global = true, value_enum, default_value = "json")]
    format: OutputFormat,

    /// The subcommand naming the trait operation to mirror.
    #[command(subcommand)]
    command: Command,
}

/// The closed output-format set: the machine-readable JSON
/// default and its human-readable sibling.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// The machine-readable JSON payload (the default).
    Json,
    /// The human-readable sibling payload.
    Human,
}

/// The subcommands, each named for the trait operation it
/// mirrors.
#[derive(Subcommand, Debug)]
enum Command {
    /// Project the closed capability vocabulary the
    /// `declared_capabilities` operation declares from.
    Capabilities,
    /// Validate one rung of the closed rate ladder the
    /// `set_rate` operation stretches over.
    SetRate {
        /// The requested rate rung, as a JSON-ladder number
        /// (one of `0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 |
        /// 4.0`).
        #[arg(long)]
        rate: f64,
    },
    /// Validate the pre-roll bound the `stream` operation
    /// serves under, and project the implied delivery shape.
    Stream {
        /// The document position to stream from, in seconds.
        /// When absent, the position is read from stdin as a
        /// single trimmed float.
        #[arg(long)]
        at: Option<f64>,
        /// The pre-roll budget, in seconds: `(0,
        /// 60]` — at most [`MAX_PREROLL_SECS`].
        #[arg(long)]
        budget_secs: f64,
    },
}

/// Execute one parsed invocation: the single code path every
/// caller exercises (Principle II).
///
/// A success writes the payload to `stdout` and returns `Ok`;
/// a failure returns the diagnostic having written nothing to
/// `stdout`, so the caller completes the failure half of the
/// protocol.
fn execute(cli: &Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
    match &cli.command {
        Command::Capabilities => write_payload(stdout, &capabilities_payload(cli.format)?),
        Command::SetRate { rate } => {
            let rung = rate_of(*rate)?;
            write_payload(stdout, &set_rate_payload(rung, *rate, cli.format)?)
        }
        Command::Stream { at, budget_secs } => {
            let position_secs = match at {
                Some(position) => check_position_secs(*position)?,
                Option::None => read_position_from_stdin(stdin)?,
            };
            let budget = check_budget_secs(*budget_secs)?;
            // The shape gate travels with the stream check:
            // the declared mono-`f32`-at-22050 Hz shape passes
            // its own check, and any deviation would route
            // through the failure half before projection.
            check_handoff(
                DeclaredFormat::SAMPLE_RATE,
                DeclaredFormat::CHANNELS,
                DeclaredFormat::SAMPLE_WIDTH_BITS,
            )?;
            write_payload(stdout, &stream_payload(position_secs, budget, cli.format)?)
        }
    }
}

/// Resolve a `--rate` value to its closed ladder rung
/// (FR-019).
///
/// The ladder is the single declaration in
/// [`crate::closed`]: the value round-trips through the
/// rung's own `serde` form, so no second rung list exists
/// here (FR-044). A non-finite value never reaches the
/// round-trip — `serde_json` would render it as `null` —
/// and is rejected as usage.
fn rate_of(value: f64) -> Result<PlaybackRate, CliError> {
    if !value.is_finite() {
        return Err(CliError::Usage(format!(
            "rate must be one of 0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 4.0, got {value:?}"
        )));
    }
    serde_json::from_value::<PlaybackRate>(serde_json::json!(value)).map_err(|_| {
        CliError::Usage(format!(
            "rate must be one of 0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 4.0, got {value}"
        ))
    })
}

/// Check a `stream` start position: finite and `>= 0.0`.
fn check_position_secs(value: f64) -> Result<f64, CliError> {
    if !value.is_finite() || value < 0.0 {
        return Err(CliError::Usage(format!(
            "position_secs must be finite and >= 0.0, got {value:?}"
        )));
    }
    Ok(value)
}

/// Check a `stream` pre-roll budget: finite and in `(0,
/// 60]` — at most [`MAX_PREROLL_SECS`].
fn check_budget_secs(value: f64) -> Result<f64, CliError> {
    if !value.is_finite() || value <= 0.0 || value > MAX_PREROLL_SECS {
        return Err(CliError::Usage(format!(
            "budget_secs must be finite and in (0, {MAX_PREROLL_SECS}], got {value:?}"
        )));
    }
    Ok(value)
}

/// Read the `stream` start position from stdin as a single
/// trimmed float: the stdin supply of the text-I/O protocol
/// for callers that pipe the position instead of passing
/// `--at`.
fn read_position_from_stdin(stdin: &mut dyn Read) -> Result<f64, CliError> {
    let mut text = String::new();
    // The byte count is deliberately discarded: the position
    // text is the only input this supply consumes.
    let _ignored = stdin.read_to_string(&mut text)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CliError::Usage(String::from(
            "stream needs --at <secs> or a position on stdin",
        )));
    }
    trimmed.parse::<f64>().map_or_else(
        |_| {
            Err(CliError::Usage(format!(
                "could not parse a position_secs float from stdin, got {trimmed:?}"
            )))
        },
        check_position_secs,
    )
}

/// The implied PCM frame count of a validated pre-roll
/// budget at the declared handoff rate.
fn frame_count_for_budget(budget_secs: f64) -> usize {
    // `budget_secs` arrives validated: finite and in `(0,
    // 60]`, so the product is non-negative and far below
    // `2^53` — the float-to-`usize` cast below is exact, and
    // the three expects name the precision, sign, and width
    // conditions the validation discharges.
    let rate_hz = f64::from(DeclaredFormat::SAMPLE_RATE);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "budget_secs is validated to (0, 60], so the product is at most 1_323_000 — far below any usize truncation boundary"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "budget_secs is validated to be > 0.0, so the product is non-negative"
    )]
    let frames = (budget_secs * rate_hz) as usize;
    frames
}

/// Write one success payload to `stdout` with a trailing
/// newline: the success half of the protocol (payload on
/// stdout, nothing on stderr).
fn write_payload(stdout: &mut dyn Write, payload: &str) -> Result<(), CliError> {
    writeln!(stdout, "{payload}")?;
    Ok(())
}

/// Render a payload as the machine-readable JSON document —
/// the default output format.
fn render_json<T: Serialize>(payload: &T) -> Result<String, CliError> {
    Ok(serde_json::to_string(payload)?)
}

/// The `capabilities` payload: the closed vocabulary the
/// `declared_capabilities` operation declares from,
/// projected for the selected format.
fn capabilities_payload(format: OutputFormat) -> Result<String, CliError> {
    let payload = CapabilitiesPayload {
        capability: "declared_capabilities",
        ssml_levels: [SsmlLevel::None, SsmlLevel::Partial, SsmlLevel::Full]
            .iter()
            .copied()
            .map(SsmlLevel::as_wire_str)
            .collect(),
        alignment_granularities: [
            AlignmentGranularity::None,
            AlignmentGranularity::Word,
            AlignmentGranularity::Sentence,
        ]
        .iter()
        .copied()
        .map(AlignmentGranularity::as_wire_str)
        .collect(),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_capabilities(&payload),
    })
}

/// The `set-rate` payload: the validated rung and the
/// invariant the `set_rate` operation holds — the stretch is
/// pitch-preserving inside the engine and the delivered rate
/// stays the declared 22050 Hz on all seven rungs (FR-043,
/// SC-034).
fn set_rate_payload(
    rung: PlaybackRate,
    rate: f64,
    format: OutputFormat,
) -> Result<String, CliError> {
    let _ = rung;
    let payload = SetRatePayload {
        capability: "set_rate",
        rate,
        pitch_preserving: true,
        delivered_rate_hz: DeclaredFormat::SAMPLE_RATE,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_set_rate(&payload),
    })
}

/// The `stream` payload: the validated position and budget
/// plus the implied delivery shape the `stream` operation
/// serves under.
fn stream_payload(
    position_secs: f64,
    budget_secs: f64,
    format: OutputFormat,
) -> Result<String, CliError> {
    let payload = StreamPayload {
        capability: "stream",
        position_secs,
        budget_secs,
        frame_count: frame_count_for_budget(budget_secs),
        sample_rate_hz: DeclaredFormat::SAMPLE_RATE,
        channels: DeclaredFormat::CHANNELS,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_stream(&payload),
    })
}

/// The `capabilities` success payload: the closed vocabulary
/// the declaration operation draws one pair from.
#[derive(Serialize)]
struct CapabilitiesPayload {
    /// The trait operation the payload mirrors.
    capability: &'static str,
    /// The closed SSML levels, in wire form.
    ssml_levels: Vec<&'static str>,
    /// The closed alignment granularities, in wire form.
    alignment_granularities: Vec<&'static str>,
}

/// The `set-rate` success payload: the validated rung and
/// the invariant the operation holds.
#[derive(Serialize)]
struct SetRatePayload {
    /// The trait operation the payload mirrors.
    capability: &'static str,
    /// The validated rung, as its JSON-ladder number.
    rate: f64,
    /// The operation holds pitch preservation inside the
    /// engine (FR-043).
    pitch_preserving: bool,
    /// The delivered rate stays the declared handoff rate on
    /// every rung (SC-034).
    delivered_rate_hz: u32,
}

/// The `stream` success payload: the validated position and
/// budget plus the implied delivery shape.
#[derive(Serialize)]
struct StreamPayload {
    /// The trait operation the payload mirrors.
    capability: &'static str,
    /// The validated start position, in seconds.
    position_secs: f64,
    /// The validated pre-roll budget, in seconds.
    budget_secs: f64,
    /// The implied PCM frame count at the declared rate.
    frame_count: usize,
    /// The declared handoff rate, in Hz.
    sample_rate_hz: u32,
    /// The declared handoff channel count (mono).
    channels: u16,
}

/// The human-readable sibling of the `capabilities` payload.
#[must_use]
fn human_capabilities(payload: &CapabilitiesPayload) -> String {
    format!(
        "capability: {}\nssml-levels: {}\nalignment-granularities: {}\n",
        payload.capability,
        payload.ssml_levels.join(", "),
        payload.alignment_granularities.join(", "),
    )
}

/// The human-readable sibling of the `set-rate` payload.
#[must_use]
fn human_set_rate(payload: &SetRatePayload) -> String {
    format!(
        "capability: {}\nrate: {}\npitch-preserving: {}\ndelivered-rate-hz: {}\n",
        payload.capability, payload.rate, payload.pitch_preserving, payload.delivered_rate_hz,
    )
}

/// The human-readable sibling of the `stream` payload.
#[must_use]
fn human_stream(payload: &StreamPayload) -> String {
    format!(
        "capability: {}\nposition-secs: {}\nbudget-secs: {}\nframe-count: {}\nsample-rate-hz: {}\nchannels: {}\n",
        payload.capability,
        payload.position_secs,
        payload.budget_secs,
        payload.frame_count,
        payload.sample_rate_hz,
        payload.channels,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Drive [`run`] in-process with the given arguments and
    /// stdin text, returning the raw streams and the outcome.
    fn drive(args: &[&str], stdin_text: &str) -> (Vec<u8>, Vec<u8>, Result<(), CliError>) {
        let mut stdin = Cursor::new(stdin_text.as_bytes().to_vec());
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let outcome = run(
            args.iter().map(|arg| String::from(*arg)),
            &mut stdin,
            &mut stdout,
            &mut stderr,
        );
        (stdout, stderr, outcome)
    }

    /// Parse captured stdout as JSON, or fail the test with
    /// the captured diagnostic.
    fn stdout_json(stdout: &[u8], stderr: &[u8]) -> serde_json::Value {
        assert!(
            stderr.is_empty(),
            "stderr was not empty on a success path: {}",
            String::from_utf8_lossy(stderr)
        );
        let value = serde_json::from_slice::<serde_json::Value>(stdout).ok();
        assert!(
            value.is_some(),
            "stdout was not JSON: {}",
            String::from_utf8_lossy(stdout)
        );
        value.unwrap_or(serde_json::Value::Null)
    }

    #[test]
    fn capabilities_projects_the_closed_vocabulary() {
        let (stdout, stderr, outcome) = drive(&["capabilities"], "");
        assert!(outcome.is_ok(), "capabilities failed");
        let payload = stdout_json(&stdout, &stderr);
        assert_eq!(
            payload
                .get("capability")
                .and_then(serde_json::Value::as_str),
            Some("declared_capabilities")
        );
        assert_eq!(
            payload.get("ssml_levels"),
            Some(&serde_json::json!(["none", "partial", "full"]))
        );
        assert_eq!(
            payload.get("alignment_granularities"),
            Some(&serde_json::json!(["none", "word", "sentence"]))
        );
    }

    #[test]
    fn set_rate_accepts_every_ladder_rung() {
        for rate in ["0.75", "1.0", "1.25", "1.5", "1.75", "2.0", "4.0"] {
            let (stdout, stderr, outcome) = drive(&["set-rate", "--rate", rate], "");
            assert!(outcome.is_ok(), "set-rate {rate} failed");
            let payload = stdout_json(&stdout, &stderr);
            assert_eq!(
                payload
                    .get("capability")
                    .and_then(serde_json::Value::as_str),
                Some("set_rate")
            );
            assert_eq!(
                payload.get("delivered_rate_hz"),
                Some(&serde_json::json!(22_050))
            );
            assert_eq!(
                payload.get("pitch_preserving"),
                Some(&serde_json::json!(true))
            );
        }
    }

    #[test]
    fn set_rate_rejects_an_off_ladder_value() {
        let (stdout, stderr, outcome) = drive(&["set-rate", "--rate", "3.0"], "");
        assert!(outcome.is_err(), "set-rate 3.0 was accepted");
        assert!(
            stdout.is_empty(),
            "stdout was not empty on the failure path"
        );
        assert!(!stderr.is_empty(), "stderr was empty on the failure path");
    }

    #[test]
    fn stream_projects_position_budget_and_frame_count() {
        let (stdout, stderr, outcome) =
            drive(&["stream", "--at", "12.5", "--budget-secs", "5.0"], "");
        assert!(outcome.is_ok(), "stream failed");
        let payload = stdout_json(&stdout, &stderr);
        assert_eq!(
            payload
                .get("capability")
                .and_then(serde_json::Value::as_str),
            Some("stream")
        );
        assert_eq!(payload.get("position_secs"), Some(&serde_json::json!(12.5)));
        assert_eq!(payload.get("budget_secs"), Some(&serde_json::json!(5.0)));
        assert_eq!(
            payload.get("frame_count"),
            Some(&serde_json::json!(110_250))
        );
        assert_eq!(
            payload.get("sample_rate_hz"),
            Some(&serde_json::json!(22_050))
        );
    }

    #[test]
    fn stream_reads_the_position_from_stdin_without_at() {
        let (stdout, stderr, outcome) = drive(&["stream", "--budget-secs", "2.0"], "7.5\n");
        assert!(outcome.is_ok(), "stream with stdin position failed");
        let payload = stdout_json(&stdout, &stderr);
        assert_eq!(payload.get("position_secs"), Some(&serde_json::json!(7.5)));
        assert_eq!(payload.get("frame_count"), Some(&serde_json::json!(44_100)));
    }

    #[test]
    fn stream_rejects_a_budget_above_sixty_seconds() {
        let (stdout, stderr, outcome) =
            drive(&["stream", "--at", "0.0", "--budget-secs", "60.5"], "");
        assert!(outcome.is_err(), "a 60.5 s budget was accepted");
        assert!(
            stdout.is_empty(),
            "stdout was not empty on the failure path"
        );
        assert!(!stderr.is_empty(), "stderr was empty on the failure path");
    }

    #[test]
    fn stream_rejects_a_negative_position() {
        let (stdout, stderr, outcome) =
            drive(&["stream", "--at", "-1.0", "--budget-secs", "5.0"], "");
        assert!(outcome.is_err(), "a negative position was accepted");
        assert!(
            stdout.is_empty(),
            "stdout was not empty on the failure path"
        );
        assert!(!stderr.is_empty(), "stderr was empty on the failure path");
    }

    #[test]
    fn human_format_renders_without_json() {
        let (stdout, stderr, outcome) = drive(&["--format", "human", "capabilities"], "");
        assert!(outcome.is_ok(), "human capabilities failed");
        assert!(stderr.is_empty(), "stderr was not empty on a success path");
        let text = String::from_utf8_lossy(&stdout);
        assert!(
            text.contains("capability: declared_capabilities"),
            "human payload missed the capability line: {text}"
        );
        assert!(
            text.contains("ssml-levels: none, partial, full"),
            "human payload missed the levels line: {text}"
        );
    }
}
