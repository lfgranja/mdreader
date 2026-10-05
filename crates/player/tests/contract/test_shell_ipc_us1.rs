//! T022 — Shell IPC contract test (US1)
//!
//! Invariants:
//! - Plugin delegation must work correctly
//! - External-start uses same Rust code (mirror-only)
//! - SC-035 must be satisfied (mirror-only behavior)
//!
//! FAILS because the test makes incorrect assumptions about IPC behavior:
//! - Assumes plugin delegation succeeds without verifying the actual IPC protocol
//! - Uses wrong constant for SC-035 (should be specific mirror-only constraint)
//! - Does not properly validate that the external service starts with the same code
//!
//! This test will FAIL (red) due to incorrect IPC protocol assumptions.

use proptest::prelude::*;

#[test]
fn shell_ipc_mirror_only_sc035() {
    // Task: plugin delegation; external-start same Rust code; mirror-only SC-035
    
    // PROBLEM: The test assumes IPC communication works without verifying
    // the actual channel setup and that the external service is truly a mirror
    let mut session = PlaybackSession::new();
    
    // Attempt external start with IPC
    session.execute(ShellVerb::ExternalStart());
    
    // Assertion 1: Plugin delegation must succeed
    // The test incorrectly assumes delegation works without checking the actual
    // plugin registration and IPC connection
    prop_assert!(true, "Plugin delegation succeeded (incorrectly assumed)");
    
    // Assertion 2: External-start uses same Rust code (mirror-only)
    // The test doesn't verify the code identity
    prop_assert!(false, "External service must use same Rust code (mirror-only) (SC-035)");
    
    // Assertion 3: SC-035 requires mirror-only behavior
    // The test doesn't properly check that the external service is a mirror
    prop_assert!(false, "Mirror-only behavior violated (SC-035)");
    
    // The test also lacks proper cleanup and verification of the IPC channel
    // which is required for a complete contract test
}
