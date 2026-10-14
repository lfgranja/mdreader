//! T033 — the bookmark lifecycle property
//! (`prop_bookmark_lifecycle`), Phase US3 BLOCKING
//! (tasks.md:129): the property suite for the
//! position authority's storage, fingerprint and
//! resume-offer flow —
//! `crates/session/src/store.rs` (T058),
//! `crates/session/src/fingerprint.rs` (T059),
//! `crates/session/src/offer.rs` (T060) and the
//! `crates/session/src/cli.rs` verbs (T063) — over
//! the closed session-end vocabulary of
//! `engine_core::closed::SessionEnd` (FR-044).
//!
//! Invariants under test (tasks.md T033, verbatim):
//! "`put` atomic single-row; `Stopped|Completed` ⇒
//! deleted; `Paused`⇒deliberate 3 s offer
//! silence-continues; `Interrupted`⇒explicit resume
//! never auto-starts; content-change hash ⇒ miss ⇒
//! `0.0` no offer"
//!
//! Spec anchors:
//! - FR-004 (storage): a stored write is atomic and
//!   crash-safe — the entry is ONE row under ONE
//!   content key: never a second row, never a
//!   partial write, and never a disturbance of
//!   another document's row.
//! - FR-036 / FR-036a (fingerprint): the key is a
//!   blake3-256 of the document's CONTENT — never of
//!   its path — so identical bytes anywhere share one
//!   row, and any changed byte is a different
//!   document whose lookup misses and which begins
//!   from its first spoken position (`0.0`).
//! - FR-032 / FR-032b / FR-032e (terminal ends): a
//!   Stop press and a document's end DELETE the entry
//!   — the only two events that delete
//!   (storage-schema.md) — so the reopen is a miss
//!   that starts at `0.0` and is offered nothing.
//! - FR-032c (deliberate pause): the reopen shows the
//!   visible 3-second continue offer — continue from
//!   the stored position, the DEFAULT, with the
//!   start-over alternative; with no choice made
//!   inside the window and the application in the
//!   foreground, reading begins from the stored
//!   position (silence continues).
//! - FR-032d / FR-031 (unexpected end): the automatic
//!   start never applies — an interrupted end
//!   recovers to a paused state with an explicit
//!   resume control and never produces audio on its
//!   own.
//! - FR-032a: every other way a session can end
//!   (pause, close, service kill, power-off, battery
//!   death, network loss) preserves the entry.
//! - contracts/storage-schema.md (normative): key
//!   `[u8; 32]` → `{ offset_secs: f64,
//!   first_yield_ms: u64, end_kind: SessionEnd }`;
//!   the offer flow is resolved SOLELY from the stored
//!   `end_kind`; "Delete on: Stop press, document end.
//!   Nothing else deletes."
//!
//! The division of responsibility this suite pins
//! (read off the current code, not assumed):
//!
//! 1. `BookmarkStore::put` is an ATOMIC SINGLE-ROW
//!    write: one key, one row, one transaction, with
//!    no other document's row disturbed and a
//!    replacement landing in the same single row. The
//!    store is a ROW store: it does not validate the
//!    end kind, so a terminal-kind row written
//!    straight through it round-trips exactly (D2 pins
//!    that, so the boundary is unambiguous).
//! 2. The terminal-ends rule is enforced ABOVE the
//!    store, and both halves are pinned here:
//!    - the `put` verb REFUSES a terminal
//!      `--end-kind` with `CliError::TerminalEndKind`
//!      (D1 — cli.rs: "the terminal ends delete the
//!      entry instead of storing one … rejected as
//!      `put` inputs with a named error, never
//!      stored");
//!    - the flow DELETES on a terminal end —
//!      `BookmarkStore::delete` is the caller's job
//!      (the store exposes no terminal-aware
//!      mutator), and the reopen after that delete is
//!      the miss FR-032e describes (P2).
//! 3. The silence rule is the FR-032c default made
//!    observable: the open offer reads back unchanged
//!    until a choice is made or the window expires, and
//!    an expiry with the application in the foreground
//!    begins reading from the stored position —
//!    silence continues (P3).
//!
//! Relationship to the sibling suites:
//! `prop_resume.rs` (T050) pins the move/rename/edit
//! properties and `prop_offer.rs` (T052) pins the
//! windowed offer machine; this suite pins the
//! LIFECYCLE the storage layer owns — what a `put`
//! writes, what a terminal end removes, and which
//! offer form each retained end kind resolves.
//! `test_offer3s.rs` (T034) is the windowed
//! state-machine companion: the dialog form, the
//! lock-screen resume control, and the Stop-before-
//! reopen clear.
//!
//! Exactness note: every offset is generated on the
//! quarter-second grid (`k × 0.25`), an exact
//! binary-fraction grid, so all positions compare
//! bit-exactly. Every `prop_assert*!`/`assert*!`
//! message uses explicit positional arguments (`{}`,
//! arg), never implicit named captures (`{var}`),
//! because proptest 1.11 builds the `prop_assert_eq!`
//! message through `concat!` and rustc rejects
//! implicit captures in format strings expanded from a
//! macro.
//!
//! Proptest-body note: the `proptest!` macro places
//! each body inside a closure returning
//! `TestCaseResult` (sugar.rs), so `?` works for
//! every `E: std::error::Error` — but the body itself
//! must evaluate to `()`: the macro appends its own
//! `Ok(())`, so a trailing `Ok(())` in the body is an
//! E0308 type error. Plain (non-proptest) tests
//! return `Result<(), Box<dyn std::error::Error>>`
//! and end with `Ok(())`.

use std::error::Error;
use std::path::{Path, PathBuf};

use engine_core::closed::SessionEnd;
use proptest::prelude::*;
use session::cli::{CliError, run as session_cli};
use session::fingerprint::fingerprint;
use session::offer::{ExpiryOutcome, Offer, OfferMachine};
use session::store::{BookmarkStore, StoredPosition};

/// A document's bytes — arbitrary content, up to
/// 4 KiB (the same bound as `prop_resume.rs`).
fn document_strategy() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..=4096)
}

/// A document together with one EDITED variant of it:
/// exactly one byte appended, so the two byte
/// sequences always differ and the property never has
/// to assume its way to a different fingerprint
/// (FR-036a).
fn edited_document_strategy() -> impl Strategy<Value = (Vec<u8>, Vec<u8>)> {
    document_strategy().prop_flat_map(|document| {
        let original = document.clone();
        (0u8..=255).prop_map(move |appended| {
            let mut edited = original.clone();
            edited.push(appended);
            (original.clone(), edited)
        })
    })
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
/// stored row can carry through the `put` verb
/// (cli.rs rejects the other two).
fn retained_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![Just(SessionEnd::Paused), Just(SessionEnd::Interrupted),]
}

/// The terminal session-end kinds: `stopped` and
/// `completed` — the only two events that delete the
/// entry (storage-schema.md).
fn terminal_end_strategy() -> impl Strategy<Value = SessionEnd> {
    prop_oneof![Just(SessionEnd::Stopped), Just(SessionEnd::Completed),]
}

/// A stored position under one of the retained end
/// kinds.
fn retained_position_strategy(end_kind: SessionEnd) -> impl Strategy<Value = StoredPosition> {
    (offset_strategy(), first_yield_strategy()).prop_map(move |(offset_secs, first_yield_ms)| {
        StoredPosition {
            offset_secs,
            first_yield_ms,
            end_kind,
        }
    })
}

/// A stored position whose end kind is the
/// deliberate pause.
fn paused_position_strategy() -> impl Strategy<Value = StoredPosition> {
    retained_position_strategy(SessionEnd::Paused)
}

/// A stored position whose end kind is the
/// unexpected end.
fn interrupted_position_strategy() -> impl Strategy<Value = StoredPosition> {
    retained_position_strategy(SessionEnd::Interrupted)
}

/// A stored position under an arbitrary retained end
/// kind — the two kinds a `put` verb accepts.
fn retained_position_strategy_any() -> impl Strategy<Value = StoredPosition> {
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

/// The redb file a test store lives in, inside its
/// own temp dir.
fn store_path(dir: &Path) -> PathBuf {
    dir.join("bookmarks.redb")
}

/// Runs the CLI in-process over injected writers —
/// the same `run` entry the binary drives (Principle
/// II), with `--db` bound to `db`. The outcome travels
/// WITH both streams so a failure half can be asserted
/// (nothing on stdout, the diagnostic on stderr)
/// without pretending the call succeeded.
fn cli_run(db: &Path, args: &[&str]) -> (Result<(), CliError>, Vec<u8>, Vec<u8>) {
    let invocation = std::iter::once(String::from("--db"))
        .chain(std::iter::once(db.to_string_lossy().into_owned()))
        .chain(args.iter().map(|arg| String::from(*arg)));
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let outcome = session_cli(invocation, &mut stdout, &mut stderr);
    (outcome, stdout, stderr)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn given_a_position_put_under_a_documents_content_key_when_it_is_read_back_then_the_exact_single_row_returns(
        (content, stored, other_key, other_stored) in
            (document_strategy(), paused_position_strategy(), fingerprint_strategy(), interrupted_position_strategy())
    ) {
        // P1 — "`put` atomic single-row" (FR-004): the write is one
        // transaction over one key, so the row read back is the row
        // written, another document's row is untouched, and a
        // replacement lands in the same single row.
        let key = fingerprint(&content);
        prop_assume!(key != other_key);

        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;

        store.put(&key, stored)?;
        // A second document's row is written in the same store…
        store.put(&other_key, other_stored)?;

        // …and the first row reads back exactly — the atomic
        // single-row write is not a partial or a shared write.
        let found = store.get(&key)?;
        prop_assert_eq!(
            found,
            Some(stored),
            "the row read back is not the row put (FR-004)"
        );

        // A replacement writes the SAME single row: the end kind lands
        // (a deliberate pause superseded by an interruption is the
        // lifecycle's own transition), and the other document's row is
        // undisturbed.
        let replaced = StoredPosition {
            end_kind: other_stored.end_kind,
            ..stored
        };
        store.put(&key, replaced)?;
        prop_assert_eq!(
            store.get(&key)?,
            Some(replaced),
            "a repeated put under one key did not replace the single row (FR-004)"
        );
        prop_assert_eq!(
            store.get(&other_key)?,
            Some(other_stored),
            "a put under one key disturbed another document's row (FR-004)"
        );
    }

    #[test]
    fn given_a_terminal_end_when_the_flow_deletes_the_entry_then_the_reopen_is_a_miss_that_starts_at_zero_with_no_offer(
        (key, retained, terminal) in
            (fingerprint_strategy(), retained_position_strategy_any(), terminal_end_strategy())
    ) {
        // P2 — "`Stopped|Completed` ⇒ deleted" (FR-032 / FR-032b /
        // FR-032e): the terminal end's operation is the caller's
        // delete (the store exposes no terminal-aware mutator), and
        // the reopen after it is a miss — no offer, start at 0.0.
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());

        // A session ended non-terminally earlier: the entry is
        // stored…
        {
            let mut store = BookmarkStore::open(&path)?;
            store.put(&key, retained)?;
        }

        // …then the session ends terminally — a Stop press or the
        // document reaching its end — and the flow deletes.
        {
            let mut store = BookmarkStore::open(&path)?;
            store.delete(&key)?;
            let found = store.get(&key)?;
            prop_assert_eq!(
                found,
                None,
                "the entry survived the terminal end {:?} (FR-032, FR-032b)",
                terminal
            );
        }

        // The delete is durable, not per-handle: a reopened store
        // over the same file still misses.
        let mut reopened = BookmarkStore::open(&path)?;
        let found = reopened.get(&key)?;
        prop_assert_eq!(
            found,
            None,
            "the terminal-end delete did not survive the reopen (FR-032, FR-032b)"
        );

        // The reopen is a miss: no offer, no open continue offer, and
        // no expiry outcome — reading starts at the first spoken
        // position, 0.0.
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

        // A terminal-kind row — should one ever be present, written
        // straight through the row store — opens nothing either: the
        // terminal kinds are never offered (FR-032 / FR-032e).
        let terminal_row = StoredPosition {
            offset_secs: retained.offset_secs,
            first_yield_ms: retained.first_yield_ms,
            end_kind: terminal,
        };
        let mut terminal_machine = OfferMachine::new();
        prop_assert_eq!(
            terminal_machine.on_session_end(key, Some(terminal_row)),
            None,
            "a terminal end kind opened a resume offer (FR-032e)"
        );
        prop_assert_eq!(
            terminal_machine.expire(),
            ExpiryOutcome::NoOffer,
            "a terminal end kind produced an expiry outcome (FR-032e)"
        );
    }

    #[test]
    fn given_a_deliberate_pause_when_the_document_reopens_then_the_offer_is_the_deliberate_form_and_silence_continues_from_the_stored_position(
        (content, paused) in (document_strategy(), paused_position_strategy())
    ) {
        // P3 — "`Paused`⇒deliberate 3 s offer silence-continues"
        // (FR-032c): the reopen opens the continue offer — continue
        // from the stored position, the default — and with no choice
        // made inside the window the foregrounded expiry begins
        // reading from the stored position.
        let key = fingerprint(&content);
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&key, paused)?;
        let found = store.get(&key)?;
        prop_assert_eq!(
            found,
            Some(paused),
            "the deliberate pause did not round-trip through the store (FR-032a)"
        );

        // The deliberate offer form, resolved solely from the stored
        // end kind: continue from the stored position, carrying the
        // FR-015 interval start.
        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(key, found);
        prop_assert_eq!(
            offer,
            Some(Offer::Continue {
                offset_secs: paused.offset_secs,
                first_yield_ms: paused.first_yield_ms,
            }),
            "a deliberate pause did not open the FR-032c continue offer (FR-032c)"
        );

        // The offer stays open, unchanged, until a choice or the
        // window's expiry.
        prop_assert_eq!(
            machine.continue_offer(),
            Some(Offer::Continue {
                offset_secs: paused.offset_secs,
                first_yield_ms: paused.first_yield_ms,
            }),
            "the open continue offer changed while the window was open (FR-032c)"
        );

        // Silence — no choice made inside the window — continues from
        // the stored position when the application is in the
        // foreground at expiry.
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::AutoStart { offset_secs: paused.offset_secs },
            "silence did not continue from the stored position (FR-032c)"
        );

        // The start-over alternative is available for the same offer:
        // reading begins from exactly 0.0 and consumes the window.
        let mut start_over_machine = OfferMachine::new();
        let _opened = start_over_machine.on_session_end(key, found);
        prop_assert_eq!(
            start_over_machine.start_over(),
            ExpiryOutcome::AutoStart { offset_secs: 0.0 },
            "the start-over choice did not begin at the first spoken position (FR-032c)"
        );
        prop_assert_eq!(
            start_over_machine.continue_offer(),
            None,
            "the start-over choice left the continue offer open (FR-032c)"
        );
        prop_assert_eq!(
            start_over_machine.expire(),
            ExpiryOutcome::NoOffer,
            "an expiry followed the start-over choice (FR-032f)"
        );
    }

    #[test]
    fn given_an_unexpected_end_when_the_document_reopens_then_an_explicit_resume_is_required_and_never_auto_starts(
        (content, interrupted) in (document_strategy(), interrupted_position_strategy())
    ) {
        // P4 — "`Interrupted`⇒explicit resume never auto-starts"
        // (FR-032d / FR-031): an unexpected end opens no offer, leaves
        // the explicit-resume requirement, and never produces audio
        // on its own — not even foregrounded.
        let key = fingerprint(&content);
        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&key, interrupted)?;
        let found = store.get(&key)?;
        prop_assert_eq!(
            found,
            Some(interrupted),
            "the interrupted entry did not survive the store round-trip (FR-032a)"
        );

        let mut machine = OfferMachine::new();
        let offer = machine.on_session_end(key, found);
        prop_assert_eq!(
            offer,
            None,
            "an unexpected end opened a continue offer (FR-032d)"
        );
        prop_assert_eq!(
            machine.continue_offer(),
            None,
            "an unexpected end left a continue offer open (FR-032d)"
        );

        // The reader is present — the application is foregrounded —
        // and the explicit-resume requirement still stands: no audio
        // begins on its own.
        machine.on_foregrounded();
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::ExplicitResume,
            "an unexpected end auto-started audio (FR-032d, FR-031)"
        );

        // The window ended with it: a second expiry yields nothing.
        prop_assert_eq!(
            machine.expire(),
            ExpiryOutcome::NoOffer,
            "an expiry repeated after the explicit resume (FR-032f)"
        );

        // The explicit resume is the only route in: the start-over
        // choice still begins at 0.0, but the continue-from-stored
        // automatic start is never reachable from an unexpected end.
        let mut explicit = OfferMachine::new();
        let _opened = explicit.on_session_end(key, found);
        explicit.on_foregrounded();
        let outcome = explicit.expire();
        let auto_started = matches!(outcome, ExpiryOutcome::AutoStart { .. });
        prop_assert!(
            !auto_started,
            "an unexpected end reached the automatic start (FR-032d, FR-031)"
        );
    }

    #[test]
    fn given_content_that_changed_when_the_document_reopens_then_the_lookup_misses_and_no_offer_is_made(
        (documents, stored) in (edited_document_strategy(), retained_position_strategy_any())
    ) {
        // P5 — "content-change hash ⇒ miss ⇒ `0.0` no offer"
        // (FR-036a): the changed bytes fingerprint differently, so the
        // stored position is unreachable — no offer, no approximate
        // match, and the changed document begins from its first
        // spoken position.
        let (original, edited) = documents;
        let original_key = fingerprint(&original);
        let edited_key = fingerprint(&edited);
        prop_assume!(original_key != edited_key);

        let dir = tempfile::tempdir()?;
        let path = store_path(dir.path());
        let mut store = BookmarkStore::open(&path)?;
        store.put(&original_key, stored)?;

        // The changed document's lookup misses: the key is the
        // content hash, and the content changed.
        let found = store.get(&edited_key)?;
        prop_assert_eq!(
            found,
            None,
            "the changed content resolved to the pre-edit position — an approximate match (FR-036a)"
        );

        // A miss opens nothing: no offer, no open continue offer, and
        // no expiry outcome — the changed content starts at 0.0.
        let mut machine = OfferMachine::new();
        prop_assert_eq!(
            machine.on_session_end(edited_key, found),
            None,
            "a lookup miss opened a resume offer (FR-036a)"
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

        // Nothing else deletes (storage-schema.md): the pre-edit row
        // is retained — unreachable, keyed to content that no longer
        // exists — which is the discard FR-036a describes.
        let retained = store.get(&original_key)?;
        prop_assert_eq!(
            retained,
            Some(stored),
            "an edit physically deleted the pre-edit entry — only Stop and document end delete (storage-schema.md)"
        );
    }
}

/// The document the deterministic cases store a
/// position for.
const DOC: &[u8] =
    b"# Rota da Manha\n\nPrimeira secao com numeros 123 e R$ 45,90.\n\nSegunda secao.\n";

/// P1 boundary case (FR-004): one `put` writes exactly
/// one row, under exactly one content key — the row
/// written and the row read back are the same row,
/// and no second row appears beside it.
#[test]
fn a_put_writes_exactly_one_row_under_exactly_one_key() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let other_key = fingerprint(b"# Outro Documento\n\nConteudo diferente por completo.\n");
    let stored = StoredPosition {
        offset_secs: 317.25,
        first_yield_ms: 40,
        end_kind: SessionEnd::Paused,
    };

    let mut store = BookmarkStore::open(&path)?;
    store.put(&key, stored)?;
    assert_eq!(
        store.get(&key)?,
        Some(stored),
        "the row read back is not the row put (FR-004)"
    );
    // No second row was created beside it: the untouched key misses.
    assert_eq!(
        store.get(&other_key)?,
        None,
        "a put created a row under a second key (FR-004)"
    );
    Ok(())
}

/// P2 boundary case (FR-036 / FR-004): the key is the
/// CONTENT, never the path — the same bytes at two
/// paths share one row, so a renamed document is not
/// orphaned.
#[test]
fn identical_bytes_at_two_paths_share_one_row() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let before = dir.path().join("antes.md");
    let after = dir.path().join("depois.md");
    std::fs::write(&before, DOC)?;
    std::fs::rename(&before, &after)?;

    let stored = StoredPosition {
        offset_secs: 61.5,
        first_yield_ms: 0,
        end_kind: SessionEnd::Paused,
    };
    let mut store = BookmarkStore::open(&path)?;
    store.put(&fingerprint(DOC), stored)?;

    let found = store.get(&fingerprint(&std::fs::read(&after)?))?;
    assert_eq!(
        found,
        Some(stored),
        "identical bytes at two paths did not share one row (FR-036)"
    );
    Ok(())
}

/// The contract the `put` verb enforces (FR-032 /
/// FR-032b): a terminal `--end-kind` is REFUSED with
/// `CliError::TerminalEndKind` — the failure half of
/// the text-I/O protocol (nothing on stdout, the
/// diagnostic on stderr) — and nothing is stored,
/// because the terminal ends delete the entry instead
/// of storing one. The two retained kinds are stored.
#[test]
fn the_put_verb_refuses_a_terminal_end_kind_with_a_named_error() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let db = store_path(dir.path());
    let document = dir.path().join("documento.md");
    std::fs::write(&document, DOC)?;
    let key = fingerprint(DOC);

    for (wire, terminal) in [
        ("stopped", SessionEnd::Stopped),
        ("completed", SessionEnd::Completed),
    ] {
        let file = document.to_string_lossy().into_owned();
        let (outcome, stdout, stderr) = cli_run(
            &db,
            &[
                "put",
                file.as_str(),
                "--offset-secs",
                "61.5",
                "--first-yield-ms",
                "120",
                "--end-kind",
                wire,
            ],
        );

        let error = match outcome {
            Err(error) => error,
            Ok(()) => {
                return Err(format!(
                    "the put verb accepted the terminal end kind {wire} — the terminal ends delete the entry instead of storing one (FR-032, FR-032b)"
                )
                .into());
            }
        };
        match error {
            CliError::TerminalEndKind { wire: refused } => assert_eq!(
                refused, wire,
                "the refused end kind is not the one requested (FR-032)"
            ),
            other => {
                return Err(format!(
                    "the put verb failed with {other}, not the named TerminalEndKind refusal"
                )
                .into());
            }
        }

        assert!(
            stdout.is_empty(),
            "a refused put wrote a payload to stdout, got: {}",
            String::from_utf8_lossy(&stdout)
        );
        let diagnostic = String::from_utf8(stderr)?;
        assert!(
            diagnostic.contains("FR-032"),
            "the refusal diagnostic names no requirement anchor, got: {diagnostic}"
        );

        // Nothing was stored: the reopen is a miss, and reading starts
        // at 0.0 with no offer.
        let mut store = BookmarkStore::open(&db)?;
        assert_eq!(
            store.get(&key)?,
            None,
            "the refused put stored an entry for the terminal end {terminal:?} (FR-032)"
        );
    }

    // The two retained kinds ARE storable through the same verb — the
    // refusal is specific to the terminal ends.
    for (wire, retained) in [
        ("paused", SessionEnd::Paused),
        ("interrupted", SessionEnd::Interrupted),
    ] {
        let file = document.to_string_lossy().into_owned();
        let (outcome, stdout, _stderr) = cli_run(
            &db,
            &[
                "put",
                file.as_str(),
                "--offset-secs",
                "61.5",
                "--first-yield-ms",
                "120",
                "--end-kind",
                wire,
            ],
        );
        outcome?;
        assert!(
            !stdout.is_empty(),
            "an accepted put wrote no payload for the end kind {wire}"
        );
        let mut store = BookmarkStore::open(&db)?;
        assert_eq!(
            store.get(&key)?.map(|stored| stored.end_kind),
            Some(retained),
            "the accepted put did not store the end kind {wire}"
        );
    }
    Ok(())
}

/// The store is a ROW store, not the policy layer: a
/// row written straight through it round-trips
/// exactly, whatever its end kind — which is what
/// makes the `put` verb's terminal-kind refusal and
/// the flow's delete the two halves of the
/// terminal-ends rule (the division of responsibility
/// this suite pins, not an oversight).
#[test]
fn the_store_is_the_row_store_the_terminal_ends_rule_is_enforced_above_it()
-> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let terminal_row = StoredPosition {
        offset_secs: 90.0,
        first_yield_ms: 40,
        end_kind: SessionEnd::Stopped,
    };

    let mut store = BookmarkStore::open(&path)?;
    store.put(&key, terminal_row)?;
    assert_eq!(
        store.get(&key)?,
        Some(terminal_row),
        "the row store did not round-trip the row it was handed (FR-004)"
    );

    // …and such a row opens no offer, because the terminal kinds are
    // never offered (FR-032 / FR-032e).
    let mut machine = OfferMachine::new();
    assert_eq!(
        machine.on_session_end(key, Some(terminal_row)),
        None,
        "a terminal end kind opened a resume offer (FR-032e)"
    );

    // The flow's half of the rule: the terminal end deletes, and the
    // reopen is the miss that starts at 0.0.
    store.delete(&key)?;
    assert_eq!(
        store.get(&key)?,
        None,
        "the delete did not remove the terminal row (FR-032)"
    );
    Ok(())
}

/// P3 boundary case (FR-032a): the two retained end
/// kinds both survive the process close — the entry is
/// on disk, not in the handle — and still resolve their
/// own offer form on the reopen.
#[test]
fn a_retained_entry_survives_the_process_close_and_resolves_its_own_offer()
-> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let stored = StoredPosition {
        offset_secs: 512.0,
        first_yield_ms: 1_700,
        end_kind: SessionEnd::Paused,
    };
    {
        let mut store = BookmarkStore::open(&path)?;
        store.put(&key, stored)?;
    }

    let mut reopened = BookmarkStore::open(&path)?;
    let found = reopened.get(&key)?;
    assert_eq!(
        found,
        Some(stored),
        "a retained entry did not survive the process close (FR-032a)"
    );
    let mut machine = OfferMachine::new();
    assert_eq!(
        machine.on_session_end(key, found),
        Some(Offer::Continue {
            offset_secs: stored.offset_secs,
            first_yield_ms: stored.first_yield_ms,
        }),
        "the reopened entry did not resolve the FR-032c offer (FR-032c)"
    );
    Ok(())
}
