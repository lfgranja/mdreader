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
    pub const ALL: [Auditee; 6] = [
        Auditee::StatusViewHtml,
        Auditee::StatusViewRs,
        Auditee::MediaMirrorKt,
        Auditee::DesignSonaDir,
        Auditee::TauriConfJson,
        Auditee::ConchaRLogoSvg,
    ];

    /// Resolve the auditee path relative to `CARGO_MANIFEST_DIR/..`.
    pub fn path(&self, manifest_dir: &Path) -> PathBuf {
        let root = manifest_dir.parent().unwrap_or(manifest_dir);
        match self {
            Auditee::StatusViewHtml => root.join("src-tauri/src/status_view.html"),
            Auditee::StatusViewRs => root.join("src-tauri/src/status_view.rs"),
            Auditee::MediaMirrorKt => {
                root.join("android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt")
            }
            Auditee::DesignSonaDir => root.join("DESIGN-SONA"),
            Auditee::TauriConfJson => root.join("src-tauri/tauri.conf.json"),
            Auditee::ConchaRLogoSvg => {
                root.join("DESIGN-SONA/assets/logo/logo-primary-concha-r.svg")
            }
        }
    }

    /// Resolve and verify existence; missing auditee => named error.
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
pub fn resolve_all(manifest_dir: &Path) -> Result<Vec<(Auditee, PathBuf)>, String> {
    let mut out = Vec::with_capacity(6);
    for a in Auditee::ALL {
        let p = a.resolve(manifest_dir)?;
        out.push((a, p));
    }
    Ok(out)
}

pub mod served_identity;

pub mod accent;
