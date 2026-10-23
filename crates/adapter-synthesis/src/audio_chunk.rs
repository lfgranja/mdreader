//! Delivery unit — the `AudioChunk` the adapter
//! delivers to `player` (data-model E2,
//! `data-model.md:43-61`; the wire/interface shape
//! at `contracts/adapter-handoff.md:36-46`).
//!
//! The chunk is the adapter's only output unit: the
//! synthesized mono-`f32` buffer at exactly 22 050 Hz,
//! the declared rate and channel constants, the duration
//! DERIVED from the buffer's own length (never a
//! parameter — a caller cannot supply a duration that
//! disagrees with the buffer; data-model E2), an echo of
//! the delivered segment's `source_range` (never
//! adapter-computed — segment-resume CHK030; the
//! `player` is the sole playback authority,
//! `constitution.md:486-490`), and the pre-roll
//! remaining, `Some` only while pre-roll is buffered
//! (FR-015a / R10; `data-model.md:145`).
//!
//! Delivery validates fail-closed, in order (FR-024b /
//! SC-035 — a refused delivery never starts playback
//! silently):
//!
//! 1. an empty `pcm_data` is a corrupt synthesis result,
//!    refused as [`DeliveryError::Format`] naming
//!    [`EngineError::InitFailed`]
//!    (`contracts/adapter-handoff.md:52`, "`InitFailed`
//!    if synthesis corrupt" — an empty buffer is the
//!    silent-success defect mode `src/synth.rs` exists to
//!    prevent);
//! 2. the shape gate is delegated to [`check_delivery`]
//!    with the derived duration — the single shape
//!    authority (FR-044: the shape is declared exactly
//!    once, in `engine_core::format::DeclaredFormat`),
//!    so any deviation names a member of the closed
//!    [`EngineError`] set and playback does NOT start
//!    (R1/R2; FR-035); the 32-bit width is a type-level
//!    fact of `Vec<f32>`, so [`DECLARED_WIDTH_BITS`] is
//!    passed as the width;
//! 3. a `pre_roll_remaining_ms` above
//!    [`AudioChunk::MAX_PRE_ROLL_MS`] is refused with
//!    [`DeliveryError::PreRollTooLong`] — never clamped:
//!    a silent clamp would be a silent misreport, the
//!    same policy as `EngineFailure::report` refusing an
//!    over-bound detail with
//!    `FailureReportError::DetailTooLong`
//!    (`src/failure.rs:183-205`).
//!
//! [`DeliveryError`] is a construction-time refusal, not
//! an engine-observed failure: a pre-roll over-bound is
//! not a member of the `spec.md:86` condition→variant
//! table, so mapping it to one of the seven [`EngineError`]
//! members would corrupt the closed failure taxonomy
//! (C4-A — no invented members).
//!
//! Immutable after delivery (`spec.md:103`;
//! `contracts/adapter-handoff.md:30`): the struct exposes
//! no setter, and the adapter never modifies a field after
//! delivery. No field is a `String` — the payload is pure
//! `f32` PCM, so no document text can ride the delivery
//! (FR-045; `contracts/adapter-handoff.md:56`).
//!
//! Pre-roll stops when playback pauses, and there is no
//! path that synthesizes continuously while idle
//! (`contracts/adapter-handoff.md:63`; R10) — a
//! behavioural contract of the synthesizer, pinned by the
//! delivery-unit tests, not a field this type can express.

use crate::chunk::SourceRange;
use crate::failure::EngineError;
use crate::format_check::{
    DECLARED_CHANNELS, DECLARED_RATE_HZ, DECLARED_WIDTH_BITS, check_delivery,
};

/// One delivered audio unit — the adapter's output to
/// `player`.
///
/// The unit is declared at data-model E2
/// (`data-model.md:43-61`; the wire/interface shape
/// at `contracts/adapter-handoff.md:36-46`). Immutable
/// after delivery (`spec.md:103`): every field is set
/// once by [`AudioChunk::deliver`] and never
/// modified afterwards.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioChunk {
    /// The delivered PCM samples: mono `f32` at exactly
    /// 22 050 Hz (R1; `engine_core::format::DeclaredFormat`
    /// — `SAMPLE_RATE` 22 050, `CHANNELS` 1,
    /// `SAMPLE_WIDTH_BITS` 32). The engine's synthesis
    /// output, delivered as synthesized — the adapter
    /// never resamples or downmixes it
    /// (`contracts/adapter-handoff.md:32`).
    pub pcm_data: Vec<f32>,
    /// The delivered sample rate, in Hz — always
    /// [`DECLARED_RATE_HZ`] (22 050). A deviation is
    /// refused at the gate as [`DeliveryError::Format`]
    /// naming [`EngineError::RateMismatch`] (R2; FR-035).
    pub rate_hz: u32,
    /// The delivered channel count — always
    /// [`DECLARED_CHANNELS`] (mono). A deviation is
    /// refused as [`DeliveryError::Format`] naming
    /// [`EngineError::ChannelMismatch`] — never a silent
    /// downmix (R2; FR-019) (FR-033, superseded by FR-019).
    pub channels: u16,
    /// The delivered duration, in milliseconds — DERIVED
    /// as `pcm_data.len() / rate_hz * 1000` (data-model
    /// E2; `contracts/adapter-handoff.md:42`), never a
    /// parameter of [`AudioChunk::deliver`], so a caller
    /// cannot supply a duration that disagrees with the
    /// buffer.
    /// The player uses it for notification progress
    /// (FR-028).
    pub duration_ms: f64,
    /// Echo of the delivered segment's `source_range`
    /// (data-model E2) — never adapter-computed
    /// (segment-resume CHK030). It links the chunk back to
    /// its `DocumentSegment` for resume alignment
    /// (`spec.md:101`).
    pub source_range_ref: SourceRange,
    /// The pre-roll remaining, in milliseconds — `Some`
    /// only while pre-roll is buffered, always `<=`
    /// [`AudioChunk::MAX_PRE_ROLL_MS`] (FR-015a / R10;
    /// `data-model.md:145`); `None` when no pre-roll is
    /// outstanding. Pre-roll stops when playback pauses,
    /// and there is no continuous synthesis while idle
    /// (`contracts/adapter-handoff.md:63`).
    pub pre_roll_remaining_ms: Option<u32>,
}

/// Why a delivery was refused at construction time.
///
/// A construction-time refusal, not an engine-observed
/// failure: the closed [`EngineError`] set reports what
/// the engine observed, and a pre-roll over-bound is not
/// a member of the `spec.md:86` condition→variant table —
/// mapping it to an [`EngineError`] member would corrupt
/// the closed failure taxonomy (C4-A). The refusal follows
/// the `FailureReportError::DetailTooLong` precedent
/// (`src/failure.rs:183-205`): an attempt over a bound is
/// refused with the offending value and the bound named,
/// never silently clamped.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeliveryError {
    /// The delivered audio failed the fail-closed format
    /// gate — the closed-set [`EngineError`] member named
    /// by [`check_delivery`]: [`EngineError::ChannelMismatch`]
    /// for a channel-count deviation,
    /// [`EngineError::RateMismatch`] for a rate or width
    /// deviation (the closed set names no width-specific
    /// variant — Principle VII), or
    /// [`EngineError::InitFailed`] for an empty, corrupt
    /// synthesis result.
    #[error("the delivered audio failed the format gate: {0:?}")]
    Format(EngineError),
    /// The pre-roll remaining exceeded
    /// [`AudioChunk::MAX_PRE_ROLL_MS`] — refused with the
    /// offending value and the bound named, never clamped:
    /// a silent clamp would be a silent misreport (the
    /// `FailureReportError::DetailTooLong` precedent,
    /// `src/failure.rs:183-205`).
    #[error("pre-roll remaining is {ms} ms; the pre-roll bound is {max_ms} ms")]
    PreRollTooLong {
        /// The refused pre-roll remaining, in milliseconds.
        ms: u32,
        /// The pre-roll bound the value exceeded.
        max_ms: u32,
    },
}

impl AudioChunk {
    /// The pre-roll bound (FR-015a / R10;
    /// `data-model.md:145`): pre-roll is buffered for at
    /// most 60 s, and pre-roll stops when playback pauses.
    pub const MAX_PRE_ROLL_MS: u32 = 60_000;

    /// Deliver a synthesized buffer as one [`AudioChunk`],
    /// validating fail-closed, in order (FR-024b / SC-035):
    ///
    /// 1. `pcm_data` must be non-empty — an empty delivery
    ///    is a corrupt synthesis result, refused as
    ///    [`DeliveryError::Format`] naming
    ///    [`EngineError::InitFailed`]
    ///    (`contracts/adapter-handoff.md:52`; an empty
    ///    buffer is the silent-success defect mode
    ///    `src/synth.rs` exists to prevent);
    /// 2. the shape is delegated to [`check_delivery`] —
    ///    the single shape authority (FR-044) — with the
    ///    duration derived from `pcm_data.len()`, so a
    ///    rate, width or channel deviation names a member
    ///    of the closed [`EngineError`] set and playback
    ///    does not start silently (R1/R2; FR-035). The
    ///    32-bit width is a type-level fact of `Vec<f32>`,
    ///    so [`DECLARED_WIDTH_BITS`] is passed as the
    ///    width;
    /// 3. `pre_roll_remaining_ms`, when `Some`, must not
    ///    exceed [`AudioChunk::MAX_PRE_ROLL_MS`] — an
    ///    attempt above the bound is refused with
    ///    [`DeliveryError::PreRollTooLong`], never
    ///    clamped.
    ///
    /// `source_range_ref` is echoed verbatim onto the
    /// chunk (segment-resume CHK030) — never
    /// adapter-computed.
    ///
    /// # Errors
    ///
    /// * [`DeliveryError::Format`] when `pcm_data` is
    ///   empty (naming [`EngineError::InitFailed`]) or
    ///   when the derived shape fails [`check_delivery`]
    ///   (naming the closed-set member the gate names).
    /// * [`DeliveryError::PreRollTooLong`] when
    ///   `pre_roll_remaining_ms` is `Some` and exceeds
    ///   [`AudioChunk::MAX_PRE_ROLL_MS`].
    pub fn deliver(
        pcm_data: Vec<f32>,
        source_range_ref: SourceRange,
        pre_roll_remaining_ms: Option<u32>,
    ) -> Result<Self, DeliveryError> {
        // Gate 1 — a corrupt synthesis result fails
        // closed (`contracts/adapter-handoff.md:52`,
        // "InitFailed if synthesis corrupt"): an empty
        // buffer is the silent-success defect mode
        // `src/synth.rs` exists to prevent, so it is
        // refused, never delivered.
        if pcm_data.is_empty() {
            return Err(DeliveryError::Format(EngineError::InitFailed));
        }
        // Gate 2 — the shape gate is delegated to
        // format_check::check_delivery, the single
        // shape authority (FR-044), with the duration
        // DERIVED from the buffer's own length
        // (data-model E2): pcm_data.len() / rate_hz *
        // 1000 — the identical computation the
        // delivery-unit tests freeze
        // (tests/audio_chunk.rs:231-232, :516-518).
        // The declared rate, channels and width are
        // passed as declared — the `Vec<f32>` type
        // makes the 32-bit width a type-level fact.
        let sample_count = pcm_data.len();
        #[expect(
            clippy::cast_precision_loss,
            reason = "data-model E2 mandates deriving duration_ms as pcm_data.len() / rate_hz * 1000; f64 has no lossless From<usize>, so the usize -> f64 cast is unavoidable — the identical computation is frozen by the delivery-unit tests (tests/audio_chunk.rs:231-232, :516-518) and is measured against the half-sample band that format_check.rs:130-133 sanctions the same cast for; a >2^53-sample buffer (over 2.8 years of audio at 22 050 Hz) is unreachable in memory"
        )]
        let duration_ms = sample_count as f64 / f64::from(DECLARED_RATE_HZ) * 1000.0;
        check_delivery(
            DECLARED_RATE_HZ,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            sample_count,
            duration_ms,
        )
        .map_err(DeliveryError::Format)?;
        // Gate 3 — the pre-roll bound (FR-015a / R10):
        // an attempt above the bound is refused with the
        // offending value and the bound named, never
        // clamped — a silent clamp would be a silent
        // misreport (the FailureReportError::DetailTooLong
        // refusal precedent, src/failure.rs:183-205).
        if let Some(ms) = pre_roll_remaining_ms
            && ms > Self::MAX_PRE_ROLL_MS
        {
            return Err(DeliveryError::PreRollTooLong {
                ms,
                max_ms: Self::MAX_PRE_ROLL_MS,
            });
        }
        Ok(Self {
            pcm_data,
            rate_hz: DECLARED_RATE_HZ,
            channels: DECLARED_CHANNELS,
            duration_ms,
            source_range_ref,
            pre_roll_remaining_ms,
        })
    }
}
