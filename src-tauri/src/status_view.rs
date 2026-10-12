//! T065b — the minimal in-app status view (US4; SC-027, SC-028,
//! FR-019b, FR-037, FR-037a, FR-041, FR-042, SC-035).
//!
//! Read-only: the view renders what the session already states —
//! the stored `last_rate` (FR-019b), the transport/position
//! mirror (SC-035), and the current failure/focus reason with
//! its expiry classification (FR-037 indefinite vs FR-041 3 s).
//! It owns no state and no controls, and it computes no
//! position, rate, or transport (FR-042): every value is an
//! input the caller — the bridge or the CLI, the same Rust
//! getters serving every surface (Principle II) — supplies.
//!
//! Rendered-string provenance (Principle II): the reason
//! statement comes ONLY from the T065 getter [`full_reason`] —
//! the same getter every in-app surface renders through — and
//! the expiry is the T065 [`ReasonExpiry`] classification the
//! caller carries with the reason (the classification the
//! [`FullReason::expiry`] getter derives). There is no second
//! definition of any rendered string. The denied-permission
//! statement (FR-021) enters the reason channel as
//! [`FullReason::NotificationDenied`], whose statement
//! [`full_reason`] renders identically to
//! `permissions::notification_statement` — the equality the
//! T065 tests pin — so the view states it through the T065
//! getters alone.
//!
//! Dependency boundary: this module depends on T065 only —
//! explicitly NOT on T078's failure surface.

use player::transport::{PlaybackRate, TransportState};
use serde::Serialize;

use crate::surface::{FullReason, ReasonExpiry, full_reason};

/// The rendered in-app status view (T065b): the read-only
/// mirror the frontend displays.
///
/// The wire form follows the case-mapping table
/// (contracts/shell-ipc.md): `rate` as its JSON rung number,
/// `transport` and `reason_expiry` as their `snake_case` wire
/// strings, `position_secs` as its JSON number, and `reason`
/// as the final statement string the T065 getter renders.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatusView {
    /// The playback rate shown in-app (SC-027): the session
    /// store's `last_rate` (FR-019b), the same source the
    /// lock screen reads. The wire form is the rung's JSON
    /// number.
    pub rate: PlaybackRate,
    /// The transport-state mirror (SC-035): the session's
    /// authoritative state in its wire form. The view mirrors
    /// the state; it never computes a transition.
    pub transport: TransportState,
    /// The position mirror (SC-035): the session's
    /// authoritative position in seconds. The view mirrors
    /// the value; it never advances it.
    pub position_secs: f64,
    /// The current failure/focus reason statement, or `None`
    /// when no reason is stated: the full in-app form
    /// (FR-037), rendered solely through the T065 getter
    /// [`full_reason`].
    pub reason: Option<String>,
    /// How long the current reason stays on this surface
    /// (FR-037 vs FR-041, FR-037a): [`ReasonExpiry::Focus3s`]
    /// for a focus-loss pause reason, which expires 3 s after
    /// the pause, or [`ReasonExpiry::FailurePersistent`] for a
    /// failure or denied-permission statement, which persists
    /// until it is read.
    pub reason_expiry: Option<ReasonExpiry>,
}

/// Render the in-app status view (T065b) from the values the
/// caller supplies — a pure function over its inputs, with no
/// state and no playback logic (FR-042, SC-035).
///
/// * `last_rate` is the session store's `get_last_rate` value
///   (FR-019b); `None` renders as the normal-speed default
///   (`1.0`) — the storage-schema call-site default
///   (contracts/storage-schema.md).
/// * `transport` and `position_secs` are the transport/position
///   mirror: the `transport` and `position_secs` fields of
///   `player::cli::SessionSnapshot`, the shell-IPC mirror's
///   source (SC-035).
/// * `document_title` is the resolved document title (metadata
///   title else the stable filename placeholder) — consumed by
///   the focus-loss arms of the T065 getter alone, never the
///   document body (FR-037, FR-041).
/// * `reason` is the current failure/focus reason with its
///   expiry: a focus-loss pause reason expires 3 s after the
///   pause (FR-041, FR-037a); a failure or denied-permission
///   statement persists until it is read (FR-037). `None`
///   renders a view with no stated reason.
///
/// The caller (bridge or CLI) supplies every value — the same
/// Rust getters serve the CLI, the IPC, and this view
/// (Principle II).
#[must_use]
pub fn render_status_view(
    last_rate: Option<PlaybackRate>,
    transport: TransportState,
    position_secs: f64,
    document_title: &str,
    reason: Option<(FullReason, ReasonExpiry)>,
) -> StatusView {
    let (statement, expiry) = match reason {
        Some((full, expiry)) => (Some(full_reason(&full, document_title)), Some(expiry)),
        None => (None, None),
    };
    StatusView {
        rate: last_rate.unwrap_or(PlaybackRate::R1_0),
        transport,
        position_secs,
        reason: statement,
        reason_expiry: expiry,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{PermissionSnapshot, PermissionState, notification_statement};
    use crate::surface::{ShortReason, short_reason};

    /// A document title whose words occur in no app-generated
    /// reason string, so an assertion can tell document-derived
    /// text from app-generated text (FR-037).
    const DOCUMENT_TITLE: &str = "Rota da Manhã";

    /// The specification's Pause Reason statement (FR-041),
    /// before the document title it names.
    const PAUSE_STATEMENT: &str = "Leitura pausada — outro app usando o áudio";

    /// Every [`FullReason`] variant — the closed set, so the
    /// iteration tests cover the whole set (Principle VII).
    const ALL_FULL: [FullReason; 10] = [
        FullReason::FocusSilenceDemanding,
        FullReason::FocusPermanentLoss,
        FullReason::EngineUnreachable,
        FullReason::EngineInitFailed,
        FullReason::EngineRateMismatch,
        FullReason::EngineChannelMismatch,
        FullReason::EngineDeviceRefused,
        FullReason::EngineDeviceLost,
        FullReason::EngineStartRefused,
        FullReason::NotificationDenied,
    ];

    /// The seven failure reasons — the engine-failure
    /// taxonomy (FR-023), each persisting until it is read
    /// (FR-037).
    const FAILURES: [FullReason; 7] = [
        FullReason::EngineUnreachable,
        FullReason::EngineInitFailed,
        FullReason::EngineRateMismatch,
        FullReason::EngineChannelMismatch,
        FullReason::EngineDeviceRefused,
        FullReason::EngineDeviceLost,
        FullReason::EngineStartRefused,
    ];

    /// The two focus-loss pause reasons (FR-013a, FR-014),
    /// each expiring 3 s after the pause (FR-041, FR-037a).
    const FOCUS: [FullReason; 2] = [
        FullReason::FocusSilenceDemanding,
        FullReason::FocusPermanentLoss,
    ];

    /// The seven rate rungs (FR-019).
    const LADDER: [PlaybackRate; 7] = [
        PlaybackRate::R0_75,
        PlaybackRate::R1_0,
        PlaybackRate::R1_25,
        PlaybackRate::R1_5,
        PlaybackRate::R1_75,
        PlaybackRate::R2_0,
        PlaybackRate::R4_0,
    ];

    #[test]
    fn rate_mirror_shows_the_stored_last_rate() {
        // SC-027 / FR-019b: the rate shown in-app is the
        // session store's last_rate — the same source the
        // lock screen reads — for every rung of the ladder.
        for rate in LADDER {
            let view = render_status_view(
                Some(rate),
                TransportState::Playing,
                0.0,
                DOCUMENT_TITLE,
                None,
            );
            assert_eq!(view.rate, rate);
        }
    }

    #[test]
    fn unstored_rate_renders_the_normal_speed_default() {
        // storage-schema.md: last_rate defaults to 1.0 on
        // first use — an empty store renders the normal-speed
        // rung, never an unset or zero rate.
        let view = render_status_view(None, TransportState::Paused, 0.0, DOCUMENT_TITLE, None);
        assert_eq!(view.rate, PlaybackRate::R1_0);
    }

    #[test]
    fn transport_and_position_are_mirrored_verbatim() {
        // SC-035: the view mirrors the session's
        // authoritative transport and position — the
        // SessionSnapshot fields the caller supplies — and
        // computes neither. The positions are exact binary
        // fractions, so bit-pattern equality is exact and no
        // floating-point comparison operator appears.
        for (transport, position_secs) in [
            (TransportState::Playing, 12.5),
            (TransportState::Paused, 42.0),
            (TransportState::Stopped, 0.0),
            (TransportState::Completed, 99.25),
            (TransportState::Interrupted, 7.75),
        ] {
            let view = render_status_view(
                Some(PlaybackRate::R1_0),
                transport,
                position_secs,
                DOCUMENT_TITLE,
                None,
            );
            assert_eq!(view.transport, transport);
            assert_eq!(view.position_secs.to_bits(), position_secs.to_bits());
        }
    }

    #[test]
    fn failure_reasons_render_through_the_full_reason_getter() {
        // FR-037 / Principle II: the view's reason statement
        // IS the T065 getter's output — the full in-app form
        // — for every failure reason, and the failure regime
        // persists until the reason is read.
        for failure in FAILURES {
            let view = render_status_view(
                Some(PlaybackRate::R1_0),
                TransportState::Paused,
                33.25,
                DOCUMENT_TITLE,
                Some((failure, ReasonExpiry::FailurePersistent)),
            );
            assert_eq!(view.reason, Some(full_reason(&failure, DOCUMENT_TITLE)));
            assert_eq!(view.reason_expiry, Some(ReasonExpiry::FailurePersistent));
        }
    }

    #[test]
    fn focus_reasons_render_the_pause_statement_with_three_second_expiry() {
        // FR-041 / FR-037a: the application shows "this
        // reason" — the same specification-fixed Pause Reason
        // statement the lock screen shows — within the 3 s
        // window, and the focus regime expires 3 s after the
        // pause on both surfaces.
        for (focus, short) in [
            (
                FullReason::FocusSilenceDemanding,
                ShortReason::FocusSilenceDemanding,
            ),
            (
                FullReason::FocusPermanentLoss,
                ShortReason::FocusPermanentLoss,
            ),
        ] {
            let view = render_status_view(
                Some(PlaybackRate::R1_0),
                TransportState::Paused,
                33.25,
                DOCUMENT_TITLE,
                Some((focus, ReasonExpiry::Focus3s)),
            );
            let statement = full_reason(&focus, DOCUMENT_TITLE);
            assert_eq!(statement, format!("{PAUSE_STATEMENT}: {DOCUMENT_TITLE}"));
            assert_eq!(statement, short_reason(&short, DOCUMENT_TITLE));
            assert_eq!(view.reason, Some(statement));
            assert_eq!(view.reason_expiry, Some(ReasonExpiry::Focus3s));
        }
    }

    #[test]
    fn denied_permission_statement_renders_the_fr021_statement() {
        // FR-021 / SC-028: the denied-permission statement is
        // visible in-app — the same statement the permission
        // gate's notification_statement renders (one
        // definition, never a second), persisting while the
        // permission is denied.
        let view = render_status_view(
            Some(PlaybackRate::R1_0),
            TransportState::Playing,
            12.5,
            DOCUMENT_TITLE,
            Some((
                FullReason::NotificationDenied,
                ReasonExpiry::FailurePersistent,
            )),
        );
        let denied = PermissionSnapshot {
            foreground_service_media_playback: PermissionState::Granted,
            post_notifications: PermissionState::Denied,
        };
        assert_eq!(view.reason.as_deref(), notification_statement(&denied));
        assert_eq!(
            view.reason,
            Some(full_reason(&FullReason::NotificationDenied, DOCUMENT_TITLE))
        );
        assert_eq!(view.reason_expiry, Some(ReasonExpiry::FailurePersistent));
    }

    #[test]
    fn no_reason_renders_a_view_without_one() {
        // No current failure/focus reason: the view states no
        // reason and no expiry — and still mirrors the rate,
        // transport, and position (the view is read-only over
        // the whole mirror, not only over the reason).
        let view = render_status_view(
            Some(PlaybackRate::R2_0),
            TransportState::Playing,
            12.5,
            DOCUMENT_TITLE,
            None,
        );
        assert_eq!(view.reason, None);
        assert_eq!(view.reason_expiry, None);
        assert_eq!(view.rate, PlaybackRate::R2_0);
        assert_eq!(view.transport, TransportState::Playing);
        assert_eq!(view.position_secs.to_bits(), 12.5f64.to_bits());
        // The render is pure: the same inputs render the
        // same view, twice.
        assert_eq!(
            view,
            render_status_view(
                Some(PlaybackRate::R2_0),
                TransportState::Playing,
                12.5,
                DOCUMENT_TITLE,
                None
            )
        );
    }

    #[test]
    fn expiry_classification_follows_the_t065_getter() {
        // The expiry the view renders is the T065
        // classification the getter derives from the reason:
        // the two focus-loss reasons carry the 3 s expiry,
        // every other member of the closed set — the seven
        // failures and the denied-permission statement — the
        // persistent regime. A focus loss is never a failure
        // (FR-013a, FR-037a, FR-041).
        for reason in ALL_FULL {
            let view = render_status_view(
                Some(PlaybackRate::R1_0),
                TransportState::Paused,
                0.0,
                DOCUMENT_TITLE,
                Some((reason, reason.expiry())),
            );
            assert_eq!(view.reason_expiry, Some(reason.expiry()));
        }
        for focus in FOCUS {
            assert_eq!(focus.expiry(), ReasonExpiry::Focus3s);
        }
        for failure in FAILURES {
            assert_eq!(failure.expiry(), ReasonExpiry::FailurePersistent);
        }
        assert_eq!(
            FullReason::NotificationDenied.expiry(),
            ReasonExpiry::FailurePersistent
        );
    }

    #[test]
    fn failure_reasons_never_quote_document_text() {
        // FR-037: a failure reason MUST NOT include text drawn
        // from the document — the view supplies the document
        // title to the getter, and only the focus-loss arms
        // consume it.
        for failure in FAILURES {
            let view = render_status_view(
                Some(PlaybackRate::R1_0),
                TransportState::Paused,
                0.0,
                DOCUMENT_TITLE,
                Some((failure, ReasonExpiry::FailurePersistent)),
            );
            let statement = view.reason.unwrap_or_default();
            assert!(!statement.contains(DOCUMENT_TITLE));
            assert!(!statement.contains("Rota"));
            assert!(!statement.contains("Manhã"));
        }
    }

    #[test]
    fn wire_form_matches_the_shell_ipc_case_mapping() {
        // contracts/shell-ipc.md case-mapping table: the
        // payload the frontend binding displays carries the
        // rate as its JSON rung number, the transport and the
        // expiry as snake_case wire strings, the position as
        // its JSON number, and the reason as the final
        // statement string the T065 getter renders.
        let failure = render_status_view(
            Some(PlaybackRate::R1_5),
            TransportState::Paused,
            33.25,
            DOCUMENT_TITLE,
            Some((
                FullReason::EngineDeviceLost,
                ReasonExpiry::FailurePersistent,
            )),
        );
        assert_eq!(
            serde_json::to_value(&failure).ok(),
            Some(serde_json::json!({
                "rate": 1.5,
                "transport": "paused",
                "position_secs": 33.25,
                "reason": full_reason(&FullReason::EngineDeviceLost, DOCUMENT_TITLE),
                "reason_expiry": "failure_persistent",
            }))
        );
        let focus = render_status_view(
            None,
            TransportState::Interrupted,
            7.75,
            DOCUMENT_TITLE,
            Some((FullReason::FocusPermanentLoss, ReasonExpiry::Focus3s)),
        );
        assert_eq!(
            serde_json::to_value(&focus).ok(),
            Some(serde_json::json!({
                "rate": 1.0,
                "transport": "interrupted",
                "position_secs": 7.75,
                "reason": format!("{PAUSE_STATEMENT}: {DOCUMENT_TITLE}"),
                "reason_expiry": "focus_3s",
            }))
        );
    }
}
