//! T195 — Contract test (TDD RED): `pick_directory` action initiates directory
//! scanning, emits chapter metadata, and starts audio within TTFA < 3.0 s.
//!
//! Host-checkable: asserts the bridge parses and routes the action, calls the
//! shell scan entry point, publishes chapter metadata, and declares the budget.

use src_tauri::commands::parse_action;

const BRIDGE: &str = include_str!("../src/lib.rs");
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

/// TTFA budget (seconds) from spec 010 US1.
const TTFA_BUDGET_SECS: f64 = 3.0;

#[test]
fn parse_action_recognizes_pick_directory() {
    assert!(
        parse_action("pick_directory").is_ok(),
        "pick_directory must be a recognized bridge action"
    );
}

#[test]
fn bridge_initiates_directory_scan_on_pick_directory() {
    assert!(
        BRIDGE.contains("BridgeAction::PickDirectory"),
        "Bridge must match BridgeAction::PickDirectory"
    );
    assert!(
        BRIDGE.contains("pick_directory_from_shell"),
        "Bridge must call pick_directory_from_shell to start the directory scan"
    );
}

#[test]
fn bridge_emits_chapter_metadata() {
    assert!(
        BRIDGE.contains("chapter_index") && BRIDGE.contains("chapter_count"),
        "Published state must carry chapter_index and chapter_count metadata"
    );
}

#[test]
fn directory_playback_declares_ttfa_budget() {
    assert!(
        BRIDGE.contains("DIRECTORY_TTFA_BUDGET_SECS"),
        "Bridge must declare DIRECTORY_TTFA_BUDGET_SECS"
    );
    const {
        assert!(TTFA_BUDGET_SECS <= src_tauri::publish::DIRECTORY_TTFA_BUDGET_SECS);
    }
}

#[test]
fn status_view_sends_pick_directory() {
    assert!(
        STATUS_VIEW.contains("pick_directory"),
        "status_view.html must send the pick_directory action"
    );
}
