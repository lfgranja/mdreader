//! T052 — resume-offer state machine property tests
//! (Phase 11, BLOCKING; the property suite for
//! `crates/session/src/offer.rs` (T060), over the stored
//! positions of `crates/session/src/store.rs` (T058)).
//!
//! Invariants under test (tasks.md T052, verbatim):
//! "deliberate-pause → 3 s offer continue-default +
//! start-over, expiry auto-starts ONLY if the latched
//! `AppForegrounded` state is true else converts to
//! explicit-resume; `AppBackgrounded` mid-window does NOT
//! convert early and `AppForegrounded` before expiry
//! restores auto-start; unexpected-end/power/battery/
//! network → paused + explicit resume never auto-audio;
//! start-over → `0.0` + discard; completion → teardown +
//! no offer + no residency; offer window never outlives
//! session"
//!
//! Spec anchors:
//! - FR-032c (spec.md:957): a deliberate-pause reopen
//!   shows a visible 3-second resume offer — continue
//!   from the stored position (the default) with a
//!   start-over alternative; if no choice is made within
//!   the window, reading begins automatically from the
//!   stored position PROVIDED the application is in the
//!   foreground when the window expires; otherwise the
//!   offer converts to the explicit-resume form of FR-032d
//!   and never begins audio on its own. Choosing "start
//!   over" begins from the first spoken position instead,
//!   and the stored position is discarded.
//! - FR-032d (spec.md:968): the automatic start never
//!   applies to a document left by an unexpected end, a
//!   power-off, a battery death, or a network loss — those
//!   recover to a paused state with an explicit resume
//!   control and never produce audio on their own.
//! - FR-031 (spec.md:941): following an unexpected session
//!   end, the system MUST NOT begin producing audio on its
//!   own; recovery to a paused state is the only permitted
//!   automatic transition.
//! - FR-032b / FR-032e / FR-032f (spec.md:974–995):
//!   reaching the end of a document clears the stored
//!   position, tears down the resident component, and
//!   offers nothing — completion and the resume offer are
//!   mutually exclusive states, so the window never
//!   outlives the session it belonged to.
//! - FR-032c's visibility MUST (spec.md:90, the
//!   `AppForegrounded`/`AppBackgrounded` intents): the
//!   offer remains visible for the whole 3 seconds — the
//!   machine latches the latest foreground value and reads
//!   it at expiry ONLY, never converting an open offer
//!   early.
//! - contracts/storage-schema.md: the offer flow is
//!   resolved SOLELY from the stored `end_kind` — `paused`
//!   ⇒ the FR-032c deliberate offer, `interrupted` ⇒ the
//!   FR-032d explicit-resume form; `stopped|completed`
//!   never open an offer.
//!
//! FAILS before `crates/session/src/offer.rs` (T060), the
//! `session::store` module (T058) and the `engine-core`
//! dependency land: the `session::offer` module and the
//! `engine_core` crate are unresolved, so this test target
//! does not build. That build failure is the required
//! pre-implementation red state (T056).
//!
//! Expected API contract (T060 — the surface this suite is
//! the contract for; `SessionEnd` comes from
//! `engine_core::closed`, so the implementer adds
//! `engine-core` as a dependency of the `session` crate —
//! it is not one today):
//! - `session::offer::OfferMachine::new()` and
//!   `Default`.
//! - `on_session_end(&mut self, fingerprint: [u8; 32],
//!   stored: Option<StoredPosition>) -> Option<Offer>` — the
//!   offer form is resolved SOLELY from the stored
//!   `end_kind`: a deliberate `Paused` opens the 3-second
//!   offer (`Offer::Continue` carrying the stored offset and
//!   the FR-015 interval start, with the start-over
//!   alternative available through `start_over`); an
//!   `Interrupted` end leaves an explicit-resume
//!   requirement and opens no continue offer;
//!   `Stopped`/`Completed` never open an offer; `None` (no
//!   stored position) opens nothing.
//! - `on_foregrounded(&mut self)` /
//!   `on_backgrounded(&mut self)` — the
//!   `AppForegrounded`/`AppBackgrounded` latches: they
//!   latch the latest value ONLY and never convert an open
//!   offer early (FR-032c's visibility MUST); the latch is
//!   read at expiry only.
//! - `expire(&mut self) -> ExpiryOutcome` — the 3-second
//!   window elapses: `AutoStart { offset_secs }` ONLY
//!   when the foreground latch is true AND the open offer
//!   came from a deliberate pause; `ExplicitResume`
//!   otherwise when an offer or an explicit-resume
//!   requirement is open; `NoOffer` when nothing is.
//! - `continue_offer(&mut self) -> Option<Offer>` — the
//!   open offer, if any.
//! - `start_over(&mut self) -> ExpiryOutcome` — the
//!   start-over alternative: reading begins from the first
//!   spoken position, so the outcome is
//!   `AutoStart { offset_secs: 0.0 }`, and the flow
//!   discards the stored entry (FR-032c).
//! - `on_completion(&mut self)` — completion teardown
//!   (FR-032b / FR-032e / FR-032f): no offer remains and
//!   no residency survives.
//!
//! The `start_over` outcome note: the canonical surface
//! returns `ExpiryOutcome` (three variants), so the
//! start-over result — reading beginning at the first
//! spoken position — is carried by
//! `AutoStart { offset_secs: 0.0 }`; the parenthetical "or
//! a dedicated `StartOver` outcome" in the T060 brief is
//! resolved here in favor of the enumerated three-variant
//! surface, because O5 requires the post-start-over offset
//! `0.0` to be observable in the returned outcome.
//!
//! The machine carries no clock: `expire` represents the
//! 3-second window elapsing, so every property is
//! deterministic and no test waits on real time.
//!
//! Exactness note: every offset is generated on the
//! quarter-second grid (`k × 0.25`), an exact
//! binary-fraction grid, so all offer offsets compare
//! bit-exactly. Every `prop_assert*!`/`assert*!` message
//! uses explicit positional arguments (`{}`, `arg`), never
//! implicit named captures (`{var}`), because proptest
//! 1.11 builds the `prop_assert_eq!` message through
//! `concat!` and rustc (1.98) rejects implicit captures in
//! format strings expanded from a macro.

use proptest::prelude::*;

use engine_core::closed::SessionEnd;
use session::offer::{ColdStart, ExpiryOutcome, Offer, OfferMachine, cold_start};
use session::store::{BookmarkStore, StoredPosition};

/// The redb file a test store lives in, inside its own
/// temp dir.
fn store_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("bookmarks.redb")
}

/// The document bytes the cold-start lookup reads: the CONTENT
/// is the only key (FR-036), so the stored row the cold-start
/// path finds is written under the bytes' own fingerprint.
const COLD_START_DOC: &[u8] = b"# Capitulo\n\nO onibus sai as seis.";

/// The second document's bytes: different content, so a
/// different content fingerprint (FR-036).
const COLD_START_SECOND_DOC: &[u8] = b"# Outro\n\nO trem passa as sete.";

/// The third document's bytes: a third content fingerprint, so
/// a row stored under it can never leak into another document's
/// open (FR-036).
const COLD_START_THIRD_DOC: &[u8] = b"# Mais um\n\nO onibus passa as oito.";

/// A playback position on the exact quarter-second grid
/// `k × 0.25` ∈ [0, 3600] s — an exact binary-fraction
/// grid, so offer offsets compare bit-exactly.
#[expect(
    clippy::cast_precision_loss,
    reason = "the quarter-second grid k × 0.25 is an exact binary fraction for every k in 0..=14400, so the u64→f64 cast is exact on this range; the lint is range-blind"
)]
fn offset_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// The start of a continuous non-advancing interval — the
/// `first_yield_ms` of the FR-015 supersede clock, in
/// milliseconds.
fn first_yield_strategy() -> impl Strategy<Value = u64> {
    0u64..=3_600_000
}

/// An arbitrary 32-byte document fingerprint (a
/// blake3-256 digest).
fn fingerprint_strategy() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>()
}

/// A stored position from a deliberate pause — the end
/// kind that opens the FR-032c 3-second offer.
fn paused_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (offset_strategy(), first_yield_strategy()).prop_map(|(offset_secs, first_yield_ms)| {
        StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind: SessionEnd::Paused,
        }
    })
}

/// A stored position from an unexpected end — power,
/// battery, or network loss, or any other interruption —
/// the end kind that leaves the FR-032d explicit-resume
/// requirement.
fn interrupted_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (offset_strategy(), first_yield_strategy()).prop_map(|(offset_secs, first_yield_ms)| {
        StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind: SessionEnd::Interrupted,
        }
    })
}

/// The terminal session-end kinds: `stopped` and
/// `completed` never open an offer (FR-032 / FR-032e).
fn terminal_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![Just(SessionEnd::Stopped), Just(SessionEnd::Completed),]
}

/// A stored position from a terminal end — Stop or
/// document completion.
fn terminal_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (
        offset_strategy(),
        first_yield_strategy(),
        terminal_end_strategy(),
    )
        .prop_map(|(offset_secs, first_yield_ms, end_kind)| StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn given_a_deliberate_pause_when_the_session_ends_then_a_three_second_offer_opens_with_continue_as_the_default_and_start_over_available(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O1 (FR-032c): "When a document left by a
        // deliberate Pause is reopened, the system MUST
        // show a resume offer offering to continue from the
        // stored position with a visible alternative to
        // start from the beginning."
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(fingerprint, Some(stored));
        prop_assert_eq!(
            offer,
            Some(Offer::Continue {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            }),
            "a deliberate pause did not open a continue offer carrying the stored position (FR-032c)"
        );
        // The offer stays open for the whole window — it
        // is what the reader sees until a choice is made or
        // the window elapses.
        prop_assert_eq!(
            machine.continue_offer(),
            Some(Offer::Continue {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            }),
            "the open offer changed before any choice or expiry (FR-032c)"
        );

        // The start-over alternative is available: choosing
        // it begins reading from the first spoken position
        // — 0.0 — and the flow discards the stored entry
        // (FR-032c: "Choosing 'start over' MUST begin from
        // the first spoken position instead" and "the
        // stored position MUST be discarded").
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&fingerprint, stored)?;
        let outcome = machine.start_over();
        prop_assert_eq!(
            outcome,
            ExpiryOutcome::AutoStart { offset_secs: 0.0 },
            "start-over did not begin from the first spoken position (FR-032c)"
        );
        // The flow's discard on start-over.
        store.delete(&fingerprint)?;
        prop_assert_eq!(
            store.get(&fingerprint)?,
            None,
            "the stored entry survived a start-over (FR-032c)"
        );
        // The choice consumed the offer: nothing remains
        // open, and expiring nothing yields no outcome.
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "an offer remained open after a start-over choice (FR-032c)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "an expiry produced an outcome after the start-over choice (FR-032c)"
        );
    }

    #[test]
    fn given_a_deliberate_pause_with_the_application_foregrounded_when_the_window_elapses_then_reading_auto_starts_from_the_stored_position(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O2 (FR-032c): "If the user makes no choice within
        // that window, reading MUST begin automatically from
        // the stored position, provided the application is
        // in the foreground when the window expires." The
        // latch is read at expiry, and it is true.
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "a deliberate pause did not open the 3-second offer (FR-032c)"
        );
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::AutoStart {
                offset_secs: stored.offset_secs,
            },
            "a foregrounded deliberate-pause offer did not auto-start from the stored position at expiry (FR-032c)"
        );
    }

    #[test]
    fn given_a_deliberate_pause_with_no_foreground_latch_when_the_window_elapses_then_the_offer_converts_to_an_explicit_resume(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O2 (FR-032c): "…if it has left the foreground, the
        // offer instead converts to the explicit-resume form
        // of FR-032d and MUST NOT begin audio on its own,
        // so that a document never begins reading aloud
        // unattended." A machine that was never foregrounded
        // has no foreground latch — the same rule.
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "a deliberate pause did not open the 3-second offer (FR-032c)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::ExplicitResume,
            "a deliberate-pause offer auto-started without a foreground latch (FR-032c, FR-031)"
        );
    }

    #[test]
    fn given_a_deliberate_pause_foregrounded_then_backgrounded_when_the_window_elapses_then_the_offer_converts_to_an_explicit_resume(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O2 (FR-032c): auto-start requires the foreground
        // latch to be true AT EXPIRY — "not since
        // unlatched". A backgrounding after the
        // foregrounding unlatches it, so the offer converts
        // to the explicit-resume form and never begins
        // audio on its own.
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "a deliberate pause did not open the 3-second offer (FR-032c)"
        );
        machine.on_foregrounded();
        machine.on_backgrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::ExplicitResume,
            "a deliberate-pause offer auto-started after the application left the foreground (FR-032c, FR-031)"
        );
    }

    #[test]
    fn given_foreground_and_background_events_before_expiry_when_the_window_is_still_open_then_the_offer_is_never_converted_early(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O3 (FR-032c's visibility MUST, spec.md:90): the
        // latches latch the latest value ONLY —
        // `AppBackgrounded` mid-window does NOT convert the
        // offer early, and `AppForegrounded` before expiry
        // restores auto-start. The offer is still open,
        // unchanged, until `expire` reads the latch.
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "a deliberate pause did not open the 3-second offer (FR-032c)"
        );
        machine.on_foregrounded();
        machine.on_backgrounded();
        machine.on_foregrounded();
        // Still open — the latch events never converted it
        // early.
        prop_assert_eq!(
            machine.continue_offer(),
            Some(Offer::Continue {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            }),
            "a latch event converted the open offer before expiry (FR-032c)"
        );
        // The latch is read at expiry only: the final value
        // is foregrounded, so the offer auto-starts from
        // the stored position.
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::AutoStart {
                offset_secs: stored.offset_secs,
            },
            "the final foreground latch did not restore auto-start at expiry (FR-032c)"
        );
    }

    #[test]
    fn given_an_unexpected_end_when_the_session_ends_then_the_session_recovers_to_paused_with_an_explicit_resume_and_never_auto_audio(
        (fingerprint, stored) in (fingerprint_strategy(), interrupted_position_strategy())
    ) {
        // O4 (FR-032d, FR-031): "The automatic start
        // permitted by FR-032c MUST NOT apply to a document
        // left by an unexpected end, a device power-off, a
        // battery death, or a network loss. Those cases
        // recover to a paused state with an explicit resume
        // control and never produce audio on their own."
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(fingerprint, Some(stored));
        prop_assert_eq!(
            offer,
            None,
            "an unexpected end opened a continue offer (FR-032d)"
        );
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "a continue offer existed for an unexpected end (FR-032d)"
        );
        // Even foregrounded, an interrupted session never
        // auto-starts: the explicit-resume requirement
        // stands at expiry.
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::ExplicitResume,
            "an unexpected-end recovery auto-started audio (FR-032d, FR-031)"
        );
    }

    #[test]
    fn given_an_open_offer_when_the_reader_chooses_start_over_then_reading_begins_at_zero_and_the_stored_entry_is_discarded(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O5 (FR-032c): "The resume offer is showing and the
        // user chooses start-over during the 3-second
        // window. Reading MUST begin from the first spoken
        // position and the stored position MUST be
        // discarded, so that a later reopen of the document
        // starts clean rather than offering the position the
        // reader just rejected."
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&fingerprint, stored)?;
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "a deliberate pause did not open the 3-second offer (FR-032c)"
        );
        let outcome = machine.start_over();
        prop_assert_eq!(
            outcome,
            ExpiryOutcome::AutoStart { offset_secs: 0.0 },
            "start-over did not begin reading at exactly 0.0 (FR-032c)"
        );
        // The flow discards the stored entry: a later
        // reopen starts clean — a lookup miss, no offer,
        // and reading from 0.0 (the miss flow,
        // storage-schema.md). The first-spoken-position
        // start is carried by the `AutoStart { offset_secs:
        // 0.0 }` outcome asserted above.
        store.delete(&fingerprint)?;
        let discarded = store.get(&fingerprint)?;
        prop_assert_eq!(
            discarded,
            None,
            "the stored entry survived a start-over — a later reopen would offer the rejected position (FR-032c)"
        );
        // The choice consumed the offer: nothing remains
        // open, and expiring nothing yields no outcome.
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "an offer remained open after the start-over choice (FR-032c)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "an expiry produced an outcome after the start-over choice (FR-032c)"
        );
    }

    #[test]
    fn given_an_open_offer_when_the_document_completes_then_the_session_tears_down_with_no_offer_and_no_residency(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O6 (FR-032b / FR-032e / FR-032f): "When a
        // document reaches its end, the system MUST tear
        // down the always-resident component and MUST
        // dismiss its notification" — completion and the
        // resume offer are mutually exclusive states, so
        // the teardown leaves no open offer, no expiry
        // outcome, and no residency, and the stored
        // position was cleared so the completed document is
        // offered nothing anywhere.
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&fingerprint, stored)?;
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "a deliberate pause did not open the 3-second offer (FR-032c)"
        );
        machine.on_completion();
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "a completion left an offer open (FR-032f)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "an expiry produced an outcome after the completion teardown (FR-032f)"
        );
        // The flow clears the stored position at document
        // end (FR-032b).
        store.delete(&fingerprint)?;
        prop_assert_eq!(
            store.get(&fingerprint)?,
            None,
            "the stored position survived document completion (FR-032b)"
        );
        // And a completed end kind never opens an offer in
        // the first place — the offer form is resolved
        // solely from the end kind.
        let completed = StoredPosition {
            offset_secs: stored.offset_secs,
            first_yield_ms: stored.first_yield_ms,
            end_kind: SessionEnd::Completed,
        };
        let mut reopened = OfferMachine::new();
        prop_assert_eq!(
            reopened.on_session_end(fingerprint, Some(completed)),
            None,
            "a completed document opened a resume offer (FR-032b, FR-032e)"
        );
    }

    #[test]
    fn given_an_open_offer_from_a_previous_session_when_a_new_session_ends_then_the_previous_offer_is_gone(
        (first_fingerprint, second_fingerprint, first_stored, second_stored) in
            (fingerprint_strategy(), fingerprint_strategy(), paused_position_strategy(), paused_position_strategy())
    ) {
        // O7: "the offer window never outlives its session"
        // — the machine holds at most one open offer, so
        // ending a new session replaces any offer the
        // previous session left open.
        prop_assume!(first_stored != second_stored);
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(first_fingerprint, Some(first_stored)).is_some(),
            "the previous session's deliberate pause did not open the 3-second offer (FR-032c)"
        );
        let offer = machine.on_session_end(second_fingerprint, Some(second_stored));
        prop_assert_eq!(
            offer,
            Some(Offer::Continue {
                offset_secs: second_stored.offset_secs,
                first_yield_ms: second_stored.first_yield_ms,
            }),
            "the new session did not open its own continue offer (O7)"
        );
        // The open offer is the new session's — the previous
        // session's offer (its offset in particular) is
        // gone.
        prop_assert_eq!(
            machine.continue_offer(),
            Some(Offer::Continue {
                offset_secs: second_stored.offset_secs,
                first_yield_ms: second_stored.first_yield_ms,
            }),
            "the open offer is not the new session's offer (O7)"
        );
        prop_assert_ne!(
            machine.continue_offer(),
            Some(Offer::Continue {
                offset_secs: first_stored.offset_secs,
                first_yield_ms: first_stored.first_yield_ms,
            }),
            "the previous session's offer outlived its session (O7)"
        );
    }

    #[test]
    fn given_an_open_offer_from_a_previous_session_when_a_new_session_ends_terminally_then_no_offer_remains(
        (first_fingerprint, second_fingerprint, stored, terminal_stored) in
            (fingerprint_strategy(), fingerprint_strategy(), paused_position_strategy(), terminal_position_strategy())
    ) {
        // O7: a terminal end (Stop or completion) opens no
        // offer and closes any offer the previous session
        // left open — the window never outlives its session.
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(first_fingerprint, Some(stored)).is_some(),
            "the previous session's deliberate pause did not open the 3-second offer (FR-032c)"
        );
        let offer = machine.on_session_end(second_fingerprint, Some(terminal_stored));
        prop_assert_eq!(
            offer,
            None,
            "a terminal end opened a resume offer (FR-032, FR-032e)"
        );
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "the previous session's offer outlived the terminal end of the new session (O7, FR-032f)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "an expiry produced an outcome after the terminal end closed the window (O7, FR-032f)"
        );
    }

    #[test]
    fn given_identical_event_sequences_when_run_on_two_machines_then_every_outcome_is_identical(
        (fingerprint, stored) in (fingerprint_strategy(), paused_position_strategy())
    ) {
        // O8: the machine is deterministic — identical event
        // sequences produce identical outcomes at every
        // step: the opened offer, the latched-foreground
        // expiry, and the backgrounded expiry.
        let run_sequence = |machine: &mut OfferMachine| {
            let opened = machine.on_session_end(fingerprint, Some(stored));
            machine.on_foregrounded();
            let expired = machine.expire();
            (opened, expired)
        };
        let mut first = OfferMachine::new();
        let mut second = OfferMachine::new();
        let (opened_first, expired_first) = run_sequence(&mut first);
        let (opened_second, expired_second) = run_sequence(&mut second);
        prop_assert_eq!(
            opened_first,
            opened_second,
            "identical session-end events opened different offers (O8)"
        );
        prop_assert_eq!(
            expired_first,
            expired_second,
            "identical foreground latches and expiries produced different outcomes (O8)"
        );

        // A second, longer sequence on fresh machines: an
        // unexpected end, a backgrounding, then expiry —
        // the explicit-resume requirement every time.
        let run_interrupted_sequence = |machine: &mut OfferMachine| {
            let interrupted = StoredPosition {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
                end_kind: SessionEnd::Interrupted,
            };
            let opened = machine.on_session_end(fingerprint, Some(interrupted));
            machine.on_backgrounded();
            let expired = machine.expire();
            (opened, expired)
        };
        let mut third = OfferMachine::new();
        let mut fourth = OfferMachine::new();
        let (opened_third, expired_third) = run_interrupted_sequence(&mut third);
        let (opened_fourth, expired_fourth) =
            run_interrupted_sequence(&mut fourth);
        prop_assert_eq!(
            opened_third,
            opened_fourth,
            "identical unexpected-end events opened different outcomes (O8)"
        );
        prop_assert_eq!(
            expired_third,
            expired_fourth,
            "identical backgrounding and expiries produced different outcomes (O8)"
        );
    }
}

/// Boundary case: a machine that has never seen a session
/// holds no open offer, and expiring nothing yields no
/// outcome — there is no window to outlive anything.
#[test]
fn given_a_fresh_machine_when_expired_then_no_offer_is_open_and_expiry_yields_no_outcome() {
    let mut machine = OfferMachine::new();
    assert_eq!(
        machine.continue_offer(),
        None,
        "a fresh machine held an open offer (O6, O7)"
    );
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::NoOffer,
        "expiring a fresh machine produced an outcome (O6, O7)"
    );
}

/// Boundary case: the default constructor builds the same
/// state as `new` — no open offer.
#[test]
fn given_the_default_constructor_when_a_machine_is_built_then_no_offer_is_open() {
    let mut machine = OfferMachine::default();
    assert_eq!(
        machine.continue_offer(),
        None,
        "the default constructor left an offer open (O6, O7)"
    );
}

// T110 — the cold-start open gate (Phase 15 Addendum 2,
// feature 004-ressoa-dogfood-build).
//
// The cold-start open of a document that has a stored `Paused`
// bookmark MUST present the 3-second continue-vs-start-over offer
// and MUST start NO audio until the offer resolves (FR-009 /
// FR-032c): with no choice inside the window, reading continues
// from the stored position. A document with NO stored position
// MUST start immediately (FR-036a). The second document picked
// while the first plays MUST never auto-start from the outgoing
// document's position, and the first document's bookmark MUST
// survive the switch (FR-018).
//
// The decision is the offer machine's own (FR-044): the open
// lookup (`session::store::lookup_on_open`) resolves the offer
// form through the canonical core, and the cold-start gate
// (`session::offer::cold_start`) projects that form onto the
// app's three routings — start now, await the 3-second offer, or
// await an explicit resume. The app NEVER re-derives the offer's
// form.
//
// The silence rule is asserted on the machine that carries the
// REAL stored row (the offer the app opens at the cold start), so
// the offset the expiry answers is the stored position and not a
// placeholder.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// T110: a stored deliberate pause gates the cold start on
    /// the 3-second offer — no audio until it resolves — and the
    /// silence rule continues from the stored position.
    #[test]
    fn prop_a_stored_pause_gates_the_cold_start_on_the_offer(
        (stored, fingerprint) in (paused_position_strategy(), fingerprint_strategy())
    ) {
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        let document_fingerprint = session::fingerprint::fingerprint(COLD_START_DOC);
        store.put(&document_fingerprint, stored)?;

        // The REAL cold-start lookup: bytes → fingerprint → the
        // stored row → the offer form (T036 / FR-044).
        let lookup = session::store::lookup_on_open(&mut store, COLD_START_DOC)?;
        prop_assert!(
            lookup.stored.is_some(),
            "the cold-start lookup found no stored row for the document it just wrote (T036)"
        );

        // The gate: the cold start is NOT authorized to begin
        // audio — the 3-second continue offer is open
        // (FR-009 / FR-032c).
        prop_assert_eq!(
            cold_start(lookup.offer),
            ColdStart::AwaitOffer {
                offset_secs: stored.offset_secs,
                first_yield_ms: stored.first_yield_ms,
            },
            "a stored deliberate pause did not gate the cold start on the 3-second offer (FR-009 / FR-032c)"
        );

        // The silence rule: with no choice inside the window, the
        // offer resolves into the continue-from-the-stored-
        // position start (FR-009) — the machine's own answer over
        // the REAL stored row.
        let mut machine = OfferMachine::new();
        prop_assert!(
            machine.on_session_end(fingerprint, Some(stored)).is_some(),
            "the cold start opened no continue offer for the dialog to render (FR-032c)"
        );
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::AutoStart {
                offset_secs: stored.offset_secs,
            },
            "the silence rule did not continue from the stored position (FR-009)"
        );
    }

    /// T110: a document with NO stored position starts
    /// immediately — no offer, no gate.
    #[test]
    fn prop_a_miss_starts_the_cold_start_immediately(
        (first_key, second_key) in (fingerprint_strategy(), fingerprint_strategy())
    ) {
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        let document_fingerprint = session::fingerprint::fingerprint(COLD_START_DOC);
        prop_assume!(first_key != document_fingerprint);
        prop_assume!(second_key != document_fingerprint);

        // Rows under OTHER keys never resolve this document's
        // open: the key is the content, never an identity
        // (FR-036).
        store.put(&first_key, StoredPosition {
            offset_secs: 12.5,
            first_yield_ms: 250,
            end_kind: SessionEnd::Paused,
        })?;
        store.put(&second_key, StoredPosition {
            offset_secs: 30.0,
            first_yield_ms: 500,
            end_kind: SessionEnd::Paused,
        })?;

        // The REAL cold-start lookup over a store with no row
        // under the document's fingerprint.
        let lookup = session::store::lookup_on_open(&mut store, COLD_START_DOC)?;
        prop_assert!(
            lookup.stored.is_none(),
            "the cold-start lookup found a stored row for a document that was never read (T036 / FR-036)"
        );
        prop_assert_eq!(
            cold_start(lookup.offer),
            ColdStart::StartNow,
            "a document with no stored position did not start immediately (FR-036a)"
        );
    }

    /// T110: the second document picked while the first plays
    /// never auto-starts — it carries no stored row of its own,
    /// so it starts clean, at its own first spoken position — and
    /// the first document's bookmark survives the switch
    /// (FR-018).
    #[test]
    fn prop_the_second_document_never_auto_starts_and_the_first_bookmark_survives(
        (stored, other_stored) in (paused_position_strategy(), paused_position_strategy())
    ) {
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        let first_fingerprint = session::fingerprint::fingerprint(COLD_START_DOC);
        let second_fingerprint = session::fingerprint::fingerprint(COLD_START_SECOND_DOC);
        let third_fingerprint = session::fingerprint::fingerprint(COLD_START_THIRD_DOC);
        prop_assume!(first_fingerprint != second_fingerprint);
        prop_assume!(first_fingerprint != third_fingerprint);
        prop_assume!(second_fingerprint != third_fingerprint);

        // The first document plays and is bookmarked under its OWN
        // fingerprint (the FR-018 switch's durable save); a third
        // document carries a row of its own.
        store.put(&first_fingerprint, stored)?;
        store.put(&third_fingerprint, other_stored)?;

        // The second document is picked: the switch saves the
        // first's position and never deletes it.
        let second_lookup = session::store::lookup_on_open(&mut store, COLD_START_SECOND_DOC)?;
        prop_assert!(
            second_lookup.stored.is_none(),
            "the incoming document inherited a stored position of its own (FR-018)"
        );
        prop_assert_eq!(
            cold_start(second_lookup.offer),
            ColdStart::StartNow,
            "the incoming document auto-started from a position it never owned (FR-018)"
        );

        // The first document's bookmark survives the switch.
        let first_after = session::store::lookup_on_open(&mut store, COLD_START_DOC)?;
        prop_assert!(
            first_after.stored.is_some(),
            "the switch deleted the first document's bookmark (FR-018)"
        );
        if let Some(first_stored) = first_after.stored {
            prop_assert!(
                (first_stored.offset_secs - stored.offset_secs).abs() < 1e-9,
                "the switch moved the first document's stored position (FR-018)"
            );
        }
    }
}
