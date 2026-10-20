//! T114 — the explicit-resume cold start SURVIVES the offer-window
//! expiries (Phase 16, feature 004-ressoa-dogfood-build).
//!
//! RED-FIRST: this suite FAILS against the current code
//! (Principle III). The failing property is
//! `prop_the_explicit_resume_cold_start_survives_repeated_offer_window_expiries`.
//!
//! The finding (tasks.md T114, F4 — `contradicts`, CRITICAL):
//! a document left by an unexpected end — the `Interrupted` row the
//! 20 s heartbeat, the app-background write, or a focus-loss write
//! leaves — MUST cold start into the explicit-resume form: the
//! reading waits paused with an explicit resume control on the
//! lock-screen surface, and audio NEVER begins on its own
//! (FR-009 / FR-018 / FR-032d / FR-031). The parked cold start
//! MUST SURVIVE the offer-window expiries: the requirement is a
//! latch, not a one-shot answer.
//!
//! The defect this suite pins:
//!
//! - `crates/session/src/offer.rs:340-363` — `expire()` clears
//!   `explicit_resume` on the FIRST call and answers `NoOffer` on
//!   the SECOND.
//! - `src-tauri/src/lib.rs:1791-1805` — the `ExplicitResume`
//!   branch re-parks the SAME `PendingOpen` (the same `opened_at`,
//!   `:1578`), so the next 250 ms poll (`OFFER_WINDOW_POLL_MS`,
//!   `:1557`) re-resolves it.
//! - `src-tauri/src/lib.rs:1807-1816` — the `NoOffer` +
//!   `WindowElapsed` branch DROPS the parked open and starts
//!   nothing.
//!
//! Consequence: the kill/power-loss recovery — the normal
//! cold-start recovery — is abandoned ≈3.25 s after the open: no
//! reading, no resume control, and the stored row stays
//! `Interrupted`, so re-picking the document repeats the cycle.
//!
//! Invariants under test
//! =====================
//!
//! - I1 (FR-009 / FR-018 / FR-032d): a stored `Interrupted` row
//!   resolves to the explicit-resume form — the cold start awaits
//!   an explicit resume and NEVER resolves to "no offer / start
//!   nothing".
//! - I2 (FR-009 / FR-032d / FR-031): the explicit-resume
//!   requirement SURVIVES N offer-window expiries — expiry N
//!   answers the explicit-resume form, never "no offer", and never
//!   the automatic start; and the parked open is never abandoned.
//! - I3 (FR-009 / FR-032c): the deliberate-pause path is
//!   UNAFFECTED — the 3-second continue offer is open and its
//!   silence continues from the stored position. This half GUARDS
//!   the repair: it is green today and MUST stay green after T115.
//! - I4 (FR-036a): a document with no stored row still starts
//!   immediately.
//!
//! The app-level seam T115 must add or change
//! ===========================================
//!
//! The app-level flow is NOT reachable from this test target: the
//! parked open (`PendingOpen`, `src-tauri/src/lib.rs:1567`), its
//! resolution (`resolve_parked_open`, `:1763`) and the poll
//! (`expire_parked_open`, `:1721`) are `pub(crate)` items of the
//! `src-tauri` crate, so only that crate's own `#[cfg(test)]`
//! module can drive them. This suite therefore drives the offer
//! machine and the cold-start decision — the ONE decision the app
//! routes (FR-044) — and models the shell's branch table over that
//! decision, so the invariant is pinned where the defect lives: the
//! offer machine's own answer at the expiry.
//!
//! The defect has TWO halves, and T115 must repair BOTH:
//!
//! 1. THE MACHINE (`crates/session/src/offer.rs:340-363`) —
//!    `expire()` clears `explicit_resume` on the FIRST call and
//!    answers `NoOffer` on the SECOND. The explicit-resume
//!    requirement must be a LATCH: it stands until the reader
//!    resumes, so every expiry answers
//!    `ExpiryOutcome::ExplicitResume`. This suite pins that
//!    invariant (I2) — it is RED today.
//! 2. THE SHELL (`src-tauri/src/lib.rs:1763-1818`) — the
//!    `NoOffer` + `WindowElapsed` branch DROPS the parked open
//!    (`:1807-1816`), and the `ExplicitResume` branch re-parks the
//!    SAME `PendingOpen` with the SAME `opened_at` (`:1578`,
//!    `:1791-1805`), so the next 250 ms poll re-resolves it and
//!    drops it. The shell must keep the parked open parked until
//!    the reader resumes, and must never open a SECOND offer
//!    window over the same requirement.
//!
//! T115 also owns two follow-ups this suite cannot make:
//!
//! - `src-tauri`'s own `#[cfg(test)]` module must gain the twin
//!   assertion (the parked open survives the poll and the reader's
//!   resume starts the reading at the stored position).
//! - `crates/session/tests/prop_bookmark_lifecycle.rs:516-521`
//!   currently PINS the defective behaviour — "an expiry repeated
//!   after the explicit resume" ⇒ `ExpiryOutcome::NoOffer` — and
//!   must be amended with the repair, or the workspace cannot be
//!   green after it.
//!
//! Exactness: every offset lives on the quarter-second grid
//! (`k × 0.25`), an exact binary-fraction grid, so the offer
//! offsets compare bit-exactly. Every `prop_assert*!` message uses
//! explicit positional arguments, never implicit named captures:
//! proptest builds the message through `concat!`, and rustc
//! rejects implicit captures in an expanded format string.

use proptest::prelude::*;

use engine_core::closed::SessionEnd;
use session::offer::{ColdStart, ExpiryOutcome, Offer, OfferMachine, OpenOffer, cold_start};
use session::store::{BookmarkStore, StoredPosition};

/// The document the cold start opens: one heading and one spoken
/// paragraph. The CONTENT is the only key (FR-036).
const COLD_START_DOC: &[u8] = b"# Chapter\n\nO onibus sai as seis.";

/// A second document: different content, so a different content
/// fingerprint (FR-036) — a row stored under it can never resolve
/// the first document's open.
const OTHER_DOC: &[u8] = b"# Outro\n\nO trem passa as sete.";

/// The redb file a test store lives in, inside its own temp dir.
fn store_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("bookmarks.redb")
}

/// A playback position on the exact quarter-second grid
/// `k × 0.25` ∈ [0, 3600] s — an exact binary-fraction grid, so
/// the offer offsets compare bit-exactly.
#[expect(
    clippy::cast_precision_loss,
    reason = "the quarter-second grid k × 0.25 is an exact binary fraction for every k in 0..=14400, so the u64→f64 cast is exact on this range; the lint is range-blind"
)]
fn offset_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// The start of a continuous non-advancing interval (the FR-015
/// supersede clock), in milliseconds.
fn first_yield_strategy() -> impl Strategy<Value = u64> {
    0u64..=3_600_000
}

/// A stored position from an unexpected end — the kill, power-loss,
/// battery, or network-loss row (FR-032d): the end kind that
/// leaves the explicit-resume requirement.
fn interrupted_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (offset_strategy(), first_yield_strategy()).prop_map(|(offset_secs, first_yield_ms)| {
        StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind: SessionEnd::Interrupted,
        }
    })
}

/// A stored position from a deliberate pause — the end kind that
/// opens the FR-032c 3-second continue offer.
fn paused_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (offset_strategy(), first_yield_strategy()).prop_map(|(offset_secs, first_yield_ms)| {
        StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind: SessionEnd::Paused,
        }
    })
}

/// The number of offer-window expiries the parked cold start must
/// survive.
///
/// The bound starts at TWO: the first expiry is the 3-second window
/// elapsing, and the SECOND is the re-poll that the same
/// `opened_at` provokes — the defect's exact shape, and the first
/// expiry that can downgrade the requirement.
fn expiry_count_strategy() -> impl Strategy<Value = u32> {
    2u32..=8
}

/// The app-level parked cold start (T110), modelled after
/// `src-tauri/src/lib.rs`'s `PendingOpen` (`:1567`): the document
/// that waits, the stored position it waits at, the instant its
/// offer opened, and whether it is still parked.
///
/// The model carries DATA, never a decision: the offer machine owns
/// the decision of whether and when the reading starts (FR-044).
/// The only decision the model makes is the shell's own ROUTING of
/// the machine's answer (`src-tauri/src/lib.rs:1781`), which it
/// mirrors branch for branch.
#[derive(Debug)]
struct ParkedColdStart {
    /// The document text — the sole synthesis input (FR-018).
    document: Vec<u8>,
    /// The stored position the offer continues from, in seconds.
    offset_secs: f64,
    /// The instant the offer opened (the app's `opened_at`): the
    /// re-park KEEPS it, so the window stays elapsed for every
    /// later poll — the app never opens a second window over the
    /// same requirement.
    opened_at_poll: u32,
    /// Whether the cold start is still parked, waiting for its
    /// resolution.
    parked: bool,
}

/// The ways a parked open resolves — the shell's own vocabulary
/// (`src-tauri/src/lib.rs`'s `ParkedResolution`, `:1588`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    /// The FR-032c window elapsed with no choice.
    WindowElapsed,
    /// The reader resumed — the in-app continue choice or the
    /// lock-screen resume control.
    ReaderResumed,
    /// The reader chose "start over".
    StartOverChosen,
}

/// The shell's routing of the offer machine's answer at the
/// resolution of a parked open (`src-tauri/src/lib.rs:1781`).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Routing {
    /// The reading started at the named position.
    Started(f64),
    /// The parked open waits: the reader's explicit resume is the
    /// only start (FR-032d / FR-031).
    StillParked,
    /// The parked open was dropped: nothing is open any more, and
    /// no audio begins.
    Abandoned,
}

/// Resolve a parked cold start — the shell's own branch table
/// (`src-tauri/src/lib.rs`'s `resolve_parked_open`, `:1763`): the
/// offer machine's answer decides whether the reading starts, and
/// where.
///
/// The machine's answer is read VERBATIM: the model never re-derives
/// the offer's form (FR-044).
fn resolve_parked_open(
    machine: &mut OfferMachine,
    pending: &ParkedColdStart,
    choice: Choice,
) -> (Routing, ExpiryOutcome) {
    let outcome = match choice {
        // The reader chose "start over": the machine's own answer
        // is the first spoken position.
        Choice::StartOverChosen => machine.start_over(),
        // The window elapsed, or the reader resumed: the machine's
        // own answer decides.
        Choice::WindowElapsed | Choice::ReaderResumed => machine.expire(),
    };
    let routing = match outcome {
        ExpiryOutcome::AutoStart { offset_secs } => Routing::Started(offset_secs),
        ExpiryOutcome::ExplicitResume => {
            if choice == Choice::ReaderResumed {
                // The reader's own resume satisfies the
                // explicit-resume requirement: the reading starts
                // at the stored position the parked open holds
                // (FR-032d / FR-031).
                Routing::Started(pending.offset_secs)
            } else {
                // The window elapsed on an unexpected end: nothing
                // starts on its own, so the parked open waits
                // (FR-032d / FR-031).
                Routing::StillParked
            }
        }
        ExpiryOutcome::NoOffer => {
            if choice == Choice::ReaderResumed {
                // Nothing is open, but the reader's resume is an
                // explicit start: the reading begins at the stored
                // position the parked open holds.
                Routing::Started(pending.offset_secs)
            } else {
                // Nothing is open: the parked open is DROPPED.
                Routing::Abandoned
            }
        }
    };
    (routing, outcome)
}

/// One poll of the app's offer window — the shell's poll
/// (`src-tauri/src/lib.rs`'s `expire_parked_open`, `:1721`).
///
/// The app polls every 250 ms (`OFFER_WINDOW_POLL_MS`, `:1557`);
/// the parked open records when its offer opened, so the window
/// elapses 3 s later — and every poll AFTER that re-resolves the
/// same parked open, because the app re-parks the SAME open with
/// the SAME `opened_at` (`:1791-1805`).
fn poll_window(
    machine: &mut OfferMachine,
    parked: &mut ParkedColdStart,
) -> (Routing, ExpiryOutcome) {
    let (routing, outcome) = resolve_parked_open(machine, parked, Choice::WindowElapsed);
    match routing {
        // The reading started, or the open was dropped: nothing
        // waits any more.
        Routing::Started(_) | Routing::Abandoned => parked.parked = false,
        // The open waits for the reader's explicit resume: the app
        // re-parks the SAME open (the same `opened_at`), so the
        // window stays elapsed for the next poll.
        Routing::StillParked => {}
    }
    (routing, outcome)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T114 (I2 — THE RED PROPERTY): the explicit-resume cold
    /// start survives repeated offer-window expiries.
    ///
    /// A stored `Interrupted` row — the kill/power-loss row the
    /// 20 s heartbeat, the app-background write, or a focus-loss
    /// write leaves — cold starts into the explicit-resume form,
    /// the reading waits paused with an explicit resume control
    /// (FR-009 / FR-018 / FR-032d), and the requirement SURVIVES
    /// every expiry of the offer window: expiry N answers the
    /// explicit-resume form, never "no offer", and never the
    /// automatic start (FR-031). The parked open is never
    /// abandoned — the reader's explicit resume is the only start.
    ///
    /// Against the current code this FAILS at the SECOND expiry:
    /// `expire()` clears `explicit_resume` on the first call
    /// (`crates/session/src/offer.rs:340-363`) and answers `NoOffer`
    /// on the second, so the shell's `NoOffer` + `WindowElapsed`
    /// branch (`src-tauri/src/lib.rs:1807-1816`) DROPS the parked
    /// open and starts nothing — the cold start is abandoned
    /// ≈3.25 s after the open.
    #[test]
    fn prop_the_explicit_resume_cold_start_survives_repeated_offer_window_expiries(
        (stored, expiries) in (interrupted_position_strategy(), expiry_count_strategy())
    ) {
        // The cold-start decision (FR-044): the offer machine owns
        // it; the app routes it and never re-derives it.
        let mut machine = OfferMachine::new();
        let decision = machine.resolve_open(Some(stored));
        prop_assert_eq!(
            decision,
            OpenOffer::ExplicitResume,
            "a stored unexpected end did not resolve to the explicit-resume form (FR-032d)"
        );
        prop_assert_eq!(
            cold_start(decision),
            ColdStart::AwaitExplicitResume,
            "a stored unexpected end did not gate the cold start on an explicit resume (FR-009 / FR-032d)"
        );
        // An unexpected end opens NO continue offer: the reader's
        // explicit resume is the only start (FR-032d).
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "an unexpected end opened a continue offer (FR-032d)"
        );

        // The app parks the cold start behind the requirement: NO
        // audio begins, and the document waits at the stored
        // position.
        let mut parked = ParkedColdStart {
            document: COLD_START_DOC.to_vec(),
            offset_secs: stored.offset_secs,
            opened_at_poll: 0,
            parked: true,
        };

        for expiry in 1..=expiries {
            let (routing, outcome) = poll_window(&mut machine, &mut parked);

            // I2: the requirement SURVIVES the expiry — the
            // machine never downgrades it to "no offer", and it
            // never reaches the automatic start (FR-031).
            prop_assert_eq!(
                outcome,
                ExpiryOutcome::ExplicitResume,
                "the offer-window expiry {} downgraded the explicit-resume requirement to {:?} — the requirement must survive every expiry (FR-009 / FR-018 / FR-032d)",
                expiry,
                outcome
            );

            // The parked open is NOT abandoned: the reader's
            // explicit resume is the only start, so the open waits
            // (FR-009 / FR-032d / FR-031).
            prop_assert_eq!(
                routing,
                Routing::StillParked,
                "the offer-window expiry {} abandoned the parked cold start — the reader's explicit resume is the only start (FR-009 / FR-032d / FR-031)",
                expiry
            );
            prop_assert!(
                parked.parked,
                "the offer-window expiry {} dropped the parked cold start (FR-009 / FR-032d)",
                expiry
            );
            // The parked open still holds the document it waits
            // on: an abandoned open loses the reading too (FR-018).
            prop_assert_eq!(
                parked.document.as_slice(),
                COLD_START_DOC,
                "the offer-window expiry {} lost the parked document (FR-018)",
                expiry
            );
            // The re-park keeps the SAME open: the app never opens
            // a second offer window over the same requirement.
            prop_assert_eq!(
                parked.opened_at_poll,
                0,
                "the offer-window expiry {} re-parked the cold start under a NEW offer window (FR-009)",
                expiry
            );
        }

        // The reader's explicit resume is the ONLY start: it
        // resolves the parked open at the stored position the row
        // holds (FR-032d / FR-031).
        let (routing, _) = resolve_parked_open(&mut machine, &parked, Choice::ReaderResumed);
        prop_assert_eq!(
            routing,
            Routing::Started(parked.offset_secs),
            "the reader's explicit resume did not start the reading at the stored position (FR-032d / FR-031)"
        );
    }

    /// T114 (I1): a stored `Interrupted` row — the kill/power-loss
    /// row — resolves the cold start to the explicit-resume form
    /// over the REAL store and the REAL open lookup, NEVER to
    /// "no offer / start nothing" (FR-009 / FR-018 / FR-032d).
    #[test]
    fn prop_a_stored_interrupted_row_cold_starts_to_the_explicit_resume_form(
        stored in interrupted_position_strategy()
    ) {
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        let key = session::fingerprint::fingerprint(COLD_START_DOC);
        store.put(&key, stored)?;

        // The REAL cold-start lookup: bytes → fingerprint → the
        // stored row → the offer form (T036 / FR-044).
        let lookup = session::store::lookup_on_open(&mut store, COLD_START_DOC)?;
        prop_assert_eq!(
            lookup.stored,
            Some(stored),
            "the cold-start lookup did not find the stored row it just wrote (T036)"
        );
        prop_assert_eq!(
            lookup.offer,
            OpenOffer::ExplicitResume,
            "a stored unexpected end did not resolve to the explicit-resume form (FR-032d)"
        );
        prop_assert_eq!(
            cold_start(lookup.offer),
            ColdStart::AwaitExplicitResume,
            "a stored unexpected end cold started into 'no offer / start nothing' instead of the explicit-resume form (FR-009 / FR-018 / FR-032d)"
        );
    }

    /// T114 (I3 — THE GUARD): the deliberate-pause path is
    /// UNAFFECTED by the explicit-resume latch. The 3-second
    /// continue offer is open, and its silence continues from the
    /// stored position (FR-009 / FR-032c).
    ///
    /// This property is GREEN today and MUST stay green after the
    /// repair: the latch is for the unexpected end alone, never for
    /// the deliberate pause.
    #[test]
    fn prop_the_deliberate_pause_offer_still_silences_and_continues_from_the_stored_position(
        stored in paused_position_strategy()
    ) {
        let mut machine = OfferMachine::new();
        let decision = machine.resolve_open(Some(stored));
        prop_assert_eq!(
            decision,
            OpenOffer::Continue {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            },
            "a stored deliberate pause did not resolve to the continue offer (FR-032c)"
        );
        prop_assert_eq!(
            cold_start(decision),
            ColdStart::AwaitOffer {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            },
            "a stored deliberate pause did not gate the cold start on the 3-second offer (FR-009 / FR-032c)"
        );
        // The offer is open, and it carries the stored position
        // the silence rule continues from.
        prop_assert_eq!(
            machine.continue_offer(),
            Some(Offer::Continue {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            }),
            "the 3-second offer did not carry the stored position the silence continues from (FR-009)"
        );

        // The window elapses with the reader present (the
        // foreground latch, read at expiry ONLY): the reading
        // continues from the stored position (FR-009 / FR-032c).
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::AutoStart {
                offset_secs: stored.offset_secs,
            },
            "the 3-second offer's silence did not continue from the stored position (FR-009 / FR-032c)"
        );

        // The offer's OTHER choice — "start over" — begins at the
        // FIRST SPOKEN POSITION, never at the stored one (FR-032c).
        // The deliberate pause's start-over alternative is a
        // deliberate choice; the unexpected end's resume is the
        // only start, and it is the reader's own (FR-032d).
        let mut over = OfferMachine::new();
        prop_assert_eq!(
            over.resolve_open(Some(stored)),
            OpenOffer::Continue {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            },
            "the start-over alternative did not open on the deliberate pause (FR-032c)"
        );
        let over_parked = ParkedColdStart {
            document: COLD_START_DOC.to_vec(),
            offset_secs: stored.offset_secs,
            opened_at_poll: 0,
            parked: true,
        };
        let (routing, outcome) =
            resolve_parked_open(&mut over, &over_parked, Choice::StartOverChosen);
        prop_assert_eq!(
            outcome,
            ExpiryOutcome::AutoStart { offset_secs: 0.0 },
            "the start-over choice did not begin at the first spoken position (FR-032c)"
        );
        prop_assert_eq!(
            routing,
            Routing::Started(0.0),
            "the start-over choice did not route the reading to the first spoken position (FR-032c)"
        );
    }

    /// T114 (I4 — THE GUARD): a document with NO stored row still
    /// starts immediately — no offer, no gate (FR-036a).
    #[test]
    fn prop_a_document_with_no_stored_row_still_starts_immediately(
        foreign in paused_position_strategy()
    ) {
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        // A row under ANOTHER document's fingerprint never resolves
        // this document's open: the key is the content, never an
        // identity (FR-036).
        let other_key = session::fingerprint::fingerprint(OTHER_DOC);
        store.put(&other_key, foreign)?;

        let lookup = session::store::lookup_on_open(&mut store, COLD_START_DOC)?;
        prop_assert!(
            lookup.stored.is_none(),
            "the cold-start lookup found a stored row for a document that was never read (FR-036)"
        );
        prop_assert_eq!(
            lookup.offer,
            OpenOffer::NoOffer,
            "a document with no stored row opened an offer (FR-036a)"
        );
        prop_assert_eq!(
            cold_start(lookup.offer),
            ColdStart::StartNow,
            "a document with no stored row did not start immediately (FR-036a)"
        );

        // The machine agrees: a miss opens nothing, and its expiry
        // yields no outcome (FR-036a).
        let mut machine = OfferMachine::new();
        prop_assert_eq!(
            machine.resolve_open(None),
            OpenOffer::NoOffer,
            "a miss resolved to an offer (FR-036a)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "a miss produced an expiry outcome (FR-036a)"
        );
    }
}
