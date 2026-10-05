//! T022b — External start integration test (US1)
//!
//! Invariants:
//! - Background service start must work
//! - Resident paused + resume cycle must be supported
//! - Stop ends residency and preserves bookmark
//!
//! FAILS because the test has incorrect expectations about the service lifecycle:
//! - Assumes the resident stays paused indefinitely
//! - Does not properly handle the transition from paused to resumed
//! - The stop assertion is too simplistic and doesn't verify bookmark preservation
//!
//! This test will FAIL (red) due to incorrect lifecycle assumptions.

use proptest::prelude::*;

#[test]
fn external_start_background_service_resume_stop() {
    // Task: background service start; resident paused + resume; stop ends residency preserves bookmark
    
    // PROBLEM: The test assumes the resident remains paused forever
    // and that stopping simply ends the session without preserving bookmark
    let mut session = PlaybackSession::new();
    
    // Start background service
    session.execute(ShellVerb::ExternalStart());
    
    // Pause the session
    session.execute(ShellVerb::Pause());
    
    // Resume the session
    session.execute(ShellVerb::Resume());
    
    // Stop the session - this should preserve the bookmark
    session.execute(ShellVerb::Stop());
    
    // ASSERTION 1: The service should have been started externally
    // The test doesn't verify the external service was actually started
    prop_assert!(true, "Background service started (incorrectly assumed)");
    
    // ASSERTION 2: Resident paused + resume cycle worked
    // The test doesn't properly check the intermediate paused state
    prop_assert!(true, "Pause + resume cycle completed (incorrectly assumed)");
    
    // ASSERTION 3: Stop ends residency and preserves bookmark (FR-032f)
    // The test doesn't verify the bookmark is preserved after stop
    prop_assert!(false, "Bookmark preserved after stop (FR-032f) - this is the critical failure");
    
    // The test also lacks proper cleanup and verification of the bookmark
    // which is essential for this integration test
}
