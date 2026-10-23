//! T034 — the offer-window test (Phase US3 BLOCKING;
//! tasks.md:130): the windowed state-machine contract
//! of `crates/session/src/offer.rs` (T060) over the
//! stored positions of `crates/session/src/store.rs`
//! (T058), exercised both in-process and through the
//! CLI verbs of `crates/session/src/cli.rs` (T063) —
//! the Principle-II parity tasks.md C2 names ("T034
//! offer via session ingest CLI (T018)").
//!
//! Invariants under test (tasks.md T034, verbatim):
//! "Offer-window test in `crates/session/tests/
//! test_offer3s.rs` (deliberate reopen ⇒ visual dialog
//! 3 s continue-vs-start-over; kill/power-loss ⇒
//! paused + lock-screen resume control; `Stop` clears
//! before next open)"
//!
//! Spec anchors:
//! - FR-032c (spec.md:957): a deliberate-pause reopen
//!   shows a visible 3-second resume offer — continue
//!   from the stored position, the DEFAULT, with a
//!   start-over alternative. If no choice is made
//!   within the window, reading begins automatically
//!   from the stored position PROVIDED the application
//!   is in the foreground when the window expires;
//!   otherwise the offer converts to the
//!   explicit-resume form of FR-032d and never begins
//!   audio on its own. Choosing start-over begins from
//!   the first spoken position instead, and the stored
//!   position is discarded.
//! - FR-032c's visibility MUST (spec.md:90): the offer
//!   remains visible for the whole 3 seconds — the
//!   machine latches the latest `AppForegrounded` /
//!   `AppBackgrounded` value and reads it at expiry
//!   ONLY, never converting an open offer early.
//! - FR-032d / FR-031 (spec.md:968 / 941): following an
//!   unexpected session end — a kill, a power-off, a
//!   battery death, a network loss — the system MUST
//!   NOT begin producing audio on its own; recovery to
//!   a PAUSED STATE with an explicit resume control is
//!   the only permitted automatic transition. That is
//!   the lock-screen resume control, and it is the
//!   only form those ends resolve to.
//! - FR-032 / FR-032b / FR-032e (spec.md:947 / 974 /
//!   977): a Stop press and a document's end DELETE the
//!   stored entry — the only two events that delete
//!   (storage-schema.md) — so the next open is a miss
//!   that starts at `0.0` and is offered nothing.
//! - FR-032f: the offer window never outlives the
//!   session it belonged to — a choice or an expiry
//!   ends it, and a new session-end replaces any
//!   previously open offer.
//! - contracts/storage-schema.md (normative): the offer
//!   flow is resolved SOLELY from the stored
//!   `end_kind` — `paused` ⇒ the FR-032c deliberate
//!   offer, `interrupted` ⇒ the FR-032d
//!   explicit-resume form; `stopped`/`completed` never
//!   open one; "miss (incl. edited content, FR-036a) ⇒
//!   start at 0.0, no offer".
//!
//! What the machine is and is not, read off the code
//! before this suite was written:
//!
//! 1. The offer machine carries NO CLOCK. `expire()` is
//!    the deterministic representation of the 3-second
//!    window elapsing — no caller waits on real time —
//!    so this suite asserts the window as a
//!    state-machine contract (which offer form, which
//!    outcome, what survives the window) and the
//!    on-device 3-second timing is T051's ten-trial
//!    complement.
//! 2. **Recorded gap for T037 (RED-by-TODO, not yet
//!    wired).** There is no session-side constant
//!    naming the window: `crates/session/src/offer.rs`
//!    exposes no duration/expiry item, and the
//!    `offer` payloads in `cli.rs` carry no window
//!    field. The adapter-side twin is
//!    `adapter_synthesis::MAX_OFFER_MS = 3_000`
//!    (crates/adapter-synthesis/src/failure.rs:43), and
//!    the `session` crate does not depend on
//!    `adapter-synthesis`, so the twin cannot be bound
//!    here. The intended assertion — recorded, not run
//!    — is `assert_eq!(OFFER_WINDOW_MS, 3_000)` against
//!    the constant T036/T037 land in `offer.rs`; see
//!    [`the_deliberate_offer_window_is_a_named_three_second_constant`].
//! 3. **Recorded gap for T036/T037 (the routing
//!    surface).** The session side resolves the OFFER
//!    FORM (`Offer::Continue` vs the explicit-resume
//!    requirement); it does not yet name the SURFACE
//!    the form routes to (the in-app 3-second dialog
//!    vs the paused lock-screen control). That routing
//!    is T037's, in `src-tauri/src/lib.rs`, over the
//!    decision function T036 adds. This suite pins the
//!    decision outputs that routing consumes, so the
//!    router can be written against a fixed contract.
//!
//! Relationship to the sibling suites:
//! `prop_offer.rs` (T052) pins the machine's
//! determinacy properties, `prop_resume.rs` (T050) the
//! fingerprint/edit properties, and
//! `prop_bookmark_lifecycle.rs` (T033) the storage
//! lifecycle; this suite is the WINDOWED contract —
//! the dialog form, the lock-screen control, the
//! silence rule, and the Stop-before-open clear.
//!
//! Proptest note: this suite is deliberately NOT a
//! property suite — tasks.md rule 2 makes T034 the
//! snapshot/contract test for the fixed 3-second
//! window shape, with `prop_bookmark_lifecycle`
//! (T033) as its `prop_*` complement. Timing invariants
//! without property cover (the 3 s TTL exact) stay
//! snapshot-asserted here with T051's on-device trials
//! as complement.

use std::error::Error;
use std::path::{Path, PathBuf};

use engine_core::closed::SessionEnd;
use serde_json::Value;
use session::cli::{CliError, run as session_cli};
use session::fingerprint::fingerprint;
use session::offer::{ExpiryOutcome, Offer, OfferMachine};
use session::store::{BookmarkStore, StoredPosition};

/// The document the window cases store a position
/// for.
const DOC: &[u8] =
    b"# Rota da Manha\n\nPrimeira secao com numeros 123 e R$ 45,90.\n\nSegunda secao.\n";

/// The stored position of a deliberate pause — the
/// reopen that opens the 3-second dialog.
const PAUSED_OFFSET_SECS: f64 = 317.25;

/// The FR-015 interval start carried with the
/// deliberate pause.
const PAUSED_FIRST_YIELD_MS: u64 = 40;

/// The stored position of an unexpected end — the
/// kill/power-loss reopen that pauses with an explicit
/// resume control.
const INTERRUPTED_OFFSET_SECS: f64 = 62.0;

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
/// instead of being swallowed.
fn cli_run(db: &Path, args: &[&str]) -> (Result<(), CliError>, Vec<u8>, Vec<u8>) {
    let invocation = std::iter::once(String::from("--db"))
        .chain(std::iter::once(db.to_string_lossy().into_owned()))
        .chain(args.iter().map(|arg| String::from(*arg)));
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let outcome = session_cli(invocation, &mut stdout, &mut stderr);
    (outcome, stdout, stderr)
}

/// Runs one successful CLI invocation and parses its
/// stdout as the JSON payload the machine-readable
/// default renders.
///
/// # Errors
///
/// The invocation fails (the diagnostic is carried in
/// the message) or the payload is not valid JSON.
fn cli_json(db: &Path, args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let (outcome, stdout, stderr) = cli_run(db, args);
    if let Err(error) = outcome {
        let diagnostic = String::from_utf8_lossy(&stderr);
        return Err(format!(
            "`session {}` failed: {error} ({diagnostic})",
            args.join(" ")
        )
        .into());
    }
    Ok(serde_json::from_slice(&stdout)?)
}

/// The value at `path` — a nested key path — inside
/// `payload`.
fn field<'a>(payload: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = payload;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

/// A document file holding `DOC`, in `dir`.
///
/// # Errors
///
/// The write fails.
fn write_document(dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let path = dir.join("documento.md");
    std::fs::write(&path, DOC)?;
    Ok(path)
}

/// The store a window case reads through, over
/// `path`.
///
/// # Errors
///
/// The store cannot be opened.
fn open_store(path: &Path) -> Result<BookmarkStore, Box<dyn Error>> {
    Ok(BookmarkStore::open(path)?)
}

/// Deliberate reopen ⇒ the visual dialog's FORM
/// (FR-032c): the reopen resolves to the continue
/// offer carrying the stored position — continue from
/// the stored position, the default, with the
/// start-over alternative beside it — and silence
/// inside the window continues from the stored
/// position.
#[test]
fn a_deliberate_pause_on_reopen_resolves_the_visual_dialog_form_and_silence_continues()
-> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let stored = StoredPosition {
        offset_secs: PAUSED_OFFSET_SECS,
        first_yield_ms: PAUSED_FIRST_YIELD_MS,
        end_kind: SessionEnd::Paused,
    };

    let mut store = open_store(&path)?;
    store.put(&key, stored)?;
    let found = store.get(&key)?;

    // The dialog's continue option: the FR-032c offer,
    // resolved solely from the stored end kind.
    let mut machine = OfferMachine::new();
    assert_eq!(
        machine.on_session_end(key, found),
        Some(Offer::Continue {
            offset_secs: PAUSED_OFFSET_SECS,
            first_yield_ms: PAUSED_FIRST_YIELD_MS,
        }),
        "a deliberate reopen did not resolve the FR-032c continue offer (FR-032c)"
    );

    // The dialog stays open, unchanged, while the window runs — the
    // offer is not consumed by being read.
    assert_eq!(
        machine.continue_offer(),
        Some(Offer::Continue {
            offset_secs: PAUSED_OFFSET_SECS,
            first_yield_ms: PAUSED_FIRST_YIELD_MS,
        }),
        "the open dialog offer changed while the window was open (FR-032c)"
    );

    // Silence — no choice made inside the 3-second window — continues
    // from the stored position when the application is in the
    // foreground at expiry.
    machine.on_foregrounded();
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::AutoStart {
            offset_secs: PAUSED_OFFSET_SECS
        },
        "silence did not continue from the stored position (FR-032c)"
    );

    // …and the stored entry is still the row the silence continued
    // from: the offer window never deletes (only Stop and document
    // end delete, storage-schema.md).
    assert_eq!(
        store.get(&key)?,
        Some(stored),
        "the offer window disturbed the stored entry (storage-schema.md)"
    );
    Ok(())
}

/// The dialog's start-over alternative (FR-032c):
/// reading begins from exactly `0.0`, the choice
/// consumes the window, and the stored position is
/// discarded — the discard being the caller's, the
/// machine holding no store.
#[test]
fn the_start_over_choice_begins_at_the_first_spoken_position_and_discards_the_entry()
-> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let stored = StoredPosition {
        offset_secs: PAUSED_OFFSET_SECS,
        first_yield_ms: PAUSED_FIRST_YIELD_MS,
        end_kind: SessionEnd::Paused,
    };

    let mut store = open_store(&path)?;
    store.put(&key, stored)?;

    let mut machine = OfferMachine::new();
    let _opened = machine.on_session_end(key, store.get(&key)?);
    assert_eq!(
        machine.start_over(),
        ExpiryOutcome::AutoStart { offset_secs: 0.0 },
        "the start-over choice did not begin at the first spoken position (FR-032c)"
    );
    assert_eq!(
        machine.continue_offer(),
        None,
        "the start-over choice left the dialog offer open (FR-032c)"
    );
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::NoOffer,
        "an expiry followed the start-over choice (FR-032f)"
    );

    // The discard is the caller's job: the Stop-equivalent delete the
    // flow performs with the choice.
    store.delete(&key)?;
    assert_eq!(
        store.get(&key)?,
        None,
        "the start-over choice did not discard the stored entry (FR-032c, FR-032f)"
    );
    Ok(())
}

/// Kill / power-loss ⇒ paused + lock-screen resume
/// control (FR-032d / FR-031): an unexpected end opens
/// NO dialog, leaves the explicit-resume requirement,
/// and never produces audio on its own — the reader
/// must press the control on the lock screen.
#[test]
fn an_unexpected_end_on_reopen_resolves_the_paused_lock_screen_resume_control()
-> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let stored = StoredPosition {
        offset_secs: INTERRUPTED_OFFSET_SECS,
        first_yield_ms: 0,
        end_kind: SessionEnd::Interrupted,
    };

    let mut store = open_store(&path)?;
    store.put(&key, stored)?;
    let found = store.get(&key)?;
    assert_eq!(
        found,
        Some(stored),
        "an unexpected end did not preserve the entry (FR-032a)"
    );

    let mut machine = OfferMachine::new();
    assert_eq!(
        machine.on_session_end(key, found),
        None,
        "an unexpected end opened a continue offer — no dialog opens for it (FR-032d)"
    );
    assert_eq!(
        machine.continue_offer(),
        None,
        "an unexpected end left a continue offer open (FR-032d)"
    );

    // The reader is present — the application is foregrounded, the
    // device is unlocked — and no audio begins anyway: the explicit
    // resume control is the only route in.
    machine.on_foregrounded();
    let outcome = machine.expire();
    let auto_started = matches!(outcome, ExpiryOutcome::AutoStart { .. });
    assert!(
        !auto_started,
        "an unexpected end auto-started audio (FR-032d, FR-031): {outcome:?}"
    );
    assert_eq!(
        outcome,
        ExpiryOutcome::ExplicitResume,
        "an unexpected end resolved to something other than the explicit resume (FR-032d)"
    );

    // The paused state the control resumes from is untouched: the
    // explicit-resume path never cleared the stored position.
    assert_eq!(
        store.get(&key)?,
        Some(stored),
        "the explicit-resume path cleared a preserved entry (FR-032a, storage-schema.md)"
    );
    Ok(())
}

/// FR-032f — the window never outlives its session:
/// a choice and an expiry both end it, a later expiry
/// yields nothing, and a new session-end replaces any
/// offer still open.
#[test]
fn the_offer_window_never_outlives_its_session_or_a_choice() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let paused = StoredPosition {
        offset_secs: PAUSED_OFFSET_SECS,
        first_yield_ms: PAUSED_FIRST_YIELD_MS,
        end_kind: SessionEnd::Paused,
    };
    let interrupted = StoredPosition {
        offset_secs: INTERRUPTED_OFFSET_SECS,
        first_yield_ms: 0,
        end_kind: SessionEnd::Interrupted,
    };

    let mut store = open_store(&path)?;
    store.put(&key, paused)?;

    // The window ends with its expiry: nothing remains open, and a
    // second expiry yields no outcome.
    let mut machine = OfferMachine::new();
    let _opened = machine.on_session_end(key, store.get(&key)?);
    machine.on_foregrounded();
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::AutoStart {
            offset_secs: PAUSED_OFFSET_SECS
        },
        "the foregrounded expiry did not continue from the stored position (FR-032c)"
    );
    assert_eq!(
        machine.continue_offer(),
        None,
        "an offer remained open after its expiry (FR-032f)"
    );
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::NoOffer,
        "an expiry repeated after the window ended (FR-032f)"
    );

    // A new session-end replaces any offer still open: the dialog for
    // the deliberate pause does not survive into the next session,
    // which ended unexpectedly.
    let mut replacing = OfferMachine::new();
    let _opened = replacing.on_session_end(key, store.get(&key)?);
    store.put(&key, interrupted)?;
    let next = replacing.on_session_end(key, store.get(&key)?);
    assert_eq!(
        next, None,
        "a new session-end did not replace the open offer (FR-032f)"
    );
    assert_eq!(
        replacing.continue_offer(),
        None,
        "the previous session's offer outlived the new session (FR-032f)"
    );
    Ok(())
}

/// FR-032c's visibility MUST (spec.md:90): the offer
/// remains visible for the whole 3 seconds — a
/// mid-window `AppBackgrounded` never converts it
/// early, and a later `AppForegrounded` restores the
/// auto-start the silence rule gives.
#[test]
fn a_mid_window_visibility_change_never_converts_the_open_offer_early() -> Result<(), Box<dyn Error>>
{
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let stored = StoredPosition {
        offset_secs: PAUSED_OFFSET_SECS,
        first_yield_ms: PAUSED_FIRST_YIELD_MS,
        end_kind: SessionEnd::Paused,
    };
    let mut store = open_store(&path)?;
    store.put(&key, stored)?;
    let offer = Some(Offer::Continue {
        offset_secs: PAUSED_OFFSET_SECS,
        first_yield_ms: PAUSED_FIRST_YIELD_MS,
    });

    let mut machine = OfferMachine::new();
    let _opened = machine.on_session_end(key, store.get(&key)?);

    // The application is backgrounded mid-window: the dialog stays open
    // and visible — nothing converts early.
    machine.on_backgrounded();
    assert_eq!(
        machine.continue_offer(),
        offer,
        "a mid-window AppBackgrounded converted the open offer early (FR-032c, spec.md:90)"
    );

    // The application returns to the foreground before the window
    // expires: the silence rule's auto-start is restored.
    machine.on_foregrounded();
    assert_eq!(
        machine.continue_offer(),
        offer,
        "a mid-window AppForegrounded changed the open offer (FR-032c)"
    );
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::AutoStart {
            offset_secs: PAUSED_OFFSET_SECS
        },
        "the foreground restored the silence rule's auto-start (FR-032c)"
    );

    // Backgrounded at expiry, the same offer converts to the
    // explicit-resume form instead — never audio on its own.
    let mut backgrounded = OfferMachine::new();
    let _opened = backgrounded.on_session_end(key, store.get(&key)?);
    backgrounded.on_backgrounded();
    assert_eq!(
        backgrounded.expire(),
        ExpiryOutcome::ExplicitResume,
        "a backgrounded expiry began audio on its own (FR-032c, FR-031)"
    );
    Ok(())
}

/// `Stop` clears before the next open (FR-032 /
/// FR-032b / FR-032e): the terminal end deletes the
/// entry, so the next open is a miss — no offer, and
/// reading starts at `0.0`.
#[test]
fn stop_before_the_next_open_clears_the_entry_and_the_reopen_starts_at_zero_with_no_offer()
-> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let path = store_path(dir.path());
    let key = fingerprint(DOC);
    let stored = StoredPosition {
        offset_secs: PAUSED_OFFSET_SECS,
        first_yield_ms: PAUSED_FIRST_YIELD_MS,
        end_kind: SessionEnd::Paused,
    };

    let document = write_document(dir.path())?;
    let file = document.to_string_lossy().into_owned();

    // A session was paused mid-document and the entry exists. The
    // handle is dropped before any CLI invocation: one handle on the
    // file at a time, the pattern `integration_ingest.rs` (T014)
    // establishes.
    {
        let mut store = open_store(&path)?;
        store.put(&key, stored)?;
        assert_eq!(
            store.get(&key)?,
            Some(stored),
            "the deliberate pause was not stored before the Stop (FR-032c)"
        );
    }

    // The Stop press deletes it — the `clear` verb's operation, the
    // same delete the Stop and document-end paths perform. The success
    // half of the protocol: the payload on stdout, nothing on stderr.
    let (outcome, stdout, stderr) = cli_run(&path, &["clear", file.as_str()]);
    outcome?;
    assert!(
        !stdout.is_empty(),
        "the clear verb wrote no payload (the text-I/O protocol's success half)"
    );
    assert!(
        stderr.is_empty(),
        "the clear verb wrote a diagnostic: {}",
        String::from_utf8_lossy(&stderr)
    );

    {
        let mut store = open_store(&path)?;
        assert_eq!(
            store.get(&key)?,
            None,
            "the Stop press did not clear the stored entry (FR-032)"
        );
    }

    // The next open is a miss: no offer, no outcome, and the ingest
    // verb reports the start-at-zero resolution the miss resolves to.
    let mut machine = OfferMachine::new();
    assert_eq!(
        machine.on_session_end(key, None),
        None,
        "a reopened document after Stop opened a resume offer (FR-032e)"
    );
    assert_eq!(
        machine.expire(),
        ExpiryOutcome::NoOffer,
        "a reopened document after Stop produced an expiry outcome (FR-032e)"
    );

    let payload = cli_json(&path, &["ingest", file.as_str()])?;
    assert_eq!(
        field(&payload, &["resume"]),
        Some(&Value::Bool(false)),
        "a reopen after Stop reported a resume (FR-032e)"
    );
    assert_eq!(
        field(&payload, &["offset"]).and_then(Value::as_f64),
        Some(0.0),
        "a reopen after Stop did not start at 0.0 (FR-032e, FR-036a)"
    );
    Ok(())
}

/// CLI parity (tasks.md C2): the same decision
/// function, driven over the wire. A stored deliberate
/// pause resolves to the continue offer, silence
/// continues from the stored position when the
/// foreground latch holds, a backgrounded expiry
/// converts, and an unexpected end resolves to the
/// explicit resume — never an auto-start.
#[test]
fn the_offer_verbs_resolve_the_same_forms_over_the_wire() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let db = store_path(dir.path());
    let document = write_document(dir.path())?;
    let file = document.to_string_lossy().into_owned();

    // A deliberate pause is stored through the `put` verb…
    let _stored_paused = cli_json(
        &db,
        &[
            "put",
            file.as_str(),
            "--offset-secs",
            "317.25",
            "--first-yield-ms",
            "40",
            "--end-kind",
            "paused",
        ],
    )?;

    // …the reopen resolves the continue offer (the dialog's continue
    // option)…
    let shown = cli_json(&db, &["offer", "show", file.as_str()])?;
    assert_eq!(
        field(&shown, &["offer", "kind"]).and_then(Value::as_str),
        Some("continue"),
        "the offer verb did not resolve the FR-032c continue offer (FR-032c)"
    );
    assert_eq!(
        field(&shown, &["offer", "offset_secs"]).and_then(Value::as_f64),
        Some(PAUSED_OFFSET_SECS),
        "the continue offer did not carry the stored position (FR-032c)"
    );
    assert_eq!(
        field(&shown, &["offer", "first_yield_ms"]).and_then(Value::as_u64),
        Some(PAUSED_FIRST_YIELD_MS),
        "the continue offer did not carry the FR-015 interval start (FR-032c)"
    );

    // …silence in the window, foregrounded, continues from the stored
    // position. `--foreground` is a FLAG (clap's SetTrue on a bool):
    // its presence latches `AppForegrounded`; its absence leaves the
    // `AppBackgrounded` latch holding (cli.rs).
    let continued = cli_json(&db, &["offer", "expire", file.as_str(), "--foreground"])?;
    assert_eq!(
        field(&continued, &["outcome", "kind"]).and_then(Value::as_str),
        Some("auto_start"),
        "a foregrounded expiry did not auto-start from the stored position (FR-032c)"
    );
    assert_eq!(
        field(&continued, &["outcome", "offset_secs"]).and_then(Value::as_f64),
        Some(PAUSED_OFFSET_SECS),
        "the silence rule did not continue from the stored position (FR-032c)"
    );

    // …and the same expiry, backgrounded, converts to the explicit
    // resume instead.
    let backgrounded = cli_json(&db, &["offer", "expire", file.as_str()])?;
    assert_eq!(
        field(&backgrounded, &["outcome", "kind"]).and_then(Value::as_str),
        Some("explicit_resume"),
        "a backgrounded expiry did not convert to the explicit resume (FR-032c, FR-031)"
    );

    // The start-over choice over the wire: reading begins at 0.0 and
    // the stored entry is discarded.
    let start_over = cli_json(&db, &["offer", "start-over", file.as_str()])?;
    assert_eq!(
        field(&start_over, &["outcome", "kind"]).and_then(Value::as_str),
        Some("auto_start"),
        "the start-over choice did not begin reading (FR-032c)"
    );
    assert_eq!(
        field(&start_over, &["outcome", "offset_secs"]).and_then(Value::as_f64),
        Some(0.0),
        "the start-over choice did not begin at the first spoken position (FR-032c)"
    );
    assert_eq!(
        field(&start_over, &["discarded"]),
        Some(&Value::Bool(true)),
        "the start-over choice did not discard the stored entry (FR-032c, FR-032f)"
    );

    // An unexpected end — the kill/power-loss reopening — resolves to
    // the explicit resume over the wire, never an auto-start.
    let _stored_interrupted = cli_json(
        &db,
        &[
            "put",
            file.as_str(),
            "--offset-secs",
            "62.0",
            "--first-yield-ms",
            "0",
            "--end-kind",
            "interrupted",
        ],
    )?;
    let interrupted_show = cli_json(&db, &["offer", "show", file.as_str()])?;
    assert_eq!(
        field(&interrupted_show, &["offer"]),
        Some(&Value::Null),
        "an unexpected end opened a continue offer over the wire (FR-032d)"
    );
    let interrupted_expire = cli_json(&db, &["offer", "expire", file.as_str(), "--foreground"])?;
    assert_eq!(
        field(&interrupted_expire, &["outcome", "kind"]).and_then(Value::as_str),
        Some("explicit_resume"),
        "an unexpected end auto-started audio over the wire (FR-032d, FR-031)"
    );
    Ok(())
}

/// T037 GAP — recorded, not run (tasks.md: "the 3 s
/// window … if absent, record the gap for T037 with a
/// TODO").
///
/// The session side has NO named constant for the
/// offer window: `crates/session/src/offer.rs` exposes
/// no duration/expiry item, and the CLI's `offer`
/// payloads carry no window field. The machine models
/// the window as the clockless `expire()` hop, so the
/// 3 seconds live in the ADAPTER (`MAX_OFFER_MS =
/// 3_000`, crates/adapter-synthesis/src/failure.rs:43)
/// and in the UI layer T037 wires — never in a
/// session-side name the dialog and the test could
/// share.
///
/// The intended assertion, once the constant lands in
/// `offer.rs`, is:
///
/// ```ignore
/// assert_eq!(session::offer::OFFER_WINDOW_MS, 3_000, "...");
/// ```
///
/// Until then this case stays `#[ignore]`d: the mirror
/// below is the RECORDED value of the adapter twin
/// (`session` does not depend on `adapter-synthesis`,
/// so the twin cannot be bound from here), and the
/// assertion pins the requirement, not a symbol.
#[test]
#[ignore = "T037 gap: session::offer exposes no offer-window constant (the 3 s window is the clockless expire() hop); un-ignore and bind the assertion to the constant T036/T037 land in offer.rs"]
fn the_deliberate_offer_window_is_a_named_three_second_constant() {
    /// The recorded window value: the FR-032c
    /// deliberate-pause offer window is 3 seconds
    /// (spec.md:957), the same `3_000` ms the
    /// adapter-side `MAX_OFFER_MS` carries.
    const RECORDED_OFFER_WINDOW_MS: u32 = 3_000;

    assert_eq!(
        RECORDED_OFFER_WINDOW_MS, 3_000,
        "the deliberate-pause offer window is 3 s (FR-032c, spec.md:957) — \
         TODO(T036/T037): name it in crates/session/src/offer.rs as the window \
         constant the dialog and this suite share, mirroring \
         adapter_synthesis::MAX_OFFER_MS (3_000)"
    );
}
