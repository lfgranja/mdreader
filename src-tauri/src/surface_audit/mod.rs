//! Surface-audit twins for feature 005 (host checks C-H1..C-H8).
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
    /// All six auditees.
    pub const ALL: [Self; 6] = [
        Self::StatusViewHtml,
        Self::StatusViewRs,
        Self::MediaMirrorKt,
        Self::DesignSonaDir,
        Self::TauriConfJson,
        Self::ConchaRLogoSvg,
    ];

    /// Resolve the auditee path relative to `CARGO_MANIFEST_DIR/..`.
    #[must_use]
    pub fn path(&self, manifest_dir: &Path) -> PathBuf {
        let root = manifest_dir.parent().unwrap_or(manifest_dir);
        match self {
            Self::StatusViewHtml => root.join("src-tauri/src/status_view.html"),
            Self::StatusViewRs => root.join("src-tauri/src/status_view.rs"),
            Self::MediaMirrorKt => {
                root.join("android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt")
            }
            Self::DesignSonaDir => root.join("DESIGN-SONA"),
            Self::TauriConfJson => root.join("src-tauri/tauri.conf.json"),
            Self::ConchaRLogoSvg => root.join("DESIGN-SONA/assets/logo/logo-primary-concha-r.svg"),
        }
    }

    /// Resolve and verify existence; missing auditee => named error.
    ///
    /// # Errors
    /// Returns a named error if the auditee is missing.
    pub fn resolve(&self, manifest_dir: &Path) -> Result<PathBuf, String> {
        let p = self.path(manifest_dir);
        if !p.exists() {
            return Err(format!(
                "surface_audit: missing auditee {:?} at {}",
                self,
                p.display()
            ));
        }
        Ok(p)
    }
}

/// Resolve all six auditees; fail-closed on any miss.
///
/// # Errors
/// Returns a named error if any auditee is missing.
pub fn resolve_all(manifest_dir: &Path) -> Result<Vec<(Auditee, PathBuf)>, String> {
    let mut out = Vec::with_capacity(6);
    for a in Auditee::ALL {
        let p = a.resolve(manifest_dir)?;
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
