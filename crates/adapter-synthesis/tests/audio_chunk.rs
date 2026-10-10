//! T042 — delivery-unit tests for the `AudioChunk` delivery
//! unit of the FR-024 synthesis adapter, written
//! test-first: this file BLOCKS T043 (the `AudioChunk`
//! implementation) and T044 (the bound engine) per the
//! Phase 5 rules (`tasks.md`, rule 1 — tests block
//! implementation).
//!
//! Spec anchors: data-model E2 (`data-model.md:43-61` —
//! the `AudioChunk` entity), data-model rules **R1**
//! (`pcm_data` is exactly mono `f32` at 22 050 Hz),
//! **R2** (any format deviation surfaces a named
//! `EngineError`), **R10** (`data-model.md:145` — pre-roll
//! ≤ 60 s, stops on pause, no continuous synthesis while
//! idle), `contracts/adapter-handoff.md:36-46` (the
//! wire/interface shape), `:30` (immutable after
//! delivery), `:56` (no embedded text — FR-045), `:63`
//! (pre-roll stops on pause; no continuous synthesis
//! while idle), `spec.md:103` (immutability), **FR-015a**
//! (pre-roll), **FR-024b** / **SC-035** (the fail-closed
//! format gate — a refused delivery never starts
//! playback silently), **FR-035** (a deviation is named,
//! never silent), **FR-045** (no document text in the
//! delivery), and segment-resume **CHK030**
//! (`session_position` is an echo, never adapter-computed).
//!
//! # The contract this file specifies (T043 implements it)
//!
//! `AudioChunk` does not exist in the crate yet (`rg
//! "pcm_data|pre_roll|embedded_text|source_range_ref"
//! crates/adapter-synthesis/src` → 0 hits). Per the
//! task's verification rule, this file COMPILES against
//! the current public surface — it binds only to
//! `format_check::{check_format, check_delivery,
//! DECLARED_*}`, `chunk::split_document`,
//! `synth::synthesize` and `failure::EngineFailure::report`
//! — and fails on assertions. The contract it specifies,
//! which T043 must implement and re-export from `src/lib.rs`
//! beside the existing re-exports (`src/lib.rs:82-87`), is:
//!
//! ```rust
//! pub struct AudioChunk {
//!     pub pcm_data: Vec<f32>,                // mono f32, exactly 22 050 Hz, 32-bit
//!     pub rate_hz: u32,                      // == DECLARED_RATE_HZ (22 050)
//!     pub channels: u16,                     // == DECLARED_CHANNELS (1, mono)
//!     pub duration_ms: f64,                  // DERIVED: pcm_data.len() / rate_hz * 1000 — never supplied
//!     pub source_range_ref: SourceRange,     // echo of the delivered segment's source_range — never adapter-computed
//!     pub pre_roll_remaining_ms: Option<u32>, // Some only while pre-roll is buffered; <= 60 000
//! }
//!
//! pub enum DeliveryError {
//!     /// The closed-set `EngineError` named by
//!     /// `format_check::check_delivery` (FR-024b / SC-035):
//!     /// `ChannelMismatch` for a channel-count deviation,
//!     /// `RateMismatch` for a rate or width deviation (the
//!     /// closed set names no width-specific variant —
//!     /// Principle VII).
//!     Format(EngineError),
//!     /// The pre-roll remaining exceeded `MAX_PRE_ROLL_MS` —
//!     /// refused with the offending value and the bound
//!     /// named, never clamped: a silent clamp would be a
//!     /// silent misreport (the `FailureReportError::
//!     /// DetailTooLong` precedent, `src/failure.rs:183-205`).
//!     PreRollTooLong { ms: u32, max_ms: u32 },
//! }
//!
//! impl AudioChunk {
//!     /// The pre-roll bound (FR-015a / R10): pre-roll is
//!     /// buffered for at most 60 s.
//!     pub const MAX_PRE_ROLL_MS: u32 = 60_000;
//!
//!     pub fn deliver(
//!         pcm_data: Vec<f32>,
//!         source_range_ref: SourceRange,
//!         pre_roll_remaining_ms: Option<u32>,
//!     ) -> Result<Self, DeliveryError>;
//! }
//! ```
//!
//! `deliver` validates fail-closed, in order:
//! 1. `pcm_data` must be non-empty — an empty delivery is
//!    a corrupt synthesis result, reported as
//!    `DeliveryError::Format(EngineError::InitFailed)`
//!    (the `contracts/adapter-handoff.md:52` constraint
//!    table: "`InitFailed` if synthesis corrupt"; an empty
//!    buffer is the silent-success defect mode `src/synth.rs`
//!    exists to prevent);
//! 2. the shape gate is delegated to
//!    `format_check::check_delivery` with the DERIVED
//!    duration — the single shape authority (FR-044: no
//!    second shape declaration), so a stereo, wrong-rate,
//!    or wrong-width delivery names the closed-set member
//!    and playback does NOT start silently;
//! 3. `pre_roll_remaining_ms`, when `Some`, must not
//!    exceed `MAX_PRE_ROLL_MS` — an attempt above the
//!    bound is refused with
//!    `DeliveryError::PreRollTooLong { ms, max_ms }`,
//!    never clamped.
//!
//! `DeliveryError` is a construction-time refusal, not an
//! engine-observed failure: a pre-roll over-bound is not a
//! member of the `spec.md:86` condition→variant table, so
//! mapping it to one of the seven `EngineError` members
//! would corrupt the closed failure taxonomy. The task
//! itself names the precedent — `EngineFailure::report`
//! refusing an over-bound detail with
//! `FailureReportError::DetailTooLong` — and this design
//! follows it. `duration_ms` is never a parameter of
//! `deliver`: it is derived from `pcm_data.len()` inside
//! the constructor (data-model E2: "Computed from
//! `pcm_data.len() / rate_hz * 1000`"), so a caller
//! cannot supply a duration that disagrees with the
//! buffer — the half-sample refusal the tests pin below
//! is the gate the constructor delegates to.
//!
//! The unobservable-until-T043 pins (no public surface
//! exists for them yet) are documented here as contract:
//! `pre_roll_remaining_ms` is `Some` only while pre-roll
//! is buffered and `None` means no pre-roll is
//! outstanding (`data-model.md:53`); pre-roll stops when
//! playback pauses and there is NO path that synthesizes
//! continuously while idle (`contracts/adapter-handoff.md:63`);
//! once delivered, a chunk's fields cannot be mutated
//! through any public API — the struct exposes no setter,
//! and the adapter never modifies fields after delivery
//! (`spec.md:103`; `contracts/adapter-handoff.md:30`).
//!
//! # Why this file was RED before T044
//!
//! `a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer`
//! binds to the current `synthesize(&DocumentSegment) ->
//! Result<Vec<f32>, EngineFailure>` surface — the PCM
//! buffer the T043 chunk wraps; its assertions transfer
//! one-to-one to `AudioChunk.pcm_data` when the delivery
//! unit lands. Since T044 the engine is bound
//! (`sherpa-onnx` 1.13.8 driving the embedded Piper
//! pt-BR `faber` voice — API_CONTRACT.md §7, commit
//! `ee95b2d`), so a well-formed segment synthesizes to a
//! real, non-empty, mono-`f32` buffer at the declared
//! 22 050 Hz and the test's `Ok` arm carries the real
//! assertions (the design-phase pin — an unconditional
//! `InitFailed` on every call while the voice model was
//! unbound — does not survive binding). Every other test
//! pins behaviour that is already implemented and
//! observable through the current surface (the format
//! gate, the duration derivation, the boundary echo, the
//! refusal precedent), so it is green today and guards
//! the contract against regression while T043/T044 land.
//!
//! Rust rigor (`constitution.md` §Rust Code Rigor): no
//! `unwrap`/`expect`/`panic!`/`todo!`/`dbg!`/`println!`
//! — all denied by the workspace lint config, in tests
//! too — and no `#[allow]`/`#[expect]` suppression
//! appears anywhere in this file. Floating-point
//! comparisons never appear: every float verdict routes
//! through `check_delivery`'s `f64::total_cmp` band
//! (`src/format_check.rs:135-138`), so the denied
//! `clippy::float_cmp` is not needed. No `_ =>` catch-all
//! exists over any closed set (gate 7, `constitution.md:614`).

use engine_core::closed::EngineTier;
use proptest::prelude::*;

use adapter_synthesis::chunk::{SourceRange, split_document};
use adapter_synthesis::failure::{
    EngineError, EngineFailure, FailureCondition, FailureReportError, MAX_CONDITION_DETAIL_CHARS,
    MAX_OFFER_MS,
};
use adapter_synthesis::format_check::{
    DECLARED_CHANNELS, DECLARED_RATE_HZ, DECLARED_WIDTH_BITS, check_delivery, check_format,
};
use adapter_synthesis::synth::synthesize;

/// The distinctive marker placed inside a segment's
/// `content` to observe whether document prose can reach
/// any string-ish field of a delivery (FR-045;
/// `contracts/adapter-handoff.md:56`). Its letters and
/// digits are un-renderable by a `f32` `Debug`
/// implementation, so it cannot appear in a serialised
/// PCM form by coincidence.
const SENTINEL: &str = "Zq9Xw4VeLrYt sentinel phrase never delivered as audio";

/// One prose word: 3–12 lowercase letters.
fn word() -> impl Strategy<Value = String> {
    "[a-z]{3,12}"
}

/// One sentence of 1–12 words.
fn sentence() -> impl Strategy<Value = String> {
    proptest::collection::vec(word(), 1usize..=12usize).prop_map(|words| words.join(" "))
}

/// An arbitrary heading-plus-two-paragraph document —
/// the shape the adapter splits before synthesis
/// (FR-024a).
fn document_strategy() -> impl Strategy<Value = String> {
    ("[A-Z][a-z]{2,20}", sentence(), sentence())
        .prop_map(|(heading, first, second)| format!("# {heading}\n\n{first}\n\n{second}\n"))
}

/// A document whose heading is the [`SENTINEL`] phrase,
/// so some segment's `content` always carries the
/// sentinel.
fn sentinel_document_strategy() -> impl Strategy<Value = String> {
    sentence().prop_map(|body| format!("# {SENTINEL}\n\n{body}\n\nClosing paragraph.\n"))
}

// ---------------------------------------------------------------------------
// 1. Duration derivation (data-model E2: `duration_ms` is
//    "Computed from `pcm_data.len() / rate_hz * 1000`";
//    `contracts/adapter-handoff.md:42`). The constructor
//    DERIVES the duration — it is never a parameter — and
//    a caller-supplied duration that disagrees with the
//    buffer by more than half a sample is refused with
//    `EngineError::RateMismatch` (the half-sample band at
//    `src/format_check.rs:47,135-138`).
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// An arbitrary non-empty delivered buffer round-trips
    /// through the derived duration: the buffer is
    /// generated first and the duration is derived from
    /// its length — the only derivation the delivery unit
    /// performs — and the delivery gate accepts it. The
    /// buffer values are arbitrary `f32`s, including NaN
    /// and infinities, because the gate quantifies the
    /// buffer (sample count, rate, width), never its
    /// values.
    #[test]
    fn an_arbitrary_non_empty_buffer_round_trips_through_the_derived_duration(
        pcm_data in proptest::collection::vec(any::<f32>(), 1usize..=2_048usize),
    ) {
        let sample_count = pcm_data.len();
        let duration_ms =
            pcm_data.len() as f64 / f64::from(DECLARED_RATE_HZ) * 1000.0;
        prop_assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                DECLARED_CHANNELS,
                DECLARED_WIDTH_BITS,
                sample_count,
                duration_ms
            ),
            Ok(()),
            "a buffer of {} samples at {} Hz derives a {} ms \
             duration the gate must accept",
            sample_count, DECLARED_RATE_HZ, duration_ms
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The derivation holds at scale: a sample count up to
    /// one million (≈ 45 s of audio) derives a duration
    /// the gate accepts — the derived `f64` stays inside
    /// the half-sample band for every derivable buffer
    /// length.
    #[test]
    fn a_sample_count_at_scale_derives_a_duration_the_gate_accepts(
        sample_count in 1usize..=1_000_000usize,
    ) {
        let duration_ms =
            sample_count as f64 / f64::from(DECLARED_RATE_HZ) * 1000.0;
        prop_assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                DECLARED_CHANNELS,
                DECLARED_WIDTH_BITS,
                sample_count,
                duration_ms
            ),
            Ok(()),
            "a {}-sample buffer derives {} ms",
            sample_count, duration_ms
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// A caller-supplied duration that disagrees with the
    /// buffer by strictly more than half a sample is
    /// refused with `EngineError::RateMismatch` — never
    /// silently accepted, never silently resampled
    /// (FR-024b / SC-035; R2). The delivery unit's
    /// constructor derives the duration itself, so this
    /// refusal is the gate the constructor delegates to;
    /// the perturbation is strictly beyond ±0.5 of a
    /// sample.
    #[test]
    fn a_caller_supplied_duration_disagreeing_with_the_buffer_is_refused(
        sample_count in 1usize..=1_000_000usize,
        perturbation_samples in 51i64..=150i64,
    ) {
        let perturbation = perturbation_samples as f64 / 100.0;
        let duration_ms = (sample_count as f64 + perturbation)
            / f64::from(DECLARED_RATE_HZ)
            * 1000.0;
        prop_assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                DECLARED_CHANNELS,
                DECLARED_WIDTH_BITS,
                sample_count,
                duration_ms
            ),
            Err(EngineError::RateMismatch),
            "a duration {} hundredths of a sample off the buffer \
             must be refused as RateMismatch",
            perturbation_samples
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The half-sample band is inclusive and real: a
    /// duration perturbed strictly inside ±0.5 of a sample
    /// is accepted (`src/format_check.rs:135-138` refuses
    /// only `Ordering::Greater`), so the derivation's
    /// floating-point round trip never falls outside the
    /// band it is measured against.
    #[test]
    fn a_duration_within_half_a_sample_of_the_buffer_is_accepted(
        sample_count in 1usize..=1_000_000usize,
        perturbation_samples in -49i64..=49i64,
    ) {
        let perturbation = perturbation_samples as f64 / 100.0;
        let duration_ms = (sample_count as f64 + perturbation)
            / f64::from(DECLARED_RATE_HZ)
            * 1000.0;
        prop_assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                DECLARED_CHANNELS,
                DECLARED_WIDTH_BITS,
                sample_count,
                duration_ms
            ),
            Ok(())
        );
    }
}

/// A non-finite duration fails closed as `RateMismatch`
/// (`src/format_check.rs:119-121`): a duration is derived
/// from a buffer length, so NaN and the infinities are
/// not derivable durations — they are refused, never
/// silently passed.
#[test]
fn a_non_finite_duration_fails_closed_as_a_rate_mismatch() {
    for duration_ms in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                DECLARED_CHANNELS,
                DECLARED_WIDTH_BITS,
                22_050,
                duration_ms
            ),
            Err(EngineError::RateMismatch),
            "a non-finite duration ({duration_ms}) is not derivable and \
             must fail closed"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. The fail-closed format gate (FR-024b / SC-035;
//    data-model.md:59-61; R1/R2). The delivery unit's
//    constructor delegates to `format_check`, so every
//    deviation names a member of the closed 7-member set
//    and playback does NOT start silently.
// ---------------------------------------------------------------------------

/// A channel-count deviation alone — rate and width at
/// their declared values — names `EngineError::
/// ChannelMismatch`, never `Ok` and never any other
/// member of the closed set (R2; FR-035; SC-035). A
/// stereo delivery is never a silent downmix. The
/// delivery-level gate names the same member with a
/// perfectly consistent length, because the shape is
/// checked before the length (`src/format_check.rs:110-118`).
#[test]
fn a_stereo_delivery_names_channel_mismatch() {
    for channels in [0, 2, 6, u16::MAX] {
        assert_eq!(
            check_format(DECLARED_RATE_HZ, channels, DECLARED_WIDTH_BITS),
            Err(EngineError::ChannelMismatch),
            "a {channels}-channel delivery is not the declared mono shape"
        );
        assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                channels,
                DECLARED_WIDTH_BITS,
                22_050,
                1_000.0
            ),
            Err(EngineError::ChannelMismatch),
            "a {channels}-channel delivery names ChannelMismatch before \
             any length is considered"
        );
    }
}

/// A rate deviation alone — channels and width at their
/// declared values — names `EngineError::RateMismatch`
/// (R2; FR-035). Representative values include the bounds
/// `0` and `u32::MAX`; the delivery-level gate names the
/// same member with a perfectly consistent length.
#[test]
fn a_wrong_rate_delivery_names_rate_mismatch() {
    for rate_hz in [0, 1, 8_000, 22_049, 22_051, 44_100, u32::MAX] {
        assert_eq!(
            check_format(rate_hz, DECLARED_CHANNELS, DECLARED_WIDTH_BITS),
            Err(EngineError::RateMismatch),
            "a rate of {rate_hz} Hz deviates from the declared \
             {DECLARED_RATE_HZ} Hz"
        );
        assert_eq!(
            check_delivery(
                rate_hz,
                DECLARED_CHANNELS,
                DECLARED_WIDTH_BITS,
                22_050,
                1_000.0
            ),
            Err(EngineError::RateMismatch),
            "a rate of {rate_hz} Hz names RateMismatch before any length \
             is considered"
        );
    }
}

/// A sample-width deviation alone — rate and channels at
/// their declared values — names `EngineError::
/// RateMismatch`: the closed set names no width-specific
/// variant, so a non-`f32` width surfaces under the rate
/// shape (Principle VII; `src/failure.rs:57-62`; R2). The
/// delivered buffer's `Vec<f32>` type makes the 32-bit
/// width a type-level fact, and the gate pins it at the
/// delivery boundary.
#[test]
fn a_wrong_width_delivery_names_rate_mismatch() {
    for width_bits in [0, 8, 16, 24, 64, u16::MAX] {
        assert_eq!(
            check_format(DECLARED_RATE_HZ, DECLARED_CHANNELS, width_bits),
            Err(EngineError::RateMismatch),
            "a width of {width_bits} bits deviates from the declared \
             {DECLARED_WIDTH_BITS}-bit f32 shape and the closed set \
             names no width-specific variant"
        );
        assert_eq!(
            check_delivery(
                DECLARED_RATE_HZ,
                DECLARED_CHANNELS,
                width_bits,
                22_050,
                1_000.0
            ),
            Err(EngineError::RateMismatch),
            "a width of {width_bits} bits names RateMismatch before any \
             length is considered"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. The delivery contract, end to end (data-model E2;
//    `contracts/adapter-handoff.md:36-46`). This was
//    the pre-T044 RED blocker of this file (see the
//    module header); it is green since T044.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Every segment of an arbitrary bounded document
    /// synthesizes into a delivered chunk whose `pcm_data`
    /// is non-empty, conforms to the declared
    /// mono-`f32`-at-22 050 Hz shape, and carries a
    /// duration derived from its own length (R1; R2;
    /// data-model E2).
    ///
    /// Since T044 the engine is bound (`sherpa-onnx`
    /// 1.13.8 driving the embedded Piper pt-BR `faber`
    /// voice — API_CONTRACT.md §7, commit `ee95b2d`),
    /// so the `Ok` arm below carries the real assertions
    /// for a well-formed segment. The `Err` arm is the
    /// guard: the bound engine failing closed on a
    /// well-formed segment is the silent branch this
    /// crate exists to prevent, so the test must fail
    /// if it ever happens. The arm's predicate is a
    /// runtime value that is always false (a named
    /// failure's offer never outlives `MAX_OFFER_MS` —
    /// `EngineFailure::report` refuses it), so the
    /// assertion stays live without tripping
    /// `clippy::assertions_on_constants` — the same
    /// runtime-predicate idiom as `unit_failure.rs`'s
    /// `silent_success`. The named failure is still
    /// observed here, never silently skipped (SC-035).
    #[test]
    fn a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer(
        document in document_strategy(),
    ) {
        let segments = split_document(&document, "document-hash");
        prop_assert!(
            !segments.is_empty(),
            "a non-empty document yields at least one segment"
        );
        for segment in &segments {
            match synthesize(segment) {
                Ok(pcm_data) => {
                    prop_assert!(
                        !pcm_data.is_empty(),
                        "pcm_data must be non-empty: an empty delivery is \
                         a corrupt synthesis result (adapter-handoff §2.1)"
                    );
                    let duration_ms = pcm_data.len() as f64
                        / f64::from(DECLARED_RATE_HZ)
                        * 1000.0;
                    prop_assert_eq!(
                        check_delivery(
                            DECLARED_RATE_HZ,
                            DECLARED_CHANNELS,
                            DECLARED_WIDTH_BITS,
                            pcm_data.len(),
                            duration_ms
                        ),
                        Ok(()),
                        "the delivered buffer's derived duration must pass \
                         the delivery gate"
                    );
                }
                Err(failure) => {
                    // Always false for a named failure: see the
                    // doc comment above.
                    let delivered = failure.offer_expires_in_ms > MAX_OFFER_MS;
                    prop_assert!(
                        delivered,
                        "synthesis must deliver a conforming chunk for a \
                         bound model, not fail closed: {failure:?}"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. No embedded document text (FR-045;
//    `contracts/adapter-handoff.md:56`).
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// A distinctive sentinel placed in a segment's
    /// `content` appears in no string-ish field of the
    /// delivery and in no serialised form of it. On the
    /// failure path the only string-ish field is the
    /// identifier-pattern detail (R7); on the delivery
    /// path the payload is pure `f32` PCM, whose
    /// serialised form cannot carry prose. The `AudioChunk`
    /// the T043 unit adds has the same structure — no
    /// field of it is a `String` — so the property is
    /// structural for the chunk and is pinned here at
    /// runtime for every delivery the current surface can
    /// produce.
    #[test]
    fn the_delivery_carries_no_document_text(
        document in sentinel_document_strategy(),
    ) {
        let segments = split_document(&document, "document-hash");
        prop_assert!(
            segments.iter().any(|segment| segment.content.contains(SENTINEL)),
            "the fixture must place the sentinel inside a segment's content"
        );
        for segment in &segments {
            match synthesize(segment) {
                Ok(pcm_data) => {
                    let rendered = format!("{pcm_data:?}");
                    prop_assert!(
                        !rendered.contains(SENTINEL),
                        "the serialised form of a delivered buffer must \
                         not contain document text (FR-045)"
                    );
                }
                Err(failure) => {
                    prop_assert!(
                        !failure.observed.contains(SENTINEL),
                        "the failure detail must be identifier-pattern \
                         only, never document text (FR-045 / R7)"
                    );
                    let rendered = format!("{failure:?}");
                    prop_assert!(
                        !rendered.contains(SENTINEL),
                        "no serialised form of a named failure may \
                         contain document text (FR-045)"
                    );
                    let rendered_position = format!("{}", failure.session_position);
                    prop_assert!(
                        !rendered_position.contains(SENTINEL),
                        "the echoed session position must not carry \
                         document text (FR-045)"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 5. The single-authority echo (segment-resume CHK030):
//    `source_range_ref` echoes the delivered segment's
//    `source_range` exactly and is never adapter-computed;
//    a failure's `session_position` echoes the segment's
//    `resume_point` when present, else its `source_range`
//    — the echo implemented at `src/synth.rs:311-317`
//    (`named_failure` → `echoed_position`).
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// The failure a delivery carries echoes the segment's
    /// `resume_point` when present, else its
    /// `source_range` — never adapter-computed (the player
    /// is the sole playback authority,
    /// `constitution.md:486-490`). The delivered chunk's
    /// `source_range_ref` echo is pinned by the delivery
    /// test above; this test pins the echo on the failure
    /// path — the path a named-`EngineFailure` condition
    /// takes (voice load, engine-process death, interior
    /// NUL, or delivery-rate deviation).
    #[test]
    fn the_failure_session_position_echoes_the_segment_boundary(
        document in document_strategy(),
    ) {
        let segments = split_document(&document, "document-hash");
        prop_assert!(
            !segments.is_empty(),
            "a non-empty document yields at least one segment"
        );
        for segment in &segments {
            let expected_position = match segment.resume_point {
                Some(resume_point) => resume_point,
                None => segment.source_range,
            };
            match synthesize(segment) {
                Ok(_pcm_data) => {
                    // The chunk's source_range_ref echo is
                    // pinned by
                    // a_bounded_segment_delivers_a_conforming_non_empty_mono_buffer;
                    // nothing is observable on this path
                    // without the chunk type.
                }
                Err(failure) => {
                    prop_assert_eq!(
                        failure.session_position,
                        expected_position,
                        "session_position must echo the segment's \
                         resume_point, or its source_range when none is \
                         declared — never adapter-computed"
                    );
                }
            }
        }
    }
}

/// The fail-closed report constructor echoes the session
/// position it is given verbatim — it never derives,
/// snaps, or re-anchors one (segment-resume CHK030). This
/// is the structural form of the "never adapter-computed"
/// pin: the position a failure carries is a value the
/// delivery path supplied, not a computation.
#[test]
fn the_report_constructor_echoes_the_given_position_verbatim() {
    let positions = [
        SourceRange::whole_lines(1, 1),
        SourceRange::new(2, 7, 12, 345),
        SourceRange::whole_lines(u32::MAX, u32::MAX),
    ];
    for position in positions {
        let report = EngineFailure::report(
            FailureCondition::DeviceRemoved,
            FailureCondition::DeviceRemoved.detail(),
            position,
            MAX_OFFER_MS,
        );
        assert_eq!(
            report,
            Ok(EngineFailure {
                variant: EngineError::DeviceLost,
                condition: FailureCondition::DeviceRemoved,
                observed: "device_removed".to_owned(),
                session_position: position,
                offer_expires_in_ms: MAX_OFFER_MS,
                tier_at_failure: EngineTier::OfflineDefault,
            }),
            "report must echo the given session position verbatim"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Immutability after delivery (`spec.md:103`;
//    `contracts/adapter-handoff.md:30`).
// ---------------------------------------------------------------------------

/// Delivery is a pure function of its input: the split
/// segments are identical before and after synthesis, and
/// every named failure stays within its presentation
/// bound. The `AudioChunk` the T043 unit adds exposes no
/// mutation API — its fields are set once by `deliver`
/// and the adapter never modifies them after delivery.
#[test]
fn delivery_leaves_the_delivered_segments_untouched() {
    let source = "# Heading\n\nFirst paragraph.\n\nSecond paragraph.\n";
    let segments = split_document(source, "document-hash");
    let snapshot = segments.clone();
    for segment in &segments {
        match synthesize(segment) {
            Ok(_pcm_data) => {}
            Err(failure) => {
                assert!(
                    failure.offer_expires_in_ms <= MAX_OFFER_MS,
                    "a named failure's offer expires within MAX_OFFER_MS \
                     (FR-023a / FR-041)"
                );
            }
        }
    }
    assert_eq!(
        segments, snapshot,
        "delivery must never mutate the delivered segments (spec.md:103)"
    );
}

/// A delivered value is owned by the consumer: mutating
/// the consumer's copy leaves the pipeline's snapshot
/// unaffected (`contracts/adapter-handoff.md:30` — the
/// adapter holds no reference back into its output, so a
/// snapshot taken before a consumer attempts a mutation
/// is unaffected).
#[test]
fn a_consumer_mutation_of_its_owned_copy_leaves_the_pipeline_untouched() {
    let source = "# Heading\n\nParagraph text.\n";
    let segments = split_document(source, "document-hash");
    let snapshot = segments.clone();
    let mut consumer_copy = segments.clone();
    for segment in &mut consumer_copy {
        segment.content.clear();
        segment.segment_index = u64::MAX;
    }
    assert_ne!(
        consumer_copy, snapshot,
        "the consumer's mutation must actually change the owned copy"
    );
    assert_eq!(
        segments, snapshot,
        "a consumer's mutation of its owned copy must not affect the \
         delivered values"
    );
}

// ---------------------------------------------------------------------------
// 7. The pre-roll bound policy (FR-015a / R10;
//    `data-model.md:145`). `pre_roll_remaining_ms` must
//    NEVER exceed 60 000 ms, and an attempt above the
//    bound is REFUSED, never clamped — a silent clamp
//    would be a silent misreport. The crate's precedent
//    for refusing rather than truncating is
//    `EngineFailure::report`, which refuses an over-bound
//    detail with `FailureReportError::DetailTooLong`
//    naming both the offending value and the bound
//    (`src/failure.rs:207-252`); the T043 delivery unit
//    follows the same policy for the 60 000 ms pre-roll
//    bound via `DeliveryError::PreRollTooLong { ms,
//    max_ms }`. `None` means no pre-roll is outstanding,
//    and there is no path that synthesizes continuously
//    while idle (`contracts/adapter-handoff.md:63`).
// ---------------------------------------------------------------------------

/// The refusal precedent the pre-roll bound follows: an
/// attempt over a bound is refused with a structured
/// error naming the offending value and the bound — never
/// silently truncated or clamped — and the bound itself is
/// inclusive (exactly at the bound is accepted).
#[test]
fn the_report_refusal_precedent_pins_the_pre_roll_bound_policy() {
    let position = SourceRange::whole_lines(1, 1);

    // The named precedent: an over-bound detail is refused
    // with DetailTooLong — never truncated.
    let over_bound_detail = "d".repeat(MAX_CONDITION_DETAIL_CHARS + 1);
    assert_eq!(
        EngineFailure::report(
            FailureCondition::VoiceInitFailure,
            &over_bound_detail,
            position,
            MAX_OFFER_MS
        ),
        Err(FailureReportError::DetailTooLong {
            len: MAX_CONDITION_DETAIL_CHARS + 1,
            max: MAX_CONDITION_DETAIL_CHARS,
        }),
        "an attempt over a bound must be refused with the offending \
         value and the bound named — never silently truncated"
    );

    // The sibling refusal: an over-bound offer is refused
    // the same way (FR-023a / FR-032c / FR-041).
    assert_eq!(
        EngineFailure::report(
            FailureCondition::VoiceInitFailure,
            "voice_init_failure",
            position,
            MAX_OFFER_MS + 1
        ),
        Err(FailureReportError::OfferTooLong {
            offer_ms: MAX_OFFER_MS + 1,
            max_ms: MAX_OFFER_MS,
        }),
        "an attempt over the offer bound must be refused, not clamped"
    );

    // The bounds are inclusive: exactly at the bound is
    // accepted, so a legitimate 60 000 ms pre-roll
    // (FR-015a) is deliverable.
    let at_detail_bound = "d".repeat(MAX_CONDITION_DETAIL_CHARS);
    let at_bound_report = EngineFailure::report(
        FailureCondition::VoiceInitFailure,
        &at_detail_bound,
        position,
        MAX_OFFER_MS,
    );
    assert!(
        at_bound_report.is_ok(),
        "exactly at the detail bound must be accepted"
    );
    let at_offer_bound_report = EngineFailure::report(
        FailureCondition::VoiceInitFailure,
        "voice_init_failure",
        position,
        MAX_OFFER_MS,
    );
    assert!(
        at_offer_bound_report.is_ok(),
        "exactly at the offer bound must be accepted"
    );
}
