//! Android voice-asset extraction (Android-only).
//!
//! Voice files ship inside the APK under `assets/voices/...`, where
//! the synthesis engine cannot open them by path: the Tauri resource
//! dir resolves to an `asset://` URL, not a filesystem location. On
//! first startup this module copies the tree through the platform
//! `AssetManager` into the app data dir and returns the real path,
//! which the bridge hands to
//! [`set_asset_dir`](crate::synth::set_asset_dir). A marker file
//! skips re-extraction on later starts.

use std::ffi::CString;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Asset prefix of the voice tree inside the APK.
const ASSET_ROOT: &str = "voices/pt_BR-faber-medium";
/// Model file proving a complete tree.
const MODEL_FILE: &str = "pt_BR-faber-medium.onnx";
/// Marker file proving a complete extraction.
const MARKER_FILE: &str = ".extracted";
/// Build-time manifest: every voice file relative to [`ASSET_ROOT`],
/// one per line. Directory listing on device proved unreliable, so
/// the copy follows this list instead of walking the asset tree.
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

/// Copy the packaged voice tree into `data_dir/voices/...` and
/// return the extracted dir.
///
/// Idempotent: when the marker and model files already exist the
/// copy is skipped.
///
/// # Errors
///
/// * [`ExtractError::Handles`] when the JVM/activity handles cannot
///   be attached.
/// * [`ExtractError::OpenAsset`] when an asset is missing.
/// * [`ExtractError::Io`] when a write fails.
pub fn extract_voices(data_dir: &Path) -> Result<PathBuf, ExtractError> {
    let out = data_dir.join("voices").join("pt_BR-faber-medium");
    if is_fresh(&out) {
        return Ok(out);
    }
    // Clear a stale partial tree (e.g. a previous run that stored a
    // subdirectory as an empty file); the copy below rebuilds it.
    let _ignored = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| ExtractError::Io(e.to_string()))?;
    let manager = asset_manager().map_err(ExtractError::Handles)?;
    copy_dir(&manager, ASSET_ROOT, &out)?;
    std::fs::write(out.join(MARKER_FILE), "1").map_err(|e| ExtractError::Io(e.to_string()))?;
    Ok(out)
}

/// A tree is fresh when the marker, the model, and the espeak data
/// dir (a real dir, not an empty file) all exist.
fn is_fresh(out: &Path) -> bool {
    out.join(MARKER_FILE).exists()
        && out.join(MODEL_FILE).exists()
        && out.join("espeak-ng-data").is_dir()
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
/// Files are opened by explicit path: `open()` on files is proven
/// on device, while nested directory listing is not, so no listing
/// call participates. A file that will not open is a named failure —
/// never silently skipped.
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
