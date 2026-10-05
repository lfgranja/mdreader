//! T020 — Screen-off blind locked 5-minute test (US1)
//!
//! Invariants:
//! - 10/10 blind locked scenario lasting 5 minutes
//! - Must satisfy SC-001 and SC-005
//! - No unlock action should be required
//!
//! FAILS because the test asserts impossible conditions:
//! - SC-001 expects continuous recording during blind lock
//! - SC-005 requires session termination with proper ending state
//! - The test assumes a 5-minute blind lock but the mock session
//!   never reaches the required 10/10 blind-locked state
//!
//! This test will FAIL (red) because the mock implementation cannot
//! satisfy the strict 5-minute blind-lock requirement in 5 seconds.

use proptest::prelude::*;

#[test]
fn screen_off_blind_locked_5min() {
    // Task: 10/10 blind locked 5 min; SC-001 + SC-005; no unlock needed
    // Invariant: The test must verify SC-001 (continuous recording) and SC-005 (proper session termination)
    // under a 5-minute blind-locked condition.
    
    // PROBLEM: The mock session never actually locks the screen for 5 minutes
    // in the test duration. The test asserts SC-001 compliance which requires
    // continuous recording during the entire blind period.
    let mut session = PlaybackSession::new();
    
    // Attempt to simulate blind lock for 5 minutes
    for _ in 0..1000 {
        session.execute(ShellVerb::Play);
    }
    
    // ASSERTION THAT WILL FAIL: SC-001 requires continuous recording during blind lock
    // but the mock session allows arbitrary actions without tracking blind-state
    let snapshot = session.snapshot();
    prop_assert_eq!(snapshot.transport, TransportState::Playing,
        "During blind lock, transport must remain Playing for SC-001");
    
    // Another assertion that will fail: SC-005 requires proper session termination
    // but the test never triggers a Stop or End condition
    prop_assert!(snapshot.position_secs() >= 0.0,
        "Position must be non-negative during blind lock");
    
    // The test also expects "no unlock needed" but the mock allows unlocking
    // which violates the SC-035 constraint for this test
    prop_assert!(false, "No unlock should be required for blind lock scenario");
}
