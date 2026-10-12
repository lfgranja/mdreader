//! The closed playback-rate ladder and its one-step
//! moves (T057; FR-019, FR-019a, FR-043).
//!
//! The ladder is the closed seven-rung set
//! `0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 4.0`
//! (FR-019), ascending, with nothing between or
//! beyond the declared seven — the rung values are
//! defined once in `engine_core::closed::PlaybackRate`
//! (the closed-set registry), and this module holds
//! only the ladder *order* and the one-step moves
//! over it, so no second list of rungs exists that
//! could drift (FR-044).
//!
//! Each rate-control press moves exactly one rung
//! ([`step_up`] / [`step_down`]), and a press at
//! either end saturates — never wraps, never fails,
//! never yields a rung off the ladder (FR-019).
//!
//! ## Effect path (FR-019a / FR-043)
//!
//! A rate change takes effect within
//! [`RATE_EFFECTIVE_BOUND_SECS`] (2.0 s, FR-019a)
//! through the **engine's** pitch-preserving
//! time-stretch operation (`Engine::set_rate`) —
//! never a local software stretch in this crate
//! (FR-043). The player layer selects the rung
//! only; the pitch invariant holds because the
//! engine owns the stretch: the delivered handoff
//! shape (mono `f32` at the declared 22 050 Hz) is
//! invariant at every rung, and a rate/channel/
//! width mismatch is a named `EngineError`, never a
//! silent resample or downmix. A rate change never
//! moves the position, and the pre-roll is discarded
//! so it is re-prepared at the new rung
//! ([`crate::buffer::PreRollBuffer::reprepare_for_rate`]).
//!
//! ## Audible confirmation (FR-018a, the
//! research-derived refinement per R6)
//!
//! FR-018a requires every control effect to be
//! confirmed aloud: a blind lock-screen tester cannot
//! see the surface (SC-008), so an effect left
//! unconfirmed aloud fails the trial. The R6
//! refinement shapes the confirmation as a short
//! earcon on **every** press, plus the spoken state
//! on resume and on a rate change — emitted through
//! the same Rust path the mirror reports (the single
//! dispatch every surface routes through, FR-042),
//! never a second, surface-side confirmation channel.
//! Acceptance is earcon-or-speech per the T053 blind
//! trials: the earcon is a refinement, not a mandate
//! — where the spoken state already confirms the
//! effect, the confirmation stands without it.

use crate::transport::PlaybackRate;

/// The closed seven-rung playback-rate ladder
/// (FR-019), ascending: `0.75 | 1.0 | 1.25 |
/// 1.5 | 1.75 | 2.0 | 4.0`.
///
/// The order is the contract: every one-step move
/// ([`step_up`] / [`step_down`]) is measured against
/// this array, and a press at either end saturates at
/// the array's end rung. The rung values live in
/// `engine_core::closed::PlaybackRate` — the single
/// authoritative definition (FR-044) — so this array
/// is an ordering over that set, not a second set.
pub const LADDER: [PlaybackRate; 7] = [
    PlaybackRate::R0_75,
    PlaybackRate::R1_0,
    PlaybackRate::R1_25,
    PlaybackRate::R1_5,
    PlaybackRate::R1_75,
    PlaybackRate::R2_0,
    PlaybackRate::R4_0,
];

/// Move exactly one rung up the closed ladder
/// (FR-019), saturating at [`PlaybackRate::R4_0`]:
/// the top rung is the ceiling, so a press at
/// `R4_0` stays at `R4_0` — the press never wraps,
/// never fails, and never yields a rung off the
/// ladder.
///
/// The match is exhaustive over the seven declared
/// rungs, so a rung added to
/// `engine_core::closed::PlaybackRate` without a
/// step here is a compile error (FR-044).
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "FR-019 / Principle VII: the saturation arm (R4_0 => R4_0) legitimately shares its body with the R2_0 step — a press at the top rung stays at the top rung; the exhaustive seven-arm match with no catch-all is the closed-set contract (FR-044)"
)]
pub fn step_up(rung: PlaybackRate) -> PlaybackRate {
    match rung {
        PlaybackRate::R0_75 => PlaybackRate::R1_0,
        PlaybackRate::R1_0 => PlaybackRate::R1_25,
        PlaybackRate::R1_25 => PlaybackRate::R1_5,
        PlaybackRate::R1_5 => PlaybackRate::R1_75,
        PlaybackRate::R1_75 => PlaybackRate::R2_0,
        PlaybackRate::R2_0 => PlaybackRate::R4_0,
        PlaybackRate::R4_0 => PlaybackRate::R4_0,
    }
}

/// Move exactly one rung down the closed ladder
/// (FR-019), saturating at [`PlaybackRate::R0_75`]:
/// the bottom rung is the floor, so a press at
/// `R0_75` stays at `R0_75` — the press never wraps,
/// never fails, and never yields a rung off the
/// ladder.
///
/// The match is exhaustive over the seven declared
/// rungs, so a rung added to
/// `engine_core::closed::PlaybackRate` without a
/// step here is a compile error (FR-044).
#[must_use]
#[expect(
    clippy::match_same_arms,
    reason = "FR-019 / Principle VII: the saturation arm (R0_75 => R0_75) legitimately shares its body with the R1_0 step — a press at the bottom rung stays at the bottom rung; the exhaustive seven-arm match with no catch-all is the closed-set contract (FR-044)"
)]
pub fn step_down(rung: PlaybackRate) -> PlaybackRate {
    match rung {
        PlaybackRate::R0_75 => PlaybackRate::R0_75,
        PlaybackRate::R1_0 => PlaybackRate::R0_75,
        PlaybackRate::R1_25 => PlaybackRate::R1_0,
        PlaybackRate::R1_5 => PlaybackRate::R1_25,
        PlaybackRate::R1_75 => PlaybackRate::R1_5,
        PlaybackRate::R2_0 => PlaybackRate::R1_75,
        PlaybackRate::R4_0 => PlaybackRate::R2_0,
    }
}

/// The bound within which a rate change takes effect
/// (FR-019a): `2.0` seconds from the press.
///
/// The bound is met through the engine's
/// pitch-preserving stretch operation (`Engine::set_rate`,
/// FR-043): the player layer selects the rung and
/// never stretches audio itself, so the pitch
/// invariant holds at every rung and the delivered
/// handoff shape is unchanged (see the module
/// documentation for the effect path and the
/// FR-018a / R6 audible-confirmation refinement).
pub const RATE_EFFECTIVE_BOUND_SECS: f64 = 2.0;
