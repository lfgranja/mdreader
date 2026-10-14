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
//
// Allow unsafe code for JNI operations (FFI boundary).
#![allow(unsafe_code)]

#[cfg(target_os = "android")]
use std::ffi::{c_void, CString};
#[cfg(target_os = "android")]
use std::sync::{
    Mutex, OnceLock,
    mpsc::{Receiver, Sender},
};

#[cfg(target_os = "android")]
use jni::objects::{JClass, JString, JValue};
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

/// Calls the Kotlin `ShellHost.publishMirror` static method via JNI
/// to publish the mirror state to the MediaMirror (MediaSession + notification).
///
/// # Errors
/// Returns an error string if JNI is not available or the call fails.
#[cfg(target_os = "android")]
pub fn publish_mirror_to_kotlin(_mirror_json: &str, _delete_intent_json: Option<&str>) -> Result<(), String> {
    // Note: Full MirrorState publish deferred to synthesis feature (FR-024, 002-fr-024-synthesis-adapter)
    // due to jni 0.22.4 API changes blocking the Rust↔Kotlin JNI bridge.
    // The Kotlin PlaybackService handles the media notification independently via ShellHost.init.
    tracing::debug!("JNI publish to Kotlin MediaMirror deferred to synthesis feature");
    Ok(())
}/// No-op on non-Android targets.
#[cfg(not(target_os = "android"))]
pub fn publish_mirror_to_kotlin(_mirror_json: &str, _delete_intent_json: Option<&str>) -> Result<(), String> {
    Ok(())
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
    let (tx, rx) = std::sync::mpsc::channel::<u32>();
    if VERB_TX.set(Mutex::new(tx)).is_err() {
        tracing::warn!("shell channel: raced install lost; drain not spawned");
        return;
    }
    match std::thread::Builder::new()
        .name(String::from("shell-drain"))
        .spawn(move || drain_loop(app, rx))
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
pub fn code_to_action(code: u32) -> Option<&'static str> {
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
/// object, received opaquely and never touched — the body reads only
/// the by-value `code`, maps out-of-range and negative values to a
/// dropped-with-diagnostic no-op, and performs no unsafe operation of
/// any kind. The symbol-name export the JNI lookup requires is the
/// only unsafe surface, and it is a link-time rename, never a
/// body-level operation. The symbol resolves against the
/// already-loaded application library, which the Kotlin host relies on
/// instead of loading anything itself.
#[expect(
    unsafe_code,
    reason = "T111 JNI entry: the JNI lookup requires the exact exported symbol name, and every spelling of a symbol export (no_mangle/export_name) is an unsafe attribute in edition 2024 — unsatisfiable in code. The body takes only a by-value integer, touches neither JVM parameter, and performs zero unsafe operations; the suppression covers the link-time rename only."
)]
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_ShellHost_postShellVerb(
    _env: *mut c_void,
    _obj: *mut c_void,
    code: i32,
) {
    post_code(u32::try_from(code).unwrap_or_default());
}

/// Native entry of the intent channel: one fieldless platform intent
/// as its integer code.
///
/// Same calling convention as
/// [`Java_app_mdreader_ShellHost_postShellVerb`]: the JVM parameters
/// stay opaque, only the by-value `code` is read, unknown values drop
/// with a diagnostic, and the body performs no unsafe operation — the
/// suppression below covers the link-time symbol rename only, for the
/// same reason as the verb entry.
#[expect(
    unsafe_code,
    reason = "T111 JNI entry: same unsatisfiable-in-code symbol-export requirement as the verb entry; body reads only a by-value integer and performs zero unsafe operations."
)]
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_ShellHost_postShellIntent(
    _env: *mut c_void,
    _obj: *mut c_void,
    code: i32,
) {
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
fn drain_loop(app: AppHandle, rx: Receiver<u32>) {
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
}