//! T074 — property coverage for skip controls while the resident
//! session is paused between app sessions (FR-002, FR-042).

use proptest::prelude::*;

use player::cli::{PlaybackSession, ShellVerb};
use player::transport::TransportState;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// A sectionless lock-screen skip from a paused resident session
    /// applies the FR-002 ten-second fallback, clamped at document end,
    /// and leaves transport paused.
    #[test]
    fn prop_paused_resident_skip_forward_advances_ten_seconds(
        duration in 10u16..=10_000,
        position in 0u16..=10_000,
    ) {
        let duration = f64::from(duration);
        let position = f64::from(position).min(duration);
        let mut session = PlaybackSession::with_duration(duration);
        prop_assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(position);
        prop_assert!(session.execute(ShellVerb::Pause).is_ok());

        let result = session.execute(ShellVerb::SkipForward);
        prop_assert!(result.is_ok(), "resident paused session rejected skip-forward");
        if let Ok(snapshot) = result {
            let expected = (position + 10.0).min(duration);
            prop_assert_eq!(snapshot.transport, TransportState::Paused);
            prop_assert!(
                (snapshot.position_secs - expected).abs() < 1e-9,
                "paused resident skip from {position} / {duration} must land at {expected}, got {}",
                snapshot.position_secs,
            );
        }
    }

    /// A sectionless backward skip from a paused resident session
    /// applies the same ten-second rule and clamps at document start.
    #[test]
    fn prop_paused_resident_skip_backward_retreats_ten_seconds(
        duration in 10u16..=10_000,
        position in 0u16..=10_000,
    ) {
        let duration = f64::from(duration);
        let position = f64::from(position).min(duration);
        let mut session = PlaybackSession::with_duration(duration);
        prop_assert!(session.execute(ShellVerb::Play).is_ok());
        session.on_audible_progress(position);
        prop_assert!(session.execute(ShellVerb::Pause).is_ok());

        let result = session.execute(ShellVerb::SkipBackward);
        prop_assert!(result.is_ok(), "resident paused session rejected skip-backward");
        if let Ok(snapshot) = result {
            let expected = (position - 10.0).max(0.0);
            prop_assert_eq!(snapshot.transport, TransportState::Paused);
            prop_assert!(
                (snapshot.position_secs - expected).abs() < 1e-9,
                "paused resident skip from {position} / {duration} must land at {expected}, got {}",
                snapshot.position_secs,
            );
        }
    }
}
