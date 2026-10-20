//! The resume-offer state machine (T060): the
//! FR-032c / FR-032d offer flow, resolved SOLELY
//! from the stored `end_kind`
//! (contracts/storage-schema.md).
//!
//! A deliberate `Paused` end opens the visible
//! 3-second continue offer (FR-032c) — continue
//! from the stored position, the default, with the
//! start-over alternative; an `Interrupted` end
//! leaves the explicit-resume requirement and opens
//! no offer (FR-032d / FR-031); `Stopped` /
//! `Completed` never open one (FR-032 / FR-032e /
//! FR-032f). The machine carries no clock:
//! `expire` represents the 3-second window
//! elapsing, so every transition is deterministic
//! and no caller waits on real time.
//!
//! The `AppForegrounded` / `AppBackgrounded`
//! intents latch the latest visibility ONLY and
//! are read at expiry ONLY — the offer remains
//! visible for the whole 3 seconds, never
//! converting an open offer early (FR-032c's
//! visibility MUST, spec.md:90).
//!
//! A session open resolves the SAME decision with
//! no fingerprint in hand: [`OpenOffer`] (T036) is
//! the form the lookup [`crate::store::lookup_on_open`]
//! carries, routed through
//! [`OfferMachine::resolve_open`] — the one core
//! `on_session_end` also runs — so the open
//! decision exists exactly once and no caller
//! re-derives it (FR-044).

use engine_core::closed::SessionEnd;

use crate::store::StoredPosition;

/// The offer a session end opens (FR-032c /
/// FR-032d): the continue offer a deliberate
/// pause presents, or the start-over alternative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Offer {
    /// Continue from the stored position — the
    /// deliberate-pause offer (FR-032c), carrying
    /// the stored offset and the FR-015 interval
    /// start.
    Continue {
        /// The stored last spoken position, in
        /// seconds.
        offset_secs: f64,
        /// The start of the continuous
        /// non-advancing interval, in milliseconds
        /// (FR-015).
        first_yield_ms: u64,
    },
    /// The start-over alternative (FR-032c): begin
    /// from the first spoken position.
    StartOver,
}

/// The outcome of the 3-second window elapsing
/// (FR-032c / FR-032d / FR-032f).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpiryOutcome {
    /// Reading begins from the stored position —
    /// the auto-start of a foregrounded
    /// deliberate-pause offer at expiry (SC-020),
    /// or the start-over first-spoken-position
    /// start.
    AutoStart {
        /// The position reading begins from, in
        /// seconds.
        offset_secs: f64,
    },
    /// No audio begins: an explicit-resume control
    /// is all the session offers — an unexpected
    /// end (SC-021 / FR-032d) or a backgrounded
    /// expiry (FR-032c / FR-031).
    ExplicitResume,
    /// Nothing is open — no outcome: the window
    /// never outlives its session (FR-032f).
    NoOffer,
}

/// The offer form a session-open lookup resolves
/// (T036): the decision the stored session end
/// carries (FR-044).
///
/// The three forms are the whole outcome space of
/// an open: a deliberate pause continues, an
/// unexpected end demands an explicit resume, and
/// everything else — a terminal end or no stored
/// row — opens nothing.
///
/// `Eq` is precluded by the `f64` field; every
/// offset lives on the quarter-second grid, an
/// exact binary-fraction grid, so `PartialEq` is
/// exact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenOffer {
    /// Continue from the stored position — the
    /// deliberate-pause offer (FR-032c), the
    /// default, carrying the stored offset and the
    /// FR-015 interval start.
    Continue {
        /// The stored last spoken position, in
        /// seconds.
        offset_secs: f64,
        /// The start of the continuous
        /// non-advancing interval, in milliseconds
        /// (FR-015).
        first_yield_ms: u64,
    },
    /// The explicit-resume requirement an
    /// unexpected end leaves (FR-032d): no offer
    /// opens, and audio never begins on its own —
    /// the reader presses resume (FR-031).
    ExplicitResume,
    /// Nothing opens — a terminal end (FR-032 /
    /// FR-032e) or no stored row (an edited
    /// document, FR-036a): the session starts at
    /// `0.0` with no offer.
    NoOffer,
}

/// The cold-start open decision (T110): whether an opened
/// document may begin audio NOW, or must wait for its resume
/// offer to resolve — and, when it waits, the position the
/// resolution continues from.
///
/// This is the app's routing decision, expressed ONCE, here: the
/// shell routes the decision and never re-derives the offer's
/// form (FR-044). The three forms are the whole outcome space of
/// a cold start.
///
/// * [`ColdStart::StartNow`] — no stored position: the session
///   begins at the first spoken position with no offer
///   (FR-036a).
/// * [`ColdStart::AwaitOffer`] — a deliberate pause: the 3-second
///   continue offer is open (FR-032c), NO audio begins until it
///   resolves, and the resolution continues from the stored
///   position (FR-009).
/// * [`ColdStart::AwaitExplicitResume`] — an unexpected end: no
///   offer opens and no audio begins on its own (FR-032d /
///   FR-031) — the reader's explicit resume is the only start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColdStart {
    /// Start now — the first spoken position, no offer.
    StartNow,
    /// The 3-second continue offer is open (FR-032c): no audio
    /// until the offer resolves, and the resolution continues
    /// from the stored position.
    AwaitOffer {
        /// The stored last-spoken position the resolution
        /// continues from, in seconds.
        offset_secs: f64,
        /// The start of the continuous non-advancing interval the
        /// resume continues (FR-015), in milliseconds.
        first_yield_ms: u64,
    },
    /// The explicit-resume requirement stands (FR-032d /
    /// FR-031): no audio begins on its own.
    AwaitExplicitResume,
}

/// The cold-start decision over an open offer form (T110): the
/// projection of [`OpenOffer`] onto the app's three routings, so
/// the shell routes the decision and never re-derives the
/// offer's form (FR-044).
///
/// The form never depends on the document's identity — it is
/// resolved solely from the stored end kind — so the decision
/// needs no fingerprint in hand.
#[must_use]
pub const fn cold_start(offer: OpenOffer) -> ColdStart {
    match offer {
        OpenOffer::Continue {
            offset_secs,
            first_yield_ms,
        } => ColdStart::AwaitOffer {
            offset_secs,
            first_yield_ms,
        },
        OpenOffer::ExplicitResume => ColdStart::AwaitExplicitResume,
        OpenOffer::NoOffer => ColdStart::StartNow,
    }
}

/// The resume-offer state machine: at most one open
/// offer, the latched application visibility, and
/// the explicit-resume requirement an unexpected
/// end leaves (FR-032c / FR-032d).
#[derive(Debug, Default)]
pub struct OfferMachine {
    /// The open continue offer, while the window is
    /// still open.
    offer: Option<Offer>,
    /// Whether an explicit-resume requirement
    /// stands — the unexpected-end form (FR-032d).
    explicit_resume: bool,
    /// The latest latched application visibility
    /// (the `AppForegrounded` / `AppBackgrounded`
    /// intents), read at expiry ONLY (FR-032c).
    foregrounded: bool,
}

impl OfferMachine {
    /// A machine with no open offer, no
    /// explicit-resume requirement, and the
    /// backgrounded latch.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The session ended with `stored` under
    /// `fingerprint`: the offer form is resolved
    /// SOLELY from the stored end kind — a
    /// deliberate `Paused` opens the 3-second
    /// continue offer (the FR-032c default), an
    /// `Interrupted` end leaves the
    /// explicit-resume requirement and opens no
    /// offer (FR-032d), `Stopped`/`Completed`
    /// never open one (FR-032 / FR-032e /
    /// FR-032f), and no stored position opens
    /// nothing. A new session-end replaces any
    /// previously open offer — the window never
    /// outlives its session (FR-032f).
    ///
    /// The routing is [`resolve_open`](Self::resolve_open)'s
    /// decision, projected onto the open offer: the
    /// continue offer is the only form that opens
    /// one (FR-032c), so the explicit-resume and
    /// no-offer forms answer `None` — their state is
    /// read through `resolve_open` / [`expire`](Self::expire),
    /// never re-derived here (FR-044).
    pub const fn on_session_end(
        &mut self,
        fingerprint: [u8; 32],
        stored: Option<StoredPosition>,
    ) -> Option<Offer> {
        // The offer form never depends on the
        // document's identity — it is resolved
        // solely from the stored end kind
        // (storage-schema.md) — so the machine
        // holds no per-document state.
        let _ = fingerprint;
        match self.resolve_open(stored) {
            OpenOffer::Continue { .. } => self.offer,
            OpenOffer::ExplicitResume | OpenOffer::NoOffer => None,
        }
    }

    /// The open decision over the stored position
    /// `stored` — the one core every open routes
    /// through (FR-044): a new end replaces any
    /// previously open offer (the window never
    /// outlives its session, FR-032f), a deliberate
    /// `Paused` opens the 3-second continue offer
    /// (FR-032c), an `Interrupted` end leaves the
    /// explicit-resume requirement and opens no
    /// offer (FR-032d / FR-031), and
    /// `Stopped`/`Completed` — or a miss — open
    /// nothing (FR-032 / FR-032e / FR-036a).
    ///
    /// A session-open lookup (the re-hash-on-open
    /// decision, [`crate::store::lookup_on_open`])
    /// carries the [`OpenOffer`] this resolves;
    /// [`on_session_end`](Self::on_session_end) projects
    /// the same decision onto the open offer. The
    /// form never depends on the document's
    /// identity, so no fingerprint is in hand here.
    #[must_use]
    pub const fn resolve_open(&mut self, stored: Option<StoredPosition>) -> OpenOffer {
        // A new session-end replaces any previously
        // open offer (the window never outlives its
        // session, FR-032f).
        self.offer = None;
        self.explicit_resume = false;
        let Some(stored) = stored else {
            return OpenOffer::NoOffer;
        };
        match stored.end_kind {
            SessionEnd::Paused => {
                let offer = Offer::Continue {
                    offset_secs: stored.offset_secs,
                    first_yield_ms: stored.first_yield_ms,
                };
                self.offer = Some(offer);
                OpenOffer::Continue {
                    offset_secs: stored.offset_secs,
                    first_yield_ms: stored.first_yield_ms,
                }
            }
            // An unexpected end leaves the
            // explicit-resume requirement: recovery
            // to a paused state is the only
            // permitted automatic transition, and no
            // continue offer opens (FR-032d /
            // FR-031).
            SessionEnd::Interrupted => {
                self.explicit_resume = true;
                OpenOffer::ExplicitResume
            }
            // Terminal ends never open an offer
            // (FR-032 / FR-032e); the flow deletes
            // their entries instead of offering them.
            SessionEnd::Stopped | SessionEnd::Completed => OpenOffer::NoOffer,
        }
    }

    /// Latch the application as foregrounded (the
    /// `AppForegrounded` intent): the latch is read
    /// at expiry ONLY and never converts an open
    /// offer early (FR-032c's visibility MUST).
    pub const fn on_foregrounded(&mut self) {
        self.foregrounded = true;
    }

    /// Latch the application as backgrounded (the
    /// `AppBackgrounded` intent): the latch is read
    /// at expiry ONLY and never converts an open
    /// offer early (FR-032c's visibility MUST).
    pub const fn on_backgrounded(&mut self) {
        self.foregrounded = false;
    }

    /// The 3-second window elapses (FR-032c /
    /// FR-032d): the latched foreground value is
    /// read NOW, at expiry only — a foregrounded
    /// continue offer auto-starts from the stored
    /// position (FR-032c / SC-020); a backgrounded
    /// offer, or an offer with no foreground
    /// latch, converts to the explicit-resume form
    /// and never begins audio on its own (FR-032c /
    /// FR-031); an explicit-resume requirement
    /// stands as itself (FR-032d / SC-021). The
    /// window ends: nothing remains open.
    #[must_use]
    pub const fn expire(&mut self) -> ExpiryOutcome {
        if let Some(offer) = self.offer.take() {
            // The window ends: the offer does not
            // outlive its expiry (FR-032f).
            if self.foregrounded {
                return match offer {
                    Offer::Continue { offset_secs, .. } => ExpiryOutcome::AutoStart { offset_secs },
                    // The start-over alternative
                    // begins from the first spoken
                    // position.
                    Offer::StartOver => ExpiryOutcome::AutoStart { offset_secs: 0.0 },
                };
            }
            return ExpiryOutcome::ExplicitResume;
        }
        if self.explicit_resume {
            // The explicit-resume requirement stands
            // at expiry (FR-032d) — and the window
            // ends with it.
            self.explicit_resume = false;
            return ExpiryOutcome::ExplicitResume;
        }
        ExpiryOutcome::NoOffer
    }

    /// The open offer, if any — unchanged until a
    /// choice is made or the window elapses
    /// (FR-032c).
    pub const fn continue_offer(&mut self) -> Option<Offer> {
        self.offer
    }

    /// The reader chose start-over (FR-032c):
    /// reading begins from the first spoken
    /// position — exactly `0.0` — and the choice
    /// consumes the offer: nothing remains open,
    /// and a later expiry yields no outcome. The
    /// flow discards the stored entry through the
    /// store (the machine holds no store); the
    /// discard is the caller's (FR-032c /
    /// FR-032f).
    #[must_use]
    pub const fn start_over(&mut self) -> ExpiryOutcome {
        self.offer = None;
        self.explicit_resume = false;
        ExpiryOutcome::AutoStart { offset_secs: 0.0 }
    }

    /// Document completion teardown (FR-032b /
    /// FR-032e / FR-032f): completion and the
    /// resume offer are mutually exclusive states —
    /// no offer remains, no residency survives,
    /// and an expiry yields no outcome.
    pub const fn on_completion(&mut self) {
        self.offer = None;
        self.explicit_resume = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stored position under `end_kind` at a
    /// quarter-second-grid offset.
    fn stored(end_kind: SessionEnd) -> StoredPosition {
        StoredPosition {
            offset_secs: 12.5,
            first_yield_ms: 250,
            end_kind,
        }
    }

    /// The closed session-end set (FR-044) — every
    /// kind the store can carry.
    const END_KINDS: [SessionEnd; 4] = [
        SessionEnd::Paused,
        SessionEnd::Stopped,
        SessionEnd::Completed,
        SessionEnd::Interrupted,
    ];

    #[test]
    fn resolve_open_routes_every_end_kind_to_its_form() {
        // T036: the open decision is resolved solely
        // from the stored end kind — a deliberate
        // pause continues, an unexpected end demands
        // the explicit resume, and the terminal ends
        // and a miss open nothing (FR-032c / FR-032d /
        // FR-032e / FR-036a).
        let mut machine = OfferMachine::new();
        assert_eq!(
            machine.resolve_open(Some(stored(SessionEnd::Paused))),
            OpenOffer::Continue {
                offset_secs: 12.5,
                first_yield_ms: 250,
            }
        );
        assert_eq!(
            machine.resolve_open(Some(stored(SessionEnd::Interrupted))),
            OpenOffer::ExplicitResume
        );
        assert_eq!(
            machine.resolve_open(Some(stored(SessionEnd::Stopped))),
            OpenOffer::NoOffer
        );
        assert_eq!(
            machine.resolve_open(Some(stored(SessionEnd::Completed))),
            OpenOffer::NoOffer
        );
        assert_eq!(machine.resolve_open(None), OpenOffer::NoOffer);
    }

    #[test]
    fn resolve_open_agrees_with_on_session_end_for_every_end_kind() {
        // FR-044: one decision, two projections —
        // `on_session_end` answers the open offer
        // exactly when `resolve_open` carries the
        // continue form, and answers `None` on the
        // explicit-resume and no-offer forms.
        let mut machine = OfferMachine::new();
        for end_kind in END_KINDS {
            let decision = machine.resolve_open(Some(stored(end_kind)));
            let opened = machine.on_session_end([0u8; 32], Some(stored(end_kind)));
            match decision {
                OpenOffer::Continue {
                    offset_secs,
                    first_yield_ms,
                } => assert_eq!(
                    opened,
                    Some(Offer::Continue {
                        offset_secs,
                        first_yield_ms
                    }),
                    "{end_kind:?} resolved Continue but on_session_end opened {opened:?}"
                ),
                OpenOffer::ExplicitResume | OpenOffer::NoOffer => assert_eq!(
                    opened, None,
                    "{end_kind:?} resolved {decision:?} but on_session_end opened {opened:?}"
                ),
            }
            // A miss opens nothing on either projection.
            assert_eq!(machine.on_session_end([0u8; 32], None), None);
        }
    }

    #[test]
    fn resolve_open_leaves_the_machine_state_the_form_names() {
        // The decision is the machine's own state: a
        // continue offer expires into the auto-start,
        // the explicit-resume form stands at expiry,
        // and a no-offer form expires to no outcome
        // (FR-032c / FR-032d / FR-032f).
        let mut continued = OfferMachine::new();
        assert_eq!(
            continued.resolve_open(Some(stored(SessionEnd::Paused))),
            OpenOffer::Continue {
                offset_secs: 12.5,
                first_yield_ms: 250,
            }
        );
        assert_eq!(
            continued.continue_offer(),
            Some(Offer::Continue {
                offset_secs: 12.5,
                first_yield_ms: 250,
            })
        );

        let mut interrupted = OfferMachine::new();
        assert_eq!(
            interrupted.resolve_open(Some(stored(SessionEnd::Interrupted))),
            OpenOffer::ExplicitResume
        );
        assert_eq!(interrupted.expire(), ExpiryOutcome::ExplicitResume);

        let mut stopped = OfferMachine::new();
        assert_eq!(
            stopped.resolve_open(Some(stored(SessionEnd::Stopped))),
            OpenOffer::NoOffer
        );
        assert_eq!(stopped.expire(), ExpiryOutcome::NoOffer);
    }

    #[test]
    fn resolve_open_replaces_any_previously_open_offer() {
        // FR-032f: the window never outlives its
        // session — a later open decision replaces
        // the offer the previous one opened.
        let mut machine = OfferMachine::new();
        let _opened = machine.on_session_end([0u8; 32], Some(stored(SessionEnd::Paused)));
        assert_eq!(
            machine.resolve_open(Some(stored(SessionEnd::Interrupted))),
            OpenOffer::ExplicitResume
        );
        assert_eq!(machine.continue_offer(), None);
    }
}
