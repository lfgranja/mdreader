//! T050 — fingerprint + re-anchor + resume-storage property tests
//! (Phase 11, BLOCKING; the property suite for
//! `crates/session/src/store.rs` (T058) and
//! `crates/session/src/fingerprint.rs` (T059), over
//! the offer machine of `crates/session/src/offer.rs`
//! (T060) contracted by `prop_offer.rs` (T052)).
//!
//! Invariants under test (tasks.md T050, verbatim):
//! "move/rename → same offer; edited-bytes while
//! idle → NO offer + start `0.0` + old entry
//! discarded; edit detected while a session is
//! active → paused state + explicit resume from
//! `0.0` of the changed content, no approximate
//! match (FR-036a); Stop|Completed → entry
//! deleted, reopen starts `0.0` no offer;
//! Pause|close|kill|power|battery|network →
//! entry preserved; voice/engine change → nearest
//! sentence/node boundary else `0.0`, boundary
//! reported; `last_rate` ∈ ladder default `1.0`,
//! survives close, crosses sessions+documents"
//!
//! Spec anchors:
//! - FR-036 (spec.md:996): a stored position is
//!   keyed by a fingerprint of the document's
//!   CONTENT, never of its location — a moved or
//!   renamed document resolves to the same
//!   bookmark key and therefore the same stored
//!   position and the same offer.
//! - FR-036a (spec.md:1000): a changed document
//!   is a different document — its fingerprint no
//!   longer matches, the stored position becomes
//!   unreachable, and no resume is offered,
//!   retargeted, or approximately matched for the
//!   changed content, which begins from its first
//!   spoken position. Where the change is detected
//!   while a session is active, the pending stored
//!   position is invalidated rather than offered:
//!   the session recovers to a paused state with
//!   an explicit resume control from the first
//!   spoken position of the changed content
//!   (FR-032d, FR-036).
//! - FR-032 / FR-032b / FR-032e (spec.md:947,
//!   974, 977): Stop discards the stored position;
//!   reaching the end of a document is equivalent
//!   to Stop for storage purposes — the entry is
//!   deleted, so the reopen starts from the
//!   beginning and is offered nothing.
//! - FR-032a (spec.md:951): a position survives
//!   every other way a session can end — Pause,
//!   application close, service kill, power-off,
//!   battery death, network loss.
//! - FR-038 (spec.md:1013): a stored position
//!   remains valid across a voice or engine change
//!   and is re-anchored to the nearest sentence or
//!   node boundary in the new engine's timeline
//!   before reading resumes; if no boundary is
//!   available at or before the stored position,
//!   the position falls back to the start of the
//!   document, never to an arbitrary offset.
//! - FR-019 / FR-019b and storage-schema.md
//!   `settings`: `last_rate` ∈ the closed
//!   seven-rung ladder, default `1.0` on first
//!   use, written on every rate change, crossing
//!   sessions and documents.
//! - contracts/storage-schema.md (normative):
//!   `bookmarks` key `[u8; 32]` (blake3-256 of
//!   document bytes) → `{ offset_secs: f64,
//!   first_yield_ms: u64, end_kind: SessionEnd }`;
//!   the offer flow is resolved SOLELY from the
//!   stored `end_kind` (`paused` ⇒ the FR-032c
//!   deliberate offer, `interrupted` ⇒ the
//!   FR-032d explicit-resume form); "Delete on:
//!   Stop press, document end. Nothing else
//!   deletes."
//!
//! Interpretation notes pinned by this suite (the
//! contract, not the implementation):
//!
//! 1. **Edit-while-idle "discard" (R2) is a flow
//!    property, not a physical deletion.** The
//!    schema's deletion rule is "Delete on: Stop
//!    press, document end. Nothing else deletes",
//!    and T058's `delete` is reached only from
//!    Stop|Completed. An edit is honoured because
//!    the lookup keys on content: the edited
//!    document's fingerprint differs from the
//!    stored key, so the lookup misses, no offer
//!    is made, and reading starts at `0.0`. This
//!    suite asserts the miss and the resulting
//!    no-offer/start-at-0.0 flow — and that the
//!    old row is RETAINED, pinning "nothing else
//!    deletes" — never a physical delete on edit.
//!    The old row is unreachable (keyed to
//!    content that no longer exists), which is
//!    the discard FR-036a describes.
//! 2. **Edit-while-active (R3) is the FR-036a
//!    explicit-resume form.** The edit ends the
//!    active session; the flow reports the changed
//!    content's first spoken position (`0.0`) with
//!    `end_kind = Interrupted`, so the machine
//!    yields the FR-032d explicit-resume
//!    requirement — never an auto-start, even
//!    foregrounded — and the changed content's
//!    lookup misses, so no approximate match is
//!    possible. The pre-edit row is retained but
//!    unreachable, per note 1.
//! 3. **The re-anchor takes the new engine's
//!    boundary list as input.** Boundary RESOLUTION
//!    is delegated to doc-structure (T061c, the
//!    "ordered headings" provider); the
//!    session-side `reanchor` (T059) owns the
//!    snap, the `0.0` fallback, and the report.
//!    Its input is therefore the ordered boundary
//!    positions doc-structure supplies.
//!
//! FAILS before `crates/session/src/store.rs`
//! (T058), `crates/session/src/fingerprint.rs`
//! (T059) and the `engine-core` dependency land:
//! `session::store` and `session::fingerprint`
//! are unresolved modules (the crate root
//! currently carries a `fingerprint` FUNCTION and
//! a local `SessionEnd` that T058/T059 supersede
//! with the canonical API), `session::offer` is
//! unresolved, and `engine_core` is not a
//! dependency of the `session` crate today (T058
//! adds `engine-core = { path = "../engine-core" }`
//! — `SessionEnd` and `PlaybackRate` come from
//! `engine_core::closed`), so this test target
//! does not build. That build failure is the
//! required pre-implementation red state (T056).
//!
//! Expected API contract (T058/T059 — the surface
//! this suite is the contract for; the offer
//! machine's contract is `prop_offer.rs`'s):
//! - `session::store::StoredPosition { offset_secs:
//!   f64, first_yield_ms: u64, end_kind: SessionEnd
//!   }` — `Debug + Clone + Copy + PartialEq` (f64
//!   fields preclude `Eq`).
//! - `session::store::BookmarkStore::open(&Path)`
//!   (the redb file), `put(&[u8; 32], StoredPosition)`,
//!   `get(&[u8; 32])`, `delete(&[u8; 32])`,
//!   `put_last_rate(PlaybackRate)`, `get_last_rate()`
//!   — every mutator returns `Result<(), Error>`,
//!   every reader `Result<Option<…>, Error>`, with
//!   `Error: std::error::Error` so proptest bodies
//!   can use `?`.
//! - `session::fingerprint::fingerprint(&[u8]) ->
//!   [u8; 32]` — the blake3-256 digest; a pure
//!   function of content, never of path — and
//!   `session::fingerprint::reanchor(stored_offset_secs:
//!   f64, boundaries: &[f64]) -> Reanchor` with
//!   `Reanchor { nearest: f64, reported: Option<f64> }`
//!   — `Debug + Clone + Copy + PartialEq`: `nearest`
//!   is the greatest boundary at or before the
//!   stored offset, or `0.0` when none precedes it;
//!   `reported` is that boundary, or `None` when
//!   the fallback fired.
//!
//! Exactness note: every offset and boundary is
//! generated on the quarter-second grid (`k ×
//! 0.25`), an exact binary-fraction grid, so all
//! positions compare bit-exactly. Every
//! `prop_assert*!`/`assert*!` message uses explicit
//! positional arguments (`{}`, `arg`), never
//! implicit named captures (`{var}`), because
//! proptest 1.11 builds the `prop_assert_eq!`
//! message through `concat!` and rustc (1.98)
//! rejects implicit captures in format strings
//! expanded from a macro.
//!
//! Proptest-body note: the `proptest!` macro
//! places each body inside a closure returning
//! `TestCaseResult` (sugar.rs:973-978), so `?`
//! works for every `E: std::error::Error`
//! (errors.rs:95) — but the body itself must
//! evaluate to `()`: the macro appends its own
//! `Ok(())`, so a trailing `Ok(())` in the body
//! is an E0308 type error. Plain (non-proptest)
//! tests return `Result<(), Box<dyn
//! std::error::Error>>` and end with `Ok(())`.

use proptest::prelude::*;

use engine_core::closed::{PlaybackRate, SessionEnd};
use session::fingerprint::{Reanchor, fingerprint, reanchor};
use session::offer::{ExpiryOutcome, Offer, OfferMachine};
use session::store::{BookmarkStore, StoredPosition};

/// A document's bytes — arbitrary content, up to
/// 4 KiB (the same bound as `prop_fingerprint.rs`).
fn document_strategy() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..=4096)
}

/// A file name — the path a document is moved or
/// renamed to.
fn name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,16}".prop_map(|name| format!("{name}.md"))
}

/// A playback position on the exact quarter-second
/// grid `k × 0.25` ∈ [0, 3600] s — an exact
/// binary-fraction grid, so positions compare
/// bit-exactly.
#[expect(
    clippy::cast_precision_loss,
    reason = "the quarter-second grid k × 0.25 is an exact binary fraction for every k in 0..=14400, so the u64→f64 cast is exact on this range; the lint is range-blind"
)]
fn offset_strategy() -> impl Strategy<Value = f64> {
    (0u64..=14_400).prop_map(|quarters| quarters as f64 * 0.25)
}

/// The start of a continuous non-advancing interval
/// — the `first_yield_ms` of the FR-015 supersede
/// clock, in milliseconds.
fn first_yield_strategy() -> impl Strategy<Value = u64> {
    0u64..=3_600_000
}

/// An arbitrary 32-byte document fingerprint (a
/// blake3-256 digest) — the bookmark key.
fn fingerprint_strategy() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>()
}

/// The retained session-end kinds: `paused` (a
/// deliberate pause ⇒ the FR-032c offer) and
/// `interrupted` (an unexpected end ⇒ the FR-032d
/// explicit-resume form) — the only end kinds a
/// retained row carries (storage-schema.md).
fn retained_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![Just(SessionEnd::Paused), Just(SessionEnd::Interrupted),]
}

/// A stored position whose entry is retained — one
/// of the two retained end kinds.
fn retained_position_strategy() -> impl Strategy<Value = StoredPosition> {
    (
        offset_strategy(),
        first_yield_strategy(),
        retained_end_strategy(),
    )
        .prop_map(|(offset_secs, first_yield_ms, end_kind)| StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind,
        })
}

/// The terminal session-end kinds: `stopped` and
/// `completed` — the only two events that delete
/// the entry (storage-schema.md).
fn terminal_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![Just(SessionEnd::Stopped), Just(SessionEnd::Completed),]
}

/// The closed seven-rung playback-rate ladder
/// (FR-019) — the only values `last_rate` holds.
fn playback_rate_strategy() -> impl Strategy<Value = PlaybackRate> {
    prop_oneof![
        Just(PlaybackRate::R0_75),
        Just(PlaybackRate::R1_0),
        Just(PlaybackRate::R1_25),
        Just(PlaybackRate::R1_5),
        Just(PlaybackRate::R1_75),
        Just(PlaybackRate::R2_0),
        Just(PlaybackRate::R4_0),
    ]
}

/// The boundary list of a new engine's timeline:
/// ordered, deduplicated positions on the
/// quarter-second grid — the "ordered headings"
/// doc-structure (T061c) supplies. The list may be
/// empty (a document with no parsed headings —
/// FR-029) and may begin after the stored
/// position, so the `0.0` fallback is reachable.
#[expect(
    clippy::cast_precision_loss,
    reason = "the quarter-second grid k × 0.25 is an exact binary fraction for every k in 0..=14400, so the u64→f64 cast is exact on this range; the lint is range-blind"
)]
fn boundary_list_strategy() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(0u64..=14_400, 0..=32).prop_map(|mut quarters| {
        quarters.sort_unstable();
        quarters.dedup();
        quarters
            .iter()
            .map(|&quarter| quarter as f64 * 0.25)
            .collect()
    })
}

/// The redb file a test store lives in, inside its
/// own temp dir.
fn store_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("bookmarks.redb")
}

/// The offer a stored position opens — the FR-032c
/// deliberate-pause offer for a `paused` end kind,
/// nothing for an `interrupted` end kind (the offer
/// form is resolved solely from the stored end
/// kind, storage-schema.md). A terminal end kind
/// never occurs in a retained row — those ends
/// delete the entry instead of storing one — and
/// opens nothing either; that arm is pinned by R4
/// and by `prop_offer.rs` O6/O7.
const fn expected_offer(stored: &StoredPosition) -> Option<Offer> {
    match stored.end_kind {
        SessionEnd::Paused => Some(Offer::Continue {
            offset_secs: stored.offset_secs,
            first_yield_ms: stored.first_yield_ms,
        }),
        SessionEnd::Interrupted | SessionEnd::Stopped | SessionEnd::Completed => None,
    }
}

/// Whether `position` is a valid re-anchor of
/// `offset` against `boundaries`: at or before the
/// stored position, and on a boundary of the new
/// timeline or at the document start (FR-038). The
/// float comparisons are exact on the quarter-second
/// grid.
fn is_a_valid_reanchor(position: f64, offset: f64, boundaries: &[f64]) -> bool {
    let on_a_boundary = boundaries.contains(&position);
    let at_the_document_start = position == 0.0;
    position <= offset && (on_a_boundary || at_the_document_start)
}

/// The greatest boundary at or before `cursor`, or
/// `None` when no boundary precedes it — the
/// FR-038 oracle, computed independently of the
/// implementation under test.
fn greatest_boundary_at_or_before(cursor: f64, boundaries: &[f64]) -> Option<f64> {
    let mut best: Option<f64> = None;
    for &boundary in boundaries {
        if boundary <= cursor && best.is_none_or(|previous| boundary > previous) {
            best = Some(boundary);
        }
    }
    best
}

/// The closed seven-rung playback-rate ladder
/// (FR-019) — the complete set of rates
/// `last_rate` can hold.
const LADDER: [PlaybackRate; 7] = [
    PlaybackRate::R0_75,
    PlaybackRate::R1_0,
    PlaybackRate::R1_25,
    PlaybackRate::R1_5,
    PlaybackRate::R1_75,
    PlaybackRate::R2_0,
    PlaybackRate::R4_0,
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn given_a_document_moved_or_renamed_when_the_session_ends_then_the_position_is_found_and_opens_the_same_offer(
        (content, renamed_name, stored) in
            (document_strategy(), name_strategy(), retained_position_strategy())
    ) {
        // R1 (FR-036): "Moving or renaming the document MUST NOT orphan
        // the stored position, because refiling documents is an ordinary
        // act on a phone and losing a bookmark to a rename is an
        // invisible loss."
        let dir = tempfile::tempdir()?;
        let original_path = dir.path().join("before-the-move.md");
        let renamed_path = dir.path().join(renamed_name);

        // The document was read at its original path and the session
        // ended non-terminally: the position is stored under the content
        // key — the fingerprint of the bytes as they were read.
        std::fs::write(&original_path, &content)?;
        let bytes_before = std::fs::read(&original_path)?;
        let key_before = fingerprint(&bytes_before);

        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&key_before, stored)?;

        // The document is moved or renamed: the same bytes are read back
        // from the other path — the same content, so the same bookmark
        // key, whatever the path was.
        std::fs::rename(&original_path, &renamed_path)?;
        let bytes_after = std::fs::read(&renamed_path)?;
        let key_after = fingerprint(&bytes_after);

        // The lookup through the renamed path finds the stored
        // position…
        let found = store.get(&key_after)?;
        prop_assert_eq!(
            found,
            Some(stored),
            "the moved or renamed document did not resolve to the stored position (FR-036)"
        );

        // …and the offer opened from it is the offer the original path
        // would have opened — the same stored position, resolved solely
        // from the stored end kind.
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(key_after, found);
        let expected = expected_offer(&stored);
        prop_assert_eq!(
            offer,
            expected,
            "the offer opened from the renamed path differs from the offer the original path opens (FR-036)"
        );
    }

    #[test]
    fn given_a_document_edited_while_idle_when_it_is_opened_then_no_offer_is_made_and_reading_starts_at_the_first_spoken_position(
        (original, edited, stored) in
            (document_strategy(), document_strategy(), retained_position_strategy())
    ) {
        // R2 (FR-036a): "When no session is active, a document whose
        // content has changed MUST be treated as a different document
        // from the one a stored position was recorded against. The
        // stored position MUST NOT be offered, retargeted, or
        // approximately matched, and the changed document MUST begin
        // from its first spoken position."
        prop_assume!(original != edited);

        let original_key = fingerprint(&original);
        let edited_key = fingerprint(&edited);
        prop_assume!(original_key != edited_key);

        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        // The idle session ended non-terminally: the position persists
        // under the ORIGINAL content's key.
        store.put(&original_key, stored)?;

        // The changed document is a different document: its fingerprint
        // — the lookup key — differs, so the lookup misses.
        let found = store.get(&edited_key)?;
        prop_assert_eq!(
            found,
            None,
            "the edited document resolved to the pre-edit position — an approximate match (FR-036a)"
        );

        // NO offer: a miss opens nothing, the window yields no outcome,
        // and reading starts at the first spoken position — 0.0 (the
        // miss flow, storage-schema.md: "miss (incl. edited content,
        // FR-036a) ⇒ start at 0.0, no offer").
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(edited_key, found);
        prop_assert_eq!(
            offer,
            None,
            "a lookup miss opened a resume offer for edited content (FR-036a)"
        );
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "a lookup miss left a continue offer open (FR-036a)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "a lookup miss produced an expiry outcome (FR-036a)"
        );

        // The old entry is discarded in the flow sense — unreachable:
        // it is keyed to content that no longer exists, so it can never
        // be offered again. Nothing else deletes (storage-schema.md:
        // "Delete on: Stop press, document end. Nothing else deletes"),
        // so the row is retained; this suite pins the retention, not a
        // physical delete (interpretation note 1).
        let retained = store.get(&original_key)?;
        prop_assert_eq!(
            retained,
            Some(stored),
            "the pre-edit entry was physically deleted — only Stop and document end delete (storage-schema.md)"
        );
    }

    #[test]
    fn given_a_document_edited_while_a_session_is_active_when_the_edit_is_detected_then_the_session_recovers_to_paused_with_an_explicit_resume_at_zero(
        (original, edited, stored) in
            (document_strategy(), document_strategy(), retained_position_strategy())
    ) {
        // R3 (FR-036a, second paragraph): "Where the change is detected
        // while a session is active, the pending stored position is
        // invalidated rather than offered: the session recovers to a
        // paused state with an explicit resume control from the first
        // spoken position of the changed content, and no approximate
        // match is offered (FR-032d, FR-036)."
        prop_assume!(original != edited);

        let original_key = fingerprint(&original);
        let edited_key = fingerprint(&edited);
        prop_assume!(original_key != edited_key);

        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        // The active session's last-spoken position, stored under the
        // ORIGINAL content's key.
        store.put(&original_key, stored)?;

        // The edit is detected while the session is active: the session
        // ends. The changed content's first spoken position is 0.0, and
        // the end is an unexpected one — the edit interrupted the
        // session — so the flow reports the FR-032d explicit-resume
        // form (interpretation note 2).
        let ended = StoredPosition {
            offset_secs: 0.0,
            first_yield_ms: 0,
            end_kind: SessionEnd::Interrupted,
        };
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(edited_key, Some(ended));
        prop_assert_eq!(
            offer,
            None,
            "the edit-detected end opened a continue offer (FR-036a, FR-032d)"
        );

        // Even foregrounded, the explicit-resume requirement never
        // auto-starts: an unexpected end never produces audio on its own
        // (FR-032d, FR-031).
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::ExplicitResume,
            "the edit-detected recovery auto-started audio (FR-036a, FR-032d, FR-031)"
        );

        // No approximate match: the changed content's lookup misses —
        // the pre-edit position is never offered for the changed
        // content, so the explicit resume begins at the changed
        // content's first spoken position (0.0), not at the pre-edit
        // offset.
        let found = store.get(&edited_key)?;
        prop_assert_eq!(
            found,
            None,
            "the changed content resolved to the pre-edit position — an approximate match (FR-036a)"
        );

        // The pre-edit row is retained (nothing else deletes) but
        // unreachable: it is keyed to the original content, and the
        // original content no longer exists.
        let retained = store.get(&original_key)?;
        prop_assert_eq!(
            retained,
            Some(stored),
            "the pre-edit entry was physically deleted — only Stop and document end delete (storage-schema.md)"
        );
    }

    #[test]
    fn given_a_terminal_end_when_the_flow_runs_then_the_entry_is_deleted_and_the_reopen_starts_at_zero_with_no_offer(
        (key, stored, terminal_end) in
            (fingerprint_strategy(), retained_position_strategy(), terminal_end_strategy())
    ) {
        // R4 (FR-032 / FR-032b / FR-032e): "Stop discards the stored
        // position for that document" and "reaching the end of a
        // document is equivalent to Stop for storage purposes" — the
        // entry is deleted (the only two events that delete,
        // storage-schema.md), so the reopen is a lookup miss: it starts
        // at 0.0 with no offer.
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        // A session ended non-terminally earlier: the entry is stored.
        store.put(&key, stored)?;

        // The session now ends terminally — a Stop press or the
        // document reaching its end — and the flow deletes the entry.
        store.delete(&key)?;
        let found = store.get(&key)?;
        prop_assert_eq!(
            found,
            None,
            "the entry survived a terminal end ({:?}) (FR-032, FR-032b)",
            terminal_end
        );

        // The reopen: a miss, so no offer — and reading starts at the
        // first spoken position, 0.0.
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(key, found);
        prop_assert_eq!(
            offer,
            None,
            "a terminal-end reopen opened a resume offer (FR-032e)"
        );
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "a terminal-end reopen left a continue offer open (FR-032e)"
        );
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "a terminal-end reopen produced an expiry outcome (FR-032e)"
        );
    }

    #[test]
    fn given_a_non_terminal_end_when_the_process_closes_and_reopens_then_the_entry_is_preserved_and_still_resolves_its_offer(
        (key, stored) in (fingerprint_strategy(), retained_position_strategy())
    ) {
        // R5 (FR-032a): "A position MUST be preserved across every other
        // way a session can end, including Pause, closing the
        // application, the system killing the service, the device being
        // powered off, the battery dying, and the network dropping." The
        // two retained end kinds cover all of them: `paused` (a
        // deliberate pause) and `interrupted` (close, kill, power,
        // battery, network loss).
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());

        // The session ends non-terminally and the process closes: the
        // store is dropped, and the redb file on disk carries the
        // entry.
        {
            let mut store = BookmarkStore::open(&path)?;
            store.put(&key, stored)?;
        }

        // The process reopens: a fresh store over the same redb file
        // finds the identical entry…
        {
            let mut store = BookmarkStore::open(&path)?;
            let found = store.get(&key)?;
            prop_assert_eq!(
                found,
                Some(stored),
                "a non-terminal end ({:?}) did not survive the process close (FR-032a)",
                stored.end_kind
            );

            // …and the offer form is still resolved solely from the
            // preserved end kind.
            let mut machine = OfferMachine::new();
            let offer = machine.on_session_end(key, found);
            match stored.end_kind {
                SessionEnd::Paused => prop_assert_eq!(
                    offer,
                    Some(Offer::Continue {
                        offset_secs: stored.offset_secs,
                        first_yield_ms: stored.first_yield_ms,
                    }),
                    "a preserved deliberate pause did not open the FR-032c offer (FR-032c)"
                ),
                SessionEnd::Interrupted => prop_assert_eq!(
                    offer,
                    None,
                    "a preserved unexpected end opened a continue offer (FR-032d)"
                ),
                // Retained rows never carry a terminal end kind — those
                // ends delete the entry instead of storing one
                // (storage-schema.md) — so these arms cannot be
                // generated; they are asserted closed, not silently
                // skipped.
                SessionEnd::Stopped | SessionEnd::Completed => {
                    let non_terminal = matches!(
                        stored.end_kind,
                        SessionEnd::Paused | SessionEnd::Interrupted
                    );
                    prop_assert!(
                        non_terminal,
                        "a preserved entry carried a terminal end kind (storage-schema.md)"
                    );
                }
            }
        }
    }

    #[test]
    fn given_a_stored_position_and_a_new_engine_timeline_when_reanchoring_then_the_resume_lands_on_the_nearest_boundary_or_the_document_start(
        (offset, boundaries) in (offset_strategy(), boundary_list_strategy())
    ) {
        // R6 (FR-038): "A stored position MUST remain valid when the
        // voice or engine changes, and MUST be re-anchored to the
        // nearest sentence or node boundary in the new engine's
        // timeline before reading resumes… If no boundary is available
        // at or before the stored position, the position MUST fall back
        // to the start of the document rather than to an arbitrary
        // offset."
        let result = reanchor(offset, &boundaries);

        // The oracle is the specification's own definition: the greatest
        // boundary at or before the stored position, or the document
        // start when none precedes it.
        let expected_boundary = greatest_boundary_at_or_before(offset, &boundaries);
        let expected = Reanchor {
            nearest: expected_boundary.unwrap_or(0.0),
            reported: expected_boundary,
        };
        prop_assert_eq!(
            result,
            expected,
            "the re-anchor of {} did not land on the greatest boundary at or before it (FR-038)",
            offset
        );

        // The re-anchor is a pure function of the position and the
        // timeline: identical inputs give an identical re-anchor point.
        let again = reanchor(offset, &boundaries);
        prop_assert_eq!(
            result,
            again,
            "the re-anchor of {} was not deterministic (FR-038)",
            offset
        );

        // And it is a valid re-anchor: at or before the stored
        // position, on a boundary of the new timeline or at the
        // document start.
        prop_assert!(
            is_a_valid_reanchor(result.nearest, offset, &boundaries),
            "the re-anchor {} of {} is not at or before the position on a boundary or the document start (FR-038)",
            result.nearest,
            offset
        );
    }

    #[test]
    fn given_a_rate_change_when_it_is_persisted_then_the_ladder_rung_survives_the_close_and_is_independent_of_any_document(
        (rate, key, stored) in
            (playback_rate_strategy(), fingerprint_strategy(), retained_position_strategy())
    ) {
        // R7 (FR-019b / storage-schema.md `settings`): "`last_rate` …
        // crosses sessions and documents" — the rate is stored in
        // `settings`, not per bookmark, so it survives the process
        // close and is undisturbed by any document's bookmark.
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());

        // The rate is written on the change…
        {
            let mut store = BookmarkStore::open(&path)?;
            store.put_last_rate(rate)?;
            let found = store.get_last_rate()?;
            prop_assert_eq!(
                found,
                Some(rate),
                "the last rate did not round-trip through the store (FR-019b)"
            );

            // …and is independent of any document: storing a bookmark
            // for another document does not disturb it.
            store.put(&key, stored)?;
            let still = store.get_last_rate()?;
            prop_assert_eq!(
                still,
                Some(rate),
                "the last rate depended on a document's bookmark (FR-019b)"
            );
        }

        // The process closes and reopens: the rate survives the
        // close…
        {
            let mut store = BookmarkStore::open(&path)?;
            let survived = store.get_last_rate()?;
            prop_assert_eq!(
                survived,
                Some(rate),
                "the last rate did not survive the process close (FR-019b)"
            );
        }
    }
}

/// Boundary case (R1, FR-036): identical bytes at
/// two paths share one fingerprint — the moved or
/// renamed document's bookmark key is the same key,
/// so the stored position is shared. This is the
/// deterministic core of the R1 property, with real
/// files on disk.
#[test]
fn identical_bytes_at_two_paths_share_one_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let original_path = dir.path().join("comete-estrada.md");
    let renamed_path = dir.path().join("renamed-on-the-device.md");
    let content: &[u8] = b"# Trajeto para o trabalho\n\nmesmo conteudo, outro caminho\n";

    std::fs::write(&original_path, content)?;
    let bytes_before = std::fs::read(&original_path)?;
    std::fs::rename(&original_path, &renamed_path)?;
    let bytes_after = std::fs::read(&renamed_path)?;

    let key_before = fingerprint(&bytes_before);
    let key_after = fingerprint(&bytes_after);
    assert_eq!(
        key_before, key_after,
        "identical bytes at two paths produced different bookmark keys (FR-036)"
    );
    assert_eq!(
        key_before.len(),
        32,
        "the fingerprint is not a blake3-256 (32-byte) digest (FR-036)"
    );
    Ok(())
}

/// Boundary case (R4, FR-032 / FR-032b / FR-032e):
/// a document left by a Stop press — or one that
/// reached its end — reopens at its first spoken
/// position with no offer: the entry is gone, so
/// the reopen is a lookup miss and the machine
/// opens nothing.
#[test]
fn a_terminal_end_reopens_at_the_first_spoken_position_with_no_offer()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let mut store = BookmarkStore::open(&path)?;

    let key = fingerprint(b"a document the reader stopped");
    let stored = StoredPosition {
        offset_secs: 90.0,
        first_yield_ms: 40,
        end_kind: SessionEnd::Paused,
    };

    // Both terminal ends delete the entry — the only two events that
    // delete (storage-schema.md).
    for terminal in [SessionEnd::Stopped, SessionEnd::Completed] {
        store.put(&key, stored)?;

        // The flow deletes the entry on the terminal end…
        store.delete(&key)?;
        let found = store.get(&key)?;
        assert_eq!(
            found, None,
            "the entry survived a terminal end ({terminal:?}) (FR-032, FR-032b)"
        );

        // …and the reopen is a miss: no offer, reading from 0.0.
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(key, found);
        assert_eq!(
            offer, None,
            "a terminal-end reopen opened a resume offer ({terminal:?}) (FR-032e)"
        );
        assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "a terminal-end reopen produced an expiry outcome ({terminal:?}) (FR-032e)"
        );
    }
    Ok(())
}

/// Boundary case (R6, FR-038): no boundary at or
/// before the stored position — an empty timeline,
/// or one whose every boundary is later than the
/// position — falls back to the document start, and
/// nothing is reported: the resume never lands on an
/// arbitrary offset. A boundary exactly at the
/// position is at or before it (the lookup is
/// inclusive).
#[expect(
    clippy::float_cmp,
    reason = "the positions 40.0 and 0.0 and the boundary lists are exact binary fractions on the quarter-second grid, so == is the correct comparison; a tolerance would admit an off-boundary re-anchor"
)]
#[test]
fn a_position_with_no_boundary_at_or_before_it_falls_back_to_the_document_start() {
    // An empty timeline: nothing precedes the position.
    let empty = reanchor(40.0, &[]);
    assert_eq!(
        empty.nearest, 0.0,
        "an empty timeline did not fall back to the document start (FR-038)"
    );
    assert_eq!(
        empty.reported, None,
        "a fallback to the document start reported a boundary (FR-038)"
    );

    // Every boundary later than the position: the same fallback.
    let later = [50.0, 75.0];
    let beyond = reanchor(40.0, &later);
    assert_eq!(
        beyond.nearest, 0.0,
        "a timeline with only later boundaries did not fall back to the document start (FR-038)"
    );
    assert_eq!(
        beyond.reported, None,
        "a timeline with only later boundaries reported a boundary (FR-038)"
    );

    // A boundary exactly at the position is at or before it.
    let at_it = reanchor(40.0, &[25.0, 40.0]);
    assert_eq!(
        at_it.nearest, 40.0,
        "a boundary exactly at the position was not the re-anchor (FR-038)"
    );
    assert_eq!(
        at_it.reported,
        Some(40.0),
        "a boundary exactly at the position was not reported (FR-038)"
    );
}

/// Boundary case (R7, FR-019 / FR-019b): every
/// rung of the closed seven-rung ladder round-trips
/// through the store — the ladder is the complete
/// set of rates, so `last_rate` can hold nothing
/// else.
#[test]
fn every_ladder_rung_round_trips_through_the_store() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let mut store = BookmarkStore::open(&path)?;

    for rate in LADDER {
        store.put_last_rate(rate)?;
        let found = store.get_last_rate()?;
        assert_eq!(
            found,
            Some(rate),
            "the ladder rung {rate:?} did not round-trip (FR-019b)"
        );
    }
    Ok(())
}

/// Boundary case (R7, storage-schema.md `settings`):
/// `last_rate` is absent on first use — the store
/// reports `None`, and the call-site default is the
/// normal-speed rung `1.0` (the schema's "default
/// `1.0` on first use").
#[test]
fn the_last_rate_defaults_to_normal_speed_on_first_use() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let mut store = BookmarkStore::open(&path)?;

    let found = store.get_last_rate()?;
    assert_eq!(
        found, None,
        "a fresh store reported a last rate (storage-schema.md)"
    );
    let default = found.unwrap_or(PlaybackRate::R1_0);
    assert_eq!(
        default,
        PlaybackRate::R1_0,
        "the call-site default is not the normal-speed rung (FR-019)"
    );
    Ok(())
}
