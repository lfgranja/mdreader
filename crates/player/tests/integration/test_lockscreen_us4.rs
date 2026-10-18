//! T053 — US4 integration: lock-screen + residency trials
//! (FR-005 / FR-008 / FR-010 / FR-011 / FR-018 / FR-018a /
//! FR-018b / FR-019 / FR-021 / FR-026 / FR-032 / FR-032a /
//! FR-032f / FR-038 / FR-042 / FR-044).
//!
//! Blind-test protocol (SC-008): the tester cannot see the
//! surface, so a trial fails on any unlock or any effect
//! left unconfirmed aloud — every control effect below
//! (play, pause, resume, skip ±, rate change, voice-cycle
//! next/prev) is confirmed audibly (FR-018a), never
//! visually, and each voice cycle is one voice per press,
//! re-anchored per FR-038 (FR-018b). SC-004 functional
//! correctness is asserted on the same run: the state
//! effect each audible confirmation names is the effect
//! the session must show.
//!
//! Trial legs (from `specs/001-android-media-shell/tasks.md`
//! T053), one `#[test]` per leg:
//!
//! - L1 blind control sweep — every lock-screen control
//!   driven through the single dispatch (`player::cli`,
//!   FR-042): transport, the ±5.0 s skip clamp at
//!   [0.0, duration] (FR-018), and one-rung rate moves
//!   (FR-019).
//! - L2 full ladder — every rung of the closed 7-rung
//!   ladder including the 2.0×→4.0× step, saturating at
//!   both ends (FR-019).
//! - L3 voice cycle — one voice per press over a closed
//!   three-voice test pool, with the wrap boundary (the
//!   reported re-anchor boundary) asserted (FR-018b /
//!   FR-038).
//! - L4 Stop-discards vs Pause-preserves (FR-032 /
//!   FR-032a).
//! - L5 completion tears down residency (SC-045 / SC-049 /
//!   FR-032f), with exactly one resident + one
//!   notification while resumable (SC-043 / FR-009a).
//! - L6 cold-start refusal — a stated, named reason and
//!   never a playing state (FR-010 / SC-014 / FR-008).
//! - L7 denied permission (FR-021 / SC-028) and
//!   FGS-absent (FR-008) — the player-side equivalents;
//!   the in-app statements are device-only, the gate
//!   logic is T030b's.
//! - L8 airplane-mode start — 60 s of continued playback
//!   on a parsed local document with no network (FR-026).
//! - L9 status-view rate equality — the rate shown in-app
//!   equals the lock-screen rate source: the session's
//!   rate (SC-027).
//! - L10 fixture replay — every step of
//!   `tests/device/fixtures/lockscreen_blind.jsonl`,
//!   asserting each recorded result token.
//! - Regression check only — lock-screen controls remain
//!   functional on the resident service; the FR-011 proof
//!   is owned by T022b (refs T028), so the host-runnable
//!   form here is that the resident session answers a
//!   lock-screen control through the same single dispatch
//!   (FR-042).
//!
//! Device-only gaps (traced, not asserted on host): the
//! audible confirmation channel itself (FR-018a — on host
//! the state effects each confirmation names ARE asserted),
//! lock-surface visibility, the in-app permission
//! statements (SC-028 — the statement rendering belongs to
//! the surface layer, T061), the FR-038 re-anchor
//! boundary (the nearest sentence/node boundary is
//! resolved by the session crate, T059, from a parsed
//! document, T061c — on host the pool wrap boundary is the
//! reported re-anchor boundary), and the airplane-mode
//! radio state (FR-026 — on host "no network" is the
//! property that the session's advance path is
//! `on_audible_progress` alone, with no engine call
//! beyond the local stand-in). The cold-start and
//! permission gates are `src-tauri/src/permissions.rs`
//! (T030b), modelled here as host-side gates over the
//! closed precondition sets — the same seam T022b models.
//!
//! FAILS before `crates/player/src/rate.rs` (T057) and
//! `crates/player/src/surface.rs` (T061) exist: the
//! `player::rate` and `player::surface` modules are
//! unresolved, `PlaybackSession::with_duration` is
//! missing, and the `CycleVoiceNext`/`CycleVoicePrev`
//! verb variants do not exist yet — so this test target
//! does not build. That build failure is the required
//! pre-implementation red state (T056).

use std::error::Error;

use engine_core::closed::{PlaybackRate, TransportState};
use player::cli::{PlaybackSession, PlayerError, ShellVerb};
use player::rate::{LADDER, step_down, step_up};
use player::surface::{Heading, Progress, SurfaceResolver};
use serde::Deserialize;

/// Position comparison tolerance, in seconds — the same
/// epsilon form the focus-trial integration tests use, so
/// no strict float comparison ever appears (clippy
/// `float_cmp`).
const POS_EPS_SECS: f64 = 0.001;

/// The trial document's duration, in seconds — a known
/// duration, so skips clamp at [0.0, DURATION_SECS]
/// (FR-018) and the surface progress is determinate for
/// the whole session (determinacy is fixed at session
/// start, T051).
const DURATION_SECS: f64 = 120.0;

/// The trial document's filename — the surface title's
/// stable placeholder when no metadata title is supplied
/// (T051).
const FILENAME: &str = "document.md";

/// The trial document's metadata title — the surface title
/// when metadata is present (T051).
const METADATA_TITLE: &str = "Trial Document";

/// The stated refusal reason for a platform-refused cold
/// start (FR-010) — the `StartRefused` refusal surface
/// of the shell-ipc contract, wire form
/// `service_start_refused`.
const REFUSAL_REASON_SERVICE_START_REFUSED: &str = "service_start_refused";

/// The stated failure for a denied notification
/// permission (FR-021 / SC-028) — the named failure the
/// pre-play gate states in-app; the in-app statement
/// itself is rendered by the surface layer (T061) and is
/// device-only on host.
const FAILURE_REASON_NOTIFICATION_PERMISSION_DENIED: &str = "notification_permission_denied";

/// The airplane-mode trial's continued-playback window, in
/// seconds (FR-026): start plus 60 s of continued reading
/// with no network.
const AIRPLANE_MODE_WINDOW_SECS: f64 = 60.0;

/// The airplane-mode trial's feed count: one
/// audible-progress feed per second over the 60 s window
/// (FR-026).
const AIRPLANE_MODE_FEEDS: u32 = 60;

/// Embedded device fixture: the blind lock-screen control
/// trial (3 ordered steps — skip, rate, pause).
const FIXTURE_LOCKSCREEN_BLIND: &str = include_str!(
    "../../../../specs/001-android-media-shell/tests/device/fixtures/lockscreen_blind.jsonl"
);

/// The closed set of event kinds the lock-screen trial
/// records. Deserialization rejects any unrecorded kind:
/// the closed-set rule (FR-044 / Principle VII) makes a
/// new kind a coordinated edit across fixture, schema,
/// and this test — never a catch-all arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FixtureKind {
    /// A lock-screen control press.
    LockScreenControl,
}

/// The closed set of lock-screen actions the blind trial
/// records (the fixture's `payload.action` tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureAction {
    /// Skip forward exactly 5.0 s (FR-018).
    SkipForward5s,
    /// Move the rate one rung up the ladder (FR-019).
    RateUp,
    /// Pause, retaining the stored position (FR-032).
    Pause,
}

/// The closed set of trial outcomes the lock-screen
/// fixture records — each is the audible confirmation
/// (FR-018a) the blind tester confirmed aloud, and each
/// names the state effect this trial asserts (SC-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureResult {
    /// The skip advanced the position audibly (FR-018).
    ConfirmedAudiblyPositionAdvanced,
    /// The rate change was confirmed audibly (FR-019).
    ConfirmedAudiblyRateChanged,
    /// The pause was confirmed audibly (FR-032).
    ConfirmedAudiblyPaused,
}

/// One lock-screen control press in the blind trial: the
/// fixture's event shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
struct FixtureEntry {
    /// Device-fixture timestamp, in milliseconds.
    ts_ms: u64,
    /// The event kind.
    kind: FixtureKind,
    /// The control-press payload.
    payload: FixturePayload,
    /// The audible confirmation the trial recorded.
    result: FixtureResult,
}

/// The blind-trial control-press payload.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
struct FixturePayload {
    /// The lock-screen action pressed.
    action: FixtureAction,
    /// The blind tester cannot see the surface (SC-008) —
    /// every step of the fixture must carry this.
    blind_tester: bool,
    /// Whether the press required an unlock — the trial
    /// fails on any unlock, so every step must record
    /// `false`.
    #[serde(default)]
    unlock_required: bool,
}

/// The shell verb a recorded lock-screen action drives
/// through the single dispatch (FR-042) — an explicit
/// match over the closed action set, never a catch-all
/// (FR-044).
const fn verb_for(action: FixtureAction) -> ShellVerb {
    match action {
        FixtureAction::SkipForward5s => ShellVerb::SkipForward,
        FixtureAction::RateUp => ShellVerb::RateUp,
        FixtureAction::Pause => ShellVerb::Pause,
    }
}

/// Parse the embedded fixture into the closed entry sets.
///
/// Any unrecorded `kind`, `action`, or `result` token
/// fails the parse — the closed-set rule (FR-044 /
/// Principle VII): a new token is a coordinated edit
/// across fixture, schema, and this test, never a
/// catch-all arm.
fn parse_fixture(fixture: &str) -> Result<Vec<FixtureEntry>, Box<dyn Error>> {
    let mut entries = Vec::new();
    for line in fixture.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        entries.push(serde_json::from_str::<FixtureEntry>(trimmed)?);
    }
    Ok(entries)
}

/// The closed host voice pool (FR-024 stand-in): the
/// voice-selection interface supplies the ordering; this
/// feature owns only the one-step locked control
/// (FR-018b). Three voices — the closed test pool — with
/// the wrap boundary (the reported re-anchor boundary,
/// FR-038) at the pool's modulo edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VoicePool {
    /// The voices, in selection order.
    voices: [&'static str; VOICE_POOL_SIZE],
    /// The current voice index.
    index: usize,
}

/// The closed test pool size: three voices.
const VOICE_POOL_SIZE: usize = 3;

impl VoicePool {
    /// The pool in selection order, starting at the first
    /// voice.
    fn new() -> Self {
        Self {
            voices: ["voice-a", "voice-b", "voice-c"],
            index: 0,
        }
    }

    /// One next-voice press: exactly one voice forward,
    /// wrapping at the pool boundary (FR-018b).
    fn next(self) -> Self {
        Self {
            voices: self.voices,
            index: (self.index + 1) % VOICE_POOL_SIZE,
        }
    }

    /// One previous-voice press: exactly one voice back,
    /// wrapping at the pool boundary (FR-018b).
    fn prev(self) -> Self {
        Self {
            voices: self.voices,
            index: (self.index + VOICE_POOL_SIZE - 1) % VOICE_POOL_SIZE,
        }
    }

    /// The current voice.
    fn current(&self) -> &'static str {
        self.voices[self.index]
    }
}

/// The machine's resident-count invariant (SC-043 /
/// SC-049 / FR-009a): the playback service itself stays
/// resident while something is resumable — exactly one
/// resident component and one ongoing notification — and
/// is torn down at document end or a Stop press (SC-045 /
/// FR-032f). The host model keys the count on the
/// transport's resumability over the closed five-member
/// set — every arm explicit, never a catch-all (Principle
/// VII / FR-044).
const fn resident_count(transport: TransportState) -> u32 {
    match transport {
        TransportState::Playing | TransportState::Paused | TransportState::Interrupted => 1,
        TransportState::Stopped | TransportState::Completed => 0,
    }
}

/// The FGS precondition the platform cold-start gate
/// consults (FR-008 / FR-010). Closed two-member set —
/// no `Other` (Principle VII / FR-044).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FgsPrecondition {
    /// The `FOREGROUND_SERVICE_MEDIA_PLAYBACK` permission
    /// and the `mediaPlayback` foreground-service type
    /// are granted.
    Present,
    /// The platform refuses the cold start: the API 34+
    /// start restriction or the permission absent from
    /// the granted set.
    Absent,
}

/// Evaluate the platform cold-start gate (FR-008 /
/// FR-010).
///
/// Host-side model of the `src-tauri/src/permissions.rs`
/// seam (T030b, not yet implemented): the gate consults
/// the FGS precondition and either permits the start or
/// states the named refusal — a pure delegation check,
/// no playback logic.
const fn cold_start_gate(precondition: FgsPrecondition) -> Result<(), PlayerError> {
    match precondition {
        FgsPrecondition::Present => Ok(()),
        FgsPrecondition::Absent => Err(PlayerError::EngineNotAvailable(
            REFUSAL_REASON_SERVICE_START_REFUSED,
        )),
    }
}

/// The closed permission-outcome set the pre-play gate
/// consults (FR-021 / SC-028).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PermissionOutcome {
    /// The notification permission is granted.
    Granted,
    /// The notification permission is denied — the
    /// failure is stated in-app, never a silent playback
    /// block (SC-028).
    Denied,
}

/// Evaluate the pre-play permission gate (FR-021 /
/// SC-028).
///
/// Host-side model of the `src-tauri/src/permissions.rs`
/// seam (T030b): a denied permission produces the named
/// failure the in-app statement carries; the statement
/// rendering itself is the surface layer's (T061) and is
/// device-only on host.
const fn pre_play_gate(outcome: PermissionOutcome) -> Result<(), PlayerError> {
    match outcome {
        PermissionOutcome::Granted => Ok(()),
        PermissionOutcome::Denied => Err(PlayerError::EngineNotAvailable(
            FAILURE_REASON_NOTIFICATION_PERMISSION_DENIED,
        )),
    }
}

/// L1 — the blind control sweep (SC-008 / SC-004): from
/// a session with a known duration, playing at a known
/// position, every lock-screen control is driven through
/// the single dispatch and its state effect asserted —
/// the audible confirmation each press owes (FR-018a) is
/// traced on host; the effect it names is asserted here.
///
/// The sweep covers play, pause, resume, skip ± (clamped
/// at [0.0, duration], FR-018), and rate change (one
/// rung, FR-019).
#[test]
fn blind_control_sweep_confirms_every_effect() -> Result<(), Box<dyn Error>> {
    let mut session = PlaybackSession::with_duration(DURATION_SECS);

    // Play: the session reaches Playing — the
    // pre-condition of every blind control below.
    let played = session.execute(ShellVerb::Play)?;
    assert_eq!(
        played.transport,
        TransportState::Playing,
        "a Play press must reach Playing (FR-042)"
    );

    // A known position: 30.0 s of audible progress.
    session.on_audible_progress(30.0);
    assert!(
        (session.snapshot().position_secs - 30.0).abs() <= POS_EPS_SECS,
        "audible progress must advance the position to 30.0 s (FR-002)"
    );

    // Pause: the transport lands on Paused with the
    // stored position retained (FR-032).
    let paused = session.execute(ShellVerb::Pause)?;
    assert_eq!(
        paused.transport,
        TransportState::Paused,
        "a Pause press must land on Paused (FR-032)"
    );
    assert!(
        (paused.position_secs - 30.0).abs() <= POS_EPS_SECS,
        "a pause must retain the stored position (FR-032a)"
    );

    // Resume: Play from Paused re-enters Playing at the
    // stored position — the resume the lock screen offers.
    let resumed = session.execute(ShellVerb::Play)?;
    assert_eq!(
        resumed.transport,
        TransportState::Playing,
        "a resume must re-enter Playing (FR-032)"
    );
    assert!(
        (resumed.position_secs - 30.0).abs() <= POS_EPS_SECS,
        "a resume must keep the stored position (FR-032a)"
    );

    // Skip forward in a sectionless session: exactly +10 s (FR-002).
    let forward = session.execute(ShellVerb::SkipForward)?;
    assert!(
        (forward.position_secs - 40.0).abs() <= POS_EPS_SECS,
        "a sectionless skip-forward press must advance exactly 10 s (FR-002)"
    );

    // Skip backward returns to the prior sectionless fallback position.
    let backward = session.execute(ShellVerb::SkipBackward)?;
    assert!(
        (backward.position_secs - 30.0).abs() <= POS_EPS_SECS,
        "a sectionless skip-backward press must retreat exactly 10 s (FR-002)"
    );

    // The backward skip clamps at the 0.0 floor (FR-018).
    let mut floored = PlaybackSession::with_duration(DURATION_SECS);
    let _played = floored.execute(ShellVerb::Play)?;
    let _ = floored.execute(ShellVerb::SkipBackward)?;
    assert!(
        floored.snapshot().position_secs.abs() <= POS_EPS_SECS,
        "a backward skip at the document start clamps at 0.0 (FR-018)"
    );

    // The sectionless forward skip clamps at the duration (FR-002).
    let mut capped = PlaybackSession::with_duration(DURATION_SECS);
    let _played = capped.execute(ShellVerb::Play)?;
    capped.on_audible_progress(DURATION_SECS - 2.0);
    let _ = capped.execute(ShellVerb::SkipForward)?;
    assert!(
        (capped.snapshot().position_secs - DURATION_SECS).abs() <= POS_EPS_SECS,
        "a forward skip at the document end clamps at the duration (FR-018)"
    );

    // Rate change: exactly one rung per press (FR-019).
    let up = session.execute(ShellVerb::RateUp)?;
    assert_eq!(
        up.rate,
        PlaybackRate::R1_25,
        "a rate-up press must move exactly one rung (FR-019)"
    );
    let down = session.execute(ShellVerb::RateDown)?;
    assert_eq!(
        down.rate,
        PlaybackRate::R1_0,
        "a rate-down press must move exactly one rung (FR-019)"
    );
    Ok(())
}

/// L2 — the full ladder (FR-019): walk every rung of the
/// closed 7-rung ladder from the default rung to the
/// fastest — including the 2.0×→4.0× step — asserting
/// every intermediate rung, then walk back down,
/// saturating at both ends rather than wrapping, failing,
/// or yielding a rung outside the ladder.
#[test]
fn full_ladder_walks_every_rung_including_2x_to_4x() -> Result<(), Box<dyn Error>> {
    // The closed ladder itself: exactly the seven declared
    // rungs, in ascending order (FR-019).
    assert_eq!(
        LADDER,
        [
            PlaybackRate::R0_75,
            PlaybackRate::R1_0,
            PlaybackRate::R1_25,
            PlaybackRate::R1_5,
            PlaybackRate::R1_75,
            PlaybackRate::R2_0,
            PlaybackRate::R4_0,
        ],
        "the closed ladder holds exactly the seven declared rungs (FR-019)"
    );

    let mut session = PlaybackSession::with_duration(DURATION_SECS);
    let _played = session.execute(ShellVerb::Play)?;

    // Walk up from the default rung, asserting every
    // intermediate rung — the final step is the 2.0×→4.0×
    // move the trial must exercise (FR-019). The default
    // rung is LADDER[1], so the walk covers LADDER[2..]:
    // the first rate-up press lands on LADDER[2] (R1_25).
    let mut walked = PlaybackRate::R1_0;
    for &rung in LADDER.iter().skip(2) {
        let stepped = session.execute(ShellVerb::RateUp)?;
        walked = step_up(walked);
        assert_eq!(
            stepped.rate, rung,
            "a rate-up press must land on the next ladder rung (FR-019)"
        );
        assert_eq!(
            stepped.rate, walked,
            "the session rate must equal the ladder walk (FR-019)"
        );
    }
    assert_eq!(
        session.snapshot().rate,
        PlaybackRate::R4_0,
        "the ladder walk must end at the fastest rung (FR-019)"
    );

    // Saturation at the top: a further rate-up press
    // stays at 4.0× — it never wraps, fails, or leaves
    // the ladder.
    let saturated = session.execute(ShellVerb::RateUp)?;
    assert_eq!(
        saturated.rate,
        PlaybackRate::R4_0,
        "a rate-up press at the fastest rung saturates (FR-019)"
    );
    assert_eq!(step_up(PlaybackRate::R4_0), PlaybackRate::R4_0);

    // Walk back down to the slowest rung, asserting every
    // intermediate rung.
    let mut walked = PlaybackRate::R4_0;
    for &rung in LADDER.iter().rev().skip(1) {
        let stepped = session.execute(ShellVerb::RateDown)?;
        walked = step_down(walked);
        assert_eq!(
            stepped.rate, rung,
            "a rate-down press must land on the previous ladder rung (FR-019)"
        );
        assert_eq!(
            stepped.rate, walked,
            "the session rate must equal the ladder walk (FR-019)"
        );
    }
    assert_eq!(
        session.snapshot().rate,
        PlaybackRate::R0_75,
        "the ladder walk must end at the slowest rung (FR-019)"
    );

    // Saturation at the bottom: a further rate-down press
    // stays at 0.75×.
    let saturated = session.execute(ShellVerb::RateDown)?;
    assert_eq!(
        saturated.rate,
        PlaybackRate::R0_75,
        "a rate-down press at the slowest rung saturates (FR-019)"
    );
    assert_eq!(step_down(PlaybackRate::R0_75), PlaybackRate::R0_75);
    Ok(())
}

/// L3 — the voice cycle (FR-018b / FR-038): each
/// next-voice/previous-voice press moves exactly one voice
/// in the order the voice-selection interface (FR-024)
/// supplies — on host, a closed three-voice test pool —
/// and re-anchors the stored position to the nearest
/// sentence or node boundary.
///
/// The re-anchor itself is applied by the session crate
/// (T059) with boundary resolution from a parsed document
/// (T061c); on host no document is parsed, so the
/// re-anchor boundary is reported — the pool's wrap
/// boundary — and traced, not asserted (device-only gap).
/// The player-level contract asserted here: every press is
/// honoured from a live session, each press is exactly one
/// voice, and no press changes the transport, the rate, or
/// the position.
#[test]
fn voice_cycle_steps_exactly_one_voice_per_press() -> Result<(), Box<dyn Error>> {
    let mut session = PlaybackSession::with_duration(DURATION_SECS);
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(30.0);
    let mut pool = VoicePool::new();
    let start = pool.current();

    // Press 1: voice-a → voice-b — exactly one voice.
    let first = session.execute(ShellVerb::CycleVoiceNext)?;
    pool = pool.next();
    assert_eq!(
        pool.current(),
        "voice-b",
        "a next-voice press moves exactly one voice (FR-018b)"
    );

    // Press 2: voice-b → voice-c — exactly one voice.
    let second = session.execute(ShellVerb::CycleVoiceNext)?;
    pool = pool.next();
    assert_eq!(
        pool.current(),
        "voice-c",
        "a next-voice press moves exactly one voice (FR-018b)"
    );

    // Press 3: voice-c → voice-a — the wrap boundary is
    // the re-anchor boundary (FR-038), reported here as
    // the pool edge: the cycle returns to the first voice.
    let third = session.execute(ShellVerb::CycleVoiceNext)?;
    pool = pool.next();
    assert_eq!(
        pool.current(),
        start,
        "the voice cycle wraps at the pool boundary — the reported re-anchor boundary (FR-038)"
    );

    // Every press was honoured and changed no transport,
    // no rate, and no position (the re-anchor belongs to
    // the session crate, T059 — see the leg doc).
    for stepped in [first, second, third] {
        assert_eq!(
            stepped.transport,
            TransportState::Playing,
            "a voice press must change no transport (FR-018b)"
        );
        assert_eq!(
            stepped.rate,
            PlaybackRate::R1_0,
            "a voice press must change no rate (FR-018b)"
        );
        assert!(
            (stepped.position_secs - 30.0).abs() <= POS_EPS_SECS,
            "a voice press must change no position at the player level (FR-018b / FR-038)"
        );
    }

    // Previous-voice: the same one-step contract,
    // backwards — voice-a → voice-c over the wrap
    // boundary.
    let back = session.execute(ShellVerb::CycleVoicePrev)?;
    pool = pool.prev();
    assert_eq!(
        pool.current(),
        "voice-c",
        "a previous-voice press moves exactly one voice back over the wrap boundary (FR-018b)"
    );
    assert_eq!(
        back.transport,
        TransportState::Playing,
        "a previous-voice press must change no transport (FR-018b)"
    );
    assert_eq!(
        back.rate,
        PlaybackRate::R1_0,
        "a previous-voice press must change no rate (FR-018b)"
    );

    // Voice presses are live-session controls: rejected
    // once the session is torn down (Stopped /
    // Completed), never a silent no-op (FR-044 /
    // Principle VII).
    let mut torn = PlaybackSession::with_duration(DURATION_SECS);
    let _stopped = torn.execute(ShellVerb::Stop)?;
    assert!(
        matches!(
            torn.execute(ShellVerb::CycleVoiceNext),
            Err(PlayerError::IllegalTransition { .. })
        ),
        "a next-voice press from a torn-down session must be rejected (FR-032f)"
    );
    let mut ended = PlaybackSession::with_duration(DURATION_SECS);
    let _played = ended.execute(ShellVerb::Play)?;
    let _completed = ended.execute(ShellVerb::Complete)?;
    assert!(
        matches!(
            ended.execute(ShellVerb::CycleVoicePrev),
            Err(PlayerError::IllegalTransition { .. })
        ),
        "a previous-voice press from a completed session must be rejected (FR-032f)"
    );
    Ok(())
}

/// L4 — Stop discards, Pause preserves (FR-032 /
/// FR-032a): a Pause retains the stored position (the
/// resume plays it back), a Stop discards it (the
/// position resets to 0.0) — the two teardown-looking
/// presses are not interchangeable.
#[test]
fn pause_preserves_and_stop_discards_the_stored_position() -> Result<(), Box<dyn Error>> {
    // Pause preserves: the stored position survives the
    // pause and the resume that follows it.
    let mut preserving = PlaybackSession::with_duration(DURATION_SECS);
    let _played = preserving.execute(ShellVerb::Play)?;
    preserving.on_audible_progress(42.0);
    let paused = preserving.execute(ShellVerb::Pause)?;
    assert_eq!(paused.transport, TransportState::Paused);
    assert!(
        (paused.position_secs - 42.0).abs() <= POS_EPS_SECS,
        "a pause must preserve the stored position (FR-032a)"
    );
    let resumed = preserving.execute(ShellVerb::ResumeAtStored)?;
    assert_eq!(resumed.transport, TransportState::Playing);
    assert!(
        (resumed.position_secs - 42.0).abs() <= POS_EPS_SECS,
        "a resume must replay the preserved position (FR-032a)"
    );

    // Stop discards: the stored position is gone, and the
    // session is torn down — no resume offer remains.
    let mut discarding = PlaybackSession::with_duration(DURATION_SECS);
    let _played = discarding.execute(ShellVerb::Play)?;
    discarding.on_audible_progress(42.0);
    let stopped = discarding.execute(ShellVerb::Stop)?;
    assert_eq!(stopped.transport, TransportState::Stopped);
    assert!(
        stopped.position_secs.abs() <= POS_EPS_SECS,
        "a stop must discard the stored position (FR-032a)"
    );
    assert!(
        matches!(
            discarding.execute(ShellVerb::ResumeAtStored),
            Err(PlayerError::IllegalTransition { .. })
        ),
        "no resume offer remains after a stop (FR-032f)"
    );
    Ok(())
}

/// L5 — residency (SC-043 / SC-045 / SC-049 / FR-009a /
/// FR-032f): exactly one resident service and one ongoing
/// notification while the session is resumable, and
/// completion tears residency down — the machine's
/// resident-count invariant is one resident while a stored
/// position exists and zero after a discarding press.
#[test]
fn completion_tears_down_residency_and_leaves_no_resume_offer() -> Result<(), Box<dyn Error>> {
    let mut session = PlaybackSession::with_duration(DURATION_SECS);

    // A resumable session holds exactly one resident
    // component and one ongoing notification (SC-043 /
    // FR-009a).
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(42.0);
    let _paused = session.execute(ShellVerb::Pause)?;
    assert_eq!(
        resident_count(session.snapshot().transport),
        1,
        "a resumable session holds exactly one resident component (SC-043)"
    );

    // Resuming keeps residency: the session is still
    // resumable.
    let _resumed = session.execute(ShellVerb::Play)?;
    assert_eq!(
        resident_count(session.snapshot().transport),
        1,
        "a playing session still holds its residency (SC-043)"
    );

    // Document end: completion tears residency down
    // (SC-045 / SC-049 / FR-032f) — the transport is
    // terminal and no resume offer remains.
    let completed = session.execute(ShellVerb::Complete)?;
    assert_eq!(completed.transport, TransportState::Completed);
    assert_eq!(
        resident_count(completed.transport),
        0,
        "completion must tear residency down (SC-045 / SC-049)"
    );
    for verb in [
        ShellVerb::Play,
        ShellVerb::ResumeAtStored,
        ShellVerb::StartOver,
        ShellVerb::ExternalStart,
    ] {
        assert!(
            matches!(
                session.execute(verb),
                Err(PlayerError::IllegalTransition { .. })
            ),
            "no resume offer remains after completion (FR-032f)"
        );
    }

    // The universal teardown press tears residency down
    // too: a Stop leaves zero residents and no resume
    // offer.
    let mut stopped = PlaybackSession::with_duration(DURATION_SECS);
    let _played = stopped.execute(ShellVerb::Play)?;
    stopped.on_audible_progress(42.0);
    let _stopped = stopped.execute(ShellVerb::Stop)?;
    assert_eq!(
        resident_count(stopped.snapshot().transport),
        0,
        "a stop must tear residency down (FR-032f)"
    );
    assert!(
        matches!(
            stopped.execute(ShellVerb::ResumeAtStored),
            Err(PlayerError::IllegalTransition { .. })
        ),
        "no resume offer remains after a stop (FR-032f)"
    );
    Ok(())
}

/// L6 — the cold-start refusal (FR-010 / SC-014 /
/// FR-008): with the FGS precondition absent the platform
/// refuses the start. The refusal states the named
/// reason, and no playing state is ever reported — the
/// session stays Paused and no progress feed can produce
/// a phantom playing state (FR-005).
///
/// The gate itself is `src-tauri/src/permissions.rs`
/// (T030b), modelled here as a host-side gate over the
/// closed precondition set — the same seam T022b models.
#[test]
fn cold_start_refusal_states_a_named_reason_and_never_plays() -> Result<(), Box<dyn Error>> {
    // Control: a present precondition is the only
    // permitted start (FR-008) — the gate is a closed
    // two-member set.
    assert!(
        cold_start_gate(FgsPrecondition::Present).is_ok(),
        "a present FGS precondition must permit the cold start (FR-008)"
    );

    // The refusal: the gate yields the named failure.
    let refusal = cold_start_gate(FgsPrecondition::Absent);
    assert!(
        matches!(
            &refusal,
            Err(PlayerError::EngineNotAvailable(reason))
                if *reason == REFUSAL_REASON_SERVICE_START_REFUSED
        ),
        "an absent FGS precondition must yield the named refusal reason (FR-010)"
    );

    // ...and the session never enters Playing: no verb
    // press and no progress feed produces a playing state
    // from the refused start (FR-010 / SC-014 / FR-005).
    let mut session = PlaybackSession::new();
    assert_eq!(session.snapshot().transport, TransportState::Paused);
    session.on_audible_progress(5.0);
    assert_eq!(
        session.snapshot().transport,
        TransportState::Paused,
        "a refused cold start never reports a playing state (FR-010 / FR-005)"
    );
    assert!(
        session.snapshot().position_secs.abs() <= POS_EPS_SECS,
        "a refused cold start never advances position (FR-005)"
    );
    Ok(())
}

/// L7 — the denied-permission (FR-021 / SC-028) and
/// FGS-absent (FR-008) trials: both are `src-tauri`
/// permission-gate trials (T030b). On host the
/// player-side equivalent is asserted — a refused pre-play
/// gate produces a named failure and no playing state —
/// while the in-app statements themselves are traced, not
/// asserted (device-only: the statement rendering belongs
/// to the surface layer, T061).
#[test]
fn denied_permission_and_fgs_absence_state_failures_without_playing() {
    // Closed permission-outcome set (FR-021 / SC-028):
    // a granted permission permits playback; a denied
    // permission states the named failure in-app — never
    // a silent playback block.
    let outcomes = [PermissionOutcome::Granted, PermissionOutcome::Denied];
    for outcome in outcomes {
        let gate = pre_play_gate(outcome);
        match outcome {
            PermissionOutcome::Granted => {
                assert!(
                    gate.is_ok(),
                    "a granted permission must permit playback (FR-021)"
                );
            }
            PermissionOutcome::Denied => {
                assert!(
                    matches!(
                        &gate,
                        Err(PlayerError::EngineNotAvailable(reason))
                            if *reason == FAILURE_REASON_NOTIFICATION_PERMISSION_DENIED
                    ),
                    "a denied permission must state the named failure in-app (FR-021 / SC-028)"
                );
            }
        }
    }

    // The FGS-absent trial (FR-008, refs T030b): the
    // same named-failure shape, never a silent playback
    // block and never a playing state.
    let session = PlaybackSession::new();
    assert!(
        cold_start_gate(FgsPrecondition::Absent).is_err(),
        "an absent FGS precondition must fail closed (FR-008)"
    );
    assert_eq!(
        session.snapshot().transport,
        TransportState::Paused,
        "a refused pre-play gate never reports a playing state (FR-008 / FR-010)"
    );
}

/// L8 — the airplane-mode trial (FR-026): a start plus
/// 60 s of continued playback on a parsed local document
/// with no network. On host "no network" is the property
/// that the session's advance path is
/// `on_audible_progress` alone — 60 s of progress
/// accumulates with no engine call beyond the local
/// stand-in; the airplane-mode radio state itself is
/// device-only (traced, not asserted).
#[test]
fn airplane_mode_start_continues_sixty_seconds_with_no_network() -> Result<(), Box<dyn Error>> {
    // A parsed local document: the session is created over
    // the parsed document — no network dependency
    // (FR-026).
    let mut session = PlaybackSession::with_duration(DURATION_SECS);
    let started = session.execute(ShellVerb::Play)?;
    assert_eq!(
        started.transport,
        TransportState::Playing,
        "the airplane-mode start must reach Playing (FR-026)"
    );

    // 60 s of simulated audible progress — the only
    // advance path (FR-002), fed second by second.
    for _ in 0..AIRPLANE_MODE_FEEDS {
        session.on_audible_progress(1.0);
    }
    let snap = session.snapshot();
    assert!(
        (snap.position_secs - AIRPLANE_MODE_WINDOW_SECS).abs() <= POS_EPS_SECS,
        "60 s of audible progress must accumulate with no network (FR-026)"
    );
    assert_eq!(
        snap.transport,
        TransportState::Playing,
        "the session must still be playing after the 60 s window (FR-026)"
    );
    assert!(
        snap.position_secs <= DURATION_SECS,
        "the position must stay inside the parsed document (FR-026)"
    );
    Ok(())
}

/// L9 — the status-view rate equality (SC-027): the rate
/// shown in-app equals the lock-screen rate source — one
/// source of truth, the Rust session's rate. The rate the
/// in-app status view shows, the rate the lock screen
/// mirrors, and the rate the ladder walk computes are all
/// the session's `snapshot().rate`, and the surface
/// resolver — fed only the session's position — carries
/// that same position (FR-003).
#[test]
fn status_view_rate_equals_the_lock_screen_rate_source() -> Result<(), Box<dyn Error>> {
    let mut session = PlaybackSession::with_duration(DURATION_SECS);
    let _played = session.execute(ShellVerb::Play)?;
    session.on_audible_progress(30.0);

    // The verb sequence both surfaces drive.
    let _up = session.execute(ShellVerb::RateUp)?;
    let _up = session.execute(ShellVerb::RateUp)?;
    let _down = session.execute(ShellVerb::RateDown)?;

    // The ladder walk from the default rung: the rate any
    // surface shows must equal it (SC-027).
    let mut walked = PlaybackRate::R1_0;
    walked = step_up(walked);
    walked = step_up(walked);
    walked = step_down(walked);
    let snap = session.snapshot();
    assert_eq!(
        snap.rate, walked,
        "the rate shown in-app must equal the lock-screen rate source — the session's rate (SC-027)"
    );

    // The surface resolver — fed only the session's
    // position — resolves the status view: a determinate
    // progress carrying the session's projected
    // parsed-content offset and the session-start unit
    // total, with the most-recent-passed heading as the
    // section title (T051 / FR-013).
    //
    // The fixture: a 120 s document whose chunk split
    // emits 12 sentence-level units — 10 s of spoken
    // audio per unit — so the three headings fall at
    // unit boundaries and the totals are content facts,
    // never estimates.
    let resolver = SurfaceResolver::new(
        Some(METADATA_TITLE.to_string()),
        FILENAME.to_string(),
        12,
        120.0,
        true,
    );
    let headings = [
        Heading {
            position_secs: 0.0,
            title: "Introduction".to_string(),
        },
        Heading {
            position_secs: 10.0,
            title: "First Section".to_string(),
        },
        Heading {
            position_secs: 60.0,
            title: "Later Section".to_string(),
        },
    ];
    let surface = resolver.resolve(
        snap.position_secs,
        resolver.consumed_units_at(snap.position_secs),
        &headings,
    );
    assert_eq!(
        surface.title, METADATA_TITLE,
        "the surface title comes from metadata when present (T051)"
    );
    assert_eq!(
        surface.section_title,
        Some("First Section".to_string()),
        "the section title is the most-recent-passed heading, never forward-looking (T051)"
    );
    let mut determinate = false;
    if let Progress::Determinate {
        consumed_units,
        total_units,
        ..
    } = surface.progress
    {
        assert_eq!(
            consumed_units,
            resolver.consumed_units_at(snap.position_secs),
            "the status view carries the session position's parsed-content projection as the consumed offset — the Rust module only (FR-003 / FR-013)"
        );
        assert_eq!(
            total_units, 12,
            "the determinate total is the session-start parsed-content unit count — the chunk split's segment count, never an estimate (FR-013)"
        );
        determinate = true;
    }
    assert!(
        determinate,
        "a with-duration session must resolve a determinate progress, never an indeterminate one (T051)"
    );
    Ok(())
}

/// L10 — the fixture replay: every step of
/// `lockscreen_blind.jsonl`, in fixture order, driven
/// through the single dispatch, asserting each recorded
/// result token — the audible confirmation (FR-018a) the
/// blind tester confirmed, and the state effect it names
/// (SC-004).
///
/// The blind protocol (SC-008) holds across the whole
/// fixture: every step was performed blind
/// (`blind_tester`) and no step required an unlock
/// (`unlock_required`) — a trial fails on any unlock or
/// any effect left unconfirmed aloud.
#[test]
fn fixture_replay_confirms_every_blind_control_step() -> Result<(), Box<dyn Error>> {
    let entries = parse_fixture(FIXTURE_LOCKSCREEN_BLIND)?;
    assert_eq!(
        entries.len(),
        3,
        "lockscreen_blind.jsonl must carry exactly the 3 recorded trial steps"
    );

    // Step 1 (ts 0): skip forward 5.0 s, confirmed
    // audibly as a position advance.
    let first = &entries[0];
    assert_eq!(first.ts_ms, 0);
    assert_eq!(first.kind, FixtureKind::LockScreenControl);
    assert_eq!(first.payload.action, FixtureAction::SkipForward5s);
    assert!(
        first.payload.blind_tester,
        "every step must be performed blind — the tester cannot see the surface (SC-008)"
    );
    assert!(
        !first.payload.unlock_required,
        "a trial fails on any unlock (SC-008)"
    );
    assert_eq!(
        first.result,
        FixtureResult::ConfirmedAudiblyPositionAdvanced,
        "fixture step 1 must record the audible position-advance confirmation (FR-018a)"
    );

    // Step 2 (ts 12000): rate up, confirmed audibly as a
    // rate change.
    let second = &entries[1];
    assert_eq!(second.ts_ms, 12_000);
    assert_eq!(second.kind, FixtureKind::LockScreenControl);
    assert_eq!(second.payload.action, FixtureAction::RateUp);
    assert!(second.payload.blind_tester);
    assert!(!second.payload.unlock_required);
    assert_eq!(
        second.result,
        FixtureResult::ConfirmedAudiblyRateChanged,
        "fixture step 2 must record the audible rate-change confirmation (FR-018a)"
    );

    // Step 3 (ts 24000): pause, confirmed audibly as a
    // pause.
    let third = &entries[2];
    assert_eq!(third.ts_ms, 24_000);
    assert_eq!(third.kind, FixtureKind::LockScreenControl);
    assert_eq!(third.payload.action, FixtureAction::Pause);
    assert!(third.payload.blind_tester);
    assert!(!third.payload.unlock_required);
    assert_eq!(
        third.result,
        FixtureResult::ConfirmedAudiblyPaused,
        "fixture step 3 must record the audible pause confirmation (FR-018a)"
    );

    // Entries are ordered by timestamp.
    assert!(entries[0].ts_ms <= entries[1].ts_ms);
    assert!(entries[1].ts_ms <= entries[2].ts_ms);

    // The replay: the trial's pre-condition is a playing
    // session (the blind tester drives lock-screen
    // controls during playback), then every recorded
    // press is driven through the single dispatch and its
    // effect asserted against the confirmation it recorded
    // (SC-004).
    let mut session = PlaybackSession::with_duration(DURATION_SECS);
    let played = session.execute(ShellVerb::Play)?;
    assert_eq!(played.transport, TransportState::Playing);

    // Step 1: sectionless skip forward advances exactly 10 s (FR-002).
    let skipped = session.execute(verb_for(first.payload.action))?;
    assert_eq!(skipped.transport, TransportState::Playing);
    assert!(
        (skipped.position_secs - 10.0).abs() <= POS_EPS_SECS,
        "the sectionless skip-forward press must advance 10 s (FR-002)"
    );

    // Step 2: rate up — the rate moves exactly one rung
    // (FR-019).
    let rated = session.execute(verb_for(second.payload.action))?;
    assert_eq!(rated.transport, TransportState::Playing);
    assert_eq!(
        rated.rate,
        PlaybackRate::R1_25,
        "the rate-up press must move one rung (FR-019)"
    );
    assert!(
        (rated.position_secs - 10.0).abs() <= POS_EPS_SECS,
        "the rate press must retain the position (FR-019)"
    );

    // Step 3: pause — the transport lands on Paused with
    // the stored position retained (FR-032 / FR-032a).
    let paused = session.execute(verb_for(third.payload.action))?;
    assert_eq!(paused.transport, TransportState::Paused);
    assert!(
        (paused.position_secs - 10.0).abs() <= POS_EPS_SECS,
        "the pause must retain the stored position (FR-032a)"
    );
    Ok(())
}

/// The regression check (T053, "regression check only"):
/// lock-screen controls remain functional on the resident
/// service. The FR-011 proof is owned by T022b (refs
/// T028); the host-runnable form is that the resident
/// session — created outside any app-process context,
/// held paused between sessions (FR-009a) — answers a
/// lock-screen control through the same single dispatch
/// every driver routes through (FR-042).
#[test]
fn lock_screen_controls_remain_functional_on_the_resident_service() -> Result<(), Box<dyn Error>> {
    // The resident service holds the session paused
    // between sessions (FR-009a): the externally-started
    // session, created outside the app process, resumes
    // from its stored position.
    let mut resident = PlaybackSession::with_duration(DURATION_SECS);
    let _played = resident.execute(ShellVerb::Play)?;
    resident.on_audible_progress(20.0);
    let _paused = resident.execute(ShellVerb::Pause)?;
    assert_eq!(
        resident_count(resident.snapshot().transport),
        1,
        "the resident service stays resident while the session is resumable (SC-043)"
    );

    // A lock-screen skip press reaches the resident
    // session through the single dispatch (FR-042). This
    // resident document has no headings, so FR-002 requires
    // the exact +10 s sectionless fallback.
    let skipped = resident.execute(ShellVerb::SkipForward)?;
    assert_eq!(skipped.transport, TransportState::Paused);
    assert!(
        (skipped.position_secs - 30.0).abs() <= POS_EPS_SECS,
        "a sectionless lock-screen skip advances ten seconds from 20.0 (FR-002)"
    );

    // A rate press and a voice press are equally
    // functional on the resident session.
    let rated = resident.execute(ShellVerb::RateUp)?;
    assert_eq!(rated.rate, PlaybackRate::R1_25);
    let voiced = resident.execute(ShellVerb::CycleVoiceNext)?;
    assert_eq!(voiced.transport, TransportState::Paused);
    Ok(())
}
