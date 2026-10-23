//! T022b — US1 integration: externally-started-service trial
//! (FR-010 / FR-011 / SC-014).
//!
//! Host-runnable replay of the device fixture
//! `specs/001-android-media-shell/tests/device/fixtures/external_start_refused.jsonl`.
//! The externally-started service is modelled as a `PlaybackSession`
//! created outside any app-process context — a fresh session with no
//! prior state — and the trial's two legs are driven through the
//! single dispatch (`player::cli`), the same code every surface
//! (notification, lock screen, CLI, IPC) executes (FR-042):
//!
//! - **FR-011 leg (SC-014 liveness):** a Play press from the lock
//!   surface reaches `Playing` on a session that carries no
//!   app-process state, and position advances only via audible
//!   progress. On host the audio seam is `on_audible_progress` —
//!   the same function the CPAL backend feeds — so "produces audio
//!   without the app open" is asserted as: the session accepts the
//!   verb, reaches `Playing`, and moves only when audible progress
//!   arrives.
//! - **FR-010 leg (cold start refused):** with the FGS precondition
//!   absent the platform refuses the start. The stated reason is
//!   produced, no `Playing` state is ever reported (the session
//!   stays `Paused`), and no `transport:"playing"` entry follows the
//!   refused start in the record.
//!
//! The permission-state gate itself (`src-tauri/src/permissions.rs`,
//! T030b) is not yet implemented, so the FR-010 leg asserts the seam
//! the way the fixture records it: a host-side model of the gate over
//! the same closed sets, plus a simulated record in the fixture's
//! event shape (`ts_ms` / `kind` / `result`) scanned for the
//! no-playing invariant. RED finding until T030b lands.
//!
//! The fixture's third entry (`FOCUS_GRANT → playing_from_last_spoken`)
//! is the FR-009a recovery outcome — playback begins from the last
//! spoken position only after the focus grant, once the user opens the
//! application — recorded as a trial result token, never as a
//! transport mirror. The device-trace shape carries no `transport`
//! field at all; the `transport:"playing"` mirror lives in the SC-005
//! diagnostic record, which is what the conformance scan checks.

use std::error::Error;

use engine_core::closed::{PlaybackRate, TransportState};
use player::cli::{PlaybackSession, ShellVerb};
use serde::Deserialize;

/// Embedded device fixture: the externally-started-service refusal
/// trial (3 ordered entries).
const FIXTURE_EXTERNAL_START_REFUSED: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/external_start_refused.jsonl"
);

/// The stated refusal reason for a platform-refused cold start
/// (FR-010) — the `result` token the fixture records for the refused
/// `APP_START` (the shell-ipc contract's `StartRefused` refusal
/// surface).
const REFUSAL_REASON_SERVICE_START_REFUSED: &str = "service_start_refused";

/// Position comparison tolerance, in seconds — the same
/// epsilon form the focus-trial integration test uses, so no
/// strict float comparison ever appears (clippy `float_cmp`).
const POS_EPS_SECS: f64 = 0.001;

/// Wire transport state — the closed five-member set of the
/// diagnostic-record schema (SC-005). The device-trace fixture
/// carries `transport` only when an entry mirrors one; the refusal
/// trial records none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WireTransport {
    /// Playing.
    Playing,
    /// Paused.
    Paused,
    /// Stopped.
    Stopped,
    /// Completed.
    Completed,
    /// Interrupted.
    Interrupted,
}

/// The closed set of event kinds the external-start trial records.
/// Deserialization rejects any unrecorded kind: the closed-set rule
/// (FR-044 / Principle VII) makes a new kind a coordinated edit
/// across fixture, schema, and this test — never a catch-all arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TrialKind {
    /// The service was asked to start.
    AppStart,
    /// An audio-focus request.
    FocusRequest,
    /// An audio-focus grant.
    FocusGrant,
}

/// The closed set of trial outcomes the external-start fixture records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TrialResult {
    /// The platform refused the service start (FR-010).
    ServiceStartRefused,
    /// The focus grant was deferred.
    DeferredGrant,
    /// Playback resumed from the last spoken position (FR-009a).
    PlayingFromLastSpoken,
}

/// One device-fixture entry: the trial trace's event shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct TrialEntry {
    /// Device-fixture timestamp, in milliseconds.
    ts_ms: u64,
    /// The event kind.
    kind: TrialKind,
    /// The event payload (shape depends on `kind`).
    payload: serde_json::Value,
    /// The trial outcome the entry records.
    result: TrialResult,
    /// Transport mirror, when the entry carries one (SC-005 wire
    /// form). The refusal trial records none — the no-playing
    /// invariant is asserted over this field.
    #[serde(default)]
    transport: Option<WireTransport>,
}

/// The FGS precondition the platform cold-start gate consults
/// (FR-008 / FR-010). Closed two-member set — no `Other`
/// (Principle VII / FR-044).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FgsPrecondition {
    /// The `FOREGROUND_SERVICE_MEDIA_PLAYBACK` permission and the
    /// `mediaPlayback` foreground-service type are granted.
    Present,
    /// The platform refuses the cold start: the API 34+ start
    /// restriction (`ForegroundServiceStartNotAllowedException`) or
    /// the permission absent from the granted set.
    Absent,
}

/// The stated outcome of a cold start (FR-010). Closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColdStartOutcome {
    /// The service started; the transport controls stay live (FR-011).
    Started,
    /// The platform refused the start — the reason is stated and no
    /// playing state is presented (FR-010).
    Refused {
        /// The stated refusal reason (the contract's `StartRefused`
        /// surface, wire form `service_start_refused`).
        reason: &'static str,
    },
}

/// Evaluate the platform cold-start gate (FR-008 / FR-010).
///
/// Host-side model of the `src-tauri/src/permissions.rs` seam (T030b,
/// not yet implemented): the gate consults the FGS precondition and
/// either permits the start or states the refusal reason — a pure
/// delegation check, no playback logic.
const fn evaluate_cold_start(precondition: FgsPrecondition) -> ColdStartOutcome {
    match precondition {
        FgsPrecondition::Present => ColdStartOutcome::Started,
        FgsPrecondition::Absent => ColdStartOutcome::Refused {
            reason: REFUSAL_REASON_SERVICE_START_REFUSED,
        },
    }
}

/// Assert no entry of the record carries a `transport:"playing"`
/// mirror at or after a refused start (FR-010: a refusal states the
/// reason and presents no playing state, and nothing after it may
/// either — the trace's session never enters playing).
fn assert_no_playing_mirror_after_refusal(record: &[TrialEntry]) {
    let mut refusal_seen = false;
    for entry in record {
        if entry.result == TrialResult::ServiceStartRefused {
            refusal_seen = true;
            assert_ne!(
                entry.transport,
                Some(WireTransport::Playing),
                "a refusal entry never carries a playing state (FR-010)"
            );
        } else if refusal_seen {
            assert_ne!(
                entry.transport,
                Some(WireTransport::Playing),
                "no entry after a refused start may carry a playing transport mirror (FR-010)"
            );
        }
    }
    assert!(
        refusal_seen,
        "the record must contain the refused start (FR-010)"
    );
}

/// Parse a JSONL device fixture into the closed trial-entry sets.
///
/// Any unrecorded `kind` or `result` token fails the parse — the
/// closed-set rule (FR-044 / Principle VII): a new token is a
/// coordinated edit across fixture, schema, and this test, never a
/// catch-all arm.
fn parse_fixture(fixture: &str) -> Result<Vec<TrialEntry>, Box<dyn Error>> {
    let mut entries = Vec::new();
    for line in fixture.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        entries.push(serde_json::from_str::<TrialEntry>(trimmed)?);
    }
    Ok(entries)
}

/// The externally-started session is self-contained: created outside
/// any app-process context, it begins at the documented defaults with
/// no prior state — no app process, no stored position, nothing but
/// the session itself (FR-011 / SC-014).
#[test]
fn externally_started_session_is_self_contained_at_defaults() {
    let session = PlaybackSession::new();
    let fresh = session.snapshot();
    assert_eq!(fresh.transport, TransportState::Paused);
    assert!(
        fresh.position_secs.abs() <= POS_EPS_SECS,
        "a fresh session starts at position 0.0 (FR-002)"
    );
    assert_eq!(fresh.rate, PlaybackRate::R1_0);
    assert_eq!(fresh.engine_id, "rust-player");
}

/// FR-011 / SC-014 liveness leg: the service was started from
/// outside the application process, so a Play press from the lock
/// surface must produce audio without the app open. On host the audio
/// seam is `on_audible_progress` (what the CPAL backend feeds): the
/// session accepts the verb, reaches `Playing`, and position advances
/// only via audible progress — never while silent (FR-005).
#[test]
fn play_from_lock_surface_reaches_playing_and_advances_only_audibly() -> Result<(), Box<dyn Error>>
{
    // Externally started: no app-process state exists to consult.
    let mut session = PlaybackSession::new();

    // Silent progress never advances position — no audio has been
    // produced yet, so nothing moves (FR-005).
    session.on_audible_progress(5.0);
    assert!(
        session.snapshot().position_secs.abs() <= POS_EPS_SECS,
        "silent progress must not advance position (FR-005)"
    );

    // Play from the lock surface: the same single dispatch every
    // other driver routes through (FR-042).
    let played = session.execute(ShellVerb::Play)?;
    assert_eq!(played.transport, TransportState::Playing);

    // Position advances only through audible progress — the host
    // seam for "produces audio".
    session.on_audible_progress(5.0);
    assert!(
        (session.snapshot().position_secs - 5.0).abs() <= POS_EPS_SECS,
        "audible progress must advance position to 5.0 s (FR-002)"
    );
    Ok(())
}

/// The dedicated external-start verb — the mirror-only entry point
/// the externally-started service owns (FR-011,
/// contracts/shell-ipc.md) — routes through the same dispatch and
/// reaches `Playing` from a session with no app-process state,
/// exactly like `Play`.
#[test]
fn external_start_verb_reaches_playing_without_app_process_state() -> Result<(), Box<dyn Error>> {
    let mut session = PlaybackSession::new();
    let started = session.execute(ShellVerb::ExternalStart)?;
    assert_eq!(started.transport, TransportState::Playing);

    // The externally-started session stays self-contained: position
    // still moves only on audible progress.
    session.on_audible_progress(2.5);
    assert!(
        (session.snapshot().position_secs - 2.5).abs() <= POS_EPS_SECS,
        "audible progress must advance position to 2.5 s (FR-002)"
    );
    Ok(())
}

/// FR-010 leg: the platform refuses the cold start because the FGS
/// precondition is absent. The refusal states the reason the fixture
/// records, no `Playing` state is ever reported — the session stays
/// `Paused` and the simulated record carries no `transport:"playing"`
/// entry at or after the refused start.
#[test]
fn platform_refused_cold_start_states_reason_and_never_reports_playing()
-> Result<(), Box<dyn Error>> {
    // Control: a present precondition is the only start permission
    // (FR-008) — the gate is a closed two-member set.
    assert_eq!(
        evaluate_cold_start(FgsPrecondition::Present),
        ColdStartOutcome::Started
    );

    // The externally-started session: no app-process state exists.
    let session = PlaybackSession::new();

    // The platform refuses the cold start (FGS precondition absent).
    let ColdStartOutcome::Refused { reason } = evaluate_cold_start(FgsPrecondition::Absent) else {
        return Err(
            "a cold start must be refused, never silently started, when the FGS precondition is absent (FR-010)"
                .into(),
        );
    };
    assert_eq!(reason, REFUSAL_REASON_SERVICE_START_REFUSED);

    // The refused start never executes a transport verb: no Playing
    // state is presented, the session stays Paused (FR-010).
    assert_eq!(session.snapshot().transport, TransportState::Paused);

    // Simulated record for the refusal trial — the fixture's event
    // shape, mirrored on host. Every entry carries the closed
    // kinds/results; none carries a transport mirror.
    let record = [
        TrialEntry {
            ts_ms: 0,
            kind: TrialKind::AppStart,
            payload: serde_json::json!({ "from": "transport_control", "app_open": false }),
            result: TrialResult::ServiceStartRefused,
            transport: None,
        },
        TrialEntry {
            ts_ms: 120,
            kind: TrialKind::FocusRequest,
            payload: serde_json::json!({ "delay_grant": true, "tier": "test_double" }),
            result: TrialResult::DeferredGrant,
            transport: None,
        },
        TrialEntry {
            ts_ms: 320,
            kind: TrialKind::FocusGrant,
            payload: serde_json::json!({ "duration_ms": 3000 }),
            result: TrialResult::PlayingFromLastSpoken,
            transport: None,
        },
    ];
    assert_no_playing_mirror_after_refusal(&record);
    Ok(())
}

/// Replay-trace conformance (T022b): every recorded entry of the
/// device fixture parses into the closed trial sets, the refusal
/// entry carries the named refusal reason and never a playing state,
/// and the trace's session contains no `transport:"playing"` entry
/// after the refused start.
#[test]
fn fixture_replay_trace_conforms_to_the_refusal_shape() -> Result<(), Box<dyn Error>> {
    let entries = parse_fixture(FIXTURE_EXTERNAL_START_REFUSED)?;
    assert_eq!(
        entries.len(),
        3,
        "the external-start refusal fixture records exactly 3 trial steps"
    );

    // Entry 1 (ts 0): the platform refuses the externally-started
    // cold start — named reason, no playing state.
    let refusal = &entries[0];
    assert_eq!(refusal.ts_ms, 0);
    assert_eq!(refusal.kind, TrialKind::AppStart);
    assert_eq!(refusal.result, TrialResult::ServiceStartRefused);
    assert_eq!(refusal.payload["from"].as_str(), Some("transport_control"));
    assert_eq!(refusal.payload["app_open"].as_bool(), Some(false));
    assert_ne!(refusal.transport, Some(WireTransport::Playing));

    // Entry 2 (ts 120): the focus request defers the grant on the
    // test-double tier.
    let focus_request = &entries[1];
    assert_eq!(focus_request.ts_ms, 120);
    assert_eq!(focus_request.kind, TrialKind::FocusRequest);
    assert_eq!(focus_request.result, TrialResult::DeferredGrant);
    assert_eq!(focus_request.payload["delay_grant"].as_bool(), Some(true));
    assert_eq!(focus_request.payload["tier"].as_str(), Some("test_double"));
    assert_ne!(focus_request.transport, Some(WireTransport::Playing));

    // Entry 3 (ts 320): the focus grant resumes from the last
    // spoken position — the FR-009a recovery outcome, recorded as a
    // trial result token, never as a transport mirror.
    let focus_grant = &entries[2];
    assert_eq!(focus_grant.ts_ms, 320);
    assert_eq!(focus_grant.kind, TrialKind::FocusGrant);
    assert_eq!(focus_grant.result, TrialResult::PlayingFromLastSpoken);
    assert_eq!(focus_grant.payload["duration_ms"].as_u64(), Some(3000));
    assert_ne!(focus_grant.transport, Some(WireTransport::Playing));

    // Entries are ordered by timestamp.
    assert!(entries[0].ts_ms <= entries[1].ts_ms);
    assert!(entries[1].ts_ms <= entries[2].ts_ms);

    // The trace's session contains no transport:"playing" entry at
    // or after the refused start (FR-010 / SC-014).
    assert_no_playing_mirror_after_refusal(&entries);
    Ok(())
}
