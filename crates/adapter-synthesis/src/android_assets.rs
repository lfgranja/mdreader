//! Android voice-asset extraction (Android-only).
//!
//! Voice files ship inside the APK under `assets/voices/...`, where
//! the synthesis engine cannot open them by path. On first startup
//! this module copies the bundled catalog tree through the platform
//! `AssetManager` into the app data dir and returns the voices root,
//! which the bridge hands to [`set_voices_dir`](crate::synth::set_voices_dir).
//! A marker file skips re-extraction on later starts.

use std::ffi::CString;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Asset prefix of the voices catalog inside the APK.
const ASSET_ROOT: &str = "voices";
/// Marker file proving a complete extraction.
const MARKER_FILE: &str = ".extracted";
/// Catalog extraction version (bump to force re-extraction on layout changes).
const CATALOG_VERSION: &str = "1";
/// Version marker filename.
const VERSION_FILE: &str = "catalog.version";
/// Build-time manifest: every voice file relative to [`ASSET_ROOT`], one per line.
const MANIFEST: &str = include_str!(concat!(env!("OUT_DIR"), "/voice_manifest.txt"));

/// Why voice extraction failed.
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    /// The JVM or activity handles were unavailable.
    #[error("Android handles unavailable: {0}")]
    Handles(String),
    /// An asset could not be opened.
    #[error("cannot open asset {0}")]
    OpenAsset(String),
    /// A filesystem write failed.
    #[error("I/O failure: {0}")]
    Io(String),
}

/// Copy the packaged voices catalog into `data_dir/voices/...` and
/// return the extracted voices root.
///
/// Idempotent: when the marker/version and required catalog files exist
/// the copy is skipped. Migration-safe: version bump triggers rebuild.
///
/// # Errors
///
/// * [`ExtractError::Handles`] when the JVM/activity handles cannot
///   be attached.
/// * [`ExtractError::OpenAsset`] when an asset is missing.
/// * [`ExtractError::Io`] when a write fails.
pub fn extract_voices(data_dir: &Path) -> Result<PathBuf, ExtractError> {
    let out = data_dir.join("voices");
    if is_fresh(&out) {
        return Ok(out);
    }
    // Clear stale partial tree; the copy below rebuilds it.
    let _ignored = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| ExtractError::Io(e.to_string()))?;
    let manager = asset_manager().map_err(ExtractError::Handles)?;
    copy_dir(&manager, ASSET_ROOT, &out)?;
    std::fs::write(out.join(VERSION_FILE), CATALOG_VERSION)
        .map_err(|e| ExtractError::Io(e.to_string()))?;
    std::fs::write(out.join(MARKER_FILE), "1").map_err(|e| ExtractError::Io(e.to_string()))?;
    Ok(out)
}

/// A catalog tree is fresh when marker+version match and required files exist.
fn is_fresh(out: &Path) -> bool {
    let version_ok = std::fs::read_to_string(out.join(VERSION_FILE))
        .map(|v| v.trim() == CATALOG_VERSION)
        .unwrap_or(false);
    if !version_ok || !out.join(MARKER_FILE).exists() {
        return false;
    }
    // Require each bundled voice model and shared espeak dir.
    let faber = out
        .join("pt_BR-faber-medium")
        .join("pt_BR-faber-medium.onnx");
    let cadu = out.join("pt_BR-cadu-medium").join("pt_BR-cadu-medium.onnx");
    let espeak = out.join("shared").join("espeak-ng-data");
    faber.exists() && cadu.exists() && espeak.is_dir()
}

/// Attach to the JVM and wrap the activity `AssetManager`.
#[expect(
    unsafe_code,
    reason = "JNI boundary is inherently unsafe: from_raw wraps the VM pointer the \
              runtime initialized, and from_ptr wraps the AssetManager the activity \
              returned; both pointers are valid exactly here, and no safe API crosses \
              the JVM boundary"
)]
fn asset_manager() -> Result<ndk::asset::AssetManager, String> {
    unsafe {
        let ctx = ndk_context::android_context();
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).map_err(|e| e.to_string())?;
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let context = jni::objects::JObject::from_raw(ctx.context() as jni::sys::jobject);
        let assets = env
            .call_method(
                context,
                "getAssets",
                "()Landroid/content/res/AssetManager;",
                &[],
            )
            .map_err(|e| e.to_string())?
            .l()
            .map_err(|e| e.to_string())?;
        let raw = ndk_sys::AAssetManager_fromJava(env.get_raw(), assets.as_raw());
        let ptr =
            std::ptr::NonNull::new(raw).ok_or_else(|| "AssetManager handle is null".to_string())?;
        Ok(ndk::asset::AssetManager::from_ptr(ptr))
    }
}

/// Copy every manifest file from the APK into `out_dir`,
/// recreating parent dirs.
///
/// Files are opened by explicit path: a file that will not open is a
/// named failure — never silently skipped.
fn copy_dir(
    manager: &ndk::asset::AssetManager,
    asset_path: &str,
    out_dir: &Path,
) -> Result<(), ExtractError> {
    for relative in MANIFEST.lines() {
        let relative = relative.trim();
        if relative.is_empty() {
            continue;
        }
        let full = format!("{asset_path}/{relative}");
        let full_c =
            CString::new(full.as_str()).map_err(|_| ExtractError::OpenAsset(full.clone()))?;
        let mut asset = manager
            .open(&full_c)
            .ok_or_else(|| ExtractError::OpenAsset(full.clone()))?;
        let mut bytes = Vec::new();
        let _byte_count = asset
            .read_to_end(&mut bytes)
            .map_err(|e| ExtractError::Io(e.to_string()))?;
        let dest = out_dir.join(relative);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ExtractError::Io(e.to_string()))?;
        }
        std::fs::write(&dest, &bytes).map_err(|e| ExtractError::Io(e.to_string()))?;
    }
    Ok(())
}
