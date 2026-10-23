//! The engine trait and capability declaration (T016).
//!
//! Engines are reached through the [`Engine`] trait (Principle VI):
//! no vendor SDK type, no HTTP response shape, no SDK error code,
//! and no provider-specific identifier crosses into the core.
//! Every adapter declares what it can deliver through
//! [`Engine::declared_capabilities`], and the pipeline degrades
//! observably to the declared coarser level — never silently
//! (contracts/engine-handoff.md, "Engine trait").
//!
//! The delivered audio shape is fixed by the specification, not by
//! implementation: mono `f32` PCM resampled to exactly
//! [`crate::format::DeclaredFormat::SAMPLE_RATE`] by the engine
//! (FR-019) (FR-033, superseded by FR-019). Any deviation is a named
//! [`crate::closed::EngineError`]
//! — never a silent resample or downmix (FR-035, SC-034).
//!
//! The synthesis adapter of the follow-up feature predates this
//! trait and declares the same pair this trait names —
//! ([`SsmlLevel::None`], [`AlignmentGranularity::None`]) — against
//! the existing [`crate::format`] and [`crate::closed`] shapes.
//! That declaration stays valid: `None`/`None` is one pair of the
//! closed vocabulary below, not a parallel list.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::closed::{EngineError, PlaybackRate};

/// The maximum pre-roll an engine may hold outstanding: `60.0`
/// seconds (contracts/engine-handoff.md, "`stream()` pre-roll
/// source; ≤60 s outstanding").
///
/// A `stream` budget above this bound is rejected before any
/// synthesis runs. Position never advances on buffered-only
/// audio: pre-roll is a source to read from, not playback.
pub const MAX_PREROLL_SECS: f64 = 60.0;

/// The closed SSML capability level (Principle VII): exactly
/// `None | Partial | Full` — no `Other`/`Unknown` variant exists.
///
/// Every adapter declares one level through
/// [`Engine::declared_capabilities`]; the pipeline renders SSML
/// only at or below the declared level and degrades observably
/// when the declared level is coarser than the document asks
/// for (Principle VI). The wire form is the `snake_case` level
/// name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SsmlLevel {
    /// No SSML rendering: the adapter delivers plain text only.
    #[serde(rename = "none")]
    None,
    /// Partial SSML: a declared subset of SSML elements.
    #[serde(rename = "partial")]
    Partial,
    /// Full SSML: the complete neutral SSML form.
    #[serde(rename = "full")]
    Full,
}

impl SsmlLevel {
    /// The wire form of the level: `"none"`, `"partial"`, or
    /// `"full"`.
    ///
    /// The match is exhaustive over the closed three-member
    /// set — no catch-all arm — so an added member fails to
    /// compile here (constitution gate 7).
    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Partial => "partial",
            Self::Full => "full",
        }
    }
}

impl Display for SsmlLevel {
    /// The human-readable form is the wire form, so a level
    /// interpolated into a diagnostic can never drift from its
    /// machine-readable form.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_wire_str())
    }
}

/// The closed alignment-granularity level (Principle VII):
/// exactly `None | Word | Sentence` — no `Other`/`Unknown`
/// variant exists.
///
/// Every adapter declares one granularity through
/// [`Engine::declared_capabilities`]; the pipeline aligns
/// synthesis back to source nodes at most at the declared
/// granularity and degrades observably to a coarser, declared
/// granularity when the adapter cannot deliver sentence
/// alignment — never by silently highlighting nothing
/// (constitution, Audio Pipeline Contracts). The wire form is
/// the `snake_case` granularity name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentGranularity {
    /// Paragraph/heading boundary only: no word or sentence
    /// events are delivered.
    #[serde(rename = "none")]
    None,
    /// Word-level boundary events are delivered.
    #[serde(rename = "word")]
    Word,
    /// Sentence-level boundary events are delivered.
    #[serde(rename = "sentence")]
    Sentence,
}

impl AlignmentGranularity {
    /// The wire form of the granularity: `"none"`, `"word"`,
    /// or `"sentence"`.
    ///
    /// The match is exhaustive over the closed three-member
    /// set — no catch-all arm — so an added member fails to
    /// compile here (constitution gate 7).
    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Word => "word",
            Self::Sentence => "sentence",
        }
    }
}

impl Display for AlignmentGranularity {
    /// The human-readable form is the wire form, so a
    /// granularity interpolated into a diagnostic can never
    /// drift from its machine-readable form.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_wire_str())
    }
}

/// The closed stream-boundary kind (Principle VII): exactly
/// `Word | Sentence` — no `Other`/`Unknown` variant exists.
///
/// A boundary event marks the engine's position inside the
/// delivered frames at the granularity the adapter declared.
/// `None` needs no event kind: the absence of events is the
/// `None` declaration made observable. The wire form is the
/// `snake_case` kind name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryKind {
    /// A word boundary inside the delivered frames.
    #[serde(rename = "word")]
    Word,
    /// A sentence boundary inside the delivered frames.
    #[serde(rename = "sentence")]
    Sentence,
}

impl BoundaryKind {
    /// The wire form of the kind: `"word"` or `"sentence"`.
    ///
    /// The match is exhaustive over the closed two-member set
    /// — no catch-all arm — so an added member fails to
    /// compile here (constitution gate 7).
    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::Word => "word",
            Self::Sentence => "sentence",
        }
    }
}

impl Display for BoundaryKind {
    /// The human-readable form is the wire form.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_wire_str())
    }
}

/// One boundary event carried alongside PCM frames: the kind
/// of boundary and the engine-side position it marks, in
/// seconds from the start of the streamed document.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundaryEvent {
    /// The kind of boundary the engine observed.
    pub kind: BoundaryKind,
    /// The engine-side position of the boundary, in seconds.
    pub position_secs: f64,
}

/// One streamed audio chunk: mono `f32` PCM frames plus the
/// boundary events observed inside them.
///
/// The frames are delivered at the declared handoff shape
/// (mono `f32` at [`crate::format::DeclaredFormat::SAMPLE_RATE`]);
/// the events are at most at the granularity the adapter
/// declared through [`Engine::declared_capabilities`]. A chunk
/// is pre-roll source: reading it never advances the playback
/// position — only audible playback does.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamChunk {
    /// The document position the chunk starts at, in seconds.
    pub start_position_secs: f64,
    /// The mono `f32` PCM frames at the declared handoff rate.
    pub frames: Vec<f32>,
    /// The boundary events observed inside `frames`, in
    /// ascending position order.
    pub boundaries: Vec<BoundaryEvent>,
}

impl StreamChunk {
    /// The number of PCM frames in the chunk.
    #[must_use]
    pub const fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// The engine-neutral synthesis interface (Principle VI,
/// contracts/engine-handoff.md).
///
/// Implementations are swappable adapters at the edges of the
/// core; every provider-specific decision stays inside the
/// adapter. All failures surface as the closed
/// [`EngineError`] set — never as silence, never as a
/// substituted format.
pub trait Engine {
    /// Declare what the adapter can deliver: its SSML level
    /// and its alignment granularity.
    ///
    /// The pipeline degrades observably to the declared
    /// coarser level (Principle VI): a heading rendered with
    /// a break and pitch adjustment under one engine, and
    /// flattened to plain text under another, is a difference
    /// the declaration makes visible.
    fn declared_capabilities(&self) -> (SsmlLevel, AlignmentGranularity);

    /// Set the playback rate (FR-019, FR-043).
    ///
    /// The rate change is a pitch-preserving time-stretch
    /// **inside the engine**: it MUST NOT alter the delivered
    /// sample rate, which stays the declared
    /// [`crate::format::DeclaredFormat::SAMPLE_RATE`] on all
    /// seven rungs (SC-034). Measurable tolerance: the pitch
    /// shift stays within the shared SC-015 bound,
    /// [`crate::format::PITCH_TOLERANCE_CENTS`] — a deviation
    /// beyond that bound is an
    /// [`EngineError::RateMismatch`]
    /// (contracts/engine-handoff.md).
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::RateMismatch`] when the stretch
    /// cannot hold the pitch tolerance or the delivered rate;
    /// any other closed [`EngineError`] the adapter names for
    /// a rate-path failure.
    fn set_rate(&mut self, rate: PlaybackRate) -> Result<(), EngineError>;

    /// Stream pre-roll audio starting at `position_secs`, up
    /// to `max_secs` of audio.
    ///
    /// At most [`MAX_PREROLL_SECS`] seconds may be outstanding:
    /// a `max_secs` above that bound is rejected before any
    /// synthesis runs. The returned [`StreamChunk`] is
    /// buffered-only source — reading it never advances the
    /// playback position. `position_secs` is `f64`, `>= 0.0`.
    ///
    /// # Errors
    ///
    /// Returns the closed [`EngineError`] the adapter names
    /// when synthesis or delivery fails
    /// ([`EngineError::Unreachable`], [`EngineError::InitFailed`],
    /// [`EngineError::DeviceRefused`], [`EngineError::DeviceLost`],
    /// [`EngineError::StartRefused`]); a shape deviation in
    /// the delivered audio is [`EngineError::RateMismatch`] or
    /// [`EngineError::ChannelMismatch`], never a silent
    /// resample or downmix.
    fn stream(&mut self, position_secs: f64, max_secs: f64) -> Result<StreamChunk, EngineError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pre_roll_bound_is_sixty_seconds() {
        assert!((MAX_PREROLL_SECS - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ssml_levels_cover_exactly_none_partial_full() {
        fn wire_of(level: SsmlLevel) -> &'static str {
            match level {
                SsmlLevel::None => "none",
                SsmlLevel::Partial => "partial",
                SsmlLevel::Full => "full",
            }
        }
        for (level, wire) in [
            (SsmlLevel::None, "none"),
            (SsmlLevel::Partial, "partial"),
            (SsmlLevel::Full, "full"),
        ] {
            assert_eq!(level.as_wire_str(), wire);
            assert_eq!(wire_of(level), wire);
            assert_eq!(
                serde_json::to_value(level).ok(),
                Some(serde_json::Value::String(String::from(wire)))
            );
            assert_eq!(
                serde_json::from_value::<SsmlLevel>(serde_json::Value::String(String::from(wire)))
                    .ok(),
                Some(level)
            );
        }
    }

    #[test]
    fn alignment_granularities_cover_exactly_none_word_sentence() {
        fn wire_of(granularity: AlignmentGranularity) -> &'static str {
            match granularity {
                AlignmentGranularity::None => "none",
                AlignmentGranularity::Word => "word",
                AlignmentGranularity::Sentence => "sentence",
            }
        }
        for (granularity, wire) in [
            (AlignmentGranularity::None, "none"),
            (AlignmentGranularity::Word, "word"),
            (AlignmentGranularity::Sentence, "sentence"),
        ] {
            assert_eq!(granularity.as_wire_str(), wire);
            assert_eq!(wire_of(granularity), wire);
            assert_eq!(
                serde_json::to_value(granularity).ok(),
                Some(serde_json::Value::String(String::from(wire)))
            );
            assert_eq!(
                serde_json::from_value::<AlignmentGranularity>(serde_json::Value::String(
                    String::from(wire)
                ))
                .ok(),
                Some(granularity)
            );
        }
    }

    #[test]
    fn boundary_kinds_cover_exactly_word_sentence() {
        fn wire_of(kind: BoundaryKind) -> &'static str {
            match kind {
                BoundaryKind::Word => "word",
                BoundaryKind::Sentence => "sentence",
            }
        }
        for (kind, wire) in [
            (BoundaryKind::Word, "word"),
            (BoundaryKind::Sentence, "sentence"),
        ] {
            assert_eq!(kind.as_wire_str(), wire);
            assert_eq!(wire_of(kind), wire);
            assert_eq!(
                serde_json::to_value(kind).ok(),
                Some(serde_json::Value::String(String::from(wire)))
            );
            assert_eq!(
                serde_json::from_value::<BoundaryKind>(serde_json::Value::String(String::from(
                    wire
                )))
                .ok(),
                Some(kind)
            );
        }
    }

    #[test]
    fn capability_wire_strings_reject_other_and_unknown() {
        for word in ["other", "unknown", "sentence_boundary", ""] {
            let value = serde_json::Value::String(String::from(word));
            assert!(
                serde_json::from_value::<SsmlLevel>(value.clone()).is_err(),
                "SsmlLevel accepted {word:?}"
            );
            assert!(
                serde_json::from_value::<AlignmentGranularity>(value.clone()).is_err(),
                "AlignmentGranularity accepted {word:?}"
            );
            assert!(
                serde_json::from_value::<BoundaryKind>(value).is_err(),
                "BoundaryKind accepted {word:?}"
            );
        }
    }

    #[test]
    fn a_chunk_reports_its_frame_count() {
        let chunk = StreamChunk {
            start_position_secs: 12.5,
            frames: vec![0.0; 44_100],
            boundaries: vec![BoundaryEvent {
                kind: BoundaryKind::Sentence,
                position_secs: 13.0,
            }],
        };
        assert_eq!(chunk.frame_count(), 44_100);
    }
}
