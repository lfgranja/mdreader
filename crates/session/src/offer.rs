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

use engine_core::closed::SessionEnd;

use crate::store::StoredPosition;

/// The offer a session end opens (FR-032c /
/// FR-032d): the continue offer a deliberate
/// pause presents, or the start-over alternative.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
    pub fn on_session_end(
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
        // A new session-end replaces any previously
        // open offer (the window never outlives its
        // session, FR-032f).
        self.offer = None;
        self.explicit_resume = false;
        let stored = match stored {
            Some(stored) => stored,
            None => return None,
        };
        match stored.end_kind {
            SessionEnd::Paused => {
                let offer = Offer::Continue {
                    offset_secs: stored.offset_secs,
                    first_yield_ms: stored.first_yield_ms,
                };
                self.offer = Some(offer.clone());
                Some(offer)
            }
            // An unexpected end leaves the
            // explicit-resume requirement: recovery
            // to a paused state is the only
            // permitted automatic transition, and no
            // continue offer opens (FR-032d /
            // FR-031).
            SessionEnd::Interrupted => {
                self.explicit_resume = true;
                None
            }
            // Terminal ends never open an offer
            // (FR-032 / FR-032e); the flow deletes
            // their entries instead of offering them.
            SessionEnd::Stopped | SessionEnd::Completed => None,
        }
    }

    /// Latch the application as foregrounded (the
    /// `AppForegrounded` intent): the latch is read
    /// at expiry ONLY and never converts an open
    /// offer early (FR-032c's visibility MUST).
    pub fn on_foregrounded(&mut self) {
        self.foregrounded = true;
    }

    /// Latch the application as backgrounded (the
    /// `AppBackgrounded` intent): the latch is read
    /// at expiry ONLY and never converts an open
    /// offer early (FR-032c's visibility MUST).
    pub fn on_backgrounded(&mut self) {
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
    pub fn expire(&mut self) -> ExpiryOutcome {
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
    pub fn continue_offer(&mut self) -> Option<Offer> {
        self.offer.clone()
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
    pub fn start_over(&mut self) -> ExpiryOutcome {
        self.offer = None;
        self.explicit_resume = false;
        ExpiryOutcome::AutoStart { offset_secs: 0.0 }
    }

    /// Document completion teardown (FR-032b /
    /// FR-032e / FR-032f): completion and the
    /// resume offer are mutually exclusive states —
    /// no offer remains, no residency survives,
    /// and an expiry yields no outcome.
    pub fn on_completion(&mut self) {
        self.offer = None;
        self.explicit_resume = false;
    }
}
