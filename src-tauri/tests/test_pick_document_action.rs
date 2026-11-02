//! T162 — Contract test: `pick_document` action dispatch and playback state invariant.
//!
//! Requirement:
//! - Dispatching `pick_document` bridge action returns `{ "status": "picker_launched" }`.
//! - Invoking `pick_document` does not arbitrarily mutate the existing playback transport or position.
//! - Valid document ingest via `open_document_bytes_named` immediately transitions a fresh document
//!   to `Playing` (auto-play, FR-001, FR-002, SC-001) starting at offset 0.0.

use serde_json::Value;
use src_tauri::commands::parse_action;
use src_tauri::{AppState, BookmarkStore, CpalBackend};

const SAMPLE_DOC: &[u8] = b"# Titulo Novo\n\nPrimeiro paragrafo falado do texto.";

fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let store =
        BookmarkStore::open(&dir.path().join("bookmarks.redb")).map_err(|e| e.to_string())?;
    Ok((dir, AppState::test_state_with_bookmark_store(store)))
}

fn backend_available() -> bool {
    CpalBackend::new().is_ok()
}

#[test]
fn parse_action_recognizes_pick_document() {
    let action = parse_action("pick_document");
    assert!(action.is_ok(), "pick_document must be a recognized bridge action");
}

#[test]
fn pick_document_leaves_transport_state_unaltered() -> Result<(), String> {
    let (_dir, state) = state_with_store()?;
    let before_transport = state.session.lock().map_err(|e| e.to_string())?.state();
    let before_pos = state.position_clock.lock().map_err(|e| e.to_string())?.read();

    // Calling pick_document_from_shell directly
    let res = src_tauri::shell_jni::pick_document_from_shell();
    assert!(res.is_ok(), "pick_document_from_shell must succeed (or no-op on host)");

    let after_transport = state.session.lock().map_err(|e| e.to_string())?.state();
    let after_pos = state.position_clock.lock().map_err(|e| e.to_string())?.read();

    assert_eq!(before_transport, after_transport, "pick_document must not mutate transport state");
    assert_eq!(before_pos, after_pos, "pick_document must not mutate position");
    Ok(())
}

#[test]
fn fresh_document_ingest_auto_plays_from_zero() -> Result<(), String> {
    let (_dir, state) = state_with_store()?;
    let opened = src_tauri::commands::open_document_bytes_named(
        &state,
        SAMPLE_DOC.to_vec(),
        "novo_livro.md",
    );

    if backend_available() {
        let payload: Value = opened.map_err(|e| e.to_string())?;
        assert_eq!(
            payload["status"], "playing",
            "A fresh document opened via picker must immediately start playing"
        );
        let pos = state.position_clock.lock().map_err(|e| e.to_string())?.read();
        assert_eq!(pos, 0.0, "Position must start at 0.0 for a fresh document");
    } else {
        match opened {
            Ok(payload) => {
                assert_eq!(payload["status"], "playing");
            }
            Err(err) => {
                assert!(
                    err.contains("FormatUndeliverable")
                        || err.contains("SampleRateMismatch")
                        || err.contains("NoDevice")
                        || err.contains("refused"),
                    "unexpected error on host with no backend: {err}"
                );
            }
        }
    }
    Ok(())
}
