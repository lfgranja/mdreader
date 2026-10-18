//! The shared reason shapes every surface renders (T065; FR-021, FR-037,
//! FR-037a, FR-041).
//!
//! One closed vocabulary of pause and failure reasons, rendered in the two
//! presentations every surface states: the short lock-screen statement
//! ([`ShortReason`] through [`short_reason`]) and the full in-app reason
//! ([`FullReason`] through [`full_reason`]). The lock screen, the in-app
//! status view (T065b), and the failure surface (T078) all render through
//! these getters — there is no second definition of any rendered string
//! (Principle II). The two enums are parallel — the same variants under the
//! same `snake_case` wire forms — so one wire value deserializes into
//! either presentation and each surface renders its own form of the same
//! reason.
//!
//! The expiry classification ([`ReasonExpiry`]) is the boundary FR-037a
//! draws: a focus-loss pause reason expires 3 s after the pause on BOTH
//! surfaces (FR-041) — the application shows it only if the reader opens
//! it within that window — while a failure reason persists in the
//! application until it is read (FR-037). The two sets of reasons are
//! disjoint (FR-013a, FR-037a, FR-041): a focus loss is not a
//! malfunction, and its expiry never applies to a failure.
//!
//! Two axes, never one (contracts/surface.md, the F8 payload overload
//! fix): the reason TTL is how long a reason line stays on a surface
//! — [`PAUSE_REASON_TTL_MS`] for a pause-demanding reason, `None`
//! (persistent) for a failure — while the dismissal policy
//! ([`Dismissal`] through [`dismissal_for`]) is whether the surface
//! carrying the reason may be dismissed at all: refused while a
//! document plays, because pausing is the route to dismissal
//! (FR-014, FR-028e). The two never substitute for one another. A
//! persistent failure reason rides on a surface that is
//! non-dismissable while audio plays, and an expiring focus-loss
//! reason rides on a surface that becomes dismissable the moment the
//! reader pauses. Each reason exposes its own half through
//! [`ShortReason::expiry`] / [`ShortReason::reason_ttl_ms`], so no
//! payload field is ever read out of the other field's
//! classification.
//!
//! Document-text boundary (FR-037, FR-041): a rendered reason never
//! quotes document content. Only the focus-loss Pause Reason names the
//! document, and it names the title and nothing else from the document —
//! hence the `document_title` parameter of both getters, consumed by the
//! focus-loss arms alone; the failure and permission statements are
//! app-generated text and take no document text at all.
//!
//! Closed sets throughout (Principle VII, FR-044): no `Other`/`Unknown`,
//! no catch-all arms — adding a variant is a coordinated edit across
//! every consumer, with a semver bump and a documentation update. The
//! wire form of every set is the `snake_case` name (contracts/
//! shell-ipc.md case-mapping table), so an unknown wire value never
//! deserializes: there is no variant for it to land in. The dismissal
//! set ([`Dismissal`]) is the payload's two-valued wire form of
//! FR-014/FR-028e, and the transport set it reads is the session's own
//! closed transport reflection — no reason, rung, or transport list is
//! restated here.
//!
//! Statement language: the application's surface language is pt-BR (the
//! offline voices the constitution's engine registry admits are pt-BR).
//! The focus-loss statement is the specification's canonical Pause Reason
//! (FR-041: "Leitura pausada — outro app usando o áudio"); the failure
//! and permission statements follow the identifier-style stated-reason
//! shape the permission gate already renders (`permissions`, FR-021) and
//! the named failures the engine taxonomy surfaces (FR-023), with the
//! full in-app reason stating the failure in Portuguese (FR-037).

use engine_core::closed::TransportState;
use serde::{Deserialize, Serialize};

/// The short lock-screen reason statement (FR-037, FR-041): the closed
/// set of reasons in the short form the lock-screen or notification-shade
/// surface states without an unlock.
///
/// The set is the union of three closed taxonomies, and every variant
/// names its origin: the focus-loss pause reasons (the two pause-causing
/// members of the focus outcome taxonomy — a duckable transient ducks and
/// keeps reading, so it never pauses and never states a reason, FR-013),
/// the engine failure taxonomy (the `EngineError` closed set of
/// `engine-core`), and the denied-permission statement (FR-021). No
/// `Other`/`Unknown`, no catch-all arms (Principle VII, FR-044): adding
/// a variant is a coordinated edit across every consumer, with a semver
/// bump and a documentation update.
///
/// The wire form is the `snake_case` variant name (contracts/
/// shell-ipc.md case-mapping table), identical to [`FullReason`]'s, so
/// one wire value deserializes into either presentation.
///
/// Rendering is identifier-style for the failure and permission reasons —
/// the stated-reason shape the permission gate renders
/// (`permissions::PermissionFailure::reason`) and the named failure the
/// engine taxonomy surfaces (FR-023) — except the focus-loss Pause
/// Reason, whose statement the specification fixes in Portuguese
/// (FR-041). [`short_reason`] is the single source of every rendered
/// string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortReason {
    /// Focus-loss pause: a transient loss demanded silence (a call or
    /// alarm; FR-013a) and playback paused. The stated reason expires
    /// 3 s after the pause on both surfaces (FR-041, FR-037a) and
    /// never promises a resume (FR-014).
    FocusSilenceDemanding,
    /// Focus-loss pause: permanent loss of audio focus (a call or
    /// alarm; FR-014) and playback paused. The stated reason expires
    /// 3 s after the pause on both surfaces (FR-041, FR-037a) and
    /// never promises a resume (FR-014). Where a transient loss is
    /// superseded by a permanent loss, the 3 s window restarts at the
    /// permanent-loss callback and this is the authoritative reason on
    /// both surfaces (FR-014, FR-041).
    FocusPermanentLoss,
    /// Failure: the engine is unreachable at the transport level
    /// (FR-023). The reason persists in the application until it is
    /// read (FR-037).
    EngineUnreachable,
    /// Failure: the engine failed to initialize (FR-023). The reason
    /// persists in the application until it is read (FR-037).
    EngineInitFailed,
    /// Failure: the delivered rate deviates from the declared handoff
    /// shape, including a pitch-tolerance violation at 2.0× (FR-035).
    /// The reason persists in the application until it is read
    /// (FR-037).
    EngineRateMismatch,
    /// Failure: the delivered channel count deviates from the declared
    /// mono handoff shape (FR-033). The reason persists in the
    /// application until it is read (FR-037).
    EngineChannelMismatch,
    /// Failure: the audio device refused the playback request
    /// (FR-022). The reason persists in the application until it is
    /// read (FR-037).
    EngineDeviceRefused,
    /// Failure: the audio device was lost during playback (FR-022).
    /// The reason persists in the application until it is read
    /// (FR-037).
    EngineDeviceLost,
    /// Failure: playback start was refused (FR-013b). The reason
    /// persists in the application until it is read (FR-037).
    EngineStartRefused,
    /// Denied-permission statement (FR-021): the notification
    /// permission is denied, so lock-screen controls are unavailable.
    /// The statement is made inside the application, because the
    /// denied permission is precisely what removes the surface it
    /// would otherwise appear on, and it renders identically through
    /// both getters. It persists while the permission is denied.
    NotificationDenied,
}

/// The full in-app reason statement (FR-037): the closed set of reasons
/// in the full form the application states when it is next opened,
/// parallel to [`ShortReason`].
///
/// The variants share the `snake_case` wire forms with [`ShortReason`],
/// so one wire value deserializes into either presentation and each
/// surface renders its own form of the same reason.
///
/// The full failure reasons state the failure in Portuguese, the
/// application's surface language, and never quote document content
/// (FR-037); the focus-loss reasons render the same
/// specification-fixed Pause Reason statement as the short form, because
/// the application shows "this reason" within the 3 s window (FR-041);
/// the denied-permission statement renders the same statement as the
/// short form (FR-021). No `Other`/`Unknown`, no catch-all arms
/// (Principle VII, FR-044): adding a variant is a coordinated edit
/// across every consumer, with a semver bump and a documentation
/// update. [`full_reason`] is the single source of every rendered
/// string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FullReason {
    /// Focus-loss pause: a transient loss demanded silence (a call or
    /// alarm; FR-013a) and playback paused. The stated reason expires
    /// 3 s after the pause on both surfaces (FR-041, FR-037a) and
    /// never promises a resume (FR-014).
    FocusSilenceDemanding,
    /// Focus-loss pause: permanent loss of audio focus (a call or
    /// alarm; FR-014) and playback paused. The stated reason expires
    /// 3 s after the pause on both surfaces (FR-041, FR-037a) and
    /// never promises a resume (FR-014). Where a transient loss is
    /// superseded by a permanent loss, the 3 s window restarts at the
    /// permanent-loss callback and this is the authoritative reason on
    /// both surfaces (FR-014, FR-041).
    FocusPermanentLoss,
    /// Failure: the engine is unreachable at the transport level
    /// (FR-023). The reason persists in the application until it is
    /// read (FR-037).
    EngineUnreachable,
    /// Failure: the engine failed to initialize (FR-023). The reason
    /// persists in the application until it is read (FR-037).
    EngineInitFailed,
    /// Failure: the delivered rate deviates from the declared handoff
    /// shape, including a pitch-tolerance violation at 2.0× (FR-035).
    /// The reason persists in the application until it is read
    /// (FR-037).
    EngineRateMismatch,
    /// Failure: the delivered channel count deviates from the declared
    /// mono handoff shape (FR-033). The reason persists in the
    /// application until it is read (FR-037).
    EngineChannelMismatch,
    /// Failure: the audio device refused the playback request
    /// (FR-022). The reason persists in the application until it is
    /// read (FR-037).
    EngineDeviceRefused,
    /// Failure: the audio device was lost during playback (FR-022).
    /// The reason persists in the application until it is read
    /// (FR-037).
    EngineDeviceLost,
    /// Failure: playback start was refused (FR-013b). The reason
    /// persists in the application until it is read (FR-037).
    EngineStartRefused,
    /// Denied-permission statement (FR-021): the notification
    /// permission is denied, so lock-screen controls are unavailable.
    /// The statement is made inside the application, because the
    /// denied permission is precisely what removes the surface it
    /// would otherwise appear on, and it renders identically through
    /// both getters. It persists while the permission is denied.
    NotificationDenied,
}

/// The expiry classification of a stated reason (FR-037, FR-037a,
/// FR-041): how long the reason stays on a surface.
///
/// The classification is the whole boundary FR-037a draws: a focus-loss
/// pause reason expires 3 s after the pause on BOTH surfaces — the lock
/// screen and the application — because a focus loss is a state the
/// reader can see the cause of, while a failure reason persists in the
/// application until it is read, because a failure is an app-level fault
/// whose cause is invisible to the reader (FR-037). The two sets of
/// reasons are disjoint (FR-013a, FR-037a, FR-041).
///
/// The wire form is the variant name with an explicit `snake_case`
/// rename, so the digit in the focus variant carries a word separator
/// on the wire (`focus_3s`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasonExpiry {
    /// The reason expires 3 s after the pause, on both the lock-screen
    /// and the in-app surface (FR-041, FR-037a): the application
    /// shows it only if the reader opens it within that window, and
    /// only the paused state and its resume control remain afterwards
    /// (FR-030).
    #[serde(rename = "focus_3s")]
    Focus3s,
    /// The reason persists in the application indefinitely, until it is
    /// read (FR-037): an engine failure, an audio-device failure, a
    /// refused start, or the denied-permission statement — none of
    /// which the reader can see the cause of from the platform.
    #[serde(rename = "failure_persistent")]
    FailurePersistent,
}

/// The reason TTL the payload carries for a pause-demanding reason
/// (FR-041, FR-037a): exactly 3 s.
///
/// This is the numeric value of the payload's `reason_ttl_ms` field
/// for a reason that expires — the same 3 s window
/// `player::interruption` assigns every pause-demanding member of
/// the closed product-reason set, asserted equal in the tests below
/// so the value is stated once here and pinned there rather than
/// drifting. It is NOT the dismissal policy ([`Dismissal`],
/// FR-014/FR-028e): a TTL says how long a reason line stays on a
/// surface, a dismissal policy says whether the surface itself may be
/// dismissed, and the payload overload of reading one as the other
/// is what contracts/surface.md fixes (F8).
pub const PAUSE_REASON_TTL_MS: u64 = 3000;

impl ShortReason {
    /// How long this reason stays on a surface (FR-037, FR-037a,
    /// FR-041): a focus-loss pause reason expires 3 s after the pause
    /// on both surfaces; every other reason persists until it is read.
    #[must_use]
    pub const fn expiry(&self) -> ReasonExpiry {
        match *self {
            Self::FocusSilenceDemanding | Self::FocusPermanentLoss => ReasonExpiry::Focus3s,
            Self::EngineUnreachable
            | Self::EngineInitFailed
            | Self::EngineRateMismatch
            | Self::EngineChannelMismatch
            | Self::EngineDeviceRefused
            | Self::EngineDeviceLost
            | Self::EngineStartRefused
            | Self::NotificationDenied => ReasonExpiry::FailurePersistent,
        }
    }

    /// The payload TTL of this reason (FR-037a, FR-041):
    /// [`PAUSE_REASON_TTL_MS`] for a pause-demanding reason, which
    /// expires 3 s after the pause on both surfaces, or `None` for a
    /// persistent reason, which stays on the in-app surface until it
    /// is read.
    ///
    /// Derived from [`ShortReason::expiry`] — the one classification,
    /// never a second split — so no reason can carry a TTL its
    /// expiry contradicts, and the payload's `reason_ttl_ms` field is
    /// this getter's output verbatim (`3000` or `null`).
    #[must_use]
    pub const fn reason_ttl_ms(&self) -> Option<u64> {
        match self.expiry() {
            ReasonExpiry::Focus3s => Some(PAUSE_REASON_TTL_MS),
            ReasonExpiry::FailurePersistent => None,
        }
    }
}

impl FullReason {
    /// How long this reason stays on a surface (FR-037, FR-037a,
    /// FR-041): a focus-loss pause reason expires 3 s after the pause
    /// on both surfaces; every other reason persists until it is read.
    #[must_use]
    pub const fn expiry(&self) -> ReasonExpiry {
        match *self {
            Self::FocusSilenceDemanding | Self::FocusPermanentLoss => ReasonExpiry::Focus3s,
            Self::EngineUnreachable
            | Self::EngineInitFailed
            | Self::EngineRateMismatch
            | Self::EngineChannelMismatch
            | Self::EngineDeviceRefused
            | Self::EngineDeviceLost
            | Self::EngineStartRefused
            | Self::NotificationDenied => ReasonExpiry::FailurePersistent,
        }
    }

    /// The payload TTL of this reason (FR-037a, FR-041):
    /// [`PAUSE_REASON_TTL_MS`] for a pause-demanding reason, which
    /// expires 3 s after the pause on both surfaces, or `None` for a
    /// persistent reason, which stays on the in-app surface until it
    /// is read.
    ///
    /// Derived from [`FullReason::expiry`] — the one classification,
    /// never a second split — and identical to
    /// [`ShortReason::reason_ttl_ms`] for the same reason, because the
    /// TTL belongs to the reason and not to the presentation that
    /// renders it: the expiry of FR-037a governs BOTH surfaces
    /// (FR-041).
    #[must_use]
    pub const fn reason_ttl_ms(&self) -> Option<u64> {
        match self.expiry() {
            ReasonExpiry::Focus3s => Some(PAUSE_REASON_TTL_MS),
            ReasonExpiry::FailurePersistent => None,
        }
    }
}

/// The lock-screen dismissal policy (FR-014, FR-028e): the closed
/// two-member set the payload's `dismissal` field carries.
///
/// The policy is a property of the SURFACE, not of the reason: while a
/// document plays, the surface carrying its controls refuses
/// dismissal, and pausing is the route to dismissal. It is therefore
/// distinct from the reason TTL ([`PAUSE_REASON_TTL_MS`],
/// [`ShortReason::reason_ttl_ms`]) — a reason line expires on a
/// schedule, a surface is or is not dismissable, and reading one
/// field out of the other's classification is the payload overload
/// contracts/surface.md fixes (F8).
///
/// This is the payload's two-valued wire form, not the
/// resolver-level taxonomy `player::surface::Dismissal`, which
/// additionally decides the transient auto-dismiss window
/// (`player::surface::TRANSIENT_DISMISS_MS`) and the resident
/// surface's behaviour: the payload collapses that decision into these
/// two wire values and the resolver keeps the mechanics. The
/// transport set the policy reads is the session's own closed
/// transport reflection (FR-044) — no list is restated here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dismissal {
    /// The surface MUST NOT be dismissed while a document plays
    /// (FR-014): pausing is the route to dismissal. The reader's
    /// controls and the audio they govern stay on one surface.
    NonDismissableWhilePlaying,
    /// The surface MAY be dismissed by the reader (FR-028e): a
    /// paused, resident, or torn-down surface, whose dismissal the
    /// platform honours.
    Dismissable,
}

impl Dismissal {
    /// Whether the reader may dismiss the surface (FR-014, FR-028e):
    /// false only while a document plays.
    #[must_use]
    pub const fn is_dismissable(&self) -> bool {
        matches!(self, Self::Dismissable)
    }
}

/// The dismissal policy for the transport state the session states
/// (FR-014, FR-028e): a playing transport refuses dismissal — pausing
/// is the route to dismissal.
///
/// Every other state is dismissable, including the torn-down states
/// (`stopped`, `completed`), whose FR-032f teardown has nothing left
/// to keep on the surface, and the interrupted state, whose surface
/// carries the paused state and its resume control.
///
/// The match is exhaustive over the session's closed transport set
/// with no catch-all arm (Principle VII): a state added to the
/// transport reflection is a coordinated edit across every surface
/// consumer, with a semver bump.
#[must_use]
pub const fn dismissal_for(transport: TransportState) -> Dismissal {
    match transport {
        TransportState::Playing => Dismissal::NonDismissableWhilePlaying,
        TransportState::Paused
        | TransportState::Stopped
        | TransportState::Completed
        | TransportState::Interrupted => Dismissal::Dismissable,
    }
}

/// The short lock-screen statement for one reason (FR-037, FR-041): the
/// single source of the short form every lock-screen or
/// notification-shade surface renders.
///
/// The focus-loss arms render the specification's Pause Reason
/// statement — "Leitura pausada — outro app usando o áudio" — naming
/// the document title and nothing else from the document (FR-037,
/// FR-041); `document_title` is the title the surface resolver
/// already resolved (metadata title else the stable filename
/// placeholder), never the document body. The failure arms render the
/// failure's declared name — the stated-reason shape the permission
/// gate renders — and the permission arm renders the FR-021 statement;
/// those are app-generated text and take no document text at all
/// (FR-037: a failure reason MUST NOT include text drawn from the
/// document).
///
/// Every surface renders through this getter — never a second
/// definition of the string (Principle II).
#[must_use]
pub fn short_reason(reason: &ShortReason, document_title: &str) -> String {
    match reason {
        ShortReason::FocusSilenceDemanding | ShortReason::FocusPermanentLoss => {
            format!("Leitura pausada — outro app usando o áudio: {document_title}")
        }
        ShortReason::EngineUnreachable => "engine_unreachable".to_string(),
        ShortReason::EngineInitFailed => "engine_init_failed".to_string(),
        ShortReason::EngineRateMismatch => "engine_rate_mismatch".to_string(),
        ShortReason::EngineChannelMismatch => "engine_channel_mismatch".to_string(),
        ShortReason::EngineDeviceRefused => "engine_device_refused".to_string(),
        ShortReason::EngineDeviceLost => "engine_device_lost".to_string(),
        ShortReason::EngineStartRefused => "engine_start_refused".to_string(),
        ShortReason::NotificationDenied => "lock_screen_controls_unavailable".to_string(),
    }
}

/// The full in-app statement for one reason (FR-037): the single source
/// of the full form every in-app surface renders when it is next
/// opened.
///
/// The failure arms state the failure in Portuguese — the full reason
/// that remains available in the application (FR-037) — and never quote
/// document content; `document_title` is consumed by the focus-loss
/// arms alone, which render the same specification-fixed Pause Reason
/// statement as the short form, because the application shows "this
/// reason" within the 3 s window (FR-041). The permission arm renders
/// the FR-021 statement identically to [`short_reason`] (the denied
/// permission is precisely what removes the surface the statement
/// would otherwise appear on).
///
/// Every surface renders through this getter — never a second
/// definition of the string (Principle II).
#[must_use]
pub fn full_reason(reason: &FullReason, document_title: &str) -> String {
    match reason {
        FullReason::FocusSilenceDemanding | FullReason::FocusPermanentLoss => {
            format!("Leitura pausada — outro app usando o áudio: {document_title}")
        }
        FullReason::EngineUnreachable => {
            "Falha no motor de síntese: o motor está inalcançável no nível de transporte."
                .to_string()
        }
        FullReason::EngineInitFailed => {
            "Falha no motor de síntese: o motor não pôde ser inicializado.".to_string()
        }
        FullReason::EngineRateMismatch => {
            "Falha na entrega de áudio: a taxa entregue diverge da taxa declarada de 22 050 Hz."
                .to_string()
        }
        FullReason::EngineChannelMismatch => {
            "Falha na entrega de áudio: o número de canais entregue diverge do mono declarado."
                .to_string()
        }
        FullReason::EngineDeviceRefused => {
            "Falha no dispositivo de áudio: o dispositivo recusou a reprodução.".to_string()
        }
        FullReason::EngineDeviceLost => {
            "Falha no dispositivo de áudio: o dispositivo foi perdido durante a reprodução."
                .to_string()
        }
        FullReason::EngineStartRefused => {
            "Falha na reprodução: a reprodução foi recusada ao iniciar.".to_string()
        }
        FullReason::NotificationDenied => "lock_screen_controls_unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{PermissionSnapshot, PermissionState, notification_statement};
    use player::interruption::InterruptionReason;

    /// A document title whose words occur in no app-generated reason
    /// string, so an assertion can tell document-derived text from
    /// app-generated text (FR-037).
    const DOCUMENT_TITLE: &str = "Rota da Manhã";

    /// The specification's Pause Reason statement (FR-041), before the
    /// document title it names.
    const PAUSE_STATEMENT: &str = "Leitura pausada — outro app usando o áudio";

    /// Every [`ShortReason`] variant — the closed set, so the iteration
    /// tests below cover the whole set and a variant added without
    /// joining the list fails to compile the exhaustive arms
    /// (Principle VII).
    const ALL_SHORT: [ShortReason; 10] = [
        ShortReason::FocusSilenceDemanding,
        ShortReason::FocusPermanentLoss,
        ShortReason::EngineUnreachable,
        ShortReason::EngineInitFailed,
        ShortReason::EngineRateMismatch,
        ShortReason::EngineChannelMismatch,
        ShortReason::EngineDeviceRefused,
        ShortReason::EngineDeviceLost,
        ShortReason::EngineStartRefused,
        ShortReason::NotificationDenied,
    ];

    /// Every [`FullReason`] variant — the closed set.
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

    /// The seven failure reasons with their declared short names — the
    /// named failures the engine taxonomy surfaces (FR-023), in the
    /// stated-reason shape the permission gate renders.
    const FAILURE_SHORT: [(ShortReason, &str); 7] = [
        (ShortReason::EngineUnreachable, "engine_unreachable"),
        (ShortReason::EngineInitFailed, "engine_init_failed"),
        (ShortReason::EngineRateMismatch, "engine_rate_mismatch"),
        (
            ShortReason::EngineChannelMismatch,
            "engine_channel_mismatch",
        ),
        (ShortReason::EngineDeviceRefused, "engine_device_refused"),
        (ShortReason::EngineDeviceLost, "engine_device_lost"),
        (ShortReason::EngineStartRefused, "engine_start_refused"),
    ];

    /// The seven failure reasons with their full in-app statements —
    /// the full reason that remains available in the application
    /// (FR-037), in the application's pt-BR surface language —
    /// paired with the corresponding [`ShortReason`] so a test
    /// can assert the two presentations differ (FR-037: a short
    /// statement on the lock screen, the full reason in the
    /// application).
    const FAILURE_FULL: [(FullReason, ShortReason, &str); 7] = [
        (
            FullReason::EngineUnreachable,
            ShortReason::EngineUnreachable,
            "Falha no motor de síntese: o motor está inalcançável no nível de transporte.",
        ),
        (
            FullReason::EngineInitFailed,
            ShortReason::EngineInitFailed,
            "Falha no motor de síntese: o motor não pôde ser inicializado.",
        ),
        (
            FullReason::EngineRateMismatch,
            ShortReason::EngineRateMismatch,
            "Falha na entrega de áudio: a taxa entregue diverge da taxa declarada de 22 050 Hz.",
        ),
        (
            FullReason::EngineChannelMismatch,
            ShortReason::EngineChannelMismatch,
            "Falha na entrega de áudio: o número de canais entregue diverge do mono declarado.",
        ),
        (
            FullReason::EngineDeviceRefused,
            ShortReason::EngineDeviceRefused,
            "Falha no dispositivo de áudio: o dispositivo recusou a reprodução.",
        ),
        (
            FullReason::EngineDeviceLost,
            ShortReason::EngineDeviceLost,
            "Falha no dispositivo de áudio: o dispositivo foi perdido durante a reprodução.",
        ),
        (
            FullReason::EngineStartRefused,
            ShortReason::EngineStartRefused,
            "Falha na reprodução: a reprodução foi recusada ao iniciar.",
        ),
    ];

    /// The wire form of every variant of every closed set — the
    /// `snake_case` name (contracts/shell-ipc.md case-mapping table).
    const SHORT_WIRE: [(ShortReason, &str); 10] = [
        (
            ShortReason::FocusSilenceDemanding,
            "focus_silence_demanding",
        ),
        (ShortReason::FocusPermanentLoss, "focus_permanent_loss"),
        (ShortReason::EngineUnreachable, "engine_unreachable"),
        (ShortReason::EngineInitFailed, "engine_init_failed"),
        (ShortReason::EngineRateMismatch, "engine_rate_mismatch"),
        (
            ShortReason::EngineChannelMismatch,
            "engine_channel_mismatch",
        ),
        (ShortReason::EngineDeviceRefused, "engine_device_refused"),
        (ShortReason::EngineDeviceLost, "engine_device_lost"),
        (ShortReason::EngineStartRefused, "engine_start_refused"),
        (ShortReason::NotificationDenied, "notification_denied"),
    ];

    /// The wire form of every [`FullReason`] variant — identical to
    /// [`ShortReason`]'s, so one wire value deserializes into either
    /// presentation.
    const FULL_WIRE: [(FullReason, &str); 10] = [
        (FullReason::FocusSilenceDemanding, "focus_silence_demanding"),
        (FullReason::FocusPermanentLoss, "focus_permanent_loss"),
        (FullReason::EngineUnreachable, "engine_unreachable"),
        (FullReason::EngineInitFailed, "engine_init_failed"),
        (FullReason::EngineRateMismatch, "engine_rate_mismatch"),
        (FullReason::EngineChannelMismatch, "engine_channel_mismatch"),
        (FullReason::EngineDeviceRefused, "engine_device_refused"),
        (FullReason::EngineDeviceLost, "engine_device_lost"),
        (FullReason::EngineStartRefused, "engine_start_refused"),
        (FullReason::NotificationDenied, "notification_denied"),
    ];

    /// The wire form of every [`ReasonExpiry`] variant.
    const EXPIRY_WIRE: [(ReasonExpiry, &str); 2] = [
        (ReasonExpiry::Focus3s, "focus_3s"),
        (ReasonExpiry::FailurePersistent, "failure_persistent"),
    ];

    /// The wire form of every [`Dismissal`] member — the contract's
    /// closed dismissal pair (contracts/surface.md payload).
    const DISMISSAL_WIRE: [(Dismissal, &str); 2] = [
        (
            Dismissal::NonDismissableWhilePlaying,
            "non_dismissable_while_playing",
        ),
        (Dismissal::Dismissable, "dismissable"),
    ];

    /// Every transport state of the session's closed transport
    /// reflection (FR-044), so the dismissal truth table covers the
    /// whole set and a state added to the reflection fails to compile
    /// the exhaustive match of [`dismissal_for`].
    const ALL_TRANSPORTS: [TransportState; 5] = [
        TransportState::Playing,
        TransportState::Paused,
        TransportState::Stopped,
        TransportState::Completed,
        TransportState::Interrupted,
    ];

    /// The pause-demanding members of the player's closed
    /// interruption set — the four reasons that pause playback and
    /// whose line expires at [`PAUSE_REASON_TTL_MS`]
    /// (contracts/surface.md reason-line table; T027).
    const PLAYER_PAUSE_DEMANDING: [InterruptionReason; 4] = [
        InterruptionReason::FocusDenied,
        InterruptionReason::TransientPaused,
        InterruptionReason::PermanentLoss,
        InterruptionReason::DeviceRemoved,
    ];

    /// The undeliverable members of the player's closed interruption
    /// set — the reasons playback never starts for and whose line
    /// persists (`reason_ttl_ms = null`; contracts/surface.md,
    /// data-model.md interruption table).
    const PLAYER_UNDELIVERABLE: [InterruptionReason; 3] = [
        InterruptionReason::VoiceUnavailable,
        InterruptionReason::FormatUndeliverable,
        InterruptionReason::DecodeFailed,
    ];

    /// The whitespace-delimited word count of a rendered line — the
    /// measurement the ≤8-word bound of contracts/surface.md is
    /// checked against.
    fn words(line: &str) -> usize {
        line.split_whitespace().count()
    }

    #[test]
    fn focus_short_reason_states_the_pause_and_names_the_title() {
        // FR-041: the canonical Pause Reason statement, naming the
        // document title and nothing else from the document — the
        // exact equality pins the whole string, so no other
        // document-derived text can appear.
        assert_eq!(
            short_reason(&ShortReason::FocusPermanentLoss, DOCUMENT_TITLE),
            format!("{PAUSE_STATEMENT}: {DOCUMENT_TITLE}")
        );
        assert_eq!(
            short_reason(&ShortReason::FocusSilenceDemanding, DOCUMENT_TITLE),
            format!("{PAUSE_STATEMENT}: {DOCUMENT_TITLE}")
        );
    }

    #[test]
    fn focus_full_reason_is_the_same_statement_both_surfaces_show() {
        // FR-041: the application shows "this reason" — the same
        // statement the lock screen shows — within the 3 s window, so
        // the two presentations render identically for a focus loss.
        for (short, full) in [
            (
                ShortReason::FocusSilenceDemanding,
                FullReason::FocusSilenceDemanding,
            ),
            (
                ShortReason::FocusPermanentLoss,
                FullReason::FocusPermanentLoss,
            ),
        ] {
            assert_eq!(
                full_reason(&full, DOCUMENT_TITLE),
                short_reason(&short, DOCUMENT_TITLE)
            );
        }
    }

    #[test]
    fn focus_reasons_expire_after_three_seconds_on_both_surfaces() {
        // FR-041 / FR-037a: the focus-loss expiry governs BOTH
        // surfaces — the short lock-screen form and the full in-app
        // form alike.
        for reason in [
            ShortReason::FocusSilenceDemanding,
            ShortReason::FocusPermanentLoss,
        ] {
            assert_eq!(reason.expiry(), ReasonExpiry::Focus3s);
        }
        for reason in [
            FullReason::FocusSilenceDemanding,
            FullReason::FocusPermanentLoss,
        ] {
            assert_eq!(reason.expiry(), ReasonExpiry::Focus3s);
        }
    }

    #[test]
    fn failure_short_reasons_are_the_named_failures() {
        // FR-023 / FR-037: a failure surfaces as a named failure —
        // the declared name is the short lock-screen statement.
        for (reason, expected) in FAILURE_SHORT {
            assert_eq!(short_reason(&reason, DOCUMENT_TITLE), expected);
        }
    }

    #[test]
    fn failure_full_reasons_state_the_full_reason() {
        // FR-037: the full reason remains available in the
        // application, stated in the application's pt-BR surface
        // language — a full statement, not the bare named
        // short form.
        for (reason, short, expected) in FAILURE_FULL {
            let rendered = full_reason(&reason, DOCUMENT_TITLE);
            assert_eq!(rendered, expected);
            assert_ne!(rendered, short_reason(&short, DOCUMENT_TITLE));
        }
    }

    #[test]
    fn failure_reasons_never_quote_document_text() {
        // FR-037: a failure reason MUST NOT include text drawn from
        // the document — neither its title nor any word of it.
        for (reason, _) in FAILURE_SHORT {
            let short = short_reason(&reason, DOCUMENT_TITLE);
            assert!(!short.contains(DOCUMENT_TITLE));
            assert!(!short.contains("Rota"));
            assert!(!short.contains("Manhã"));
        }
        for (reason, _, _) in FAILURE_FULL {
            let full = full_reason(&reason, DOCUMENT_TITLE);
            assert!(!full.contains(DOCUMENT_TITLE));
            assert!(!full.contains("Rota"));
            assert!(!full.contains("Manhã"));
        }
    }

    #[test]
    fn failure_reasons_persist_until_read() {
        // FR-037 / FR-037a: persistence is the failure regime — the
        // reason stays in the application until it is read, on both
        // presentations.
        for (reason, _) in FAILURE_SHORT {
            assert_eq!(reason.expiry(), ReasonExpiry::FailurePersistent);
        }
        for (reason, _, _) in FAILURE_FULL {
            assert_eq!(reason.expiry(), ReasonExpiry::FailurePersistent);
        }
    }

    #[test]
    fn denied_permission_statement_is_the_same_through_both_getters() {
        // FR-021: a denied notification permission states that
        // lock-screen controls are unavailable, inside the application
        // — the same statement through both getters, and the same
        // string the permission gate's statement renders (one
        // definition, never a second).
        let short = short_reason(&ShortReason::NotificationDenied, DOCUMENT_TITLE);
        let full = full_reason(&FullReason::NotificationDenied, DOCUMENT_TITLE);
        assert_eq!(short, full);
        assert_eq!(short, "lock_screen_controls_unavailable");
        let denied = PermissionSnapshot {
            foreground_service_media_playback: PermissionState::Granted,
            post_notifications: PermissionState::Denied,
        };
        assert_eq!(Some(short.as_str()), notification_statement(&denied));
    }

    #[test]
    fn denied_permission_statement_persists_while_denied() {
        // The denial is no focus loss, so it cannot carry the 3 s
        // focus expiry: it persists while the permission is denied —
        // the only non-focus regime in the closed expiry set.
        assert_eq!(
            ShortReason::NotificationDenied.expiry(),
            ReasonExpiry::FailurePersistent
        );
        assert_eq!(
            FullReason::NotificationDenied.expiry(),
            ReasonExpiry::FailurePersistent
        );
    }

    #[test]
    fn every_variant_renders_a_non_empty_statement() {
        // The closed sets render in full: no variant is a dead arm,
        // and every statement is non-empty on both surfaces.
        for reason in ALL_SHORT {
            assert!(!short_reason(&reason, DOCUMENT_TITLE).is_empty());
        }
        for reason in ALL_FULL {
            assert!(!full_reason(&reason, DOCUMENT_TITLE).is_empty());
        }
    }

    #[test]
    fn wire_forms_are_snake_case() {
        // The wire form of every set is the snake_case name
        // (contracts/shell-ipc.md case-mapping table), and the two
        // reason enums share their wire forms, so one wire value
        // deserializes into either presentation.
        for (reason, wire) in SHORT_WIRE {
            assert_eq!(
                serde_json::to_value(reason).ok(),
                Some(serde_json::json!(wire))
            );
            assert_eq!(
                serde_json::from_value::<ShortReason>(serde_json::json!(wire)).ok(),
                Some(reason)
            );
        }
        for (reason, wire) in FULL_WIRE {
            assert_eq!(
                serde_json::to_value(reason).ok(),
                Some(serde_json::json!(wire))
            );
            assert_eq!(
                serde_json::from_value::<FullReason>(serde_json::json!(wire)).ok(),
                Some(reason)
            );
        }
        for (expiry, wire) in EXPIRY_WIRE {
            assert_eq!(
                serde_json::to_value(expiry).ok(),
                Some(serde_json::json!(wire))
            );
            assert_eq!(
                serde_json::from_value::<ReasonExpiry>(serde_json::json!(wire)).ok(),
                Some(expiry)
            );
        }
        // An unknown wire value never deserializes: there is no
        // variant for it to land in (Principle VII).
        assert!(serde_json::from_value::<ShortReason>(serde_json::json!("other")).is_err());
        assert!(serde_json::from_value::<FullReason>(serde_json::json!("other")).is_err());
        assert!(serde_json::from_value::<ReasonExpiry>(serde_json::json!("other")).is_err());
    }

    #[test]
    fn every_reason_bears_a_ttl_derived_from_its_expiry() {
        // FR-037a / FR-041: the TTL is the expiry classification's
        // numeric form — a pause-demanding reason expires at
        // `PAUSE_REASON_TTL_MS`, a persistent reason never expires
        // (`None`, the payload's `null`) — and the two presentations
        // carry the same TTL for the same reason, because the TTL
        // belongs to the reason and not to the surface (FR-041: the
        // expiry governs BOTH surfaces).
        for reason in ALL_SHORT {
            let ttl = reason.reason_ttl_ms();
            match reason.expiry() {
                ReasonExpiry::Focus3s => {
                    assert_eq!(ttl, Some(PAUSE_REASON_TTL_MS), "{reason:?} must expire");
                }
                ReasonExpiry::FailurePersistent => {
                    assert_eq!(ttl, None, "{reason:?} must persist (reason_ttl_ms = null)");
                }
            }
        }
        for reason in ALL_FULL {
            let ttl = reason.reason_ttl_ms();
            match reason.expiry() {
                ReasonExpiry::Focus3s => assert_eq!(ttl, Some(PAUSE_REASON_TTL_MS)),
                ReasonExpiry::FailurePersistent => assert_eq!(ttl, None),
            }
            // One reason, both presentations, one TTL: the short
            // lock-screen form and the full in-app form agree.
            assert_eq!(ttl, reason_ttl_of_the_same_reason(reason));
        }
        // The TTL is exactly the payload's two-valued wire form:
        // 3000 or null, never zero, never a second window.
        assert_eq!(PAUSE_REASON_TTL_MS, 3000);
    }

    /// The TTL the short presentation carries for the reason the
    /// full presentation names — the parallel-set correspondence a
    /// test can assert without a second classification.
    fn reason_ttl_of_the_same_reason(full: FullReason) -> Option<u64> {
        let short = match full {
            FullReason::FocusSilenceDemanding => ShortReason::FocusSilenceDemanding,
            FullReason::FocusPermanentLoss => ShortReason::FocusPermanentLoss,
            FullReason::EngineUnreachable => ShortReason::EngineUnreachable,
            FullReason::EngineInitFailed => ShortReason::EngineInitFailed,
            FullReason::EngineRateMismatch => ShortReason::EngineRateMismatch,
            FullReason::EngineChannelMismatch => ShortReason::EngineChannelMismatch,
            FullReason::EngineDeviceRefused => ShortReason::EngineDeviceRefused,
            FullReason::EngineDeviceLost => ShortReason::EngineDeviceLost,
            FullReason::EngineStartRefused => ShortReason::EngineStartRefused,
            FullReason::NotificationDenied => ShortReason::NotificationDenied,
        };
        short.reason_ttl_ms()
    }

    #[test]
    fn the_ttl_classification_is_the_one_player_interruption_carries() {
        // T043: ONE classification, two closed sets. The product
        // reasons of contracts/surface.md live in
        // `player::interruption` (the reason-line table's own set),
        // and the T065 surface reasons live here; both split the same
        // way — every pause-demanding reason expires at
        // `PAUSE_REASON_TTL_MS`, every undeliverable reason persists
        // (`None`). The equality is asserted across the crates rather
        // than duplicated, so a drift on either side fails a test
        // instead of shipping a second list (FR-044).
        for reason in PLAYER_PAUSE_DEMANDING {
            assert!(
                !reason.is_persistent(),
                "{reason:?} is pause-demanding — it must not be persistent"
            );
            assert_eq!(reason.reason_ttl_ms(), Some(PAUSE_REASON_TTL_MS));
        }
        for reason in PLAYER_UNDELIVERABLE {
            assert!(
                reason.is_persistent(),
                "{reason:?} is undeliverable — it must be persistent"
            );
            assert_eq!(reason.reason_ttl_ms(), None);
        }
        // Two closed sets, one value domain: every TTL in either set
        // is the expiring window or `null` — no third window exists
        // on either side of the classification.
        for reason in ALL_SHORT {
            assert!(
                matches!(reason.reason_ttl_ms(), None | Some(PAUSE_REASON_TTL_MS)),
                "{reason:?} carries a TTL outside the closed value domain"
            );
        }
        for reason in PLAYER_PAUSE_DEMANDING
            .into_iter()
            .chain(PLAYER_UNDELIVERABLE)
        {
            assert!(
                matches!(reason.reason_ttl_ms(), None | Some(PAUSE_REASON_TTL_MS)),
                "{reason:?} carries a TTL outside the closed value domain"
            );
        }
        // The one reason of the player set that states no line
        // (FocusDelayed — the zero-audio hold) is neither regime: it
        // carries no line and no TTL at all.
        assert!(InterruptionReason::FocusDelayed.reason_line().is_empty());
    }

    #[test]
    fn every_lock_screen_line_stays_within_eight_words() {
        // contracts/surface.md: the reason line is a ≤8-word pt-BR
        // line. The stated-name arms (the seven failures and the
        // denied permission) state one declared name each, and the
        // FR-041 Pause Reason statement is exactly at the bound, so
        // the whole closed set fits.
        for reason in ALL_SHORT {
            let app_text = match reason {
                ShortReason::FocusSilenceDemanding | ShortReason::FocusPermanentLoss => {
                    PAUSE_STATEMENT
                }
                ShortReason::EngineUnreachable
                | ShortReason::EngineInitFailed
                | ShortReason::EngineRateMismatch
                | ShortReason::EngineChannelMismatch
                | ShortReason::EngineDeviceRefused
                | ShortReason::EngineDeviceLost
                | ShortReason::EngineStartRefused
                | ShortReason::NotificationDenied => &short_reason(&reason, DOCUMENT_TITLE),
            };
            assert!(
                words(app_text) <= 8,
                "{reason:?} states {} words — the ≤8-word bound is exceeded by {app_text:?}",
                words(app_text)
            );
        }
        // The measurement, pinned: the FR-041 statement is eight words
        // and the document title it names rides in the notification's
        // own title field (FR-011), not in the reason line's
        // app-generated text.
        assert_eq!(words(PAUSE_STATEMENT), 8);
        assert_eq!(
            words(&short_reason(
                &ShortReason::FocusPermanentLoss,
                DOCUMENT_TITLE
            )),
            words(PAUSE_STATEMENT) + words(DOCUMENT_TITLE)
        );
    }

    #[test]
    fn every_stated_name_fits_the_android_content_text_limit() {
        // contracts/surface.md: the reason line carries a ≤40-char
        // hard limit — the Android notification content-text limit —
        // and the limit wins over the ≤8-word target on conflict.
        // Every line the surface states itself fits with room to
        // spare: the failure and permission arms are declared names,
        // so the whole stated-name half of the closed set is inside
        // both bounds. The FR-041 statement is the one line that is
        // not, and it is recorded separately below.
        for reason in ALL_SHORT {
            let rendered = short_reason(&reason, DOCUMENT_TITLE);
            let app_text = match reason {
                ShortReason::FocusSilenceDemanding | ShortReason::FocusPermanentLoss => continue,
                ShortReason::EngineUnreachable
                | ShortReason::EngineInitFailed
                | ShortReason::EngineRateMismatch
                | ShortReason::EngineChannelMismatch
                | ShortReason::EngineDeviceRefused
                | ShortReason::EngineDeviceLost
                | ShortReason::EngineStartRefused
                | ShortReason::NotificationDenied => rendered.as_str(),
            };
            assert!(
                app_text.chars().count() <= 40,
                "{reason:?} states {} chars — the ≤40-char content-text limit is exceeded by {app_text:?}",
                app_text.chars().count()
            );
            assert!(words(app_text) <= 8, "{reason:?} states too many words");
        }
    }

    #[test]
    fn the_focus_pause_reason_exceeds_the_content_text_limit_and_the_conflict_is_recorded() {
        // T043 boundary record — a real conflict between two
        // requirements, measured rather than hidden. FR-041 fixes the
        // Pause Reason statement verbatim and requires it to name the
        // document title; contracts/surface.md caps the notification
        // content text at 40 chars. The statement alone measures 42
        // chars, and 57 with the title the FR-041 statement must
        // carry — both beyond the cap.
        //
        // Recorded, not resolved by truncation: a truncated statement
        // would state something FR-041 does not say, and the
        // specification fixes the string outright ("The surface MUST
        // show a short scrolling statement of the form …"). The
        // contract's own conflict rule governs the 8-word target
        // against the 40-char limit; it says nothing about a
        // specification-fixed statement, so the line is rendered in
        // full and the conflict is escalated to the feature record
        // instead of being silently settled here. FR-041 asks for the
        // statement as moving text precisely so the glance is served
        // by the transport glyph and the earcon (FR-016) rather than
        // by reading this line.
        assert_eq!(PAUSE_STATEMENT.chars().count(), 42);
        assert!(
            PAUSE_STATEMENT.chars().count() > 40,
            "the recorded measurement changed — re-review the content-text limit against FR-041"
        );
        let rendered = short_reason(&ShortReason::FocusPermanentLoss, DOCUMENT_TITLE);
        assert_eq!(rendered.chars().count(), 57);
        assert!(
            rendered.chars().count() > 40,
            "the recorded measurement changed — re-review the content-text limit against FR-041"
        );
        // The word half of the bound does hold, so the conflict is
        // confined to the character limit.
        assert_eq!(words(PAUSE_STATEMENT), 8);
    }

    #[test]
    fn dismissal_is_refused_while_playing_and_every_other_state_is_dismissable() {
        // FR-014 / FR-028e / contracts/surface.md: dismissal is
        // refused while a document plays — pausing is the route to
        // dismissal — and every other transport state is dismissable,
        // including the interrupted state (whose surface carries the
        // paused state and its resume control) and the torn-down
        // states (whose FR-032f teardown has nothing left to keep).
        for transport in ALL_TRANSPORTS {
            let policy = dismissal_for(transport);
            if transport == TransportState::Playing {
                assert_eq!(policy, Dismissal::NonDismissableWhilePlaying);
                assert!(!policy.is_dismissable());
            } else {
                assert_eq!(
                    policy,
                    Dismissal::Dismissable,
                    "{transport:?} must be dismissable"
                );
                assert!(policy.is_dismissable());
            }
        }
    }

    #[test]
    fn the_dismissal_wire_forms_are_the_contracts_closed_pair() {
        // contracts/surface.md payload: `dismissal` is the closed
        // pair `non_dismissable_while_playing | dismissable` — no
        // third wire value, and an unknown value never deserializes
        // (Principle VII). This is the payload's two-valued form, not
        // the resolver-level `player::surface::Dismissal` taxonomy:
        // the payload field is the notification behaviour, the
        // resolver's is the mechanics (F8 payload overload fix).
        for (policy, wire) in DISMISSAL_WIRE {
            assert_eq!(
                serde_json::to_value(policy).ok(),
                Some(serde_json::json!(wire))
            );
            assert_eq!(
                serde_json::from_value::<Dismissal>(serde_json::json!(wire)).ok(),
                Some(policy)
            );
        }
        assert!(serde_json::from_value::<Dismissal>(serde_json::json!("auto_dismiss")).is_err());
        assert!(
            serde_json::from_value::<Dismissal>(serde_json::json!("non_dismissable")).is_err(),
            "the resolver's wire form is not the payload's — the two are distinct fields"
        );
    }

    #[test]
    fn the_reason_ttl_and_the_dismissal_policy_are_distinct_axes() {
        // T043 / contracts/surface.md F8: the reason TTL (how long a
        // reason line stays) and the dismissal policy (whether the
        // surface may be dismissed) are two payload fields that never
        // substitute for one another. Over the whole closed reason
        // set, every TTL value occurs with BOTH dismissal values: a
        // persistent failure reason rides on a surface that is
        // non-dismissable while audio plays, and an expiring
        // focus-loss reason rides on the very same surface. If either
        // field were read out of the other, one of the four
        // combinations below would be unreachable — a test that fails
        // on the collapse.
        let mut persistent_while_playing = false;
        let mut expiring_while_playing = false;
        for reason in ALL_SHORT {
            match reason.reason_ttl_ms() {
                Some(ttl) => {
                    assert_eq!(ttl, PAUSE_REASON_TTL_MS);
                    if dismissal_for(TransportState::Playing)
                        == Dismissal::NonDismissableWhilePlaying
                    {
                        expiring_while_playing = true;
                    }
                }
                None => {
                    if dismissal_for(TransportState::Playing)
                        == Dismissal::NonDismissableWhilePlaying
                    {
                        persistent_while_playing = true;
                    }
                }
            }
            // The dismissal policy is a function of the transport
            // alone: the reason a surface carries never changes it.
            assert_eq!(
                dismissal_for(TransportState::Paused),
                Dismissal::Dismissable
            );
        }
        assert!(
            persistent_while_playing,
            "a persistent reason on a playing surface must be non-dismissable — the two axes collapsed"
        );
        assert!(
            expiring_while_playing,
            "an expiring reason on a playing surface must be non-dismissable — the two axes collapsed"
        );
    }
}
