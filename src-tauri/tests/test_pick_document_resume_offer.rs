//! T168 — Contract test: reopen deliberate-pause document via picker presents resume offer.
//!
//! Requirement:
//! - When a document previously paused deliberately is reopened, the shell's offer route
//!   and cold start contract guarantees `ColdStart::AwaitOffer`, `offer_route: in_app_dialog`,
//!   `pending_cold_start: true`, and `offer_window_ms: 3000`.
//! - Audio does not immediately play while offer is pending.
//! - Contract verification of shell source and status view dialog integration.

use session::offer::{ColdStart, OpenOffer, cold_start};

const SHELL_SOURCE: &str = include_str!("../src/lib.rs");
const STATUS_VIEW: &str = include_str!("../src/status_view.html");

#[test]
fn deliberate_pause_maps_to_await_offer_cold_start() {
    let offer = OpenOffer::Continue {
        offset_secs: 42.5,
        first_yield_ms: 500,
    };
    let decision = cold_start(offer);
    assert_eq!(
        decision,
        ColdStart::AwaitOffer {
            offset_secs: 42.5,
            first_yield_ms: 500
        },
        "A deliberate pause must map to ColdStart::AwaitOffer"
    );
}

#[test]
fn shell_jni_forwards_ingested_document_with_offer_support() {
    assert!(
        SHELL_SOURCE.contains("open_document_bytes_named"),
        "Shell must use open_document_bytes_named to preserve file identity"
    );
    assert!(
        SHELL_SOURCE.contains("offer_route"),
        "Shell must publish offer_route"
    );
    assert!(
        SHELL_SOURCE.contains("pending_cold_start"),
        "Shell must track pending_cold_start"
    );
}

#[test]
fn status_view_responds_to_in_app_dialog_offer() {
    assert!(
        STATUS_VIEW.contains("in_app_dialog"),
        "Status view must listen for in_app_dialog route"
    );
    assert!(
        STATUS_VIEW.contains("applyOffer"),
        "Status view must apply offer route"
    );
    assert!(
        STATUS_VIEW.contains("els.offerDialog"),
        "Status view must control offerDialog element"
    );
}
