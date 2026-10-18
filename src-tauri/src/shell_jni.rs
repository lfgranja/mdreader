//! Shell JNI channel (T111, Phase 12 F5 wiring): the Kotlin-to-Rust control path.
//!
//! The Kotlin shell ([`ShellHost`](https://example.invalid) and the
//! service hosting it) classifies every platform callback at the
//! boundary into the closed shell-IPC wire vocabulary and relays it
//! here as a fieldless integer code. This module maps each code back
//! to its wire string and executes it through the single bridge entry
//! [`run_mobile_plugin`](crate::commands::run_mobile_plugin) on
//! [`AppState`](crate::AppState)'s own session — the same code path
//! every other driver calls (Principle II: one path, never a second
//! session, never a duplicated dispatch).
//!
//! The channel carries fieldless verbs and intents only: the entry
//! functions take plain integer codes by value, so no object graph
//! ever crosses the boundary and no privileged operation is needed.
//! `DeviceAppeared` / `DeviceDisappeared` keep T110's fieldless
//! fallback — the identifier never crosses; inventing identifier
//! plumbing is out of scope for this task.
//!
//! Delivery guarantees: a send failure (application not running) or
//! an unknown code drops the event with a [`tracing`] diagnostic —
//! never silent, never a crash, never a playing state.
//!
//! ## The document-ingest entry (T017 / T019)
//!
//! The Kotlin picker's `DocIngest.onDocumentIngested` is the one
//! entry that carries DATA rather than a code: a snapshot FILE
//! PATH and its BLAKE3-256 hex. This module reads that file —
//! the platform boundary's single I/O (the Kotlin side already
//! wrote it) — and forwards its BYTES to
//! [`open_document_bytes`](crate::commands::open_document_bytes)
//! on [`AppState`]. The bytes are then the sole synthesis input
//! (FR-018 / T019): no URI, no path, and no second document
//! object ever crosses into synthesis. A revoked, moved, or
//! deleted snapshot is the named [`IngestError::SnapshotUnreadable`]
//! failure to stderr — never a crash, never a playing state
//! (FR-018: re-prompt, never crash).
//
// Allow unsafe code for JNI operations (FFI boundary).
#![allow(
    unsafe_code,
    reason = "the JNI FFI boundary requires raw-pointer operations"
)]

use std::ffi::c_void;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager};

use crate::AppState;
use crate::commands::run_mobile_plugin;

/// The installed shell-channel sender: the Kotlin entries post codes
/// through this sender toward the drain thread. Set once by
/// [`install`] during application setup; absent before setup (a post
/// then drops with a diagnostic, never a crash).
static VERB_TX: OnceLock<Mutex<Sender<u32>>> = OnceLock::new();

/// Reports whether the shell channel sender is installed.
///
/// The `register_android_plugin` body consults this: an installed
/// channel means the drain thread is live on [`AppState`]'s own
/// session.
#[must_use]
pub fn channel_ready() -> bool {
    VERB_TX.get().is_some()
}

/// The JVM handle captured at one JNI entry (T133) — the handle the
/// Rust-to-Kotlin publish calls back through.
///
/// A `JavaVM` exists only inside the JVM the entries are called from,
/// so the first entry carrying an environment captures it here
/// (idempotent). The Application bootstrap forwards `AppForegrounded`
/// before any session command arrives, and an ingest starts only after
/// an entry fired, so the handle is present before the first publish.
/// The handle is `Send + Sync`; a publish from a background thread
/// attaches that thread for the call and detaches on drop.
#[cfg(target_os = "android")]
static JVM: OnceLock<jni::JavaVM> = OnceLock::new();

/// Captures the JVM handle from one JNI entry's environment.
///
/// The first entry wins; every later call observes the set handle and
/// returns without touching the environment. An unusable environment
/// or an unavailable handle is recorded as a diagnostic — the entry's
/// own work continues, and a publish reports the missing handle
/// loudly at its call site (fail-closed, never a silent half-wire).
#[cfg(target_os = "android")]
fn capture_jvm(raw: *mut jni::sys::JNIEnv) {
    if JVM.get().is_some() {
        return;
    }
    // SAFETY: `raw` is the environment pointer the JVM handed this
    // entry point; it is valid for the duration of the call, and
    // `from_raw` / `get_java_vm` only read it.
    match unsafe { jni::JNIEnv::from_raw(raw) } {
        Ok(env) => match env.get_java_vm() {
            Ok(vm) => {
                let _ = JVM.set(vm);
            }
            Err(error) => {
                tracing::warn!(
                    error = error.to_string(),
                    "shell JNI: JVM handle unavailable at entry"
                );
            }
        },
        Err(error) => {
            tracing::warn!(
                error = error.to_string(),
                "shell JNI: environment unusable at entry"
            );
        }
    }
}

/// Calls the Kotlin `ShellHost.publishMirror` static method over the
/// captured JVM handle (T133): the Rust-to-Kotlin half of the publish
/// contract.
///
/// The payload is the Kotlin surface's own wire form — built by
/// [`crate::publish::build_kotlin_mirror_json`] and parsed by
/// `MediaMirror.publishJson` on the Kotlin side; this call only
/// attaches the calling thread, invokes the static, and detaches on
/// drop. The reserved swipe-away token stays `null` while the resident
/// surface owns the intent (the Kotlin host reads its own token).
///
/// # Errors
/// Returns an error string when the JVM handle was never captured,
/// when the thread cannot be attached, or when the class lookup or the
/// invocation fails. The caller records the error as a diagnostic:
/// nothing is retried, nothing is buffered.
#[cfg(target_os = "android")]
pub fn publish_mirror_to_kotlin(
    mirror_json: &str,
    delete_intent_json: Option<&str>,
) -> Result<(), String> {
    use jni::objects::{JString, JValue};

    let vm = JVM
        .get()
        .ok_or_else(|| String::from("shell JNI: no entry has captured the JVM handle yet"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|error| format!("shell JNI: attach_current_thread failed: {error}"))?;
    let class = env
        .find_class("app/mdreader/ShellHost")
        .map_err(|error| format!("shell JNI: ShellHost lookup failed: {error}"))?;
    let mirror = env
        .new_string(mirror_json)
        .map_err(|error| format!("shell JNI: mirror string allocation failed: {error}"))?;
    let delete_intent = match delete_intent_json {
        Some(json) => env
            .new_string(json)
            .map_err(|error| format!("shell JNI: delete-intent allocation failed: {error}"))?,
        None => JString::default(),
    };
    let _ = env
        .call_static_method(
            class,
            "publishMirror",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[JValue::Object(&mirror), JValue::Object(&delete_intent)],
        )
        .map_err(|error| format!("shell JNI: ShellHost.publishMirror failed: {error}"))?;
    Ok(())
}
/// No-op on non-Android targets.
///
/// # Errors
///
/// Never fails: the host build has no shell to publish to, so the
/// call always completes as an `Ok` no-op.
#[cfg(not(target_os = "android"))]
pub const fn publish_mirror_to_kotlin(
    _mirror_json: &str,
    _delete_intent_json: Option<&str>,
) -> Result<(), String> {
    Ok(())
}

/// Commands the Kotlin `ShellHost.commandDocumentEnd` static over the
/// captured JVM handle (F1): the drain signal of the document's last
/// speech sample.
///
/// The shell's `Residency.documentEnd()` forwards the classified
/// `document_end` intent back through the drain to
/// [`commands::dispatch_document_end`](crate::commands), so the
/// session teardown stays the one the bridge already serves — this
/// call only commands it and decides nothing about the session.
/// The caller completes in-process on the error: a host or an
/// unarmed shell must never stall a drained session.
///
/// # Errors
/// Returns an error string when the JVM handle was never captured,
/// when the thread cannot be attached, or when the class lookup or
/// the invocation fails. On non-Android targets there is no shell to
/// command, so the error is returned by construction and the
/// in-process completion is the whole answer.
#[cfg(target_os = "android")]
pub fn command_document_end_to_kotlin() -> Result<(), String> {
    let vm = JVM
        .get()
        .ok_or_else(|| String::from("shell JNI: no entry has captured the JVM handle yet"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|error| format!("shell JNI: attach_current_thread failed: {error}"))?;
    let class = env
        .find_class("app/mdreader/ShellHost")
        .map_err(|error| format!("shell JNI: ShellHost lookup failed: {error}"))?;
    let _ = env
        .call_static_method(class, "commandDocumentEnd", "()V", &[])
        .map_err(|error| format!("shell JNI: ShellHost.commandDocumentEnd failed: {error}"))?;
    Ok(())
}

/// No shell to command on non-Android targets: the error routes the
/// caller to its in-process document-end completion.
///
/// # Errors
///
/// Always errors, with the "no Android shell on this host" reason:
/// nothing on this host can command a shell.
#[cfg(not(target_os = "android"))]
pub fn command_document_end_to_kotlin() -> Result<(), String> {
    Err(String::from("shell JNI: no Android shell on this host"))
}

/// Installs the shell channel and spawns its drain thread (T111).
///
/// Creates the code channel, publishes its sender to [`VERB_TX`], and
/// spawns the drain loop holding a clone of `app`, so every drained
/// code executes on [`AppState`]'s own session. Runs once during
/// application setup, before any Kotlin entry can post. A second call
/// keeps the first channel and drops the newcomer with a diagnostic —
/// there is ever exactly one drain.
///
/// A thread-spawn failure is recorded as a diagnostic; the sender
/// stays installed, so later posts drop loudly at the send site
/// instead of vanishing.
pub fn install(app: AppHandle) {
    if VERB_TX.get().is_some() {
        tracing::warn!("shell channel: already installed; second install dropped");
        return;
    }
    // Retain the handle for the document-ingest path (T017):
    // a JNI entry holds no `State` handle, and the AppState
    // must be reached through the same application.
    if APP_HANDLE.set(app.clone()).is_err() {
        tracing::warn!("shell channel: app handle already held; second install dropped");
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel::<u32>();
    if VERB_TX.set(Mutex::new(tx)).is_err() {
        tracing::warn!("shell channel: raced install lost; drain not spawned");
        return;
    }
    match std::thread::Builder::new()
        .name(String::from("shell-drain"))
        .spawn(move || drain_loop(&app, &rx))
    {
        Ok(_) => {}
        Err(error) => {
            tracing::warn!(
                error = error.to_string(),
                "shell channel: drain thread failed to spawn"
            );
        }
    }
}

/// Maps one channel code to its shell-IPC wire string (T111).
///
/// The numeric vocabulary mirrors the `ShellHost.CODE_*` constants on
/// the Kotlin side: codes 1–12 are the closed session verbs in
/// [`parse_action`](crate::commands::parse_action) order, 13–26 the
/// closed intent wires, and 27 the read-only inventory query. Codes
/// outside the table resolve to [`None`]: fail-closed, never a
/// default verb, never a playing state.
#[must_use]
pub const fn code_to_action(code: u32) -> Option<&'static str> {
    match code {
        1 => Some("play"),
        2 => Some("pause"),
        3 => Some("stop"),
        4 => Some("skip_forward5s"),
        5 => Some("skip_back5s"),
        6 => Some("rate_up"),
        7 => Some("rate_down"),
        8 => Some("cycle_voice_next"),
        9 => Some("cycle_voice_prev"),
        10 => Some("resume_at_stored"),
        11 => Some("start_over"),
        12 => Some("choose_engine"),
        13 => Some("duckable_transient"),
        14 => Some("silence_demanding"),
        15 => Some("permanent_loss"),
        16 => Some("regained"),
        17 => Some("becoming_noisy"),
        18 => Some("device_appeared"),
        19 => Some("device_disappeared"),
        20 => Some("app_foregrounded"),
        21 => Some("app_backgrounded"),
        22 => Some("resident_stop"),
        23 => Some("swipe_away"),
        24 => Some("document_end"),
        25 => Some("control_unmapped"),
        26 => Some("lifecycle_unmapped"),
        27 => Some("diagnose"),
        _ => None,
    }
}

/// Posts one channel code toward the drain thread.
///
/// A missing installation or a gone drain drops the code with a
/// diagnostic — never silent, never a crash. A poisoned sender lock
/// is treated the same way: the event is dropped loudly and the
/// channel keeps serving later codes.
fn post_code(code: u32) {
    match VERB_TX.get() {
        None => {
            tracing::warn!(code, "shell channel: not installed; code dropped");
        }
        Some(cell) => match cell.lock() {
            Ok(guard) => {
                if let Err(error) = guard.send(code) {
                    tracing::warn!(
                        code,
                        error = error.to_string(),
                        "shell channel: drain gone; code dropped"
                    );
                }
            }
            Err(_) => {
                tracing::warn!(code, "shell channel: sender lock poisoned; code dropped");
            }
        },
    }
}

/// Native entry of the verb channel: one fieldless session verb as its
/// integer code.
///
/// The first two parameters are the JVM environment and the calling
/// object. The body reads only the by-value `code`, maps out-of-range
/// and negative values to a dropped-with-diagnostic no-op, and hands
/// the environment to [`capture_jvm`] (T133) so the Rust-to-Kotlin
/// publish has a `JavaVM` handle — the capture's own `unsafe` block
/// carries the environment read. The symbol-name export the JNI
/// lookup requires is the remaining unsafe surface, and it is a
/// link-time rename, never a body-level operation. The symbol resolves
/// against the already-loaded application library, which the Kotlin
/// host relies on instead of loading anything itself.
#[expect(
    unsafe_code,
    reason = "T111 JNI entry: the JNI lookup requires the exact exported symbol name, and every spelling of a symbol export (no_mangle/export_name) is an unsafe attribute in edition 2024 — unsatisfiable in code. The body takes a by-value integer and hands the JVM parameter to capture_jvm, whose own unsafe block carries the environment read; the body itself performs no unsafe operation, and the suppression covers the link-time rename only."
)]
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_ShellHost_postShellVerb(
    _env: *mut c_void,
    _obj: *mut c_void,
    code: i32,
) {
    #[cfg(target_os = "android")]
    capture_jvm(_env.cast::<jni::sys::JNIEnv>());
    post_code(u32::try_from(code).unwrap_or_default());
}

/// Native entry of the intent channel: one fieldless platform intent
/// as its integer code.
///
/// Same calling convention as
/// [`Java_app_mdreader_ShellHost_postShellVerb`]: the by-value `code`
/// is the only value read, the JVM environment is handed to
/// [`capture_jvm`] (T133), unknown values drop with a diagnostic, and
/// the suppression below covers the link-time symbol rename only, for
/// the same reason as the verb entry.
#[expect(
    unsafe_code,
    reason = "T111 JNI entry: same unsatisfiable-in-code symbol-export requirement as the verb entry; body reads a by-value integer and hands the JVM parameter to capture_jvm, whose own unsafe block carries the environment read."
)]
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_ShellHost_postShellIntent(
    _env: *mut c_void,
    _obj: *mut c_void,
    code: i32,
) {
    #[cfg(target_os = "android")]
    capture_jvm(_env.cast::<jni::sys::JNIEnv>());
    post_code(u32::try_from(code).unwrap_or_default());
}

/// Drains channel codes onto [`AppState`]'s own session.
///
/// Each received code maps through [`code_to_action`] and executes
/// through [`run_mobile_plugin`](crate::commands::run_mobile_plugin)
/// with no document and no payload — the fieldless contract — so the
/// drain exercises exactly the dispatch the IPC path uses. A rejected
/// dispatch (unknown wire, illegal press, empty-document play) is
/// recorded as a diagnostic; an unknown code likewise. The loop ends
/// only when every sender is gone, which outlives the application, so
/// its exit is itself recorded as a diagnostic.
fn drain_loop(app: &AppHandle, rx: &Receiver<u32>) {
    while let Ok(code) = rx.recv() {
        match code_to_action(code) {
            Some(wire) => {
                let state = app.state::<AppState>();
                match run_mobile_plugin(wire.to_owned(), None, None, app.clone(), state) {
                    Ok(_) => {}
                    Err(reason) => {
                        tracing::warn!(
                            code,
                            reason = reason.as_str(),
                            "shell channel: dispatch rejected"
                        );
                    }
                }
            }
            None => {
                tracing::warn!(code, "shell channel: unknown code; dropped");
            }
        }
    }
    tracing::warn!("shell channel: senders gone; drain exiting");
}

/// Channel-code table tests (T111): the code-to-wire table is the
/// contract the Kotlin `CODE_*` constants mirror, so every code must
/// resolve to a wire the bridge classifier accepts, anchors pin the
/// table ends and the fieldless-fallback device wires, and codes
/// outside the table resolve to nothing.
#[cfg(test)]
mod tests {
    use super::code_to_action;
    use super::{IngestError, IngestSnapshot, open_ingested_document};
    use crate::AppState;
    use crate::commands::parse_action;

    /// Every defined code resolves to a wire the bridge accepts.
    #[test]
    fn channel_codes_resolve_to_known_wires() {
        for code in 1..=27_u32 {
            let wire = code_to_action(code);
            assert!(
                wire.is_some(),
                "channel code {code} must resolve to a wire string"
            );
            if let Some(wire) = wire {
                assert!(
                    parse_action(wire).is_ok(),
                    "wire {wire:?} from code {code} must parse"
                );
            }
        }
    }

    /// Anchors pin the table ends and the fieldless device wires.
    #[test]
    fn channel_code_anchors() {
        assert!(matches!(code_to_action(1), Some("play")));
        assert!(matches!(code_to_action(18), Some("device_appeared")));
        assert!(matches!(code_to_action(19), Some("device_disappeared")));
        assert!(matches!(code_to_action(27), Some("diagnose")));
    }

    /// Codes outside the table resolve to nothing: fail-closed.
    #[test]
    fn unknown_codes_resolve_to_nothing() {
        for code in [0_u32, 28_u32, 100_u32, u32::MAX] {
            assert!(
                code_to_action(code).is_none(),
                "code {code} must resolve to nothing"
            );
        }
    }

    /// A document the tests snapshot: the content, never the
    /// path, is the key (FR-036).
    const DOC: &[u8] =
        b"# Rota da Manha\n\nO onibus sai as seis.\n\n## Volta\n\nEle passa na praca.";

    /// A fresh state whose bookmark store lives in a temp
    /// dir — one redb file per test, dropped with it.
    fn state_with_store() -> Result<(tempfile::TempDir, AppState), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let store = session::store::BookmarkStore::open(&dir.path().join("bookmarks.redb"))
            .map_err(|e| e.to_string())?;
        Ok((dir, AppState::test_state_with_bookmark_store(store)))
    }

    /// The document's content fingerprint (FR-036) — the
    /// key the open lookup resolves the stored row under.
    fn fingerprint() -> [u8; 32] {
        session::fingerprint::fingerprint(DOC)
    }

    /// One snapshot as the Kotlin picker reports it, over a
    /// written file at `path`.
    fn snapshot_at(path: &str) -> IngestSnapshot {
        IngestSnapshot::new(
            path.to_string(),
            hex::encode(fingerprint()),
            "content://picked/document.md".to_string(),
            "text/markdown".to_string(),
        )
    }

    /// Whether the backend open succeeded on this host — the
    /// device-gated twin of the lib.rs open tests' idiom.
    fn backend_available() -> bool {
        player::backend::CpalBackend::new().is_ok()
    }

    /// The T017 wire completion: a snapshot the Kotlin picker
    /// wrote to disk, arrived as its path plus the hex
    /// fingerprint, forwards EXACTLY its bytes to
    /// `open_document_bytes` — the real file round-trip, no
    /// mock. The bytes are the sole synthesis input
    /// (FR-018 / T019), proven by the fingerprint the open
    /// path derived from them.
    #[test]
    fn a_written_snapshot_forwards_its_bytes_to_the_open() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("snapshot.md");
        std::fs::write(&path, DOC).map_err(|e| e.to_string())?;
        let snapshot = snapshot_at(path.to_str().ok_or("the temp path is not UTF-8")?);

        let ingested = open_ingested_document(&state, &snapshot);

        if backend_available() {
            // Device branch: the open started reading, and the
            // fingerprint was derived from the snapshot
            // BYTES, never the URI.
            ingested.map_err(|e| e.to_string())?;
            let transport = state
                .session
                .lock()
                .map_err(|e| e.to_string())?
                .snapshot()
                .transport;
            assert_eq!(
                transport,
                player::transport::TransportState::Playing,
                "a valid open must start reading (device branch)"
            );
        } else {
            // Device-less host: the named backend refusal is
            // the only reachable rejection, and the
            // fingerprint was still derived from the bytes.
            assert!(
                matches!(
                    &ingested,
                    Err(IngestError::Open { reason })
                        if reason.contains("audio backend unavailable")
                ),
                "a device-less host must answer the named open refusal, got {ingested:?}"
            );
        }
        // Either branch proves the round-trip: the state's
        // fingerprint is the file's bytes, exactly.
        assert_eq!(
            crate::current_fingerprint(&state),
            Some(fingerprint()),
            "the open must record the snapshot BYTES' content fingerprint, exactly"
        );
        Ok(())
    }

    /// A missing snapshot file (revoked, moved, or deleted)
    /// is the named [`IngestError::SnapshotUnreadable`]
    /// failure to stderr — never a crash, never a playing
    /// state, nothing loaded (FR-018: re-prompt).
    #[test]
    fn a_missing_snapshot_is_the_named_unreadable_failure() -> Result<(), String> {
        let (_dir, state) = state_with_store()?;
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let missing = dir.path().join("revoked.md");
        let snapshot = snapshot_at(missing.to_str().ok_or("the temp path is not UTF-8")?);

        let ingested = open_ingested_document(&state, &snapshot);

        match &ingested {
            Err(IngestError::SnapshotUnreadable { path, reason }) => {
                assert_eq!(
                    path,
                    missing.to_str().ok_or("the temp path is not UTF-8")?,
                    "the named failure must carry the snapshot path it could not read"
                );
                assert!(
                    !reason.is_empty(),
                    "the named failure must carry the stated reason"
                );
            }
            other => {
                assert!(
                    matches!(other, Err(IngestError::SnapshotUnreadable { .. })),
                    "a missing snapshot must be the named unreadable failure, got {other:?}"
                );
            }
        }
        // Nothing started and nothing loaded: the failure
        // half of the text-I/O protocol leaves no document
        // behind (FR-005 / FR-018).
        assert_eq!(
            crate::current_fingerprint(&state),
            None,
            "a failed ingest must load no document"
        );
        assert!(
            state
                .surface_resolver
                .lock()
                .map_err(|e| e.to_string())?
                .is_none(),
            "a failed ingest must feed no surface resolver"
        );
        Ok(())
    }
}

/// JNI: BLAKE3-256 hash computation (T017).
///
/// Called from Kotlin `DocIngest.Blake3Hasher.nativeBlake3`
/// to compute the document fingerprint. Matches Rust
/// `session::fingerprint::fingerprint` exactly.
///
/// The Kotlin declaration lives on the *nested* object
/// `Blake3Hasher`, whose JVM class name is
/// `app.mdreader.DocIngest$Blake3Hasher`: the JNI name-mangling
/// rule encodes `$` as `_00024`, so the exported symbol carries
/// `DocIngest_00024Blake3Hasher` — a plain underscore resolves
/// nothing and the JVM throws `UnsatisfiedLinkError`, which the
/// Kotlin caller would swallow into a SHA-256 fallback (a
/// *different* fingerprint, not an error).
///
/// # Safety
/// JNI entry point — only the byte array argument is read;
/// the environment is additionally handed to [`capture_jvm`]
/// (T133). The returned byte array is a new JVM primitive
/// array.
#[cfg(target_os = "android")]
#[expect(
    unsafe_code,
    reason = "JNI entry point: the JNI export requires the exact symbol name. Body reads the input byte array, returns a new byte array, and hands the JVM environment to capture_jvm (whose own unsafe block carries the environment read)."
)]
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_DocIngest_00024Blake3Hasher_nativeBlake3(
    env: *mut jni::sys::JNIEnv,
    _this: jni::sys::jobject,
    data: jni::sys::jbyteArray,
) -> jni::sys::jbyteArray {
    use blake3::Hasher;
    use jni::JNIEnv;
    use jni::objects::JByteArray;

    capture_jvm(env);
    let env = unsafe { JNIEnv::from_raw(env).expect("valid JNIEnv") };
    let data_array = unsafe { JByteArray::from_raw(data) };

    // Get the input bytes
    let input_bytes = match env.convert_byte_array(data_array) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("JNI BLAKE3: failed to convert input array: {e}");
            return std::ptr::null_mut();
        }
    };

    // Compute BLAKE3-256
    let mut hasher = Hasher::new();
    let _ = hasher.update(&input_bytes);
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();

    // A BLAKE3-256 digest is 32 bytes: a length `jsize`
    // always holds, so the length is converted, never
    // truncated (the failure arm is unreachable for this
    // hash and answers with the JVM's null array rather
    // than a truncated one if it ever were).
    let hash_len = match i32::try_from(hash_bytes.len()) {
        Ok(len) => len,
        Err(_) => {
            tracing::error!("JNI BLAKE3: digest length exceeds jsize");
            return std::ptr::null_mut();
        }
    };

    // Create output byte array
    let output_array = match env.new_byte_array(hash_len) {
        Ok(arr) => arr,
        Err(e) => {
            tracing::error!("JNI BLAKE3: failed to allocate output array: {e}");
            return std::ptr::null_mut();
        }
    };

    // The region writer takes `jbyte` (i8): the same eight
    // bits per byte, so the digest crosses byte-for-byte,
    // never reinterpreted through a value-changing cast.
    let hash_region: Vec<i8> = hash_bytes.iter().map(|byte| byte.cast_signed()).collect();
    if let Err(e) = env.set_byte_array_region(&output_array, 0, &hash_region) {
        tracing::error!("JNI BLAKE3: failed to set output array: {e}");
        return std::ptr::null_mut();
    }

    // Return the raw pointer to the JVM array
    output_array.into_raw()
}

/// JNI: Document ingest result callback (T017).
///
/// Called from Kotlin `DocIngest.handlePickResult` after
/// successfully picking, reading, snapshotting, and hashing
/// a document. Receives the snapshot file path and the
/// fingerprint hex string, and forwards to Rust for the
/// resume-offer lookup and playback start.
///
/// # Safety
/// JNI entry point — reads only the string arguments, plus the
/// environment handed to [`capture_jvm`] (T133).
#[cfg(target_os = "android")]
#[expect(
    unsafe_code,
    reason = "JNI entry point: the JNI export requires the exact symbol name. Body reads string arguments, forwards to Rust, and hands the JVM environment to capture_jvm (whose own unsafe block carries the environment read)."
)]
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_DocIngest_onDocumentIngested(
    env: *mut jni::sys::JNIEnv,
    _this: jni::sys::jobject,
    snapshot_path: jni::sys::jstring,
    fingerprint_hex: jni::sys::jstring,
    original_uri: jni::sys::jstring,
    mime_type: jni::sys::jstring,
) {
    use jni::JNIEnv;
    use jni::objects::JString;

    capture_jvm(env);
    let mut env = unsafe { JNIEnv::from_raw(env).expect("valid JNIEnv") };

    // Convert JVM strings to Rust strings
    let snapshot_path = unsafe { JString::from_raw(snapshot_path) };
    let fingerprint_hex = unsafe { JString::from_raw(fingerprint_hex) };
    let original_uri = unsafe { JString::from_raw(original_uri) };
    let mime_type = unsafe { JString::from_raw(mime_type) };

    let snapshot_path = match env.get_string(&snapshot_path) {
        Ok(s) => s.into(),
        Err(e) => {
            tracing::error!("JNI onDocumentIngested: failed to get snapshot_path: {e}");
            return;
        }
    };
    let fingerprint_hex = match env.get_string(&fingerprint_hex) {
        Ok(s) => s.into(),
        Err(e) => {
            tracing::error!("JNI onDocumentIngested: failed to get fingerprint_hex: {e}");
            return;
        }
    };
    let original_uri = match env.get_string(&original_uri) {
        Ok(s) => s.into(),
        Err(e) => {
            tracing::error!("JNI onDocumentIngested: failed to get original_uri: {e}");
            return;
        }
    };
    let mime_type = match env.get_string(&mime_type) {
        Ok(s) => s.into(),
        Err(e) => {
            tracing::error!("JNI onDocumentIngested: failed to get mime_type: {e}");
            return;
        }
    };

    let snapshot = IngestSnapshot::new(snapshot_path, fingerprint_hex, original_uri, mime_type);
    // The bytes cross here, never the URI or the path: the
    // snapshot file the Kotlin picker wrote is the platform
    // boundary's one read, and its bytes are the sole
    // synthesis input from here on (FR-018 / T019).
    if let Err(error) = ingest_snapshot(&snapshot) {
        tracing::error!(
            snapshot_path = %snapshot.path,
            fingerprint = %snapshot.fingerprint_hex,
            error = %error,
            "document ingest failed: the picker is re-prompted, nothing started (FR-018)"
        );
    }
}

/// The named failures of the document-ingest path (T017 /
/// T019) — a closed set with no catch-all variant: every
/// rejection names its cause.
///
/// The named cause is the failure half of the text-I/O
/// protocol (the diagnostic on stderr, nothing on stdout,
/// never a playing state — FR-005 / FR-018).
#[derive(Debug)]
pub enum IngestError {
    /// The snapshot file could not be read: a revoked, moved,
    /// or deleted snapshot leaves no synthesis input. The
    /// picker is re-prompted; nothing starts and no playing
    /// state is reported (FR-018: re-prompt, never crash).
    SnapshotUnreadable {
        /// The snapshot path the Kotlin picker reported.
        path: String,
        /// Why the read failed, for the diagnostic only.
        reason: String,
    },
    /// The snapshot's bytes were refused by the open path —
    /// empty bytes, non-text bytes, no bounded segment, a
    /// synthesis failure, an unreadable store, an unavailable
    /// backend, or a press the closed transition table
    /// rejected.
    Open {
        /// The named open refusal.
        reason: String,
    },
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SnapshotUnreadable { path, reason } => {
                write!(f, "snapshot {path} unreadable: {reason}")
            }
            Self::Open { reason } => write!(f, "document open refused: {reason}"),
        }
    }
}

impl std::error::Error for IngestError {}

/// One document-ingest snapshot as the Kotlin picker reports
/// it (T017): the app-private snapshot's path plus the
/// fingerprint hex and the pick metadata the bridge logs.
///
/// The BYTES deliberately do not live here: the JNI wire
/// carries the path, and the snapshot file is read on demand
/// ([`read_snapshot_bytes`]) — the platform boundary's one
/// I/O — so nothing but bytes ever enters synthesis
/// (FR-018 / T019).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestSnapshot {
    /// The absolute path of the app-private snapshot the
    /// picker wrote (`filesDir/snapshots/<hex>.{md,txt}`).
    pub path: String,
    /// The lowercase BLAKE3-256 hex of the snapshot bytes,
    /// computed Kotlin-side through the same Rust hasher
    /// (FR-036).
    pub fingerprint_hex: String,
    /// The picked document's URI, logged for diagnostics
    /// only — it never enters synthesis (FR-018).
    pub original_uri: String,
    /// The picked document's MIME type, logged for
    /// diagnostics only.
    pub mime_type: String,
}

impl IngestSnapshot {
    /// One snapshot as the Kotlin picker reports it (T017).
    #[must_use]
    pub const fn new(
        path: String,
        fingerprint_hex: String,
        original_uri: String,
        mime_type: String,
    ) -> Self {
        Self {
            path,
            fingerprint_hex,
            original_uri,
            mime_type,
        }
    }
}

/// Forwards one ingested snapshot to the cold-start open
/// path (T017 / T019) — the JNI half of the ingest bridge.
///
/// Resolves [`AppState`] through the application handle
/// [`install`](crate::shell_jni::install) retained during setup
/// (a JNI entry holds no `State<'_, AppState>` handle, which
/// cannot be constructed outside a running Tauri app), then
/// delegates to [`open_ingested_document`]. A post before
/// setup, or a setup that never ran, drops the event with a
/// diagnostic — never a crash, never a playing state.
///
/// A read failure is a named failure to stderr, never a
/// crash and never a playing state (FR-018: re-prompt); an
/// open refusal carries the open path's own named reason.
/// The caller re-prompts the same picker on either.
///
/// # Errors
///
/// [`IngestError::SnapshotUnreadable`] when the snapshot file
/// cannot be read, or the named open refusal the open path itself
/// carries — either way the picker is re-prompted.
pub fn ingest_snapshot(snapshot: &IngestSnapshot) -> Result<(), IngestError> {
    let Some(app) = installed_app() else {
        tracing::warn!(
            snapshot_path = %snapshot.path,
            fingerprint = %snapshot.fingerprint_hex,
            original_uri = %snapshot.original_uri,
            mime_type = %snapshot.mime_type,
            "shell channel: document ingest before the app handle is held; dropped with a diagnostic"
        );
        return Ok(());
    };
    open_ingested_document(&app.state::<AppState>(), snapshot)
}

/// Reads one ingested snapshot's BYTES from its file and
/// forwards them to the cold-start open path (T017 / T019).
///
/// The snapshot file the Kotlin picker wrote is the platform
/// boundary's one I/O; from there the BYTES are the sole
/// synthesis input, handed to
/// [`open_document_bytes`](crate::commands::open_document_bytes)
/// — the same core the `open_document` IPC command wraps, so
/// the Kotlin path synthesizes through exactly the code every
/// other driver uses (Principle II). No URI, no path, and no
/// second document object ever crosses into synthesis.
///
/// # Errors
///
/// * [`IngestError::SnapshotUnreadable`] when the snapshot
///   file is absent, revoked, moved, or unreadable — a named
///   failure to stderr, never a crash and never a playing
///   state (FR-018: re-prompt, never crash).
/// * [`IngestError::Open`] when the open path refuses the
///   bytes, carrying its own named reason.
pub(crate) fn open_ingested_document(
    state: &AppState,
    snapshot: &IngestSnapshot,
) -> Result<(), IngestError> {
    let bytes = read_snapshot_bytes(&snapshot.path)?;
    let opened =
        crate::commands::open_document_bytes(state, bytes).map_err(|error| IngestError::Open {
            reason: error.to_string(),
        })?;
    tracing::info!(
        snapshot_path = %snapshot.path,
        fingerprint = %snapshot.fingerprint_hex,
        mime_type = %snapshot.mime_type,
        status = opened.get("status").and_then(serde_json::Value::as_str),
        "document ingest forwarded to the cold-start open path"
    );
    Ok(())
}

/// The installed application handle, set once by [`install`]:
/// the same handle the drain thread holds, retained so the
/// document-ingest entry can reach [`AppState`]'s own session.
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// The installed application handle, held once by
/// [`install`] during setup so the ingest path can reach
/// [`AppState`]'s own session: a `State<'_, AppState>` handle
/// cannot be constructed outside a running Tauri app, so the
/// entry resolves the state through the retained handle
/// instead.
fn installed_app() -> Option<&'static AppHandle> {
    APP_HANDLE.get()
}

/// Reads the snapshot file's bytes (T017 / T019) — the
/// platform boundary's one I/O on the document-ingest path.
///
/// The synthesized document is these bytes and nothing else:
/// no URI is opened here, the persisted grant is never
/// re-read, and the snapshot file is not re-hashed (the
/// Kotlin side already computed the fingerprint through the
/// same Rust hasher, and the open path re-derives it from
/// the bytes under FR-036).
///
/// # Errors
///
/// Returns [`IngestError::SnapshotUnreadable`] when the file
/// is absent, revoked, or unreadable — a named failure to
/// stderr, never a crash and never a playing state
/// (FR-018).
fn read_snapshot_bytes(path: &str) -> Result<Vec<u8>, IngestError> {
    let unreadable = |reason: String| IngestError::SnapshotUnreadable {
        path: path.to_string(),
        reason,
    };
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(unreadable(
            "the snapshot file is gone (revoked, moved, or deleted); the picker is re-prompted"
                .to_string(),
        )),
        Err(error) => Err(unreadable(error.to_string())),
    }
}
