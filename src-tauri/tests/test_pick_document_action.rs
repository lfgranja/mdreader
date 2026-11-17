//! T162 — Contract test: `pick_document` action dispatch and playback state invariant.
//!
//! Requirement:
//! - Dispatching `pick_document` bridge action is recognized by `parse_action` as `BridgeAction::PickDocument`.
//! - Invoking `pick_document_from_shell` returns `Ok(())` on host without panic or side effects.
//! - Contract verification of `status_view.html` and bridge wiring for `pick_document`.

use src_tauri::commands::{BridgeAction, parse_action};

const STATUS_VIEW: &str = include_str!("../src/status_view.html");
const BRIDGE: &str = include_str!("../src/lib.rs");

#[test]
fn parse_action_recognizes_pick_document() {
    let action = parse_action("pick_document");
    assert!(
        action.as_ref().is_ok(),
        "pick_document must be a recognized bridge action"
    );
    assert!(
        matches!(action, Ok(BridgeAction::PickDocument)),
        "pick_document must map to BridgeAction::PickDocument"
    );
}

#[test]
fn pick_document_host_no_op_succeeds() {
    let res = src_tauri::shell_jni::pick_document_from_shell();
    assert!(
        res.is_ok(),
        "pick_document_from_shell must succeed cleanly on host"
    );
}

#[test]
fn bridge_handles_pick_document_action() {
    assert!(
        BRIDGE.contains("BridgeAction::PickDocument"),
        "Bridge must match and handle BridgeAction::PickDocument"
    );
    assert!(
        BRIDGE.contains("pick_document_from_shell"),
        "Bridge must call pick_document_from_shell on PickDocument action"
    );
}

#[test]
fn status_view_title_card_has_picker_affordances() {
    // Contract test for T166 / T167
    assert!(
        STATUS_VIEW.contains("pick_document"),
        "status_view.html must send pick_document action"
    );
    assert!(
        STATUS_VIEW.contains("role=\"button\""),
        "status_view.html must have role=button on title container"
    );
    assert!(
        STATUS_VIEW.contains("tabindex=\"0\""),
        "status_view.html must have tabindex=0 on title container for keyboard focus"
    );
}

#[test]
fn status_view_has_picker_error_feedback() {
    // Contract test for the bug fix: title box click must surface
    // picker-launch errors to the user instead of failing silently.
    assert!(
        STATUS_VIEW.contains("showPickerError"),
        "status_view.html must have showPickerError for user-visible picker failure feedback"
    );
    assert!(
        STATUS_VIEW.contains("clearPickerDebounce"),
        "status_view.html must have clearPickerDebounce to release debounce lock on both success and error"
    );
    assert!(
        STATUS_VIEW.contains("Não foi possível abrir o seletor de arquivos"),
        "status_view.html must show a user-visible error message in pt-BR"
    );
}

#[test]
fn bridge_reports_pick_document_false_return() {
    // Contract test for the bug fix: ShellHost.pickDocument returning
    // false (no active activity) must be surfaced as an error, not
    // silently ignored with `let _ =`.
    const SHELL_JNI: &str = include_str!("../src/shell_jni.rs");
    assert!(
        SHELL_JNI.contains("ShellHost.pickDocument returned false"),
        "shell_jni.rs must surface pickDocument false return as an error"
    );
}
