//! Closed bundled offline pt-BR voice catalog for spec 018 (Option B).
//!
//! The catalog is a fixed, exhaustive set of Piper VITS voices shipped
//! with the app (CC0 provenance). Voice ids equal their bundled
//! directory names and are the only selectable identities. Unknown ids
//! are rejected at the boundary (fail-closed, no `Other`/`Unknown`).

use std::fmt;
use std::str::FromStr;

/// Error returned when parsing an unknown voice id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseVoiceIdError {
    /// The rejected input string.
    pub input: String,
}

impl fmt::Display for ParseVoiceIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown voice id: {}", self.input)
    }
}

impl std::error::Error for ParseVoiceIdError {}

/// Inference defaults for a Piper VITS voice (from its `.onnx.json`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InferenceDefaults {
    /// `noise_scale` (variability).
    pub noise_scale: f32,
    /// `noise_scale_w` / `noise_w` (duration variability).
    pub noise_scale_w: f32,
    /// `length_scale` (speaking rate factor, 1.0 = model default).
    pub length_scale: f32,
}

/// Immutable specification for one bundled catalog voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceSpec {
    /// Catalog voice identifier (directory name).
    pub voice_id: VoiceId,
    /// Non-imitative display label (no character/performer branding).
    pub label: &'static str,
    /// Bundled directory name under `assets/voices/`.
    pub dir_name: &'static str,
    /// ONNX model filename inside the voice directory.
    pub onnx: &'static str,
    /// Tokens filename inside the voice directory.
    pub tokens: &'static str,
    /// Provenance filename inside the voice directory.
    pub provenance: &'static str,
}

/// Bundled voice identity (closed catalog).
///
/// Ids are stable directory names. The catalog order is fixed and
/// deterministic: index 0 is the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceId {
    /// Default bundled voice (index 0).
    PtBrFaberMedium,
    /// Curated alternative voice (index 1).
    PtBrCaduMedium,
}

impl VoiceId {
    /// Default voice for the app (index 0).
    pub const DEFAULT: Self = Self::PtBrFaberMedium;

    /// Fixed catalog order (exhaustive, deterministic).
    pub const CATALOG: &'static [Self] = &[Self::PtBrFaberMedium, Self::PtBrCaduMedium];

    /// Returns an iterator over the fixed catalog in order.
    pub fn iter() -> std::slice::Iter<'static, Self> {
        Self::CATALOG.iter()
    }

    /// Stable directory-name id for this voice.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::PtBrFaberMedium => "pt_BR-faber-medium",
            Self::PtBrCaduMedium => "pt_BR-cadu-medium",
        }
    }

    /// Non-imitative display label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PtBrFaberMedium => "Default reading voice",
            Self::PtBrCaduMedium => "Curated alternative — calm, clear",
        }
    }

    /// Zero-based catalog index.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::PtBrFaberMedium => 0,
            Self::PtBrCaduMedium => 1,
        }
    }

    /// Returns the next voice in catalog order with deterministic wrap.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::PtBrFaberMedium => Self::PtBrCaduMedium,
            Self::PtBrCaduMedium => Self::PtBrFaberMedium,
        }
    }

    /// Returns the previous voice in catalog order with deterministic wrap.
    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Self::PtBrFaberMedium => Self::PtBrCaduMedium,
            Self::PtBrCaduMedium => Self::PtBrFaberMedium,
        }
    }

    /// Parse a voice id string (total over declared ids only).
    ///
    /// # Errors
    ///
    /// Returns [`ParseVoiceIdError`] for unknown ids (no silent remap).
    pub fn parse_id(s: &str) -> Result<Self, ParseVoiceIdError> {
        match s {
            "pt_BR-faber-medium" => Ok(Self::PtBrFaberMedium),
            "pt_BR-cadu-medium" => Ok(Self::PtBrCaduMedium),
            _ => Err(ParseVoiceIdError {
                input: s.to_owned(),
            }),
        }
    }
}

impl fmt::Display for VoiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl FromStr for VoiceId {
    type Err = ParseVoiceIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_id(s)
    }
}

/// Returns the immutable specification for `voice`.
#[must_use]
pub const fn spec(voice: VoiceId) -> VoiceSpec {
    match voice {
        VoiceId::PtBrFaberMedium => VoiceSpec {
            voice_id: VoiceId::PtBrFaberMedium,
            label: "Default reading voice",
            dir_name: "pt_BR-faber-medium",
            onnx: "pt_BR-faber-medium.onnx",
            tokens: "tokens.txt",
            provenance: "PROVENANCE.md",
        },
        VoiceId::PtBrCaduMedium => VoiceSpec {
            voice_id: VoiceId::PtBrCaduMedium,
            label: "Curated alternative — calm, clear",
            dir_name: "pt_BR-cadu-medium",
            onnx: "pt_BR-cadu-medium.onnx",
            tokens: "tokens.txt",
            provenance: "PROVENANCE.md",
        },
    }
}

/// Returns inference defaults for `voice` (from bundled `.onnx.json`).
#[must_use]
pub const fn inference_defaults(voice: VoiceId) -> InferenceDefaults {
    match voice {
        VoiceId::PtBrFaberMedium => InferenceDefaults {
            noise_scale: 0.667,
            noise_scale_w: 0.8,
            length_scale: 1.0,
        },
        VoiceId::PtBrCaduMedium => InferenceDefaults {
            noise_scale: 0.5,
            noise_scale_w: 0.333,
            length_scale: 1.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_closed_and_ordered() {
        assert_eq!(VoiceId::CATALOG.len(), 2);
        assert_eq!(VoiceId::CATALOG[0], VoiceId::PtBrFaberMedium);
        assert_eq!(VoiceId::CATALOG[1], VoiceId::PtBrCaduMedium);
        assert_eq!(VoiceId::DEFAULT, VoiceId::PtBrFaberMedium);
    }

    #[test]
    fn ids_equal_directory_names() {
        assert_eq!(VoiceId::PtBrFaberMedium.id(), "pt_BR-faber-medium");
        assert_eq!(VoiceId::PtBrCaduMedium.id(), "pt_BR-cadu-medium");
        assert_eq!(
            spec(VoiceId::PtBrFaberMedium).dir_name,
            "pt_BR-faber-medium"
        );
        assert_eq!(spec(VoiceId::PtBrCaduMedium).dir_name, "pt_BR-cadu-medium");
    }

    #[test]
    fn cycle_wraps_deterministically() {
        assert_eq!(VoiceId::PtBrFaberMedium.next(), VoiceId::PtBrCaduMedium);
        assert_eq!(VoiceId::PtBrCaduMedium.next(), VoiceId::PtBrFaberMedium);
        assert_eq!(VoiceId::PtBrFaberMedium.prev(), VoiceId::PtBrCaduMedium);
        assert_eq!(VoiceId::PtBrCaduMedium.prev(), VoiceId::PtBrFaberMedium);
    }

    #[test]
    fn parse_accepts_known_rejects_unknown() {
        assert_eq!(
            VoiceId::from_str("pt_BR-faber-medium"),
            Ok(VoiceId::PtBrFaberMedium)
        );
        assert_eq!(
            VoiceId::from_str("pt_BR-cadu-medium"),
            Ok(VoiceId::PtBrCaduMedium)
        );
        assert!(VoiceId::from_str("pt_BR-edresson-low").is_err());
        assert!(VoiceId::from_str("unknown").is_err());
        assert!(VoiceId::from_str("").is_err());
    }

    #[test]
    fn inference_defaults_match_configs() {
        let f = inference_defaults(VoiceId::PtBrFaberMedium);
        assert!((f.noise_scale - 0.667).abs() < 1e-6);
        assert!((f.noise_scale_w - 0.8).abs() < 1e-6);
        assert!((f.length_scale - 1.0).abs() < 1e-6);
        let c = inference_defaults(VoiceId::PtBrCaduMedium);
        assert!((c.noise_scale - 0.5).abs() < 1e-6);
        assert!((c.noise_scale_w - 0.333).abs() < 1e-6);
        assert!((c.length_scale - 1.0).abs() < 1e-6);
    }
}
