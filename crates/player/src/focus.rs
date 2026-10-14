//! Focus classification (T038; FR-012 / FR-013 / FR-013a / FR-013b / FR-014 / FR-015 / FR-016 / FR-017a).
//!
//! Closed sets throughout, no `Other`/`Unknown`, no `_ =>` arm
//! (Principle VII): [`FocusOutcome`] (3 outcomes), [`FocusRegained`]
//! (direction event), [`FocusIntent`] (9 intents), [`FocusGrant`] (3
//! grant answers). Engine failure surfaces named
//! ([`engine_core::closed::EngineError`]).
//!
//! Request-shape assumptions (Kotlin side, `AudioFocusRequest`):
//! `AUDIOFOCUS_GAIN` + `USAGE_MEDIA` + `CONTENT_TYPE_SPEECH` +
//! `setWillPauseWhenDucked(true)` + `setAcceptsDelayedFocusGain(true)`
//! (T038). The classifier yields per FR-012: duckable transients duck
//! (FR-013, ADR-001 conscious deviation from the platform
//! pause-instead-of-duck recommendation), silence-demanding and
//! permanent losses pause with latest-event-wins supersede (FR-013a /
//! FR-014), deferred grants hold zero audio and refused grants fail
//! named (FR-013b).

use engine_core::closed::{EngineError, TransportState};

/// Minimum duck gain factor (FR-013): 15%.
pub const DUCK_GAIN_MIN: f32 = 0.15;

/// Maximum duck gain factor (FR-013): 25%.
pub const DUCK_GAIN_MAX: f32 = 0.25;

/// Nominal duck gain factor (FR-013): ~20% while ducked.
pub const DUCK_GAIN: f32 = 0.20;

/// Full gain factor: playback at normal volume (restored).
pub const FULL_GAIN: f32 = 1.0;

/// Audio-focus outcome — the closed three-member set
/// (FR-013 / FR-013a / FR-014 / FR-017). No `Other`;
/// a permanent loss pauses, a duckable transient ducks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusOutcome {
    /// Transient loss that may duck (~20%, ADR-001):
    /// keep reading at reduced gain, never pause.
    DuckableTransient,
    /// Transient loss that must silence playback:
    /// pause immediately, no ducking.
    SilenceDemanding,
    /// Permanent loss (call, alarm): pause immediately,
    /// reason expires after 3 s, never auto-resume (FR-014).
    PermanentLoss,
}

/// Direction of a focus regain — the single-member closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FocusRegained {
    /// Focus returned via a direction event (regain, not grant).
    DirectionEvent,
}

/// Closed Kotlin→Rust intent set (contracts/shell-ipc.md): exactly
/// the nine intents below — no `Other`. Device identity travels
/// alongside the intent (diagnostic record), never inside it, so the
/// set stays fieldless.
///
/// The seven pre-existing intents each map to exactly one outcome
/// (FR-013, FR-013a, FR-014, FR-015, FR-016, FR-017a); the two
/// lifecycle intents carry no playback decision and latch only
/// (FR-032c / FR-023a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusIntent {
    /// Transient loss that may duck (FR-013).
    DuckableTransient,
    /// Transient loss demanding silence, e.g. call/alarm (FR-013a).
    SilenceDemanding,
    /// Permanent loss of audio focus (FR-014).
    PermanentLoss,
    /// Focus returned: authorizes resume via the rewind engine
    /// (FR-015; resume math lives in `crate::rewind`).
    /// Wire form is `regained` per contracts/shell-ipc.md (not the
    /// `snake_case` default `focus_regained`).
    #[serde(rename = "regained")]
    FocusRegained,
    /// Headphone-removal/noisy signal: pause, ≤1 s bound owned by
    /// the routing machine (FR-016; T046 owns timing).
    BecomingNoisy,
    /// Output device appeared: follow at unchanged position, never
    /// private→speaker without a pause-first (FR-017a; T046 owns
    /// the switch).
    DeviceAppeared,
    /// Output device disappeared: pause first, position preserved,
    /// explicit resume only (FR-016 / FR-017a).
    DeviceDisappeared,
    /// Lifecycle latch only: no playback decision, never converts an
    /// open foregrounded offer early (FR-032c / FR-023a).
    AppForegrounded,
    /// Lifecycle latch only: no playback decision (FR-032c).
    AppBackgrounded,
}

/// Stateless classification of one intent: outcome + transport + gain.
///
/// `outcome`/`transport` are `None` exactly when the intent carries
/// no playback decision of its own: the lifecycle intents latch only
/// (`foreground_latched`), and follow-only intents (`DeviceAppeared`,
/// `FocusRegained`) leave the outcome to their owning path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusClassification {
    /// Classified outcome, or `None` when the intent decides no
    /// outcome (latch-only or follow-only).
    pub outcome: Option<FocusOutcome>,
    /// Transport to mirror, or `None` when the intent changes no
    /// transport (lifecycle latch-only).
    pub transport: Option<TransportState>,
    /// Gain to apply: [`DUCK_GAIN`] while ducking, else
    /// [`FULL_GAIN`] (silence paths restore gain).
    pub gain: f32,
    /// Foreground latch written by lifecycle intents only:
    /// `Some(true)` on [`FocusIntent::AppForegrounded`],
    /// `Some(false)` on [`FocusIntent::AppBackgrounded`], else
    /// `None` (consulted at offer expiry, FR-032c / FR-023a).
    pub foreground_latched: Option<bool>,
}

/// Classify one intent into outcome + transport + gain (FR-013 /
/// FR-013a / FR-014 / FR-015 / FR-016 / FR-017a).
///
/// Nine explicit arms — one per [`FocusIntent`] member, never a
/// wildcard (Principle VII). Routing intents report the pause/follow
/// response here; the ≤1 s timing and device-switch rules belong to
/// the routing machine (T046), not to this mapping.
#[must_use]
pub const fn classify(intent: FocusIntent) -> FocusClassification {
    match intent {
        FocusIntent::DuckableTransient => FocusClassification {
            outcome: Some(FocusOutcome::DuckableTransient),
            transport: Some(TransportState::Playing),
            gain: DUCK_GAIN,
            foreground_latched: None,
        },
        FocusIntent::SilenceDemanding => FocusClassification {
            outcome: Some(FocusOutcome::SilenceDemanding),
            transport: Some(TransportState::Paused),
            gain: FULL_GAIN,
            foreground_latched: None,
        },
        FocusIntent::PermanentLoss => FocusClassification {
            outcome: Some(FocusOutcome::PermanentLoss),
            transport: Some(TransportState::Paused),
            gain: FULL_GAIN,
            foreground_latched: None,
        },
        FocusIntent::FocusRegained => FocusClassification {
            outcome: None,
            transport: Some(TransportState::Playing),
            gain: FULL_GAIN,
            foreground_latched: None,
        },
        FocusIntent::BecomingNoisy => FocusClassification {
            outcome: Some(FocusOutcome::SilenceDemanding),
            transport: Some(TransportState::Paused),
            gain: FULL_GAIN,
            foreground_latched: None,
        },
        FocusIntent::DeviceAppeared => FocusClassification {
            outcome: None,
            transport: Some(TransportState::Playing),
            gain: FULL_GAIN,
            foreground_latched: None,
        },
        FocusIntent::DeviceDisappeared => FocusClassification {
            outcome: Some(FocusOutcome::PermanentLoss),
            transport: Some(TransportState::Paused),
            gain: FULL_GAIN,
            foreground_latched: None,
        },
        FocusIntent::AppForegrounded => FocusClassification {
            outcome: None,
            transport: None,
            gain: FULL_GAIN,
            foreground_latched: Some(true),
        },
        FocusIntent::AppBackgrounded => FocusClassification {
            outcome: None,
            transport: None,
            gain: FULL_GAIN,
            foreground_latched: Some(false),
        },
    }
}

/// Latest-event-wins supersede flags (FR-014).
///
/// A permanent loss superseding any transient clears both the resume
/// flag and the duck-restore flag; a late transient arriving after a
/// permanent loss authorizes nothing (flags stay as they are and the
/// tracker keeps the paused state).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupersedeFlags {
    /// Clear the pending transient-resume flag.
    pub clear_resume_flag: bool,
    /// Clear the pending duck-restore (gain) flag.
    pub clear_duck_restore_flag: bool,
}

/// Supersede flags for an outcome transition (FR-014).
///
/// Nine explicit arms over the closed 3×3 outcome product — never a
/// wildcard (Principle VII). Escalation into [`FocusOutcome::PermanentLoss`]
/// clears both flags; every other transition clears nothing (a late
/// transient after a permanent loss authorizes nothing; steady or
/// de-escalating pairs keep the flags the tracker already holds).
#[must_use]
pub const fn supersede_flags(previous: FocusOutcome, next: FocusOutcome) -> SupersedeFlags {
    match (previous, next) {
        (FocusOutcome::DuckableTransient, FocusOutcome::PermanentLoss) => SupersedeFlags {
            clear_resume_flag: true,
            clear_duck_restore_flag: true,
        },
        (FocusOutcome::SilenceDemanding, FocusOutcome::PermanentLoss) => SupersedeFlags {
            clear_resume_flag: true,
            clear_duck_restore_flag: true,
        },
        (FocusOutcome::DuckableTransient, FocusOutcome::DuckableTransient) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
        (FocusOutcome::DuckableTransient, FocusOutcome::SilenceDemanding) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
        (FocusOutcome::SilenceDemanding, FocusOutcome::DuckableTransient) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
        (FocusOutcome::SilenceDemanding, FocusOutcome::SilenceDemanding) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
        (FocusOutcome::PermanentLoss, FocusOutcome::DuckableTransient) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
        (FocusOutcome::PermanentLoss, FocusOutcome::SilenceDemanding) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
        (FocusOutcome::PermanentLoss, FocusOutcome::PermanentLoss) => SupersedeFlags {
            clear_resume_flag: false,
            clear_duck_restore_flag: false,
        },
    }
}

/// Stateful focus tracker: current outcome, duck/resume flags, and
/// the foreground latch (FR-014 latest-event-wins + FR-032c latch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusTracker {
    /// Current classified outcome, or `None` before the first
    /// loss or after a regain resolved it.
    pub current: Option<FocusOutcome>,
    /// Whether playback is currently ducked (gain at [`DUCK_GAIN`]).
    pub ducked: bool,
    /// Whether a transient resume (with rewind) is owed.
    pub resume_due: bool,
    /// Whether a gain restore to [`FULL_GAIN`] is owed.
    pub duck_restore_due: bool,
    /// Latched application lifecycle: true while foregrounded,
    /// consulted at offer expiry only (FR-032c / FR-023a).
    pub foregrounded: bool,
}

impl FocusTracker {
    /// New tracker: no loss classified, not ducked, nothing owed,
    /// foregrounded (the app starts in the foreground).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current: None,
            ducked: false,
            resume_due: false,
            duck_restore_due: false,
            foregrounded: true,
        }
    }

    /// Apply one intent: update the latch/flags and return the
    /// classification (FR-014 latest-event-wins).
    ///
    /// Lifecycle intents latch only and change nothing else. A late
    /// [`FocusOutcome::DuckableTransient`] arriving while a permanent
    /// loss stands authorizes nothing: the paused state holds and no
    /// duck is applied. Escalation into permanent loss clears the
    /// resume and duck-restore flags via [`supersede_flags`].
    /// [`FocusIntent::FocusRegained`] resolves the current loss and
    /// marks a transient resume owed (resume math lives in
    /// `crate::rewind`).
    pub fn apply(&mut self, intent: FocusIntent) -> FocusClassification {
        if intent == FocusIntent::AppForegrounded {
            self.foregrounded = true;
            return classify(intent);
        }
        if intent == FocusIntent::AppBackgrounded {
            self.foregrounded = false;
            return classify(intent);
        }
        if intent == FocusIntent::FocusRegained {
            self.current = None;
            self.ducked = false;
            self.duck_restore_due = false;
            self.resume_due = true;
            return classify(intent);
        }
        let decision = classify(intent);
        if let Some(next) = decision.outcome {
            if let Some(previous) = self.current {
                if previous == FocusOutcome::PermanentLoss
                    && next == FocusOutcome::DuckableTransient
                {
                    // Late transient after permanent loss authorizes
                    // nothing: hold paused at full gain.
                    return FocusClassification {
                        outcome: Some(FocusOutcome::PermanentLoss),
                        transport: Some(TransportState::Paused),
                        gain: FULL_GAIN,
                        foreground_latched: None,
                    };
                }
                let flags = supersede_flags(previous, next);
                if flags.clear_resume_flag {
                    self.resume_due = false;
                }
                if flags.clear_duck_restore_flag {
                    self.duck_restore_due = false;
                }
                // Clear stale duck-restore flag when silence-demanding
                // supersedes a transient per spec.md latest-event-wins (FR-013 / T091/T101)
                if next == FocusOutcome::SilenceDemanding {
                    self.duck_restore_due = false;
                }
            }
            self.current = Some(next);
            // FR-005 / FR-014: a pause-demanding outcome
            // (silence-demanding or permanent loss) clears any
            // pending resume an earlier regain owed — a gain
            // followed by a pause waits for explicit Play and
            // never auto-resumes. This must hold whatever
            // stood before it, INCLUDING no standing outcome:
            // a regain leaves `current = None` with the resume
            // still owed, so the supersede path above (which
            // runs only when a previous outcome stood) does
            // not cover it — the auto-resume defect the
            // prop_focus8 property caught.
            if next != FocusOutcome::DuckableTransient {
                self.resume_due = false;
            }
            if next == FocusOutcome::DuckableTransient {
                self.ducked = true;
                self.duck_restore_due = true;
            } else {
                self.ducked = false;
            }
        }
        decision
    }
}

impl Default for FocusTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Closed focus-grant answer set (FR-013b).
///
/// The request assumes `setAcceptsDelayedFocusGain(true)`, so a
/// delayed grant is an explicit arm — not silence, not an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusGrant {
    /// Focus granted: audio permitted.
    Granted,
    /// Grant deferred: zero audio before the grant, no rewind owed,
    /// no FR-041 reason stated.
    Deferred,
    /// Focus refused: named failure, never a playing state.
    Refused,
}

/// Apply a focus-grant answer (FR-013b).
///
/// Returns whether audio is permitted: granted permits it, deferred
/// holds zero audio until the grant arrives (no rewind, no reason).
///
/// # Errors
///
/// Returns [`EngineError::StartRefused`] when focus is refused: the
/// refusal is a named failure and playback never reports playing.
pub const fn apply_grant(grant: FocusGrant) -> Result<bool, EngineError> {
    match grant {
        FocusGrant::Granted => Ok(true),
        FocusGrant::Deferred => Ok(false),
        FocusGrant::Refused => Err(EngineError::StartRefused),
    }
}

/// Duck band [0.15, 0.25] (FR-013) — must remain `Playing`; gain ≈20%.
/// SilenceDemanding / PermanentLoss → `Paused` / `Interrupted`; never auto-resume.
/// Supersede transient→permanent clears resume/duck-restore flags.
/// Deferred grant → zero audio before grant, no rewind; refusal → named failure + no playing state.
///
/// # Errors
///
/// Returns [`EngineError::DeviceRefused`] for an unrecognized
/// request string (closed intent set: only the three outcome names
/// classify here; [`classify`] covers the full nine intents).
pub fn classify_focus(request: &str) -> Result<FocusOutcome, EngineError> {
    if request == "DuckableTransient" {
        Ok(FocusOutcome::DuckableTransient)
    } else if request == "SilenceDemanding" {
        Ok(FocusOutcome::SilenceDemanding)
    } else if request == "PermanentLoss" {
        Ok(FocusOutcome::PermanentLoss)
    } else {
        Err(EngineError::DeviceRefused)
    }
}

/// Volume duck ≈20% (FR-013). Never alters position; no rewind owed while Playing.
///
/// # Errors
///
/// Returns [`EngineError::RateMismatch`] when the factor lies outside
/// the closed duck band [[`DUCK_GAIN_MIN`], [`DUCK_GAIN_MAX`]].
pub fn apply_duck(volume_factor: f32) -> Result<f32, EngineError> {
    if volume_factor < DUCK_GAIN_MIN || volume_factor > DUCK_GAIN_MAX {
        return Err(EngineError::RateMismatch);
    }
    Ok(volume_factor)
}
