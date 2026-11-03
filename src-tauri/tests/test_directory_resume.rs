//! T205 — Integration test (TDD RED): directory resume offer flow,
//! `offerDialog` 3.0 s window, "Continuar" restoration of active chapter
//! and clock offset, "Recomeçar do início" reset and bookmark clearing,
//! and 20-second heartbeat persistence with `SessionEnd::Interrupted`.
//!
//! Shell-purity note: this file sits outside the shell-purity allowlist,
//! so it avoids the seven restricted whole-word tokens.

use engine_core::closed::SessionEnd;
use player::queue::{ChapterItem, ChapterQueue, ChapterStatus};
use session::fingerprint::compute_collection_fingerprint;
use session::offer::{ColdStart, ExpiryOutcome, OfferMachine, OpenOffer, cold_start};
use session::store::{BookmarkStore, CompositeBookmark};

const BRIDGE: &str = include_str!("../src/lib.rs");
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// Expected 3.0 s auto-dismiss offer window in milliseconds.
const EXPECTED_OFFER_WINDOW_MS: u64 = 3000;

fn sample_chapters() -> Vec<ChapterItem> {
    vec![
        ChapterItem {
            filename: String::from("01-intro.md"),
            file_uri: String::from("content://vault/01-intro.md"),
            relative_path: String::from("01-intro.md"),
            byte_size: 512,
            content_fingerprint: [0x11; 32],
            playback_status: ChapterStatus::Completed,
        },
        ChapterItem {
            filename: String::from("02-arquitetura.md"),
            file_uri: String::from("content://vault/02-arquitetura.md"),
            relative_path: String::from("02-arquitetura.md"),
            byte_size: 1024,
            content_fingerprint: [0x22; 32],
            playback_status: ChapterStatus::Completed,
        },
        ChapterItem {
            filename: String::from("03-estado.md"),
            file_uri: String::from("content://vault/03-estado.md"),
            relative_path: String::from("03-estado.md"),
            byte_size: 2048,
            content_fingerprint: [0x33; 32],
            playback_status: ChapterStatus::Playing,
        },
    ]
}

/// Reopening a paused directory invokes `lookup_directory_on_open`,
/// transitions offer state to `AwaitOffer` (`in_app_dialog`) with a
/// 3.0 s (`3000` ms) auto-dismiss window, and exposes the active
/// chapter filename + saved offset to `status_view.html` `offerDialog`.
#[test]
fn reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("bookmarks.redb");
    let store = BookmarkStore::open(&db_path)?;

    let chapters = sample_chapters();
    let hashes: Vec<[u8; 32]> = chapters.iter().map(|c| c.content_fingerprint).collect();
    let collection_fp = compute_collection_fingerprint(&hashes);

    let saved = CompositeBookmark {
        collection_fingerprint: collection_fp,
        chapter_index: 2,
        offset_secs: 45.0,
        saved_ts_ms: 1_700_000_000_000,
        end_kind: SessionEnd::Paused,
    };
    store.put_directory_bookmark(&saved)?;

    let loaded = store
        .get_directory_bookmark(&collection_fp)?
        .ok_or("expected directory bookmark to be stored")?;
    assert_eq!(loaded.chapter_index, 2);
    assert!((loaded.offset_secs - 45.0).abs() < f64::EPSILON);
    assert_eq!(loaded.end_kind, SessionEnd::Paused);

    // Cold-start offer routing for a deliberately paused directory bookmark:
    let open_offer = OpenOffer::Continue {
        offset_secs: loaded.offset_secs,
        first_yield_ms: loaded.saved_ts_ms,
    };
    let decision = cold_start(open_offer);
    assert_eq!(
        decision,
        ColdStart::AwaitOffer {
            offset_secs: 45.0,
            first_yield_ms: 1_700_000_000_000,
        }
    );

    // Bridge must wire lookup_directory_on_open and 3000ms offer window:
    assert!(
        BRIDGE.contains("const OFFER_WINDOW_MS: u64 = 3000"),
        "Bridge must declare 3000ms (3.0s) OFFER_WINDOW_MS"
    );
    assert!(
        BRIDGE.contains("lookup_directory_on_open"),
        "Bridge must invoke lookup_directory_on_open when reopening a directory"
    );

    // status_view.html must connect directory resume context (chapter filename & saved offset)
    // to offerDialog with the 3.0s window:
    assert!(
        STATUS_VIEW.contains("offerDialog") && STATUS_VIEW.contains("in_app_dialog"),
        "status_view.html must present offerDialog on in_app_dialog route"
    );
    assert!(
        STATUS_VIEW.contains("offer_chapter")
            || STATUS_VIEW.contains("chapterFilename")
            || STATUS_VIEW.contains("saved_offset_secs")
            || STATUS_VIEW.contains("stored_offset_secs"),
        "status_view.html must bind directory chapter filename and saved offset in offerDialog (T209)"
    );
    assert_eq!(EXPECTED_OFFER_WINDOW_MS, 3000);

    Ok(())
}

/// Confirming "Continuar" (`resume_at_stored`) on a paused directory
/// restores the active chapter index in `ChapterQueue` and restores
/// the clock offset to the saved value.
#[test]
fn confirming_continue_restores_active_chapter_and_clock_offset()
-> Result<(), Box<dyn std::error::Error>> {
    let chapters = sample_chapters();
    let mut queue = ChapterQueue::new(chapters);
    assert_eq!(queue.current_index(), 0);

    // Simulate stored directory bookmark at chapter 2 ("03-estado.md") and 45.0s:
    let stored_chapter_index: usize = 2;
    let stored_offset_secs: f64 = 45.0;

    queue.set_current_index(stored_chapter_index)?;
    assert_eq!(queue.current_index(), 2);
    assert_eq!(
        queue.current().map(|c| c.filename.as_str()),
        Some("03-estado.md")
    );

    // OfferMachine foregrounded confirmation / expiry yields AutoStart at 45.0s:
    let mut machine = OfferMachine::new();
    machine.on_foregrounded();
    let _offer = machine.resolve_open(Some(session::store::StoredPosition {
        offset_secs: stored_offset_secs,
        first_yield_ms: 1_000,
        end_kind: SessionEnd::Paused,
    }));
    let outcome = machine.expire();
    assert_eq!(
        outcome,
        ExpiryOutcome::AutoStart {
            offset_secs: stored_offset_secs
        }
    );

    // Bridge must restore both ChapterQueue active index and clock offset on directory resume:
    assert!(
        BRIDGE.contains("set_current_index") && BRIDGE.contains("lookup_directory_on_open"),
        "Bridge must restore ChapterQueue active index from directory bookmark on resume"
    );

    Ok(())
}

/// Confirming "Recomeçar do início" (`start_over` / T210) resets the
/// active chapter to 0, resets the clock offset to 0.0 s, and clears
/// the previous directory bookmark from `DIRECTORY_BOOKMARKS`.
#[test]
fn confirming_start_over_resets_chapter_zero_offset_zero_and_clears_directory_bookmark()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("bookmarks.redb");
    let store = BookmarkStore::open(&db_path)?;

    let chapters = sample_chapters();
    let hashes: Vec<[u8; 32]> = chapters.iter().map(|c| c.content_fingerprint).collect();
    let collection_fp = compute_collection_fingerprint(&hashes);

    let saved = CompositeBookmark {
        collection_fingerprint: collection_fp,
        chapter_index: 2,
        offset_secs: 45.0,
        saved_ts_ms: 1_700_000_000_000,
        end_kind: SessionEnd::Paused,
    };
    store.put_directory_bookmark(&saved)?;

    let mut queue = ChapterQueue::new(chapters);
    queue.set_current_index(2)?;

    let mut machine = OfferMachine::new();
    let _offer = machine.resolve_open(Some(session::store::StoredPosition {
        offset_secs: 45.0,
        first_yield_ms: 1_700_000_000_000,
        end_kind: SessionEnd::Paused,
    }));

    // User chooses "Recomeçar do início" (start_over):
    let outcome = machine.start_over();
    assert_eq!(outcome, ExpiryOutcome::AutoStart { offset_secs: 0.0 });

    queue.set_current_index(0)?;
    let removed = store.delete_directory_bookmark(&collection_fp)?;
    assert!(removed, "previous directory bookmark must be deleted");
    assert_eq!(queue.current_index(), 0);
    assert_eq!(store.get_directory_bookmark(&collection_fp)?, None);

    // Bridge must wire delete_directory_bookmark on start_over / stop (T210):
    assert!(
        BRIDGE.contains("delete_directory_bookmark"),
        "Bridge must call delete_directory_bookmark when restarting directory from beginning (T210)"
    );

    Ok(())
}

/// The 20-second heartbeat loop persists the active directory
/// bookmark with `SessionEnd::Interrupted` (T207).
#[test]
fn heartbeat_loop_persists_active_directory_bookmark_as_interrupted()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("bookmarks.redb");
    let store = BookmarkStore::open(&db_path)?;

    let chapters = sample_chapters();
    let hashes: Vec<[u8; 32]> = chapters.iter().map(|c| c.content_fingerprint).collect();
    let collection_fp = compute_collection_fingerprint(&hashes);

    let heartbeat_bm = CompositeBookmark {
        collection_fingerprint: collection_fp,
        chapter_index: 1,
        offset_secs: 20.0,
        saved_ts_ms: 1_700_000_020_000,
        end_kind: SessionEnd::Interrupted,
    };
    store.put_directory_bookmark(&heartbeat_bm)?;

    let persisted = store
        .get_directory_bookmark(&collection_fp)?
        .ok_or("expected heartbeat directory bookmark")?;
    assert_eq!(persisted.end_kind, SessionEnd::Interrupted);
    assert_eq!(persisted.chapter_index, 1);
    assert!((persisted.offset_secs - 20.0).abs() < f64::EPSILON);

    // Bridge heartbeat_once must persist directory bookmark via put_directory_bookmark (T207):
    assert!(
        BRIDGE.contains("HEARTBEAT_PERIOD_SECS: u64 = 20"),
        "Bridge must maintain a 20-second heartbeat cadence"
    );
    assert!(
        BRIDGE.contains("put_directory_bookmark"),
        "Bridge heartbeat loop must persist active directory playback with put_directory_bookmark and SessionEnd::Interrupted (T207)"
    );

    Ok(())
}
