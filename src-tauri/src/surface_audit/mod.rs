//! Surface-audit twins for feature 005/007.
//!
//! Closed auditee resolver (A1) + a FIXED byte-scanner parser (A2). Each
//! check lives in its own child module (one file per check) so separate
//! agents never collide.

use std::path::{Path, PathBuf};

/// The closed auditee list (contracts/surface-audit.md A1). A missing
/// auditee is a NAMED failure (fail-closed, never skip).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Auditee {
    /// `src-tauri/src/status_view.html`
    StatusViewHtml,
    /// `src-tauri/src/status_view.rs`
    StatusViewRs,
    /// `android-shell/.../MediaMirror.kt` constants block
    MediaMirrorKt,
    /// `DESIGN-SONA/` directory
    DesignSonaDir,
    /// `src-tauri/tauri.conf.json`
    TauriConfJson,
    /// `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`
    ConchaRLogoSvg,
}

impl Auditee {
    /// Resolve auditee to a relative path from `manifest_dir`.
    #[must_use]
    pub fn relative_path(self) -> &'static str {
        match self {
            Auditee::StatusViewHtml => "src/status_view.html",
            Auditee::StatusViewRs => "src/status_view.rs",
            Auditee::MediaMirrorKt => {
                "android-shell/app/src/main/java/com/lfgranja/mdreader/MediaMirror.kt"
            }
            Auditee::DesignSonaDir => "DESIGN-SONA",
            Auditee::TauriConfJson => "tauri.conf.json",
            Auditee::ConchaRLogoSvg => "DESIGN-SONA/assets/logo/logo-primary-concha-r.svg",
        }
    }
}

/// Resolve all six auditees; fail-closed on any miss.
///
/// # Errors
/// Returns a named error if any auditee is missing.
pub fn resolve_all(manifest_dir: &Path) -> Result<Vec<(Auditee, PathBuf)>, String> {
    let mut out = Vec::with_capacity(6);
    let auditees = [
        Auditee::StatusViewHtml,
        Auditee::StatusViewRs,
        Auditee::MediaMirrorKt,
        Auditee::DesignSonaDir,
        Auditee::TauriConfJson,
        Auditee::ConchaRLogoSvg,
    ];
    for a in auditees {
        let p = manifest_dir.join(a.relative_path());
        if !p.exists() {
            return Err(format!("C-H1..C-H8 auditee missing: {}", p.display()));
        }
        out.push((a, p));
    }
    Ok(out)
}

#[cfg(test)]
pub mod served_identity;

#[cfg(test)]
pub mod accent;

/// aria twin.
#[cfg(test)]
pub mod aria;
#[cfg(test)]
pub mod chroma;
/// contrast twin.
#[cfg(test)]
pub mod contrast;
#[cfg(test)]
pub mod exclusivity;
/// inventory twin.
#[cfg(test)]
pub mod inventory;
/// movement twin.
#[cfg(test)]
pub mod movement;
