//! T016 — US1: the play-to-audio bound harness
//! (`specs/004-ressoa-dogfood-build/tasks.md` T016).
//!
//! Task text, verbatim: "Play-to-audio bound harness in
//! `crates/player/tests/test_play_latency.rs` (cold-start Play ⇒
//! first audible ≤3 s declared; measures and logs
//! `play_to_audio_ms` into diag `detail`)."
//!
//! What this harness declares and what it measures:
//!
//! * [`PLAY_TO_AUDIO_BOUND_MS`] (3 000 ms) is the normative SC-002
//!   bound — cold-start Play ⇒ first audible audio within three
//!   seconds. The bound terminates at the TRANSDUCER of the
//!   reference Android device, so its gating measurement is T051's
//!   10/10 on-device trial (`quickstart.md` §3), never a host run.
//! * This file measures the HOST half of that path: from the Play
//!   dispatch (segmentation + the first synthesis request) to the
//!   first non-empty PCM buffer the `offline_default` adapter
//!   returns. It records the measurement into a diagnostic JSONL
//!   record's `detail` as the `play_to_audio_ms` key (T012) and
//!   reads it back, so the telemetry leg of the contract is
//!   executed, not merely described.
//!
//! Why the 3 s bound is NOT asserted here: a host measurement is
//! dominated by a cold sherpa-onnx / Piper VITS model load in a
//! DEBUG build on x86 — 6 699 ms to 28 488 ms across twelve
//! measured runs on this host (2026-10-14, with several parallel
//! cargo test sessions loading the machine), over the device bound
//! and irrelevant to it, because the bound's endpoint is the
//! device transducer, not the adapter. Asserting 3 s on the host
//! would make the suite flaky on slow or loaded hosts for a bound
//! that was never device-scoped, so the host assertion is the
//! generous [`HOST_SANITY_CEILING_MS`] (60 000 ms) — still more
//! than twice the slowest host run measured — while a >60 s or
//! failed synthesis means the Play path itself is broken (a missing
//! voice model, a dead engine, an empty/corrupt buffer) and must
//! fail loudly.
//!
//! Bind only to public surfaces: [`adapter_synthesis::chunk`]
//! (`split_document`), [`adapter_synthesis::synth`] (`synthesize`)
//! and [`diag::writer`] (the T012 measured-latency detail keys and
//! the append-only JSONL [`DiagnosticWriter`]). No contract type is
//! re-declared locally; no `player` internals are reached through a
//! test-only back door.
//!
//! Spec anchors: SC-002 (play-to-audio bound, device-scoped);
//! FR-015a (pre-roll architecture — synthesis runs ahead of
//! real-time, which is why the adapter half is the host's share of
//! the bound); FR-039/FR-044 (`detail` is a bound, non-prose
//! `key=value` string, ≤200 chars, never document text); T012
//! (`play_to_audio_ms` typed detail key); the committed schema
//! (`specs/001-android-media-shell/contracts/
//! diagnostic-record.schema.json`, `detail` pattern).

use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use serde_json::Value;

use adapter_synthesis::EngineFailure;
use adapter_synthesis::chunk::{DocumentSegment, split_document};
use adapter_synthesis::synth::synthesize;
use diag::writer::{
    DiagnosticWriter, EventKind, RecordEvent, TransportValue, detail_from_pairs, detail_keys,
};

/// The normative SC-002 play-to-audio bound, in milliseconds:
/// cold-start Play ⇒ first audible audio within 3 000 ms.
///
/// The bound's endpoint is the reference device's transducer, so
/// this constant is DECLARED here and gated by T051's 10/10
/// on-device trial. This harness measures only the host half
/// (Play dispatch → first PCM buffer from the adapter) and never
/// asserts this bound against a host measurement — a debug x86 cold
/// model load is over it for reasons that do not exist on device.
const PLAY_TO_AUDIO_BOUND_MS: u128 = 3_000;

/// The host sanity ceiling, in milliseconds: the only bound this
/// harness asserts against a host measurement.
///
/// Sixty seconds is deliberately generous — twenty times the
/// normative bound, and more than twice the slowest of twelve runs
/// measured on this host (28 488 ms, 2026-10-14, on a machine
/// loaded by several parallel cargo test sessions) — so the suite
/// can never go flaky on a slow or loaded host. Exceeding it, or a
/// synthesis failure, means the Play path is broken rather than
/// slow.
const HOST_SANITY_CEILING_MS: u128 = 60_000;

/// The measured-latency detail key this harness records (T012).
/// Asserted to be exactly `play_to_audio_ms` — the key name is the
/// contract, so a rename is a test failure, not a silent telemetry
/// drift.
const PLAY_TO_AUDIO_KEY: &str = detail_keys::PLAY_TO_AUDIO_MS;

/// The diagnostic-record `detail` bound (FR-039): at most 200
/// characters, enforced by the schema and re-asserted here.
const DETAIL_MAX_CHARS: usize = 200;

/// A small pt-BR document, the first block of which becomes the
/// segment the cold-start Play path synthesizes first. Short on
/// purpose: the harness measures the model-load + first-buffer
/// path, not the synthesis of a long segment, so a large document
/// would only inflate the host number without exercising anything
/// the bound cares about.
const FIXTURE_DOCUMENT: &str = "\
Bom dia. Este é um documento curto em português do Brasil, usado para medir \
o intervalo entre o toque de tocar e o primeiro buffer de áudio disponível \
no adaptador de síntese.

Uma segunda linha de texto mantém o documento acima do tamanho mínimo de um \
segmento sintetizável, sem deixá-lo grande.
";

/// The record version carried by every event (schema `const` 1).
const RECORD_VERSION: u8 = 1;

/// What can stop this harness from producing a measurement.
///
/// A bound that cannot be measured is reported, never skipped: a
/// silent pass would let a broken Play path masquerade as a green
/// run.
#[derive(Debug, thiserror::Error)]
enum HarnessError {
    /// The synthesis engine refused the segment with a named
    /// [`EngineFailure`] — a member of the closed
    /// [`EngineError`](adapter_synthesis::EngineError) set.
    #[error("synthesis refused: {0:?}")]
    Engine(EngineFailure),

    /// The adapter delivered no PCM samples for a non-empty
    /// segment: a corrupt synthesis result, the silent-success
    /// defect mode the adapter exists to prevent (R2/R7).
    #[error("the adapter delivered an empty PCM buffer for a non-empty segment")]
    EmptyBuffer,

    /// The chunker produced no segment for the fixture document.
    #[error("split_document produced no segment for the fixture document")]
    NoSegment,

    /// The detail builder produced no string for a non-empty pair
    /// list — unreachable by construction, reported rather than
    /// silently turned into a detail-less event.
    #[error("detail_from_pairs built no detail for a non-empty pair list")]
    DetailAbsent,

    /// Building the non-prose detail string failed.
    #[error("detail construction failed: {0}")]
    Detail(#[from] diag::writer::DetailError),

    /// Opening or appending to the diagnostic record failed.
    #[error("diagnostic record failed: {0}")]
    Diag(#[from] diag::writer::DiagError),

    /// Serializing or parsing a record event failed.
    #[error("JSON failed: {0}")]
    Json(#[from] serde_json::Error),

    /// A temporary-directory or file operation failed.
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),

    /// The written record carries no `detail` field.
    #[error("the written record event carries no detail field")]
    MissingDetail,

    /// The written detail does not carry the measured key.
    #[error("the written detail {detail:?} carries no {key}={value} pair")]
    MissingMeasurement {
        /// The measured key that is absent.
        key: String,
        /// The measured value that is absent.
        value: String,
        /// The detail string that was read back.
        detail: String,
    },

    /// The record file holds no event line.
    #[error("the record file holds no event line")]
    EmptyRecord,
}

/// The host half of the play-to-audio path, in milliseconds: from
/// the Play dispatch (segmentation + the first synthesis request)
/// to the first non-empty PCM buffer the adapter returns.
///
/// The engine's model load is inside the window on purpose — this
/// harness reproduces the COLD-start Play of SC-002, so the first
/// call pays the one-time `OnceLock` model load.
///
/// # Errors
///
/// [`HarnessError::NoSegment`] when the chunker emits no segment,
/// [`HarnessError::Engine`] when the named engine refuses, and
/// [`HarnessError::EmptyBuffer`] when the delivered buffer is
/// empty for a non-empty segment.
fn measure_play_to_audio_ms() -> Result<u128, HarnessError> {
    let started = Instant::now();
    let segments = split_document(FIXTURE_DOCUMENT, "");
    let first: &DocumentSegment = segments.first().ok_or(HarnessError::NoSegment)?;
    let buffer = synthesize(first).map_err(HarnessError::Engine)?;
    if buffer.is_empty() {
        return Err(HarnessError::EmptyBuffer);
    }
    Ok(started.elapsed().as_millis())
}

/// Current wall-clock time in milliseconds since the Unix epoch.
/// Returns 0 when the system clock precedes the epoch (never on a
/// normal system); the value is only a record timestamp anchor.
fn now_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(0),
        Err(_) => 0,
    }
}

/// The record event carrying the measured latency: a transport
/// change to `playing` whose `detail` is the `play_to_audio_ms`
/// pair.
///
/// The 004 measured-latency keys ride as data INSIDE the nine
/// committed kinds — never as a tenth kind
/// (`contracts/diagnostic-trace.md`: "004 per-section/per-rate/
/// buffer/latency coverage rides as data inside those kinds via
/// `detail` keys, not as new kinds"), so the play-to-audio
/// measurement is a `transport_change` to `playing`.
///
/// # Errors
///
/// [`HarnessError::Detail`] when the non-prose detail cannot be
/// built — an unreachable path for a single numeric pair, still
/// reported rather than silently dropped.
fn play_to_audio_event(ts_ms: u64, measured_ms: u128) -> Result<RecordEvent, HarnessError> {
    let Some(detail) = detail_from_pairs(&[(PLAY_TO_AUDIO_KEY, &measured_ms.to_string())])? else {
        return Err(HarnessError::DetailAbsent);
    };
    Ok(RecordEvent {
        v: RECORD_VERSION,
        ts_ms,
        kind: EventKind::TransportChange,
        position_secs: 0.0,
        transport: Some(TransportValue::Playing),
        session_end_reason: None,
        supersedes_ts_ms: None,
        focus: None,
        device: None,
        error: None,
        engine: None,
        engine_tier: None,
        detail: Some(detail),
        trim: None,
    })
}

/// The `play_to_audio_ms` value a detail string carries, or `None`
/// when the key is absent or its value is not a number.
fn recorded_play_to_audio_ms(detail: &str) -> Option<u128> {
    let prefix = format!("{PLAY_TO_AUDIO_KEY}=");
    let pair = detail.split(' ').find(|pair| pair.starts_with(&prefix))?;
    let (_, raw) = pair.split_once('=')?;
    raw.parse::<u128>().ok()
}

/// Whether `detail` matches the committed schema's non-prose
/// `detail` pattern — space-separated
/// `key=value` pairs whose keys are alphanumeric/underscore and
/// whose values are alphanumeric or `./+-/_`.
///
/// The schema is a spec artifact, not a public API, so the pattern
/// is restated here as a predicate over the identifier shape (the
/// writer validates the same shape through
/// [`detail_from_pairs`]); the point is that no prose, whitespace
/// or punctuation from outside the identifier set can survive into
/// a record's `detail`.
fn matches_schema_detail_pattern(detail: &str) -> bool {
    !detail.is_empty()
        && detail.split(' ').all(|pair| match pair.split_once('=') {
            Some((key, value)) => {
                !key.is_empty()
                    && !value.is_empty()
                    && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && value
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_'))
            }
            None => false,
        })
}

/// The first non-empty line of the JSONL record at `path`,
/// deserialized as a JSON value.
///
/// # Errors
///
/// [`HarnessError::Io`] when the record cannot be read,
/// [`HarnessError::Json`] when the line does not parse, and
/// [`HarnessError::EmptyRecord`] when the file holds no event.
fn first_event_line(path: &Path) -> Result<Value, HarnessError> {
    let text = fs::read_to_string(path)?;
    let line = text
        .lines()
        .find(|line| !line.is_empty())
        .ok_or(HarnessError::EmptyRecord)?;
    Ok(serde_json::from_str(line)?)
}

/// The measured-latency key name is exactly `play_to_audio_ms`
/// (T012): the key is the telemetry contract, so a rename fails
/// here instead of drifting silently in production records.
#[test]
fn the_measured_latency_key_is_exactly_play_to_audio_ms() {
    assert_eq!(
        detail_keys::PLAY_TO_AUDIO_MS,
        "play_to_audio_ms",
        "the T012 measured-latency key must be exactly play_to_audio_ms"
    );
    assert_eq!(
        detail_keys::PLAY_TO_AUDIO_MS,
        PLAY_TO_AUDIO_KEY,
        "this harness must record under the T012 key, not a local variant"
    );
}

/// The declared bound and the host ceiling: the normative SC-002
/// bound is 3 000 ms and the host sanity ceiling is twenty times
/// it, so a host run is never held to a device-scoped bound while a
/// broken Play path still fails.
#[test]
fn the_declared_bound_and_the_host_ceiling_stay_generous() {
    assert_eq!(
        PLAY_TO_AUDIO_BOUND_MS, 3_000,
        "the SC-002 play-to-audio bound is 3000 ms"
    );
    assert_eq!(
        HOST_SANITY_CEILING_MS, 60_000,
        "the host sanity ceiling is 60000 ms — deliberately ~20x the bound"
    );
    assert!(
        PLAY_TO_AUDIO_BOUND_MS < HOST_SANITY_CEILING_MS,
        "the host ceiling must stay strictly above the device bound so a slow host never fails the suite"
    );
}

/// T016 — the play-to-audio harness itself: measure the host half
/// of the cold-start Play path, record it as the
/// `play_to_audio_ms` detail of a `transport_change` record
/// through [`DiagnosticWriter`], read the record back, and assert
/// the telemetry leg (present, schema-shaped, bounded, equal to
/// the measured value) plus the host sanity ceiling.
///
/// Asserts `play_to_audio_ms`:
///
/// * is present in the written JSONL and parseable;
/// * equals the measured value;
/// * keeps `detail` within the 200-char bound and inside the
///   schema's identifier pattern (no prose);
/// * rides a `transport_change` to `playing` on record version 1;
/// * the measured host latency is within
///   [`HOST_SANITY_CEILING_MS`] — the only timing assertion this
///   harness makes, because the 3 000 ms bound is device-scoped
///   (T051).
#[test]
fn cold_start_play_to_first_buffer_is_measured_recorded_and_within_the_host_ceiling()
-> Result<(), HarnessError> {
    let dir = tempfile::tempdir()?;
    let record_path = dir.path().join("play-latency.jsonl");

    let measured_ms = measure_play_to_audio_ms()?;

    let mut writer = DiagnosticWriter::open(&record_path)?;
    writer.append(&play_to_audio_event(now_ms(), measured_ms)?)?;
    // FR-039b: a storage-full append is NOT an error — the writer
    // records a gap marker and returns `Ok`, so a lost line would
    // otherwise surface here as a missing detail and look like a
    // broken telemetry path. The append is retried once (the caller
    // recovers from a best-effort write, by contract) and the
    // remaining gap count is asserted, so a persistent storage
    // failure still fails loudly and a transient one cannot.
    if writer.gap_count() > 0 {
        writer.append(&play_to_audio_event(now_ms(), measured_ms)?)?;
    }
    assert_eq!(
        writer.gap_count(),
        0,
        "the record append was lost to a storage failure (FR-039b gap marker): the diagnostic record at {path} could not be written — the storage path, not the play path, is broken",
        path = record_path.display()
    );
    drop(writer);

    let line = first_event_line(&record_path)?;
    let detail = line
        .get("detail")
        .and_then(Value::as_str)
        .ok_or(HarnessError::MissingDetail)?;

    assert_eq!(
        line.get("kind").and_then(Value::as_str),
        Some("transport_change"),
        "the measured latency rides a transport change to playing, per contracts/diagnostic-trace.md"
    );
    assert_eq!(
        line.get("transport").and_then(Value::as_str),
        Some("playing"),
        "the play-to-audio measurement is the dispatch of the playing transport"
    );
    assert_eq!(
        line.get("v").and_then(Value::as_u64),
        Some(u64::from(RECORD_VERSION)),
        "every record event carries record version 1 (schema const)"
    );

    let recorded_ms =
        recorded_play_to_audio_ms(detail).ok_or_else(|| HarnessError::MissingMeasurement {
            key: PLAY_TO_AUDIO_KEY.to_owned(),
            value: measured_ms.to_string(),
            detail: detail.to_owned(),
        })?;
    assert_eq!(
        recorded_ms, measured_ms,
        "the recorded play_to_audio_ms must equal the measured host latency"
    );

    assert!(
        detail.chars().count() <= DETAIL_MAX_CHARS,
        "detail must stay within the 200-char bound (FR-039), got {chars} chars",
        chars = detail.chars().count()
    );
    assert!(
        matches_schema_detail_pattern(detail),
        "detail must match the schema's non-prose key=value pattern, got {detail:?}"
    );

    assert!(
        measured_ms <= HOST_SANITY_CEILING_MS,
        "the host play-to-audio path took {measured_ms} ms, over the {HOST_SANITY_CEILING_MS} ms sanity ceiling — the Play path is broken (missing voice model, dead engine, or a failed/empty synthesis), not merely slow"
    );

    emit_measurement_record(measured_ms, &line.to_string());
    Ok(())
}

/// Emits the measurement and the written record so an operator
/// running `cargo test -- --nocapture` sees the host latency and
/// the exact JSONL line the record holds. The diagnostic record
/// itself is the durable log (written above, re-read and asserted);
/// this line is the human-readable echo.
///
/// `clippy::print_stderr` is suppressed the single sanctioned way —
/// `#[expect(lint, reason = "...")]`, never an `allow` and never a
/// weakened lint table — because the workspace denies printing in
/// every target, tests included.
#[expect(
    clippy::print_stderr,
    reason = "the harness's measured latency is the human-visible evidence of the host half of the SC-002 bound; without a --nocapture echo the only record of it is the tempdir the test itself deletes, so the operator could never compare hosts without re-deriving the number"
)]
fn emit_measurement_record(measured_ms: u128, record: &str) {
    eprintln!("T016 play_to_audio_ms={measured_ms} record={record}");
}

proptest! {
    // Plain `//`, not `///`: a doc comment cannot attach to an
    // item inside a `proptest!` block (the same note
    // `crates/adapter-synthesis/tests/prop_chunk_refined.rs`
    // records for its own blocks).

    // The detail-format leg over arbitrary measured values. The
    // timing invariant is deliberately NOT covered here — a
    // latency bound is not property-testable, so it is the
    // harness's own single measurement above, while the FORMAT
    // bound applies to every value the measurement could produce.
    #[test]
    fn any_measured_latency_builds_a_bounded_non_prose_detail(millis in 0u64..=600_000u64) {
        let measured = u128::from(millis);
        let Some(detail) = detail_from_pairs(&[(PLAY_TO_AUDIO_KEY, &measured.to_string())])? else {
            return Err(TestCaseError::fail(
                "a one-pair list must never build an absent detail",
            ));
        };
        prop_assert!(
            matches_schema_detail_pattern(&detail),
            "detail must match the schema's non-prose key=value pattern, got {detail:?}"
        );
        prop_assert!(
            detail.chars().count() <= DETAIL_MAX_CHARS,
            "detail must stay within the 200-char bound, got {chars} chars",
            chars = detail.chars().count()
        );
        prop_assert_eq!(
            recorded_play_to_audio_ms(&detail),
            Some(measured),
            "the built detail must carry the measured value back as play_to_audio_ms"
        );
    }
}
