//! Pre-roll ring buffer (T039; FR-015a / FR-015b / FR-015c).
//!
//! Holds at most [`crate::position::PREROLL_CAP_SECS`] (60 s) of
//! engine-delivered audio and never more (FR-015a).
//!
//! **Unified cap**: This type, the [`PreRollRing`] in `skip.rs`,
//! and the [`PreRollBuffer`] in `transport.rs` all depend on the
//! single [`crate::position::PREROLL_CAP_SECS`] constant — there
//! is no second cap value that can drift (Principle VII / FR-044;
//! the minimal `crate::transport::PreRollBuffer` shape reads the
//! same constant). The three types have different APIs but guarantee
//! the same 60-second bound.
//!
//! Once the reader has acted to hear audio — a Play or resume press,
//! or playback beginning — the pipeline keeps preparing audio ahead
//! of the playback point without playing it ([`PreRollBuffer::on_play_pressed`]
//! then [`PreRollBuffer::push_delivered`]). The buffer:
//!
//! * holds at most [`crate::position::PREROLL_CAP_SECS`] (60 s) of
//!   engine-delivered audio and never more (FR-015a);
//! * is measured against audio actually handed to the playback module
//!   by the engine (FR-015b): only [`PreRollBuffer::push_delivered`]
//!   grows it, and the cap is the single
//!   [`crate::position::PREROLL_CAP_SECS`] constant — there is no
//!   second cap value that can drift (Principle VII / FR-044; the
//!   minimal [`crate::transport::PreRollBuffer`] shape reads the same
//!   constant);
//! * never advances position (FR-015a): the buffer holds no position
//!   field and touches no [`crate::position::PositionClock`] — the
//!   clock advances only on audible progress, never on buffering;
//! * stays empty while idle (FR-015c): a paused session with no Play
//!   press prepares nothing and spends no synthesis budget, so
//!   [`PreRollBuffer::drain_for_idle`] empties it and leaves it
//!   non-preparing;
//! * is abandoned at session end ([`PreRollBuffer::abandon_at_end`],
//!   FR-015a);
//! * re-prepares at a new rate without shifting position
//!   ([`PreRollBuffer::reprepare_for_rate`]): the buffered audio is
//!   discarded and prepared again at the new rung while the position
//!   stays exactly where it was (spec edge case: rate changed while
//!   audio is pre-rolled but not playing; FR-019a / FR-015a). Rate
//!   stretch itself is the engine's contract op (FR-043) — this
//!   buffer only discards the stale preparation.

use engine_core::closed::PlaybackRate;
use thiserror::Error;
use tracing::{debug, instrument};

use crate::position::PREROLL_CAP_SECS;

/// Errors raised by the pre-roll buffer.
///
/// Every variant is a named, surfaced failure — the buffer never
/// fails silently and never absorbs an unexpected condition.
#[derive(Debug, Error)]
pub enum BufferError {
    /// Audio was handed over while the buffer was idle (FR-015c):
    /// pre-roll begins only on a Play press, so an idle delivery is
    /// rejected rather than buffered.
    #[error("pre-roll delivery while idle: press Play before the engine hands over audio")]
    NotPreparing,
    /// The delivered amount was negative (or not a number): only
    /// finite, non-negative durations describe handed-over audio.
    #[error("pre-roll delivery must be non-negative, got {0}")]
    NegativeDelivery(f64),
}

/// Pre-roll ring buffer: at most 60 s of engine-delivered audio held
/// ahead of the playback point without playing it (FR-015a).
///
/// The buffer is position-free by construction: it stores a buffered
/// duration only, so no buffering path can advance, shift, or reset
/// the playback position — including
/// [`reprepare_for_rate`](PreRollBuffer::reprepare_for_rate).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreRollBuffer {
    /// Seconds of engine-delivered audio currently held, within
    /// `0.0..=PREROLL_CAP_SECS`.
    buffered_secs: f64,
    /// Whether a Play press has opened preparation (FR-015a /
    /// FR-015c). Deliveries land only while preparing.
    preparing: bool,
    /// The ladder rung the held audio was prepared at. A rate change
    /// discards the held audio so mismatched audio is never played
    /// (FR-019a / FR-015a).
    prepared_rate: PlaybackRate,
}

impl PreRollBuffer {
    /// Create an empty, idle buffer at the default rung
    /// ([`PlaybackRate::R1_0`]): nothing held, nothing preparing, no
    /// synthesis spent (FR-015c).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffered_secs: 0.0,
            preparing: false,
            prepared_rate: PlaybackRate::R1_0,
        }
    }

    /// The hard pre-roll cap in seconds (FR-015a): the single
    /// [`crate::position::PREROLL_CAP_SECS`] value, never a copy.
    #[must_use]
    pub const fn capacity_secs() -> f64 {
        PREROLL_CAP_SECS
    }

    /// Seconds of engine-delivered audio currently held (FR-015b).
    #[must_use]
    pub const fn buffered_secs(&self) -> f64 {
        self.buffered_secs
    }

    /// Whether the buffer holds no audio. True while idle (FR-015c)
    /// and after abandonment at session end (FR-015a).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffered_secs <= 0.0
    }

    /// Whether preparation is open: true after a Play press, false
    /// while idle and after session end (FR-015a / FR-015c).
    #[must_use]
    pub const fn is_preparing(&self) -> bool {
        self.preparing
    }

    /// The ladder rung the held audio was prepared at.
    #[must_use]
    pub const fn prepared_rate(&self) -> PlaybackRate {
        self.prepared_rate
    }

    /// Open preparation on a Play or resume press (FR-015a /
    /// FR-015c): pre-roll begins here, never while idle. Re-pressing
    /// Play while already preparing keeps the audio held so far.
    #[instrument(skip(self))]
    pub fn on_play_pressed(&mut self) {
        self.preparing = true;
        debug!("pre-roll preparation opened on Play press");
    }

    /// Add engine-delivered audio to the buffer (FR-015b): the
    /// pipeline-side bound, measured against audio actually handed to
    /// the playback module. The total saturates at
    /// [`capacity_secs`](PreRollBuffer::capacity_secs); excess is
    /// discarded, never held beyond the bound (FR-015a).
    ///
    /// Never advances position: the buffer stores a duration only.
    ///
    /// # Errors
    ///
    /// * [`BufferError::NotPreparing`] when idle — pre-roll follows
    ///   the reader's intent to listen, not the existence of a paused
    ///   session (FR-015c).
    /// * [`BufferError::NegativeDelivery`] when `delivered_secs` is
    ///   negative or not a number.
    #[instrument(skip(self))]
    pub fn push_delivered(&mut self, delivered_secs: f64) -> Result<f64, BufferError> {
        if !self.preparing {
            return Err(BufferError::NotPreparing);
        }
        if !(delivered_secs >= 0.0) {
            return Err(BufferError::NegativeDelivery(delivered_secs));
        }
        self.buffered_secs = (self.buffered_secs + delivered_secs).min(PREROLL_CAP_SECS);
        debug!(
            buffered = self.buffered_secs,
            delivered = delivered_secs,
            "pre-roll audio buffered"
        );
        Ok(self.buffered_secs)
    }

    /// Empty the buffer and close preparation for an idle pause
    /// (FR-015c): a paused session with no Play press prepares no
    /// audio and spends no synthesis budget, so the buffer stays
    /// empty throughout the idle interval.
    #[instrument(skip(self))]
    pub fn drain_for_idle(&mut self) {
        self.buffered_secs = 0.0;
        self.preparing = false;
        debug!("pre-roll drained for idle pause");
    }

    /// Abandon the buffer at session end (FR-015a): the held audio
    /// is discarded and preparation closes, so nothing leaks past
    /// the session.
    #[instrument(skip(self))]
    pub fn abandon_at_end(&mut self) {
        self.buffered_secs = 0.0;
        self.preparing = false;
        debug!("pre-roll abandoned at session end");
    }

    /// Re-prepare at a new ladder rung without shifting position
    /// (FR-019a / FR-015a edge case): the audio held at the old rung
    /// is discarded — playing it back at a mismatched rate is
    /// prohibited — and the buffer keeps its preparing state so the
    /// engine prepares again at `rate`. The playback position is
    /// untouched: this type holds no position to shift.
    #[instrument(skip(self))]
    pub fn reprepare_for_rate(&mut self, rate: PlaybackRate) {
        self.buffered_secs = 0.0;
        self.prepared_rate = rate;
        debug!("pre-roll discarded for rate re-preparation");
    }
}

impl Default for PreRollBuffer {
    /// Empty, idle buffer at the default rung (see
    /// [`PreRollBuffer::new`]).
    fn default() -> Self {
        Self::new()
    }
}
