//! Interruption reasons — closed 8-reason set (FR-005 / data-model.md).
//!
//! Measurement note (T001b):
//!   Fixture byte size: `tests/device/fixtures/fixture-pt-2h.md`
//!   Measured via `stat -c %s` → 62424 bytes.
//!   MAX_DOC_BYTES = 62424 × 1.5 = 93636 (headroom for ~3h at 1.0x).
//!   Wire reference: FR-018 ingest refusal → `FormatUndeliverable` when exceeded.

/// Fixture-scale document byte cap (FR-018).
///
/// Fixed at plan time from the measured fixture (`fixture-pt-2h.md`:
/// 62424 bytes via `stat -c %s`). Headroom factor 1.5 yields support
/// for ~3 hours at 1.0x speed. Oversize documents refuse before
/// synthesis with `FormatUndeliverable`.
pub const MAX_DOC_BYTES: usize = 93636;

/// The closed interruption-reason set (no `Other` / `Unknown`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterruptionReason {
    /// Audio focus refused (e.g. call demands silence).
    FocusDenied,
    /// Focus acquisition delayed; zero-audio hold.
    FocusDelayed,
    /// Permanent audio-focus loss (e.g. headset removed, call won).
    PermanentLoss,
    /// Transient interruption; may duck or pause depending on class.
    TransientPaused,
    /// Output device removed; pauses with latest reason.
    DeviceRemoved,
    /// Voice/synthesis unavailable before playback starts.
    VoiceUnavailable,
    /// Document format or size exceeds deliverable bounds.
    FormatUndeliverable,
    /// Engine decode failure; never resumes from this state.
    DecodeFailed,
}

impl InterruptionReason {
    /// Exhaustive match — no `_ =>` (gate 7). ≤8-word line stub.
    pub fn reason_line(&self) -> &'static str {
        match self {
            InterruptionReason::FocusDenied => "Foco negado — tente novamente",
            InterruptionReason::FocusDelayed => "Atraso no áudio — aguardando",
            InterruptionReason::PermanentLoss => "Perda permanente — áudio parado",
            InterruptionReason::TransientPaused => "Interrupção transitória — pausado",
            InterruptionReason::DeviceRemoved => "Dispositivo removido — pausado",
            InterruptionReason::VoiceUnavailable => "Voz indisponível — falha",
            InterruptionReason::FormatUndeliverable => "Formato não suportado",
            InterruptionReason::DecodeFailed => "Decodificação falhou — erro",
        }
    }
}
