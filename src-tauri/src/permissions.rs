//! Permission-state gate (T030b, US1).
//!
//! The deterministic half of the FR-008/FR-010/FR-021
//! permission contract (Principle V): the Kotlin shell
//! reads the platform permissions and pushes the answer
//! as a [`PermissionSnapshot`] through the `update_permissions`
//! command; this module makes no Android API calls — it is
//! the pure gate that decides what the answer permits. The
//! startup gate ([`check_startup`]) runs once at boot; the
//! pre-play gate ([`check_pre_play`]) runs before every
//! Play-class verb, so a refused start can never reach a
//! playing state (FR-010). A denied notification permission
//! is deliberately not a playback blocker: it routes to the
//! in-app statement ([`notification_statement`], FR-021,
//! SC-028), which T065 renders.
//!
//! Closed sets throughout (Principle VII, FR-044): no
//! `Other`/`Unknown`, no catch-all arms — adding a variant
//! is a coordinated edit across every consumer, with a
//! semver bump and a documentation update. The wire form of
//! every set is the `snake_case` name (contracts/shell-ipc.md
//! case-mapping table), so an unknown wire value never
//! deserializes: there is no variant for it to land in.

use serde::{Deserialize, Serialize};

/// The closed three-member permission answer (Principle
/// VII): the platform's answer for one permission, exactly
/// `Granted | Denied | NotRequested`.
///
/// The wire form is the `snake_case` state name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    /// The permission is granted.
    Granted,
    /// The permission is denied.
    Denied,
    /// The permission has not been requested yet.
    NotRequested,
}

/// The immutable platform permission state the Kotlin shell
/// forwards (FR-008, FR-021): the answers for the two
/// runtime-relevant permissions, as declared in the manifest
/// (`FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `POST_NOTIFICATIONS`).
///
/// [`Default`] is the all-granted host/CI baseline; a real
/// device pushes the platform answer through the
/// `update_permissions` command. The gates in this module
/// read this snapshot and nothing else — never the manifest
/// text (the manifest declaration itself is gated statically
/// by `scripts/check-manifest.sh`, T013/T081).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionSnapshot {
    /// Answer for `FOREGROUND_SERVICE_MEDIA_PLAYBACK` — the
    /// permission that keeps media playback alive as ongoing
    /// user-visible work (FR-006, FR-008).
    pub foreground_service_media_playback: PermissionState,
    /// Answer for `POST_NOTIFICATIONS` (API 33+) — required
    /// for the lock-screen surface (FR-008, FR-021).
    pub post_notifications: PermissionState,
}

impl Default for PermissionSnapshot {
    /// The host/CI baseline: every permission granted.
    fn default() -> Self {
        Self {
            foreground_service_media_playback: PermissionState::Granted,
            post_notifications: PermissionState::Granted,
        }
    }
}

/// The closed named-failure set for the permission gates
/// (Principle VII), each with a stated reason (FR-008,
/// FR-010, FR-021): exactly
/// `ForegroundServiceMissing | NotificationDenied`.
///
/// The wire form is the `snake_case` failure name — the
/// same string [`PermissionFailure::reason`] returns, so a
/// failure interpolated into a payload never drifts from its
/// diagnostic-record form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionFailure {
    /// FR-008/FR-010: the foreground-service declaration or
    /// permission is absent — the cold start is refused with
    /// the stated reason, and no playing state is ever
    /// presented. Playback begins only after the user opens
    /// the application.
    ForegroundServiceMissing,
    /// FR-021: the notification permission is denied on
    /// API 33+. Not a playback blocker — it routes to the
    /// in-app statement (`notification_statement`), and
    /// playback itself may proceed (SC-028: playback can
    /// still be started, paused, and stopped from inside
    /// the application).
    NotificationDenied,
}

impl PermissionFailure {
    /// The stated-reason string: short, identifier-style —
    /// the same shape T065's `ShortReason` renders on the
    /// lock screen and in-app.
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::ForegroundServiceMissing => "foreground_service_missing",
            Self::NotificationDenied => "notification_denied",
        }
    }
}

/// The startup gate (FR-008/FR-010): fails closed when the
/// foreground-service permission is absent — the cold start
/// is refused with the stated reason, and no playing state
/// may be presented. Runs once at boot (the `run` setup
/// closure, T030b wiring in `src-tauri/src/lib.rs`).
///
/// # Errors
///
/// [`PermissionFailure::ForegroundServiceMissing`] when the
/// foreground-service permission is `Denied` or
/// `NotRequested` — both are "absent", and the gate fails
/// closed on either.
pub fn check_startup(snapshot: &PermissionSnapshot) -> Result<(), PermissionFailure> {
    match snapshot.foreground_service_media_playback {
        PermissionState::Granted => Ok(()),
        PermissionState::Denied | PermissionState::NotRequested => {
            Err(PermissionFailure::ForegroundServiceMissing)
        }
    }
}

/// The pre-play gate (FR-008/FR-010): fails closed when the
/// foreground-service permission is absent, so a Play-class
/// verb (`Play`, `ExternalStart`, `ResumeAtStored`,
/// `StartOver` — the verbs that can reach `Playing`) can
/// never produce a playing state from a refused start.
///
/// A denied notification permission is deliberately not a
/// failure here: FR-021 routes it to the in-app statement
/// ([`notification_statement`]), and playback itself
/// proceeds (SC-028).
///
/// # Errors
///
/// [`PermissionFailure::ForegroundServiceMissing`] when the
/// foreground-service permission is `Denied` or
/// `NotRequested`.
pub fn check_pre_play(snapshot: &PermissionSnapshot) -> Result<(), PermissionFailure> {
    match snapshot.foreground_service_media_playback {
        PermissionState::Granted => Ok(()),
        PermissionState::Denied | PermissionState::NotRequested => {
            Err(PermissionFailure::ForegroundServiceMissing)
        }
    }
}

/// The FR-021 in-app statement for a denied notification
/// permission: lock-screen controls are unavailable.
///
/// Returns the statement only when the permission is
/// `Denied` — the caller surfaces it inside the application,
/// because the denied permission is precisely what removes
/// the surface it would otherwise appear on (FR-021,
/// FR-037). `Granted` and `NotRequested` carry no statement.
#[must_use]
pub fn notification_statement(snapshot: &PermissionSnapshot) -> Option<&'static str> {
    match snapshot.post_notifications {
        PermissionState::Denied => Some("lock_screen_controls_unavailable"),
        PermissionState::Granted | PermissionState::NotRequested => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A snapshot with the foreground-service permission
    /// absent (`Denied` or `NotRequested`) and the
    /// notification permission in the given state.
    fn snapshot_without_fgs(
        fgs: PermissionState,
        notifications: PermissionState,
    ) -> PermissionSnapshot {
        PermissionSnapshot {
            foreground_service_media_playback: fgs,
            post_notifications: notifications,
        }
    }

    #[test]
    fn startup_gate_passes_when_fgs_granted() {
        assert_eq!(check_startup(&PermissionSnapshot::default()), Ok(()));
    }

    #[test]
    fn startup_gate_fails_when_fgs_denied() {
        let snapshot = snapshot_without_fgs(PermissionState::Denied, PermissionState::Granted);
        assert_eq!(
            check_startup(&snapshot),
            Err(PermissionFailure::ForegroundServiceMissing)
        );
    }

    #[test]
    fn startup_gate_fails_when_fgs_not_requested() {
        let snapshot =
            snapshot_without_fgs(PermissionState::NotRequested, PermissionState::NotRequested);
        assert_eq!(
            check_startup(&snapshot),
            Err(PermissionFailure::ForegroundServiceMissing)
        );
    }

    #[test]
    fn pre_play_fails_closed_on_missing_fgs() {
        // Both absent answers fail closed (FR-008/FR-010):
        // a refused start never reaches a playing state.
        for fgs in [PermissionState::Denied, PermissionState::NotRequested] {
            let snapshot = snapshot_without_fgs(fgs, PermissionState::Granted);
            assert_eq!(
                check_pre_play(&snapshot),
                Err(PermissionFailure::ForegroundServiceMissing)
            );
        }
    }

    #[test]
    fn pre_play_passes_with_notification_denied() {
        // FR-021 / SC-028: a denied notification permission
        // routes to the in-app statement — it is not a
        // playback blocker.
        let snapshot = snapshot_without_fgs(PermissionState::Granted, PermissionState::Denied);
        assert_eq!(check_pre_play(&snapshot), Ok(()));
    }

    #[test]
    fn notification_statement_returns_only_for_denied() {
        let denied = snapshot_without_fgs(PermissionState::Granted, PermissionState::Denied);
        assert_eq!(
            notification_statement(&denied),
            Some("lock_screen_controls_unavailable")
        );
        assert_eq!(notification_statement(&PermissionSnapshot::default()), None);
        let not_requested =
            snapshot_without_fgs(PermissionState::Granted, PermissionState::NotRequested);
        assert_eq!(notification_statement(&not_requested), None);
    }

    #[test]
    fn failure_reasons_are_stated_identifier_strings() {
        assert_eq!(
            PermissionFailure::ForegroundServiceMissing.reason(),
            "foreground_service_missing"
        );
        assert_eq!(
            PermissionFailure::NotificationDenied.reason(),
            "notification_denied"
        );
    }

    #[test]
    fn update_permissions_round_trips_through_the_command_shape() {
        // The `update_permissions` command shape: the Kotlin
        // shell pushes the platform answer as snake_case JSON
        // (case-mapping table, contracts/shell-ipc.md), the
        // command stores it, and the gates answer against the
        // pushed snapshot — the baseline passes, a refused
        // answer flips both gates to the stated-reason
        // failure (FR-008, FR-010, FR-021).
        let baseline = PermissionSnapshot::default();
        assert_eq!(check_startup(&baseline), Ok(()));
        assert_eq!(check_pre_play(&baseline), Ok(()));

        let wire = serde_json::json!({
            "foreground_service_media_playback": "denied",
            "post_notifications": "denied",
        });
        let pushed = snapshot_without_fgs(PermissionState::Denied, PermissionState::Denied);
        // `serde_json::Error` carries no `PartialEq`, so the
        // round-trip asserts on the deserialized value.
        assert_eq!(
            serde_json::from_value::<PermissionSnapshot>(wire).ok(),
            Some(pushed)
        );
        assert_eq!(
            check_startup(&pushed),
            Err(PermissionFailure::ForegroundServiceMissing)
        );
        assert_eq!(
            check_pre_play(&pushed),
            Err(PermissionFailure::ForegroundServiceMissing)
        );
        assert_eq!(
            notification_statement(&pushed),
            Some("lock_screen_controls_unavailable")
        );
    }

    #[test]
    fn permission_state_wire_forms_are_snake_case() {
        // The wire form is the snake_case state name; the
        // closed set round-trips every variant, and an
        // unknown wire value never deserializes because no
        // variant exists for it (Principle VII).
        for (state, wire) in [
            (PermissionState::Granted, "granted"),
            (PermissionState::Denied, "denied"),
            (PermissionState::NotRequested, "not_requested"),
        ] {
            assert_eq!(
                serde_json::to_value(state).ok(),
                Some(serde_json::json!(wire))
            );
            assert_eq!(
                serde_json::from_value::<PermissionState>(serde_json::json!(wire)).ok(),
                Some(state)
            );
        }
        assert!(serde_json::from_value::<PermissionState>(serde_json::json!("other")).is_err());
    }
}
