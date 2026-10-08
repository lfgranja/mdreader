//! Design-phase synthesis wrapper that fails closed
//! (`API_CONTRACT.md` §7; FR-024b / SC-035; R2 / R7).
//!
//! The Piper `faber` voice model is not bound in this design-phase crate
//! (`API_CONTRACT.md` §10 places the engine binding in `003-contract-edit`),
//! so initialisation cannot succeed: [`synthesize`] always reports the named
//! failure `FailureCondition::VoiceInitFailure` → `EngineError::InitFailed`
//! through the fail-closed constructor [`EngineFailure::report`], instead of
//! returning an empty buffer — an empty buffer would be a *silent* success
//! that plays nothing and reports nothing, exactly the failure mode SC-035
//! and data-model rules R2/R7 forbid ("a silent branch is worse than a
//! failed build"). The old `synthesize_chunk` stub returned an empty buffer
//! as a silent success; it is deleted, not aliased — a second entry point
//! returning a silent success is the defect itself.
//!
//! The real `sherpa-onnx` + Piper pt-BR (`faber`) binding — model load,
//! synthesis, mono `f32` delivery at the declared 22 050 Hz — belongs to
//! `003-contract-edit`. Until then this module owns no playback logic: the
//! `player` is the sole playback authority (`constitution.md`:486-490;
//! SC-035), and the `session_position` a failure carries is echoed from the
//! segment, never computed here (segment-resume.md CHK030).

use engine_core::closed::EngineTier;

use crate::chunk::{DocumentSegment, SourceRange};
use crate::failure::{EngineFailure, FailureCondition, FailureReportError, MAX_OFFER_MS};

/// Synthesize one bounded segment to mono `f32` at 22 050 Hz.
///
/// The Piper `faber` voice model is not bound in this design-phase crate
/// (`plan.md` §Structure Decision; integration belongs to `003-contract-edit`).
/// Until it is bound, initialisation cannot succeed — so this reports the
/// named failure `InitFailed` instead of returning an empty buffer. An empty
/// buffer would be a *silent* success for a contract that requires named
/// failure over silence (R2, R7, `SC-035`). The `session_position` the
/// failure carries is echoed from the segment's `resume_point` (falling back
/// to its `source_range` when no resume point is declared) — never
/// adapter-computed (segment-resume.md CHK030; the `player` is the sole
/// playback authority).
///
/// # Errors
///
/// Always [`EngineFailure`]: the unbound voice model is reported as
/// `FailureCondition::VoiceInitFailure` → `EngineError::InitFailed`, with
/// its stable `snake_case` identifier as `observed`, an offer of
/// [`MAX_OFFER_MS`], and `EngineTier::OfflineDefault` as the tier at
/// failure (R11).
pub fn synthesize(segment: &DocumentSegment) -> Result<Vec<f32>, EngineFailure> {
    // The session position is echoed from the segment, never computed
    // here: the resume point when the segment declares one, else the
    // segment's own source range (segment-resume.md CHK030;
    // constitution.md:486-490 — the player is the sole playback
    // authority).
    let Some(resume_point) = segment.resume_point else {
        return report_voice_init_failure(segment.source_range);
    };
    report_voice_init_failure(resume_point)
}

/// Report the design-phase voice-init failure for one echoed session
/// position, through the fail-closed constructor [`EngineFailure::report`]
/// so its bounds cannot be bypassed.
///
/// # Errors
///
/// Always [`EngineFailure`]: the unbound voice model is reported as
/// `FailureCondition::VoiceInitFailure` → `EngineError::InitFailed`, with
/// its stable `snake_case` identifier as `observed`, an offer of
/// [`MAX_OFFER_MS`], and `EngineTier::OfflineDefault` as the tier at
/// failure (R11).
fn report_voice_init_failure(session_position: SourceRange) -> Result<Vec<f32>, EngineFailure> {
    match EngineFailure::report(
        FailureCondition::VoiceInitFailure,
        FailureCondition::VoiceInitFailure.detail(),
        session_position,
        MAX_OFFER_MS,
    ) {
        Ok(failure) => Err(failure),
        // The constructor's bounds cannot refuse these compile-time
        // constants: the detail is the fixed 17-char identifier
        // `voice_init_failure` against the 200-char bound, and the
        // offer is exactly MAX_OFFER_MS against the same bound (the
        // refusal check is a strict `>`). This arm is therefore
        // unreachable at runtime, but it is still handled — the same
        // named failure is reported with the same constants — so the
        // named-failure-over-silence rule holds even in the impossible
        // refusal world.
        Err(FailureReportError::DetailTooLong { .. } | FailureReportError::OfferTooLong { .. }) => {
            Err(EngineFailure {
                variant: FailureCondition::VoiceInitFailure.variant(),
                condition: FailureCondition::VoiceInitFailure,
                observed: FailureCondition::VoiceInitFailure.detail().to_owned(),
                session_position,
                offer_expires_in_ms: MAX_OFFER_MS,
                tier_at_failure: EngineTier::OfflineDefault,
            })
        }
    }
}
