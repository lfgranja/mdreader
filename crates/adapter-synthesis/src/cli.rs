//! Text-I/O protocol for the FR-024 synthesis adapter (T041;
//! `constitution.md:251-253`, Principle II, and the
//! one-code-path rule, `constitution.md:255-257`).
//!
//! Every capability the adapter declares is drivable over
//! text: arguments and stdin in, the success payload on
//! stdout, the failure diagnostic on stderr — never mixed.
//! The machine-readable JSON payload is the default for
//! scripting (`--format json`), with a human-readable
//! sibling selectable alongside it (`--format human`).
//! The same [`run`] entry point serves the binary
//! (`src/main.rs` forwards the process's streams) and any
//! in-process caller (the `cli_protocol` conformance tests
//! inject their own readers and writers), so the CLI path
//! and the IPC path exercise identical code.
//!
//! The CLI reports what the adapter declares: it projects
//! the library's verdicts and metadata, infers nothing,
//! and never degrades silently — every failure is a named
//! [`CliError`] carried through the failure half of the
//! protocol (stdout empty, diagnostic on stderr, non-zero
//! exit status).
//!
//! FR-045 (`contracts/adapter-handoff.md:56`): no payload
//! carries document text — the `chunk` payload projects
//! segment metadata only, never segment content.

use std::fmt::Write as FmtWrite;
use std::io::{Read, Write};
use std::iter;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::chunk::{BoundaryKind, DocumentSegment, SourceRange, split_document};
use crate::failure::{EngineError, EngineFailure};
use crate::format_check::{check_delivery, check_format};
use crate::synth::synthesize;
use crate::{AlignmentGranularity, SsmlLevel, declared_capabilities};

/// The program name: the name the generated parser
/// reports under, and the argument [`run`] prepends when
/// it hands the caller's arguments — which exclude the
/// program name — to that parser.
const PROGRAM_NAME: &str = "adapter-synthesis";

/// Run one CLI invocation over the text-I/O protocol.
///
/// `args` are the arguments after the program name;
/// `stdin`, `stdout` and `stderr` are the injected
/// streams, so the protocol is drivable headless and
/// testable in-process with no spawned process.
///
/// Success writes the payload to `stdout`, leaves `stderr`
/// empty, and returns `Ok(())`. Failure writes nothing to
/// `stdout`, writes the diagnostic to `stderr`, and returns
/// the [`CliError`] for the caller to map to a non-zero
/// exit status. `--help` and `--version` are payloads, not
/// failures: the generated parser renders them to `stdout`
/// with an `Ok` result.
///
/// # Errors
///
/// * [`CliError::Usage`] when the invocation is rejected
///   before any capability runs — an unknown, missing, or
///   malformed subcommand or option, or an `--format`
///   value outside the closed format set.
/// * [`CliError::Verification`] when `format-check` or
///   `delivery-check` names a closed-set [`EngineError`]
///   deviation.
/// * [`CliError::Synthesis`] when `synthesize` fails
///   closed (the design-phase voice model is unbound).
/// * [`CliError::Io`] when reading the document (stdin or
///   `--input`) or writing a stream fails.
/// * [`CliError::Rendering`] when a machine-readable
///   payload cannot be rendered.
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
                // Writing it here rather than leaving it to the
                // caller is what keeps the streams unmixed for
                // a usage error: the caller only ever maps the
                // returned error to an exit status, so a
                // diagnostic left implicit would reach stderr
                // only if the caller chose to print it — and
                // `main` does not, because `run` owns every
                // write to the injected streams.
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
    match execute(cli, stdin, stdout) {
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
    /// The invocation was rejected before any capability
    /// ran: an unknown, missing, or malformed subcommand or
    /// option, or an `--format` value outside the closed
    /// format set. Carries the rendered diagnostic.
    #[error("{0}")]
    Usage(String),
    /// A `format-check` or `delivery-check` verification
    /// named a closed-set [`EngineError`] deviation
    /// (FR-024b; SC-035).
    #[error("verification failed: {0:?}")]
    Verification(EngineError),
    /// Synthesis failed closed: the design-phase voice
    /// model is unbound, so the named failure is reported
    /// instead of an empty buffer (R2/R7; SC-035).
    #[error("synthesis failed closed: {0:?}")]
    Synthesis(EngineFailure),
    /// Reading the document (stdin or `--input`) or writing
    /// a stream failed.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    /// Rendering the machine-readable payload failed.
    #[error("payload rendering failed: {0}")]
    Rendering(#[from] serde_json::Error),
}

impl From<EngineError> for CliError {
    /// Routes a closed-set verification deviation through
    /// the failure half of the protocol.
    fn from(error: EngineError) -> Self {
        Self::Verification(error)
    }
}

impl From<EngineFailure> for CliError {
    /// Routes a fail-closed synthesis failure through the
    /// failure half of the protocol.
    fn from(failure: EngineFailure) -> Self {
        Self::Synthesis(failure)
    }
}

/// The adapter-synthesis command line: the arguments after
/// the program name.
///
/// The parser is generated by the `clap` derive macro —
/// the grammar below is the whole parser, and the generated
/// `try_parse_from` is the only parser in the crate; no
/// argument-parsing logic is hand-written.
#[derive(Parser)]
#[command(name = PROGRAM_NAME, version)]
struct Cli {
    /// The output format: the machine-readable JSON payload
    /// is the default for scripting, with the
    /// human-readable sibling selectable alongside it
    /// (`constitution.md:251-253`).
    #[arg(long, global = true, value_enum, default_value = "json")]
    format: OutputFormat,

    /// The subcommand naming the capability to exercise.
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

/// The subcommands, each named for the capability it
/// exercises.
#[derive(Subcommand)]
enum Command {
    /// Print the declared capabilities
    /// (`declared_capabilities`).
    Capabilities,
    /// Split a document into bounded narration chunks
    /// (`split_document`), projecting segment metadata
    /// only — never segment content (FR-045).
    Chunk {
        /// Read the document from this file instead of
        /// stdin.
        #[arg(long)]
        input: Option<PathBuf>,
    },
    /// Verify a delivered audio format against the declared
    /// handoff shape (`check_format`).
    FormatCheck {
        /// The delivered sample rate, in Hz.
        #[arg(long)]
        rate_hz: u32,
        /// The delivered channel count.
        #[arg(long)]
        channels: u16,
        /// The delivered sample width, in bits.
        #[arg(long)]
        width_bits: u16,
    },
    /// Verify a delivered sample buffer — shape first, then
    /// the length its duration implies (`check_delivery`).
    DeliveryCheck {
        /// The delivered sample rate, in Hz.
        #[arg(long)]
        rate_hz: u32,
        /// The delivered channel count.
        #[arg(long)]
        channels: u16,
        /// The delivered sample width, in bits.
        #[arg(long)]
        width_bits: u16,
        /// The delivered PCM sample count.
        #[arg(long)]
        pcm_samples: usize,
        /// The buffer's duration, in milliseconds.
        #[arg(long)]
        duration_ms: f64,
    },
    /// Synthesize a document to PCM (`synthesize`), failing
    /// closed while the voice model is unbound.
    Synthesize {
        /// Read the document from this file instead of
        /// stdin.
        #[arg(long)]
        input: Option<PathBuf>,
    },
}

/// Execute one parsed invocation: the single code path
/// every caller exercises (`constitution.md:255-257`).
///
/// A success writes the payload to `stdout` and returns
/// `Ok`; a failure returns the diagnostic having written
/// nothing to `stdout`, so the caller completes the
/// failure half of the protocol.
fn execute(cli: Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
    match cli.command {
        Command::Capabilities => write_payload(stdout, &capabilities_payload(cli.format)?),
        Command::Chunk { input } => {
            let document = read_document(input, stdin)?;
            let payload = chunk_payload(&document, cli.format)?;
            write_payload(stdout, &payload)
        }
        Command::FormatCheck {
            rate_hz,
            channels,
            width_bits,
        } => {
            check_format(rate_hz, channels, width_bits)?;
            write_payload(stdout, &verification_payload("check_format", cli.format)?)
        }
        Command::DeliveryCheck {
            rate_hz,
            channels,
            width_bits,
            pcm_samples,
            duration_ms,
        } => {
            check_delivery(rate_hz, channels, width_bits, pcm_samples, duration_ms)?;
            write_payload(stdout, &verification_payload("check_delivery", cli.format)?)
        }
        Command::Synthesize { input } => {
            let document = read_document(input, stdin)?;
            let payload = synthesize_payload(&document, cli.format)?;
            write_payload(stdout, &payload)
        }
    }
}

/// Read the document to operate on: from the `--input` file
/// when one is given, else from stdin.
///
/// One code path serves both supplies
/// (`constitution.md:255-257`): the same document text
/// reaches the same projection either way, which is what
/// makes stdin and `--input` byte-identical on stdout.
fn read_document(input: Option<PathBuf>, stdin: &mut dyn Read) -> Result<String, CliError> {
    if let Some(path) = input {
        return Ok(std::fs::read_to_string(path)?);
    }
    let mut document = String::new();
    // The byte count is deliberately discarded: the document
    // text is the only output the projection consumes.
    let _ignored = stdin.read_to_string(&mut document)?;
    Ok(document)
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

/// The `capabilities` payload: the unconditional
/// declaration [`declared_capabilities`] reports, projected
/// for the selected format.
fn capabilities_payload(format: OutputFormat) -> Result<String, CliError> {
    let declaration = declared_capabilities();
    let payload = CapabilitiesPayload {
        capability: "declared_capabilities",
        ssml_level: ssml_level_text(declaration.ssml_level),
        alignment_granularity: alignment_granularity_text(declaration.alignment_granularity),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_capabilities(&payload),
    })
}

/// The `chunk` payload: the segmentation of `document`,
/// projected as segment metadata only — never segment
/// content (FR-045).
fn chunk_payload(document: &str, format: OutputFormat) -> Result<String, CliError> {
    // The document hash is owned by `session` (the `blake3`
    // fingerprint; `chunk.rs` §2.1 rule 12; R5): the CLI
    // carries no session, and the hash is never stored on a
    // segment and never influences the split, so the empty
    // identifier is passed.
    let segments = split_document(document, "");
    let payload = ChunkPayload {
        capability: "split_document",
        segment_count: segments.len(),
        segments: segments.iter().map(project_segment).collect(),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_chunk(&payload),
    })
}

/// The `format-check` / `delivery-check` payload: the
/// verdict the check reported, projected for the selected
/// format.
///
/// Only a conforming delivery reaches this function — a
/// deviation is named by the check itself and routed
/// through the failure half of the protocol before any
/// payload is projected — so the projected verdict is the
/// `true` that the check's `Ok(())` already asserted.
fn verification_payload(
    capability: &'static str,
    format: OutputFormat,
) -> Result<String, CliError> {
    let payload = CheckPayload {
        capability,
        valid: true,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_verification(&payload),
    })
}

/// The `synthesize` payload: the aggregate of the segments
/// synthesized from `document`, projected for the selected
/// format.
///
/// The design-phase `synthesize` fails closed on the first
/// segment (the unbound voice model names
/// `FailureCondition::VoiceInitFailure` →
/// `EngineError::InitFailed`), so a success payload is
/// reachable only for a document with no segments — or
/// once a voice model is bound (`003-contract-edit`).
fn synthesize_payload(document: &str, format: OutputFormat) -> Result<String, CliError> {
    let segments = split_document(document, "");
    let mut pcm_samples: usize = 0;
    for segment in &segments {
        // A failing segment routes through the failure half
        // of the protocol: nothing is written to stdout, and
        // the named failure is reported.
        let buffer = synthesize(segment)?;
        // The bound is unreachable for any buffer that fits
        // in memory (a `usize::MAX`-sample buffer would
        // need exabytes of `f32`), so the total saturates
        // instead of panicking.
        pcm_samples = pcm_samples.saturating_add(buffer.len());
    }
    let payload = SynthesizePayload {
        capability: "synthesize",
        segment_count: segments.len(),
        pcm_samples,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_synthesize(&payload),
    })
}

/// Project one segment's metadata for output — metadata
/// only, never the segment's content (FR-045).
#[must_use]
fn project_segment(segment: &DocumentSegment) -> SegmentPayload {
    SegmentPayload {
        segment_index: segment.segment_index,
        line_start: segment.source_range.line_start,
        line_end: segment.source_range.line_end,
        boundary_kind: boundary_kind_text(segment.boundary_kind),
        byte_length: segment.byte_length(),
        resume_point: segment.resume_point.map(project_resume_point),
    }
}

/// Project one segment's resume point for output.
#[must_use]
fn project_resume_point(range: SourceRange) -> ResumePointPayload {
    ResumePointPayload {
        line_start: range.line_start,
        line_end: range.line_end,
    }
}

/// The wire form of a declared SSML level. The match is
/// exhaustive over the closed three-member set — no
/// catch-all arm — so an added member fails to compile here
/// (`constitution.md` gate 7).
#[must_use]
fn ssml_level_text(level: SsmlLevel) -> &'static str {
    match level {
        SsmlLevel::None => "none",
        SsmlLevel::Partial => "partial",
        SsmlLevel::Full => "full",
    }
}

/// The wire form of a declared alignment granularity.
/// Exhaustive over the closed three-member set — no
/// catch-all arm (`constitution.md` gate 7).
#[must_use]
fn alignment_granularity_text(granularity: AlignmentGranularity) -> &'static str {
    match granularity {
        AlignmentGranularity::None => "none",
        AlignmentGranularity::Word => "word",
        AlignmentGranularity::Sentence => "sentence",
    }
}

/// The wire form of a segment's boundary kind. Exhaustive
/// over the closed two-member set — no catch-all arm
/// (`constitution.md` gate 7).
#[must_use]
fn boundary_kind_text(kind: BoundaryKind) -> &'static str {
    match kind {
        BoundaryKind::Paragraph => "paragraph",
        BoundaryKind::Heading => "heading",
    }
}

/// The `capabilities` success payload: the declaration
/// [`declared_capabilities`] reports, projected for output.
#[derive(Serialize)]
struct CapabilitiesPayload {
    /// The capability the payload names.
    capability: &'static str,
    /// The declared SSML level, in its wire form.
    ssml_level: &'static str,
    /// The declared alignment granularity, in its wire form.
    alignment_granularity: &'static str,
}

/// The `chunk` success payload: the segmentation of the
/// document, projected as segment metadata only — never
/// segment content (FR-045).
#[derive(Serialize)]
struct ChunkPayload {
    /// The capability the payload names.
    capability: &'static str,
    /// The number of segments — always the length of
    /// `segments`.
    segment_count: usize,
    /// The projected segment metadata, 0-based and
    /// consecutive.
    segments: Vec<SegmentPayload>,
}

/// One segment's projected metadata.
#[derive(Serialize)]
struct SegmentPayload {
    /// The 0-based, consecutive position in the
    /// segmentation.
    segment_index: u64,
    /// The first covered line, 1-based inclusive.
    line_start: u32,
    /// The last covered line, 1-based inclusive.
    line_end: u32,
    /// The boundary the segment opens with, in its wire
    /// form.
    boundary_kind: &'static str,
    /// The segment content's UTF-8 byte length.
    byte_length: usize,
    /// The completed boundary at-or-before the yield point,
    /// when the segment declares one.
    resume_point: Option<ResumePointPayload>,
}

/// A segment's projected resume point.
#[derive(Serialize)]
struct ResumePointPayload {
    /// The first covered line, 1-based inclusive.
    line_start: u32,
    /// The last covered line, 1-based inclusive.
    line_end: u32,
}

/// A verification success payload (`format-check`,
/// `delivery-check`).
#[derive(Serialize)]
struct CheckPayload {
    /// The capability the payload names.
    capability: &'static str,
    /// The verdict: `true` on the only path that reaches a
    /// payload — a deviation is routed through the failure
    /// half of the protocol before projection.
    valid: bool,
}

/// The `synthesize` success payload: the aggregate of the
/// synthesized segments.
#[derive(Serialize)]
struct SynthesizePayload {
    /// The capability the payload names.
    capability: &'static str,
    /// The number of segments synthesized.
    segment_count: usize,
    /// The total PCM sample count across the synthesized
    /// segments.
    pcm_samples: usize,
}

/// The human-readable sibling of the `capabilities` payload.
#[must_use]
fn human_capabilities(payload: &CapabilitiesPayload) -> String {
    format!(
        "capability: {}\nssml-level: {}\nalignment-granularity: {}\n",
        payload.capability, payload.ssml_level, payload.alignment_granularity,
    )
}

/// The human-readable sibling of the `chunk` payload: one
/// line per segment, metadata only (FR-045).
#[must_use]
fn human_chunk(payload: &ChunkPayload) -> String {
    let mut text = format!(
        "capability: {}\nsegment-count: {}\n",
        payload.capability, payload.segment_count,
    );
    for segment in &payload.segments {
        let resume = match &segment.resume_point {
            Some(point) => format!("lines {}-{}", point.line_start, point.line_end),
            None => String::from("none"),
        };
        let _ignored = writeln!(
            text,
            "segment {}: lines {}-{}, boundary-kind: {}, byte-length: {}, resume-point: {resume}",
            segment.segment_index,
            segment.line_start,
            segment.line_end,
            segment.boundary_kind,
            segment.byte_length,
        );
    }
    text
}

/// The human-readable sibling of a verification payload.
#[must_use]
fn human_verification(payload: &CheckPayload) -> String {
    format!(
        "capability: {}\nvalid: {}\n",
        payload.capability, payload.valid,
    )
}

/// The human-readable sibling of the `synthesize` payload.
#[must_use]
fn human_synthesize(payload: &SynthesizePayload) -> String {
    format!(
        "capability: {}\nsegment-count: {}\npcm-samples: {}\n",
        payload.capability, payload.segment_count, payload.pcm_samples,
    )
}
