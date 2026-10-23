//! Output-device routing (T046; FR-016 / FR-016a / FR-017 / FR-017a).
//!
//! Closed sets throughout, no `Other`/`Unknown`, no `_ =>` arm
//! (Principle VII): [`OutputDevicePrivacy`] (2 classes),
//! [`RoutingSignal`] (3 signals), [`RoutingDecision`] (3
//! decisions). Engine failure surfaces named
//! ([`engine_core::closed::EngineError`]).
//!
//! The routing machine is the sole authority for the routing
//! decision; the session applies it and never computes it
//! (SC-035). It never alters the position (R4): every decision
//! is a pure value the consumer acts on. The one-second bound of
//! FR-016 is owned here: both FR-016a mechanisms — the primary
//! `ACTION_AUDIO_BECOMING_NOISY` and the secondary
//! `AudioDeviceCallback.onAudioDevicesRemoved` — reach the pause
//! within [`ROUTING_PAUSE_BOUND_MS`] on their own, neither waits
//! for the other, and no removal may pass unhandled because the
//! platform delivered only one of them. The bound is the whole
//! privacy guarantee of this feature (SC-002): it is what keeps
//! a private document from moving to a loudspeaker.
//!
//! Behavioral rules (R1–R6, checked against the independent
//! reference model in `tests/prop_routing.rs`):
//!
//! - R1: while playing, a removal by either FR-016a mechanism —
//!   or `BecomingNoisy` — pauses within the bound, naming the
//!   removed route; the position is preserved and the resume is
//!   explicit only (FR-016 / FR-017).
//! - R2: while playing, an appearance follows at the unchanged
//!   position — no pause, restart, or repeated phrase — except a
//!   switch out of a private output to a public one, which pauses
//!   first (FR-016 privacy precedence over FR-017a). The check is
//!   fail-closed: while no output is latched, an appearance
//!   claiming the public class pauses, because the privacy
//!   guarantee is the point of the bound. A mismatched
//!   identifier/privacy claim is enforced by the claimed privacy,
//!   never by the identifier.
//! - R3: while not playing, every signal is a no-op — never an
//!   auto-resume, never a transport change (FR-017). An appearance
//!   still latches the device, so the explicit resume happens
//!   through the device the reader reconnected.
//! - R4: no decision moves the position; the session applies the
//!   decision.
//! - R5: every switch (follow, and route-change pause) carries the
//!   `output_device` record-event payload ([`DeviceEvent`]); a
//!   no-op carries none.
//! - R6: [`RoutingTracker::current_privacy`] latches: a follow
//!   latches the followed device; a removal unlatches only the
//!   removed device, and only when it was the current output;
//!   `BecomingNoisy` unlatches (the route went noisy, no device
//!   remains); a privacy-precedence pause leaves the private
//!   output latched — it remains the current output until the
//!   reader resumes (FR-016).
//!
//! Latency model: the machine is deterministic, so the pause
//! latency is a fixed, declared value ([`ROUTING_PAUSE_LATENCY_MS`])
//! rather than a measurement — always within the bound, leaving the
//! surface layer the remaining headroom to act on the decision.

use diag::writer::{DeviceDirection, DeviceEvent};
use engine_core::closed::TransportState;

/// The one-second bound of FR-016, in milliseconds (SC-002).
pub const ROUTING_PAUSE_BOUND_MS: u64 = 1000;

/// The routing machine's declared pause latency, in milliseconds.
///
/// The machine is a deterministic state machine, so the latency is
/// a fixed, simulated value rather than a measurement: a
/// conservative quarter of [`ROUTING_PAUSE_BOUND_MS`], leaving the
/// surface layer the remaining headroom to act on the pause within
/// the FR-016a one-second guarantee.
const ROUTING_PAUSE_LATENCY_MS: u64 = 250;

/// The route a `BecomingNoisy` pause names before any output device
/// is latched (R1): the platform reported the route itself going
/// noisy, so the event names the route rather than a device.
const ROUTE_FALLBACK_ID: &str = "route";

/// Closed output-device privacy set (FR-016 / FR-017a): exactly two
/// variants.
///
/// Private outputs: wired headsets, wired headphones, Bluetooth
/// headsets (A2DP/SCO worn by the reader). Public outputs: the
/// built-in loudspeaker and car Bluetooth in shared-cabin use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputDevicePrivacy {
    /// Audio reaches only the reader: wired headsets, wired
    /// headphones, and Bluetooth headsets (A2DP/SCO) worn by the
    /// reader.
    Private,
    /// Audio may reach bystanders: the built-in loudspeaker and car
    /// Bluetooth in shared-cabin use.
    Public,
}

/// Closed routing-signal set (FR-016a): exactly three variants.
///
/// The primary `ACTION_AUDIO_BECOMING_NOISY` mechanism and the
/// secondary `AudioDeviceCallback.onAudioDevicesRemoved` mechanism,
/// plus the appearance side of FR-017a.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingSignal {
    /// Primary removal signal (FR-016a): the platform reports the
    /// output route going noisy.
    BecomingNoisy,
    /// Secondary removal signal (FR-016a): an output device was
    /// removed.
    DeviceDisappeared {
        /// The identifier of the removed output device.
        device_id: String,
        /// The privacy class the removed device claims (FR-017a).
        privacy: OutputDevicePrivacy,
    },
    /// A new output device became available (FR-017a).
    DeviceAppeared {
        /// The identifier of the appearing output device.
        device_id: String,
        /// The privacy class the appearing device claims (FR-017a).
        privacy: OutputDevicePrivacy,
    },
}

/// What the routing machine decided for one signal (FR-016 /
/// FR-016a / FR-017 / FR-017a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingDecision {
    /// FR-016: pause within the one-second bound, position
    /// preserved, explicit resume only (FR-017). `device` names the
    /// removed route and carries the `output_device` record-event
    /// payload (FR-017a / FR-039).
    Pause {
        /// The machine's declared processing latency for the pause,
        /// in milliseconds — always within [`ROUTING_PAUSE_BOUND_MS`].
        latency_ms: u64,
        /// The output-device event the pause records (FR-017a /
        /// FR-039).
        device: DeviceEvent,
    },
    /// FR-017a: follow the appearing device at the unchanged
    /// position — no pause, restart, or repeated phrase. `device`
    /// carries the `output_device` record-event payload.
    Follow {
        /// The output-device event the follow records (FR-017a /
        /// FR-039).
        device: DeviceEvent,
    },
    /// The signal changes nothing for playback (an appearance while
    /// not playing, or any signal in a terminal state).
    NoChange,
}

/// The latched output device (R6): the identifier the switch events
/// name and the privacy class [`RoutingTracker::current_privacy`]
/// reports.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceLatch {
    /// The latched output-device identifier.
    device_id: String,
    /// The latched output-device privacy class.
    privacy: OutputDevicePrivacy,
}

/// Stateful routing machine: latches the current output device and
/// applies the closed signal set.
///
/// The machine is the sole authority for the routing decision; the
/// session applies it and never computes it (SC-035). It never
/// alters the position (R4): [`RoutingDecision::Pause`],
/// [`RoutingDecision::Follow`], and [`RoutingDecision::NoChange`]
/// are pure decisions the consumer acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingTracker {
    /// The current output device, or `None` before any device is
    /// latched and after a removal unlatched it.
    current: Option<DeviceLatch>,
}

impl RoutingTracker {
    /// New tracker: no output device latched.
    #[must_use]
    pub const fn new() -> Self {
        Self { current: None }
    }

    /// Apply one signal at the given transport state, returning the
    /// decision.
    ///
    /// While playing (R1 / FR-016a / R2): a removal by either
    /// FR-016a mechanism alone — or `BecomingNoisy` — pauses within
    /// [`ROUTING_PAUSE_BOUND_MS`], naming the removed route; an
    /// appearance follows at the unchanged position, except a switch
    /// out of a private output to a public one, which pauses first
    /// (FR-016 privacy precedence over FR-017a). While not playing
    /// (R3 / FR-017): every signal is a no-op — never an auto-resume,
    /// never a transport change; an appearance still latches the
    /// device, so the explicit resume happens through the device the
    /// reader reconnected.
    pub fn apply(&mut self, signal: RoutingSignal, transport: TransportState) -> RoutingDecision {
        match transport {
            TransportState::Playing => self.apply_while_playing(signal),
            TransportState::Paused
            | TransportState::Interrupted
            | TransportState::Stopped
            | TransportState::Completed => {
                // R3 / FR-017: never auto-resume, and never change
                // the transport. The latch still follows appearances,
                // so the current output is known for the explicit
                // resume the reader owns (FR-017).
                if let RoutingSignal::DeviceAppeared { device_id, privacy } = signal {
                    self.current = Some(DeviceLatch { device_id, privacy });
                }
                RoutingDecision::NoChange
            }
        }
    }

    /// The privacy of the output audio currently occupies, or `None`
    /// before any device is latched.
    #[must_use]
    pub fn current_privacy(&self) -> Option<OutputDevicePrivacy> {
        self.current.as_ref().map(|latch| latch.privacy)
    }

    /// Apply one signal while playing: the FR-016 pause, the FR-017a
    /// follow, or the privacy-precedence pause.
    fn apply_while_playing(&mut self, signal: RoutingSignal) -> RoutingDecision {
        match signal {
            // FR-016a primary: the route went noisy — pause within
            // the bound, naming the latched route (the route itself
            // before any device is latched), and unlatch it (R6): no
            // device remains.
            RoutingSignal::BecomingNoisy => {
                let device_id = self.current.as_ref().map_or_else(
                    || ROUTE_FALLBACK_ID.to_string(),
                    |latch| latch.device_id.clone(),
                );
                self.current = None;
                RoutingDecision::Pause {
                    latency_ms: ROUTING_PAUSE_LATENCY_MS,
                    device: DeviceEvent {
                        direction: DeviceDirection::Disappeared,
                        device_id,
                    },
                }
            }
            // FR-016a secondary: a removal — pause within the bound,
            // whatever the removed device's class (a Bluetooth
            // disconnect carries exactly the wired removal's
            // consequence, FR-016), unlatching only when the removed
            // device was the current output (R6).
            RoutingSignal::DeviceDisappeared { device_id, .. } => {
                let removed_is_current = self
                    .current
                    .as_ref()
                    .is_some_and(|latch| latch.device_id == device_id);
                if removed_is_current {
                    self.current = None;
                }
                RoutingDecision::Pause {
                    latency_ms: ROUTING_PAUSE_LATENCY_MS,
                    device: DeviceEvent {
                        direction: DeviceDirection::Disappeared,
                        device_id,
                    },
                }
            }
            // FR-017a: follow the appearing device — except a switch
            // out of a private output to a public one, which pauses
            // first (FR-016 privacy precedence over FR-017a).
            RoutingSignal::DeviceAppeared { device_id, privacy } => {
                // Fail-closed while no output is latched: an unknown
                // output is treated as private, because the privacy
                // guarantee is the point of the bound (SC-002). A
                // mismatched identifier/privacy claim is enforced by
                // the claimed privacy, never by the identifier (R2).
                let leaves_a_private_output = self
                    .current
                    .as_ref()
                    .is_none_or(|latch| latch.privacy == OutputDevicePrivacy::Private)
                    && privacy == OutputDevicePrivacy::Public;
                if leaves_a_private_output {
                    // The private output remains the current output
                    // until the reader resumes (FR-016): the pause
                    // latches nothing, and the newly available device
                    // is re-evaluated under FR-017a before the
                    // resume.
                    RoutingDecision::Pause {
                        latency_ms: ROUTING_PAUSE_LATENCY_MS,
                        device: DeviceEvent {
                            direction: DeviceDirection::Appeared,
                            device_id,
                        },
                    }
                } else {
                    self.current = Some(DeviceLatch {
                        device_id: device_id.clone(),
                        privacy,
                    });
                    RoutingDecision::Follow {
                        device: DeviceEvent {
                            direction: DeviceDirection::Appeared,
                            device_id,
                        },
                    }
                }
            }
        }
    }
}

impl Default for RoutingTracker {
    fn default() -> Self {
        Self::new()
    }
}
