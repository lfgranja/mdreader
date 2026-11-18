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
use crate::synth::synthesize_with_voice;
use crate::voice::VoiceId;
use crate::{AlignmentGranularity, SsmlLevel, declared_capabilities};

/// The program name.
const PROGRAM_NAME: &str = "adapter-synthesis";

/// Run one CLI invocation over the text-I/O protocol.
///
/// # Errors
///
/// * [`CliError::Usage`] for invalid invocation/arguments.
/// * [`CliError::Verification`] when a verification check fails.
/// * [`CliError::Synthesis`] when synthesis fails closed.
/// * [`CliError::Io`] on I/O errors.
/// * [`CliError::Rendering`] when JSON rendering fails.
pub fn run(
    args: impl Iterator<Item = String>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    let invocation = iter::once(String::from(PROGRAM_NAME)).chain(args);
    let cli = match Cli::try_parse_from(invocation) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.render();
            if error.use_stderr() {
                let _ignored = write!(stderr, "{rendered}");
                return Err(CliError::Usage(rendered.to_string()));
            }
            write!(stdout, "{rendered}")?;
            return Ok(());
        }
    };
    match execute(cli, stdin, stdout) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ignored = writeln!(stderr, "{error}");
            Err(error)
        }
    }
}

/// Why a [`run`] invocation failed.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The invocation was rejected before any capability ran.
    #[error("{0}")]
    Usage(String),
    /// A verification named a closed-set [`EngineError`] deviation.
    #[error("verification failed: {0:?}")]
    Verification(EngineError),
    /// Synthesis failed closed: named [`EngineFailure`].
    #[error("synthesis failed closed: {0:?}")]
    Synthesis(EngineFailure),
    /// I/O failure.
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
    /// Payload rendering failure.
    #[error("payload rendering failed: {0}")]
    Rendering(#[from] serde_json::Error),
}

impl From<EngineError> for CliError {
    fn from(error: EngineError) -> Self {
        Self::Verification(error)
    }
}

impl From<EngineFailure> for CliError {
    fn from(failure: EngineFailure) -> Self {
        Self::Synthesis(failure)
    }
}

/// The adapter-synthesis command line.
#[derive(Parser)]
#[command(name = PROGRAM_NAME, version)]
struct Cli {
    /// Output format.
    #[arg(long, global = true, value_enum, default_value = "json")]
    format: OutputFormat,
    /// Capability subcommand.
    #[command(subcommand)]
    command: Command,
}

/// Closed output-format set.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    /// Machine-readable JSON (default).
    Json,
    /// Human-readable text.
    Human,
}

/// Subcommands.
#[derive(Subcommand)]
enum Command {
    /// Print the declared capabilities (`declared_capabilities`).
    Capabilities,
    /// List bundled offline catalog voices (spec 018).
    Voices,
    /// Split a document into bounded narration chunks (`split_document`).
    Chunk {
        /// Read document from this file instead of stdin.
        #[arg(long)]
        input: Option<PathBuf>,
    },
    /// Verify a delivered audio format (`check_format`).
    FormatCheck {
        /// Sample rate, in Hz.
        #[arg(long)]
        rate_hz: u32,
        /// Channel count.
        #[arg(long)]
        channels: u16,
        /// Sample width, in bits.
        #[arg(long)]
        width_bits: u16,
    },
    /// Verify a delivered sample buffer (`check_delivery`).
    DeliveryCheck {
        /// Sample rate, in Hz.
        #[arg(long)]
        rate_hz: u32,
        /// Channel count.
        #[arg(long)]
        channels: u16,
        /// Sample width, in bits.
        #[arg(long)]
        width_bits: u16,
        /// PCM sample count.
        #[arg(long)]
        pcm_samples: usize,
        /// Duration, in milliseconds.
        #[arg(long)]
        duration_ms: f64,
    },
    /// Synthesize a document to PCM (`synthesize`).
    Synthesize {
        /// Read document from this file instead of stdin.
        #[arg(long)]
        input: Option<PathBuf>,
        /// Bundled catalog voice id. Omit for active/default voice.
        #[arg(long)]
        voice: Option<String>,
    },
}

/// Execute one parsed invocation.
fn execute(cli: Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
    match cli.command {
        Command::Capabilities => write_payload(stdout, &capabilities_payload(cli.format)?),
        Command::Voices => write_payload(stdout, &voices_payload(cli.format)?),
        Command::Chunk { input } => {
            let document = read_document(input, stdin)?;
            write_payload(stdout, &chunk_payload(&document, cli.format)?)
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
        Command::Synthesize { input, voice } => {
            let document = read_document(input, stdin)?;
            let voice_id = resolve_voice(voice.as_deref())?;
            write_payload(
                stdout,
                &synthesize_payload(&document, voice_id, cli.format)?,
            )
        }
    }
}

/// Resolve CLI voice argument to catalog [`VoiceId`].
fn resolve_voice(arg: Option<&str>) -> Result<VoiceId, CliError> {
    arg.map_or_else(
        || Ok(crate::synth::active_voice()),
        |id| VoiceId::parse_id(id).map_err(|e| CliError::Usage(e.to_string())),
    )
}

/// Read document from `--input` file or stdin.
fn read_document(input: Option<PathBuf>, stdin: &mut dyn Read) -> Result<String, CliError> {
    if let Some(path) = input {
        return Ok(std::fs::read_to_string(path)?);
    }
    let mut document = String::new();
    let _ignored = stdin.read_to_string(&mut document)?;
    Ok(document)
}

/// Write one success payload to stdout with trailing newline.
fn write_payload(stdout: &mut dyn Write, payload: &str) -> Result<(), CliError> {
    writeln!(stdout, "{payload}")?;
    Ok(())
}

/// Render payload as JSON.
fn render_json<T: Serialize>(payload: &T) -> Result<String, CliError> {
    Ok(serde_json::to_string(payload)?)
}

/// `capabilities` payload.
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

/// `voices` payload (spec 018 catalog).
fn voices_payload(format: OutputFormat) -> Result<String, CliError> {
    let active = crate::synth::active_voice();
    let voices: Vec<VoiceEntryPayload> = VoiceId::CATALOG
        .iter()
        .copied()
        .map(|v| {
            let s = crate::voice::spec(v);
            let dir = crate::synth::voice_dir(v);
            VoiceEntryPayload {
                id: s.voice_id.id().to_owned(),
                label: s.label.to_owned(),
                default: v == VoiceId::DEFAULT,
                active: v == active,
                present: dir.join(s.onnx).is_file() && dir.join(s.tokens).is_file(),
            }
        })
        .collect();
    let payload = VoicesPayload {
        capability: "voices",
        default_voice: VoiceId::DEFAULT.id().to_owned(),
        active_voice: active.id().to_owned(),
        voices,
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_voices(&payload),
    })
}

/// `chunk` payload (metadata only, never content).
fn chunk_payload(document: &str, format: OutputFormat) -> Result<String, CliError> {
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

/// Verification success payload.
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

/// `synthesize` payload.
fn synthesize_payload(
    document: &str,
    voice: VoiceId,
    format: OutputFormat,
) -> Result<String, CliError> {
    let segments = split_document(document, "");
    let mut pcm_samples: usize = 0;
    for segment in &segments {
        let buffer = synthesize_with_voice(voice, segment)?;
        pcm_samples = pcm_samples.saturating_add(buffer.len());
    }
    let payload = SynthesizePayload {
        capability: "synthesize",
        segment_count: segments.len(),
        pcm_samples,
        voice: voice.id().to_owned(),
    };
    Ok(match format {
        OutputFormat::Json => render_json(&payload)?,
        OutputFormat::Human => human_synthesize(&payload),
    })
}

/// Project one segment's metadata (never content).
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

#[must_use]
const fn project_resume_point(range: SourceRange) -> ResumePointPayload {
    ResumePointPayload {
        line_start: range.line_start,
        line_end: range.line_end,
    }
}

#[must_use]
const fn ssml_level_text(level: SsmlLevel) -> &'static str {
    match level {
        SsmlLevel::None => "none",
        SsmlLevel::Partial => "partial",
        SsmlLevel::Full => "full",
    }
}

#[must_use]
const fn alignment_granularity_text(granularity: AlignmentGranularity) -> &'static str {
    match granularity {
        AlignmentGranularity::None => "none",
        AlignmentGranularity::Word => "word",
        AlignmentGranularity::Sentence => "sentence",
    }
}

#[must_use]
const fn boundary_kind_text(kind: BoundaryKind) -> &'static str {
    match kind {
        BoundaryKind::Paragraph => "paragraph",
        BoundaryKind::Heading => "heading",
    }
}

#[derive(Serialize)]
struct CapabilitiesPayload {
    capability: &'static str,
    ssml_level: &'static str,
    alignment_granularity: &'static str,
}

#[derive(Serialize)]
struct VoicesPayload {
    capability: &'static str,
    default_voice: String,
    active_voice: String,
    voices: Vec<VoiceEntryPayload>,
}

#[derive(Serialize)]
struct VoiceEntryPayload {
    id: String,
    label: String,
    default: bool,
    active: bool,
    present: bool,
}

#[derive(Serialize)]
struct ChunkPayload {
    capability: &'static str,
    segment_count: usize,
    segments: Vec<SegmentPayload>,
}

#[derive(Serialize)]
struct SegmentPayload {
    segment_index: u64,
    line_start: u32,
    line_end: u32,
    boundary_kind: &'static str,
    byte_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    resume_point: Option<ResumePointPayload>,
}

#[derive(Serialize)]
struct ResumePointPayload {
    line_start: u32,
    line_end: u32,
}

#[derive(Serialize)]
struct CheckPayload {
    capability: &'static str,
    valid: bool,
}

#[derive(Serialize)]
struct SynthesizePayload {
    capability: &'static str,
    segment_count: usize,
    pcm_samples: usize,
    /// Additive field (spec 018): voice id used for synthesis.
    voice: String,
}

#[must_use]
fn human_capabilities(payload: &CapabilitiesPayload) -> String {
    format!(
        "capability: {}\nssml-level: {}\nalignment-granularity: {}\n",
        payload.capability, payload.ssml_level, payload.alignment_granularity,
    )
}

#[must_use]
fn human_voices(payload: &VoicesPayload) -> String {
    let mut out = format!(
        "capability: {}\ndefault-voice: {}\nactive-voice: {}\nvoices: {}",
        payload.capability,
        payload.default_voice,
        payload.active_voice,
        payload.voices.len()
    );
    for v in &payload.voices {
        let _ = FmtWrite::write_fmt(
            &mut out,
            format_args!(
                "\n- {}{}{} [{}] present={}",
                v.id,
                if v.default { " (default)" } else { "" },
                if v.active { " (active)" } else { "" },
                v.label,
                v.present
            ),
        );
    }
    out.push('\n');
    out
}

#[must_use]
fn human_chunk(payload: &ChunkPayload) -> String {
    let mut text = format!(
        "capability: {}\nsegment-count: {}\n",
        payload.capability, payload.segment_count,
    );
    for segment in &payload.segments {
        let resume = segment.resume_point.as_ref().map_or_else(
            || String::from("none"),
            |point| format!("lines {}-{}", point.line_start, point.line_end),
        );
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

#[must_use]
fn human_verification(payload: &CheckPayload) -> String {
    format!(
        "capability: {}\nvalid: {}\n",
        payload.capability, payload.valid
    )
}

#[must_use]
fn human_synthesize(payload: &SynthesizePayload) -> String {
    format!(
        "capability: {}\nvoice: {}\nsegment-count: {}\npcm-samples: {}\n",
        payload.capability, payload.voice, payload.segment_count, payload.pcm_samples,
    )
}
