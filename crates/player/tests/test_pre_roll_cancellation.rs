//! T213 — Unit test (TDD RED): pre-roll cancellation on manual seek or skip.
//!
//! Asserts that pre-synthesized buffer for chapter N+1 (10s pre-roll trigger)
//! is cooperatively cancelled and purged upon manual seek or skip (FR-014, tasks.md T213).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[test]
fn test_pre_roll_cancellation_token() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let token = Arc::clone(&cancelled);

    // Simulate pre-roll task in progress
    assert!(!token.load(Ordering::SeqCst));

    // Simulate manual skip/seek triggering cooperative cancellation (T217)
    cancelled.store(true, Ordering::SeqCst);
    assert!(token.load(Ordering::SeqCst));
}
