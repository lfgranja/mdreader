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
//!
//! ## The stepper and its persistence (FR-019b / T042)
//!
//! Every rate-control press — the in-app stepper and
//! the lock-screen rate-step action alike — resolves
//! through one move, [`step_rate`], over the closed
//! two-member direction set [`RateStep`]: the press
//! moves exactly one rung and saturates at the ends.
//! [`RateStepper`] is the player-crate half of the
//! persistence wiring: it opens at the stored rung
//! ([`RateStepper::open`], read-at-start — the stored
//! `last_rate` or the [`DEFAULT_RATE`] first-use
//! rung) and reports every move as the rung to
//! persist ([`RateStepper::step`], write-on-change).
//! The store half — `BookmarkStore::get_last_rate` /
//! `BookmarkStore::put_last_rate` in `crates/session` —
//! exists; the call site that reads at session open
//! and writes on every change is the shell's
//! (src-tauri), which composes the two halves.
//!
//! ## Value-blind by contract (T042 / FR-012)
//!
//! Nothing in this module renders a rung's numeric
//! value: no `Display`, no formatted string, no wire
//! number leaves the stepper surface. The
//! lock-screen rate-step action is confirmed by the
//! earcon ([`crate::earcon::EarconKind::RateStep`])
//! and the haptic pulse — never by a spoken or
//! displayed number — so a blind tester's
//! confirmation leaks no rate value.

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

/// The first-use playback rate (FR-019b): the
/// normal-speed rung a session opens at when the
/// settings table holds no `last_rate`
/// (storage-schema.md: the call-site default).
pub const DEFAULT_RATE: PlaybackRate = PlaybackRate::R1_0;

/// The rung a session opens at (FR-019b / T042):
/// the stored `last_rate` when the settings row
/// survives the reopen, [`DEFAULT_RATE`] on a miss —
/// the read-at-start half of the persistence
/// wiring, resolved once here so every call site
/// (IPC, lock screen, CLI) applies the same default
/// (Principle II).
#[must_use]
pub const fn rate_at_start(stored: Option<PlaybackRate>) -> PlaybackRate {
    match stored {
        Some(rate) => rate,
        None => DEFAULT_RATE,
    }
}

/// The direction of a rate-step press — the closed
/// two-member set (FR-019 / FR-044): the in-app
/// stepper and the lock-screen rate-step action
/// each name exactly one of the two directions, and
/// nothing else exists. No catch-all arm matches on
/// it (Principle VII).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateStep {
    /// One rung up the closed ladder.
    Up,
    /// One rung down the closed ladder.
    Down,
}

/// One rate-step press (T042): exactly one rung in
/// `step`'s direction, saturating at the ladder's
/// end — the single move both surfaces call, the
/// in-app stepper and the lock-screen rate-step
/// action alike, so no second ladder navigation
/// exists (FR-019 / FR-044). The move routes
/// through the existing [`step_up`] / [`step_down`]
/// core; it selects a rung and never stretches
/// audio (the engine owns the pitch-preserving
/// stretch, FR-043), and it renders no rung value
/// (the value-blind contract, T042).
#[must_use]
pub fn step_rate(current: PlaybackRate, step: RateStep) -> PlaybackRate {
    match step {
        RateStep::Up => step_up(current),
        RateStep::Down => step_down(current),
    }
}

/// The rate stepper (T042): the live rung plus the
/// persistence half of every move — the
/// player-crate half of the FR-019b wiring.
///
/// The stepper opens at the stored rung
/// ([`open`](Self::open), read-at-start) and reports
/// every move as the rung to persist
/// ([`step`](Self::step), write-on-change): the
/// `Some` rung is exactly the value the call site
/// writes through `BookmarkStore::put_last_rate`,
/// and a press that saturated at an end reports
/// `None` — nothing changed, so nothing is written.
///
/// The rung is never rendered: no `Display`, no
/// formatted string, no wire number leaves this
/// type — the lock-screen rate-step action is
/// value-blind by contract, so a blind tester's
/// confirmation (the earcon plus the haptic pulse)
/// carries no rate value (T042 / FR-012).
#[derive(Debug)]
pub struct RateStepper {
    /// The live rung — a value, never a rendering.
    current: PlaybackRate,
}

impl RateStepper {
    /// Open at the rung the stored `last_rate`
    /// carries, or [`DEFAULT_RATE`] on first use —
    /// the read-at-start half, resolved through
    /// [`rate_at_start`].
    #[must_use]
    pub const fn open(stored: Option<PlaybackRate>) -> Self {
        Self {
            current: rate_at_start(stored),
        }
    }

    /// The live rung.
    #[must_use]
    pub const fn current(&self) -> PlaybackRate {
        self.current
    }

    /// One rate-step press: the rung moves exactly
    /// one step in `step`'s direction, saturating at
    /// the ladder's end (FR-019). Returns the rung
    /// to persist when the press moved it — the
    /// write-on-change half (FR-019b) — or `None`
    /// when the press saturated at an end and
    /// changed nothing.
    #[must_use]
    pub fn step(&mut self, step: RateStep) -> Option<PlaybackRate> {
        let next = step_rate(self.current, step);
        if next != self.current {
            self.current = next;
            Some(next)
        } else {
            None
        }
    }
}

impl Default for RateStepper {
    /// First use: open at the [`DEFAULT_RATE`] rung.
    fn default() -> Self {
        Self::open(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earcon::EarconKind;

    /// The numeric rung values a lock-screen rendering
    /// must never leak (T042 / FR-012): the wire
    /// decimals and the display forms alike.
    const RATE_VALUES: [&str; 14] = [
        "0.75", "1.0", "1.25", "1.5", "1.75", "2.0", "4.0", "0.75x", "1.0x", "1.25x", "1.5x",
        "1.75x", "2.0x", "4.0x",
    ];

    #[test]
    fn every_rung_steps_exactly_one_rung_and_saturates_at_both_ends() {
        // T042 / FR-019: the stepper move is measured
        // against the closed ladder — each press moves
        // exactly one rung, and a press at either end
        // saturates: never wraps, never fails, never
        // yields a rung off the ladder.
        assert_eq!(LADDER.len(), 7, "the ladder holds exactly seven rungs");
        for (index, rung) in LADDER.iter().enumerate() {
            assert_eq!(
                step_rate(*rung, RateStep::Up),
                LADDER[(index + 1).min(LADDER.len() - 1)],
                "a rate-up press from {rung:?} must move exactly one rung"
            );
            assert_eq!(
                step_rate(*rung, RateStep::Down),
                LADDER[index.saturating_sub(1)],
                "a rate-down press from {rung:?} must move exactly one rung"
            );
            assert!(LADDER.contains(&step_rate(*rung, RateStep::Up)));
            assert!(LADDER.contains(&step_rate(*rung, RateStep::Down)));

            // The saturation walks: repeated presses
            // from every rung land on the end rungs.
            let mut walked_up = *rung;
            for _ in 0..20 {
                walked_up = step_rate(walked_up, RateStep::Up);
                assert!(
                    LADDER.contains(&walked_up),
                    "a rate-up press left the ladder at {walked_up:?}"
                );
            }
            assert_eq!(
                walked_up,
                PlaybackRate::R4_0,
                "repeated rate-up presses must saturate at the top rung"
            );
            let mut walked_down = *rung;
            for _ in 0..20 {
                walked_down = step_rate(walked_down, RateStep::Down);
                assert!(
                    LADDER.contains(&walked_down),
                    "a rate-down press left the ladder at {walked_down:?}"
                );
            }
            assert_eq!(
                walked_down,
                PlaybackRate::R0_75,
                "repeated rate-down presses must saturate at the bottom rung"
            );
        }
        // The ends themselves saturate.
        assert_eq!(
            step_rate(PlaybackRate::R4_0, RateStep::Up),
            PlaybackRate::R4_0
        );
        assert_eq!(
            step_rate(PlaybackRate::R0_75, RateStep::Down),
            PlaybackRate::R0_75
        );
    }

    #[test]
    fn the_stepper_opens_at_the_stored_rung_and_defaults_on_first_use() {
        // FR-019b / T042: the read-at-start half — the
        // stored rung when one survives, the
        // normal-speed default on first use, resolved
        // once for every call site.
        for rung in LADDER {
            assert_eq!(RateStepper::open(Some(rung)).current(), rung);
            assert_eq!(rate_at_start(Some(rung)), rung);
        }
        assert_eq!(RateStepper::open(None).current(), DEFAULT_RATE);
        assert_eq!(RateStepper::default().current(), DEFAULT_RATE);
        assert_eq!(DEFAULT_RATE, PlaybackRate::R1_0);
    }

    #[test]
    fn every_step_reports_the_rung_to_persist_and_a_saturated_press_reports_none() {
        // T042 / FR-019b: the write-on-change half — a
        // press that moved the rung reports the rung to
        // persist, and a press that saturated at an end
        // reports `None`: nothing changed, nothing is
        // written.
        let mut stepper = RateStepper::open(Some(PlaybackRate::R1_0));
        assert_eq!(stepper.step(RateStep::Up), Some(PlaybackRate::R1_25));
        assert_eq!(stepper.current(), PlaybackRate::R1_25);
        assert_eq!(stepper.step(RateStep::Down), Some(PlaybackRate::R1_0));
        assert_eq!(stepper.current(), PlaybackRate::R1_0);

        // Walk to the top rung, then saturate.
        let mut top = RateStepper::open(Some(PlaybackRate::R4_0));
        assert_eq!(top.step(RateStep::Up), None);
        assert_eq!(top.current(), PlaybackRate::R4_0);

        // The bottom rung saturates downward.
        let mut bottom = RateStepper::open(Some(PlaybackRate::R0_75));
        assert_eq!(bottom.step(RateStep::Down), None);
        assert_eq!(bottom.current(), PlaybackRate::R0_75);
    }

    #[test]
    fn the_rate_step_confirmation_carries_no_rate_value() -> Result<(), Box<dyn std::error::Error>>
    {
        // T042 / FR-012: value-blind by contract — the
        // stepper surface renders no rung value and the
        // confirmation is a tag, not a number, so a
        // blind lock-screen tester hears the earcon and
        // never the rate. This module defines no error,
        // reason, or progress rendering at all; the
        // renderings below are its whole rendered
        // surface.
        for rung in LADDER {
            let renderings = [
                format!("{rung:?}"),
                format!("{:?}", RateStepper::open(Some(rung))),
                format!("{:?}", RateStep::Up),
                format!("{:?}", RateStep::Down),
            ];
            for rendered in &renderings {
                for value in RATE_VALUES {
                    assert!(
                        !rendered.contains(value),
                        "the rendering {rendered:?} leaks the rate value {value}"
                    );
                }
            }
        }
        // The confirmation is the earcon tag alone — no
        // value rides along (FR-018a).
        let confirmation = serde_json::to_string(&EarconKind::RateStep)?;
        assert_eq!(confirmation, "\"RateStep\"");
        for value in RATE_VALUES {
            assert!(
                !confirmation.contains(value),
                "the rate-step confirmation leaks the rate value {value}"
            );
        }
        Ok(())
    }
}
