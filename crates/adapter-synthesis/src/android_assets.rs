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
    if out.join(MARKER_FILE).exists() && out.join(MODEL_FILE).exists() {
        return Ok(out);
    }
    std::fs::create_dir_all(&out).map_err(|e| ExtractError::Io(e.to_string()))?;
    let manager = asset_manager().map_err(ExtractError::Handles)?;
    copy_dir(&manager, ASSET_ROOT, &out)?;
    std::fs::write(out.join(MARKER_FILE), "1").map_err(|e| ExtractError::Io(e.to_string()))?;
    Ok(out)
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

/// Copy one asset dir into `out_dir`, recursing into subdirs.
///
/// An entry that opens as a file is copied byte-for-byte; an entry
/// that only opens as a dir is recursed into; anything else is a
/// named failure — never silently skipped.
fn copy_dir(
    manager: &ndk::asset::AssetManager,
    asset_path: &str,
    out_dir: &Path,
) -> Result<(), ExtractError> {
    let dir_name =
        CString::new(asset_path).map_err(|_| ExtractError::OpenAsset(asset_path.to_owned()))?;
    let dir = manager
        .open_dir(&dir_name)
        .ok_or_else(|| ExtractError::OpenAsset(asset_path.to_owned()))?;
    let entries: Vec<CString> = dir.collect();
    for entry in &entries {
        let name = entry.to_string_lossy().into_owned();
        let full = format!("{asset_path}/{name}");
        let full_c =
            CString::new(full.as_str()).map_err(|_| ExtractError::OpenAsset(full.clone()))?;
        if let Some(mut asset) = manager.open(&full_c) {
            let mut bytes = Vec::new();
            let _byte_count = asset
                .read_to_end(&mut bytes)
                .map_err(|e| ExtractError::Io(e.to_string()))?;
            std::fs::write(out_dir.join(&name), &bytes)
                .map_err(|e| ExtractError::Io(e.to_string()))?;
        } else if manager.open_dir(&full_c).is_some() {
            let sub = out_dir.join(&name);
            std::fs::create_dir_all(&sub).map_err(|e| ExtractError::Io(e.to_string()))?;
            copy_dir(manager, &full, &sub)?;
        } else {
            return Err(ExtractError::OpenAsset(full));
        }
    }
    Ok(())
}
