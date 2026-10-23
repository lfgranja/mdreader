//! T112 — end-kind discriminant convergence tests (Phase 12, BLOCKING TEST).
//!
//! Pins the canonical `end_kind` vocabulary of
//! `crates/session/src/store.rs` (`end_kind_to_code` /
//! `end_kind_from_code`): `Paused` → `0`, `Stopped` → `1`,
//! `Completed` → `2`, `Interrupted` → `3` (FR-044).
//!
//! The discriminant functions are private, so this suite exercises
//! them through the PUBLIC surface only — the
//! `BookmarkStore::put` / `get` round-trip plus the canonical
//! `OfferMachine` in `crates/session/src/offer.rs` — reusing the
//! patterns of `prop_offer.rs` (T052) and `prop_resume.rs` (T050).
//!
//! The contradiction this suite converges (Phase 12):
//! - `crates/session/src/lib.rs` legacy `resume_offer` maps
//!   `Some(2)` → `ExplicitResume` (a `Completed` end offered resume —
//!   the SC-019 violation) and `Some(0) | Some(1)` →
//!   `DeliberatePause` (a `Stopped` end offered continue).
//! - `crates/player/tests/integration/test_session_recovery_us1.rs`
//!   pins a local `END_KIND_INTERRUPTED = 2`, contradicting the
//!   canonical `Interrupted` → `3`.
//!
//! Canonical behavior pinned here (all GREEN):
//! - every one of the four closed-set variants round-trips through
//!   the store distinctly (no two variants share a discriminant);
//! - `Stopped` / `Completed` ends never produce any resume offer
//!   through the canonical `OfferMachine` (the `prop_offer.rs` O6
//!   style, via the CANONICAL API only).
//!
//! The suite also carries ONE red test —
//! `legacy_code_2_completed_must_not_offer_explicit_resume` — which
//! FAILS against the legacy `session::resume_offer` today (code `2`
//! returns `ExplicitResume`). T113 deletes that test together with
//! the legacy function. The legacy `session::resume_offer` is
//! imported ONLY in that one red test; every guard test uses the
//! canonical `OfferMachine` / store API.
//!
//! Exactness note: every offset is generated on the
//! quarter-second grid (`k × 0.25`), an exact
//! binary-fraction grid, so all positions compare
//! bit-exactly. Every `prop_assert*!`/`assert*!` message
//! uses explicit positional arguments (`{}`, `arg`), never
//! implicit named captures (`{var}`), because proptest
//! 1.11 builds the `prop_assert_eq!` message through
//! `concat!` and rustc rejects implicit captures in
//! format strings expanded from a macro.

use proptest::prelude::*;

use engine_core::closed::SessionEnd;
use session::offer::{ExpiryOutcome, OfferMachine};
use session::store::{BookmarkStore, StoredPosition};

/// The redb file a test store lives in, inside its own
/// temp dir.
fn store_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("bookmarks.redb")
}

/// A playback position on the exact quarter-second grid
/// `k × 0.25` ∈ [0, 3600] s — an exact binary-fraction
/// grid, so positions compare bit-exactly.
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
/// blake3-256 digest) — the bookmark key.
fn fingerprint_strategy() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>()
}

/// Every member of the closed session-end set (FR-044) —
/// the complete vocabulary the discriminant covers.
fn end_kind_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![
        Just(SessionEnd::Paused),
        Just(SessionEnd::Stopped),
        Just(SessionEnd::Completed),
        Just(SessionEnd::Interrupted),
    ]
}

/// A stored position carrying any member of the closed
/// session-end set.
fn any_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (
        offset_strategy(),
        first_yield_strategy(),
        end_kind_strategy(),
    )
        .prop_map(|(offset_secs, first_yield_ms, end_kind)| StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind,
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
    fn given_any_end_kind_when_stored_then_the_entry_round_trips_exactly(
        (fingerprint, stored) in (fingerprint_strategy(), any_position_strategy())
    ) {
        // Guard (FR-044): every member of the closed
        // four-variant set survives the store round-trip
        // exactly — the discriminant covers the whole set
        // and names no end kind outside it.
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&fingerprint, stored)?;
        let found = store.get(&fingerprint)?;
        prop_assert_eq!(
            found,
            Some(stored),
            "the stored end kind {:?} did not round-trip through the store (FR-044)",
            stored.end_kind
        );
    }

    #[test]
    fn given_a_terminal_end_when_the_session_ends_then_no_offer_opens_and_expiry_yields_no_outcome(
        (fingerprint, stored) in (fingerprint_strategy(), terminal_position_strategy())
    ) {
        // Guard (FR-032 / FR-032e, the `prop_offer.rs` O6
        // style through the CANONICAL `OfferMachine` only):
        // a `Stopped` / `Completed` end never produces any
        // resume offer — no continue offer opens, none stays
        // open, and the window yields no outcome.
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(fingerprint, Some(stored));
        prop_assert_eq!(
            offer,
            None,
            "a terminal end ({:?}) opened a resume offer (FR-032, FR-032e)",
            stored.end_kind
        );
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "a terminal end ({:?}) left a continue offer open (FR-032, FR-032e)",
            stored.end_kind
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "a terminal end ({:?}) produced an expiry outcome (FR-032, FR-032e)",
            stored.end_kind
        );
    }
}

/// Guard (FR-044): the four closed-set variants carry four
/// DISTINCT discriminants — all four entries coexist in one
/// store and each reads back exactly. A collision (two
/// variants sharing one code) would surface here as a
/// mismatched read-back.
#[test]
fn all_four_end_kinds_carry_distinct_discriminants() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let mut store = BookmarkStore::open(&path)?;

    let variants = [
        SessionEnd::Paused,
        SessionEnd::Stopped,
        SessionEnd::Completed,
        SessionEnd::Interrupted,
    ];
    let fingerprints: [[u8; 32]; 4] = [[1; 32], [2; 32], [3; 32], [4; 32]];
    for (fingerprint, end_kind) in fingerprints.iter().zip(variants) {
        let stored = StoredPosition {
            offset_secs: 120.0,
            first_yield_ms: 40,
            end_kind,
        };
        store.put(fingerprint, stored)?;
    }
    for (fingerprint, end_kind) in fingerprints.iter().zip(variants) {
        let found = store.get(fingerprint)?;
        assert_eq!(
            found.map(|position| position.end_kind),
            Some(end_kind),
            "the end kind {end_kind:?} did not survive alongside the other three variants (FR-044)"
        );
    }
    Ok(())
}

/// Guard (FR-032c / FR-032d, canonical API): the retained end
/// kinds keep their offer forms — a `Paused` end opens the
/// deliberate continue offer, an `Interrupted` end leaves the
/// explicit-resume requirement and opens no continue offer.
/// Together with the terminal-no-offer guard above, this pins
/// the full four-way offer table of storage-schema.md.
#[test]
fn retained_end_kinds_keep_their_canonical_offer_forms() {
    let fingerprint = [9; 32];

    // A deliberate pause opens the 3-second continue offer
    // carrying the stored position (FR-032c).
    let paused = StoredPosition {
        offset_secs: 120.0,
        first_yield_ms: 40,
        end_kind: SessionEnd::Paused,
    };
    let mut machine = OfferMachine::new();
    let offer = machine.on_session_end(fingerprint, Some(paused));
    assert!(
        offer.is_some(),
        "a deliberate pause did not open the continue offer (FR-032c)"
    );

    // An unexpected end opens no continue offer, and even
    // foregrounded the expiry stands as the explicit-resume
    // requirement — never an auto-start (FR-032d / FR-031).
    let interrupted = StoredPosition {
        offset_secs: 120.0,
        first_yield_ms: 40,
        end_kind: SessionEnd::Interrupted,
    };
    let mut machine = OfferMachine::new();
    let offer = machine.on_session_end(fingerprint, Some(interrupted));
    assert_eq!(
        offer, None,
        "an unexpected end opened a continue offer (FR-032d)"
    );
    machine.on_foregrounded();
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::ExplicitResume,
        "an unexpected-end recovery auto-started audio (FR-032d, FR-031)"
    );
}

// T113 removed the legacy `session::resume_offer` surface together
// with the red test that pinned its SC-019 violation (code `2` is
// the canonical `Completed` end and must never be offered resume).
// The guards above pin the canonical `OfferMachine` / store API.
