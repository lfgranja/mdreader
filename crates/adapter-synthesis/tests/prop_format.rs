//! Property tests for `crates/adapter-synthesis` task **T018** — the
//! format gate (`src/format_check.rs`).
//!
//! Spec anchors: **FR-024b** (the delivered audio format is checked
//! before playback), **FR-035** (a format deviation is a named
//! failure, never a silent resample/downmix), **SC-035** (failure is
//! named, never silent), data-model rules **R1** (`AudioChunk` is
//! exactly mono `f32` at 22050 Hz) and **R2** (any format deviation
//! surfaces a named `EngineError`), checklist **CHK005** (all seven
//! `EngineError` variants mapped with condition→variant rules),
//! **CHK007** (the wire format is quantified:
//! `pcm_data.len() == (duration_ms / 1000.0) * 22050.0`,
//! `rate_hz == 22050`, `channels == 1`) and **CHK025** (the
//! mono-f32-at-22050-Hz assumption is validated by the
//! `format_check` mechanism, with `RateMismatch`/`ChannelMismatch`
//! specified for every deviation). All invariants below are normative
//! per **`API_CONTRACT` §6** (frozen interface) and §0 global rules.
//!
//! These tests bind to the frozen contract, not to the current stub:
//! `check_delivery` and the `DECLARED_*` constants do not exist yet
//! (rule 1 — tests BLOCK the core wave), so a red run is the expected
//! state. They exercise the *real* crate surface
//! (`adapter_synthesis::format_check::{check_format, check_delivery,
//! DECLARED_RATE_HZ, DECLARED_CHANNELS, DECLARED_WIDTH_BITS}`) and
//! the real `engine_core::closed::EngineError` 7-member closed set —
//! no local re-declaration of `EngineError` or of the check itself
//! (the F1 root cause), and no `unwrap`/`expect`/`panic!`/`println!`
//! — all denied by the workspace lint config, in tests too
//! (`constitution.md` §Rust Code Rigor). No `#[allow]`/`#[expect]`
//! suppression appears anywhere: the half-sample tolerance comparison
//! below routes through `f64::total_cmp` into the `Ordering` domain
//! so that no floating-point comparison operator — `clippy::float_cmp`,
//! denied via pedantic — is needed.
//!
//! `serde_json` is not a dev-dependency of this crate, so the serde
//! wire-name round-trip is not exercised here; the closed set is
//! pinned by the wildcard-free exhaustive `match` (a hypothetical
//! eighth member fails to compile — Principle VII), with the wire
//! names recorded as the contract table they document.

use std::cmp::Ordering;

use engine_core::closed::EngineError;
use proptest::prelude::*;

use adapter_synthesis::format_check::{
    DECLARED_CHANNELS, DECLARED_RATE_HZ, DECLARED_WIDTH_BITS, check_delivery, check_format,
};

/// Half a sample — the delivery length tolerance (`API_CONTRACT` §6:
/// `check_delivery` compares `pcm_sample_count` against
/// `(duration_ms / 1000.0) * DECLARED_RATE_HZ` "with a half-sample
/// tolerance, because `duration_ms` is a derived `f64`").
const HALF_SAMPLE: f64 = 0.5;

/// The seven `EngineError` members with their exact serde wire names
/// (`engine-core` `closed.rs`). The order is the declaration order of
/// the closed set — exactly these seven, no `Other`/`Unknown`
/// (Principle VII; CHK005; data-model E3).
const ENGINE_ERRORS: [(EngineError, &str); 7] = [
    (EngineError::Unreachable, "engine_unreachable"),
    (EngineError::InitFailed, "engine_init_failed"),
    (EngineError::RateMismatch, "rate_mismatch"),
    (EngineError::ChannelMismatch, "channel_mismatch"),
    (EngineError::DeviceRefused, "device_refused"),
    (EngineError::DeviceLost, "device_lost"),
    (EngineError::StartRefused, "start_refused"),
];

/// The wire name of each `EngineError` member. Wildcard-free
/// exhaustive `match`: it compiles only when the enum has exactly
/// the seven named variants — a hypothetical eighth member fails
/// this test target's build, which makes "exactly the seven
/// members, no `Other`/`Unknown`" mechanically decidable
/// (Principle VII; `constitution.md` §Closed Sets).
const fn wire_name(error: EngineError) -> &'static str {
    match error {
        EngineError::Unreachable => "engine_unreachable",
        EngineError::InitFailed => "engine_init_failed",
        EngineError::RateMismatch => "rate_mismatch",
        EngineError::ChannelMismatch => "channel_mismatch",
        EngineError::DeviceRefused => "device_refused",
        EngineError::DeviceLost => "device_lost",
        EngineError::StartRefused => "start_refused",
    }
}

/// Whether `pcm_sample_count` agrees with `duration_ms` at the
/// declared rate within half a sample (`API_CONTRACT` §6; CHK007:
/// `pcm_data.len() == (duration_ms / 1000.0) * 22050.0`). The
/// comparison goes through `f64::total_cmp` into the `Ordering`
/// domain so no floating-point comparison operator appears in this
/// file — `clippy::float_cmp` is denied workspace-wide (pedantic)
/// and this crate carries no lint suppressions (`API_CONTRACT` §0
/// rule 3).
#[expect(
    clippy::cast_precision_loss,
    reason = "the buffer length is bounded at 1 000 000 samples — far below f64's exact-integer range (2^53); the cast converts exactly"
)]
fn agrees_within_half_sample(pcm_sample_count: usize, duration_ms: f64) -> bool {
    let expected_samples = (duration_ms / 1000.0) * f64::from(DECLARED_RATE_HZ);
    let deviation = (pcm_sample_count as f64 - expected_samples).abs();
    !matches!(deviation.total_cmp(&HALF_SAMPLE), Ordering::Greater)
}

/// The `EngineError` closed set is exactly the seven members of
/// `engine_core::closed`, enumerated exhaustively with no `_ =>`
/// arm (Principle VII; CHK005). A hypothetical eighth member fails
/// to compile — the exhaustiveness is structural, not asserted.
///
/// The wire names are the contract table (`closed.rs` serde
/// renames). `serde_json` is not a dev-dependency of this crate, so
/// the round-trip itself is not exercised here; the table stands as
/// the documented contract and the exhaustive `match` pins the
/// member set.
#[test]
fn the_engine_error_closed_set_is_exactly_the_seven_members() {
    for (variant, wire) in ENGINE_ERRORS {
        assert_eq!(
            wire_name(variant),
            wire,
            "variant {variant:?} must serialize as the wire name {wire}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `check_format` is `Ok(())` **if and only if** the triple is
    /// exactly the declared handoff shape `(22050, 1, 32)` (R1;
    /// CHK025; API_CONTRACT §6). Any other triple is `Err` naming
    /// either `RateMismatch` or `ChannelMismatch` — never `Ok`, never
    /// any other member of the closed set (R2; FR-035; SC-035: a
    /// deviation is named, never silently passed).
    #[test]
    fn check_format_is_ok_iff_exactly_the_declared_handoff_shape(
        rate_hz in any::<u32>(),
        channels in any::<u16>(),
        width_bits in any::<u16>(),
    ) {
        let result = check_format(rate_hz, channels, width_bits);
        if rate_hz == DECLARED_RATE_HZ
            && channels == DECLARED_CHANNELS
            && width_bits == DECLARED_WIDTH_BITS
        {
            prop_assert_eq!(result, Ok(()));
        } else {
            prop_assert!(
                matches!(
                    result,
                    Err(EngineError::RateMismatch | EngineError::ChannelMismatch)
                ),
                "a deviation of ({rate_hz}, {channels}, {width_bits}) must \
                 name RateMismatch or ChannelMismatch — never Ok, never any \
                 other member; got {result:?}"
            );
        }
    }
}

/// Exactly the declared shape passes the gate: `(22050, 1, 32)` is
/// the only `Ok` triple (R1; CHK001; CHK025).
#[test]
fn exactly_the_declared_shape_is_ok() {
    assert_eq!(
        check_format(DECLARED_RATE_HZ, DECLARED_CHANNELS, DECLARED_WIDTH_BITS),
        Ok(())
    );
}

/// Condition→variant table row 1 (`API_CONTRACT` §6; data-model E2):
/// a rate deviation alone — with channels and width at their
/// declared values — names `RateMismatch`, never `Ok` and never
/// any other member. Representative values include the bounds
/// `0` and `u32::MAX` (CHK005; FR-035).
#[test]
fn a_rate_deviation_alone_names_rate_mismatch() {
    for rate_hz in [0, 1, 8_000, 22_049, 22_051, 44_100, u32::MAX] {
        assert_eq!(
            check_format(rate_hz, DECLARED_CHANNELS, DECLARED_WIDTH_BITS),
            Err(EngineError::RateMismatch),
            "rate {rate_hz} Hz deviates from the declared {DECLARED_RATE_HZ} Hz"
        );
    }
}

/// Condition→variant table row 2 (`API_CONTRACT` §6; data-model E2):
/// a channel-count deviation alone — with rate and width at their
/// declared values — names `ChannelMismatch`, never `Ok` and never
/// any other member. A stereo delivery is never a silent downmix
/// (CHK005; CHK006; FR-033). Representative values include `0`
/// and `u16::MAX`.
#[test]
fn a_channel_deviation_alone_names_channel_mismatch() {
    for channels in [0, 2, 6, u16::MAX] {
        assert_eq!(
            check_format(DECLARED_RATE_HZ, channels, DECLARED_WIDTH_BITS),
            Err(EngineError::ChannelMismatch),
            "channel count {channels} deviates from the declared mono \
             {DECLARED_CHANNELS}"
        );
    }
}

/// Condition→variant table row 3 (`API_CONTRACT` §6; data-model E2):
/// a sample-width deviation alone — with rate and channels at their
/// declared values — names `RateMismatch`: the closed set names no
/// width-specific variant, so a non-`f32` width surfaces under the
/// rate shape (Principle VII; CHK005). Representative values
/// include `0` and `u16::MAX`.
#[test]
fn a_width_deviation_names_rate_mismatch_because_the_set_has_no_width_variant() {
    for width_bits in [0, 8, 16, 24, 64, u16::MAX] {
        assert_eq!(
            check_format(DECLARED_RATE_HZ, DECLARED_CHANNELS, width_bits),
            Err(EngineError::RateMismatch),
            "width {width_bits} bits deviates from the declared \
             {DECLARED_WIDTH_BITS}-bit f32 shape and the closed set names \
             no width-specific variant"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// A delivered buffer whose sample count matches
    /// `(duration_ms / 1000.0) * 22050.0` within half a sample is
    /// accepted, and any disagreement is `RateMismatch` (CHK007;
    /// API_CONTRACT §6: the length check names `RateMismatch`). The
    /// shape is held at the declared values so only the length
    /// agreement varies. Non-finite `duration_ms` is excluded: it is
    /// not a derivable duration (`duration_ms` is computed from a
    /// buffer length), and the two comparison forms a conforming
    /// implementation may take diverge on NaN.
    #[test]
    fn delivery_sample_count_agrees_with_duration_within_half_a_sample(
        pcm_sample_count in any::<usize>(),
        duration_ms in any::<f64>(),
    ) {
        prop_assume!(duration_ms.is_finite());
        let result = check_delivery(
            DECLARED_RATE_HZ,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            pcm_sample_count,
            duration_ms,
        );
        if agrees_within_half_sample(pcm_sample_count, duration_ms) {
            prop_assert_eq!(result, Ok(()));
        } else {
            prop_assert_eq!(result, Err(EngineError::RateMismatch));
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The accepted half-sample band is real, not vacuous: a buffer
    /// whose length is the declared rate times the duration, perturbed
    /// strictly inside ±0.5 of a sample, is `Ok` (CHK007;
    /// API_CONTRACT §6). The perturbation is scaled hundredths of a
    /// sample, so it stays strictly inside the half-sample tolerance
    /// despite floating-point round-trip error.
    #[expect(
        clippy::cast_precision_loss,
        reason = "the strategy bounds the buffer length at 1 000 000 samples and the perturbation at ±1.50 samples — both far below f64's exact-integer range (2^53); every cast converts exactly"
    )]
    #[test]
    fn a_length_within_half_a_sample_of_the_duration_is_accepted(
        pcm_sample_count in 0usize..=1_000_000,
        perturbation_samples in -49i64..=49i64,
    ) {
        let perturbation = perturbation_samples as f64 / 100.0;
        let duration_ms =
            (pcm_sample_count as f64 + perturbation) / f64::from(DECLARED_RATE_HZ) * 1000.0;
        let result = check_delivery(
            DECLARED_RATE_HZ,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            pcm_sample_count,
            duration_ms,
        );
        prop_assert_eq!(result, Ok(()));
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The rejected side of the band: a buffer whose length disagrees
    /// with the duration by strictly more than half a sample is
    /// `RateMismatch` (CHK007; API_CONTRACT §6). The perturbation is
    /// strictly beyond ±0.5 of a sample.
    #[expect(
        clippy::cast_precision_loss,
        reason = "the strategy bounds the buffer length at 1 000 000 samples and the perturbation at ±1.50 samples — both far below f64's exact-integer range (2^53); every cast converts exactly"
    )]
    #[test]
    fn a_length_more_than_half_a_sample_off_the_duration_is_a_rate_mismatch(
        pcm_sample_count in 0usize..=1_000_000,
        perturbation_samples in 51i64..=150i64,
    ) {
        let perturbation = perturbation_samples as f64 / 100.0;
        let duration_ms =
            (pcm_sample_count as f64 + perturbation) / f64::from(DECLARED_RATE_HZ) * 1000.0;
        let result = check_delivery(
            DECLARED_RATE_HZ,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            pcm_sample_count,
            duration_ms,
        );
        prop_assert_eq!(result, Err(EngineError::RateMismatch));
    }
}

/// Shape is checked before length (`API_CONTRACT` §6: "Verify a
/// delivered sample buffer: shape first, then that the buffer
/// length matches the declared rate"). A wrong rate with a length
/// that is perfectly consistent with the duration still names the
/// shape error — here `RateMismatch` for 44100 Hz against a
/// 22050-sample, 1000 ms buffer (CHK005; FR-035; SC-035).
#[test]
fn a_wrong_rate_with_a_consistent_length_still_names_the_shape_error() {
    assert_eq!(
        check_delivery(
            44_100,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            22_050,
            1_000.0
        ),
        Err(EngineError::RateMismatch)
    );
    // The same holds when the length is also inconsistent: the
    // shape error is what is named, not the length error.
    assert_eq!(
        check_delivery(44_100, DECLARED_CHANNELS, DECLARED_WIDTH_BITS, 0, 1_000.0),
        Err(EngineError::RateMismatch)
    );
}

/// Shape is checked before length, and the channel shape names
/// `ChannelMismatch`: a stereo delivery with a perfectly
/// consistent length names the channel deviation, which a
/// length-only error could never produce — the discriminating
/// case for "shape first" (CHK005; CHK006; FR-033).
#[test]
fn a_wrong_channel_count_with_a_consistent_length_names_channel_mismatch() {
    assert_eq!(
        check_delivery(DECLARED_RATE_HZ, 2, DECLARED_WIDTH_BITS, 22_050, 1_000.0),
        Err(EngineError::ChannelMismatch)
    );
}

/// Shape is checked before length for the width too: a non-`f32`
/// width with a consistent length names `RateMismatch` — the
/// width deviation surfaces under the rate shape because the
/// closed set names no width-specific variant (Principle VII;
/// CHK005).
#[test]
fn a_wrong_width_with_a_consistent_length_names_rate_mismatch() {
    assert_eq!(
        check_delivery(DECLARED_RATE_HZ, DECLARED_CHANNELS, 16, 22_050, 1_000.0),
        Err(EngineError::RateMismatch)
    );
}

/// The declared shape with a length that agrees with the duration
/// is delivered: 22050 samples is exactly 1000 ms at 22050 Hz,
/// and zero samples is exactly 0 ms (R1; CHK007).
#[test]
fn the_declared_shape_with_a_consistent_length_is_delivered() {
    assert_eq!(
        check_delivery(
            DECLARED_RATE_HZ,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            22_050,
            1_000.0
        ),
        Ok(())
    );
    assert_eq!(
        check_delivery(
            DECLARED_RATE_HZ,
            DECLARED_CHANNELS,
            DECLARED_WIDTH_BITS,
            0,
            0.0
        ),
        Ok(())
    );
}
