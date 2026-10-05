# Research: Android Media Shell

**Feature**: `001-android-media-shell` | **Date**: 2026-10-02 | **Status**: all unknowns resolved

No NEEDS CLARIFICATION remains in the spec (checklist confirms zero markers); the unknowns
below are plan-level technical decisions. Each follows Decision / Rationale / Alternatives.

## R1. Tauri 2.12.1 Android plugin bridge

- **Decision**: Rust↔Kotlin over `PluginApi::register_android_plugin` +
  `PluginHandle::run_mobile_plugin`. Kotlin plugin class constructed `(Landroid/app/Activity;)V`.
- **Rationale**: Sanctioned Tauri 2 path; `PluginHandle` is the only Rust-side token for
  calling into Kotlin. `registerService` and `mobile::update_manifest` do not exist in
  Tauri 2 (only iOS `update_info_plist`), so no plugin API can declare the service.
- **Alternatives considered**: raw JNI without the plugin layer (rejected — bypasses
  `PluginManager` lifecycle); `tauri-plugin-background-service` (rejected — generic
  background service, not `mediaPlayback`-typed; manifest edit still required).

## R2. Foreground-service manifest declaration

- **Decision**: Manual `<service android:foregroundServiceType="mediaPlayback">` in the
  generated `AndroidManifest.xml` + `FOREGROUND_SERVICE`,
  `FOREGROUND_SERVICE_MEDIA_PLAYBACK` (+ `POST_NOTIFICATIONS` on API 33+) permissions;
  SC-006 automated check re-run on every Tauri upgrade.
- **Rationale**: Android 10+ treats background audio as user-visible work only under this
  declaration; Tauri regenerates the manifest, so only an automated check (not inspection)
  keeps the declaration from regressing (FR-007).
- **Alternatives considered**: declaring the `<service>` in a plugin manifest (rejected —
  the typed service belongs in the app manifest via merger/script, permissions may ride
  the plugin manifest).

## R3. Resident service + Android 14 start restrictions

- **Decision**: The playback `Service` itself stays resident paused between sessions
  (FR-009a: exactly one resident component, one notification); cold-start refusal on
  API 34+ (`ForegroundServiceStartNotAllowedException`) surfaces a stated reason with no
  playing state, and audio begins only after an app open (FR-010).
- **Rationale**: API 34 refuses `startForegroundService()` from the background; a live
  resident process is the only way a lock-screen Play press works with the app closed.
  Teardown at document end (FR-032f) bounds the battery cost.
- **Alternatives considered**: `WorkManager`/`AlarmManager`/`BOOT_COMPLETED` restart
  (all rejected — deferrable-work schedulers or alarm-restricted paths that cannot take
  a foreground service from the background on API 34+).

## R4. Rust PCM playback: cpal over oboe

- **Decision**: `cpal` 0.18.2, AAudio backend, mono f32 stream at the declared rate;
  `supported_output_configs()` exposes the device format; conversion to device format in
  the Rust playback module; resampling is the engine's job, never the module's
  (FR-033/FR-034/FR-043).
- **Rationale**: Production AAudio backend on `aarch64-linux-android`; matches the handoff
  contract exactly (engine resamples, module converts format). Mismatch (rate, channels,
  width) surfaces as a named engine failure (FR-035 + stereo edge case).
- **Alternatives considered**: `oboe` 0.6.x (rejected for now — built-in conversion is
  convenient but adds a C++ dependency and a less-mature Rust binding; revisit only if
  Rust-side conversion proves complex).

## R5. Focus classification + routing

- **Decision**: `AudioFocusRequest.Builder(AUDIOFOCUS_GAIN)` (API 26+); classify by the
  interrupter's request — `…_MAY_DUCK` → duck to ~20% and keep reading (FR-013),
  demand-silence (call/alarm) or permanent loss → pause, never auto-resume (FR-013a/FR-014);
  `ACTION_AUDIO_BECOMING_NOISY` → pause (FR-016); new-device-appears → follow switch at
  unchanged position + record output-device event (FR-017a), never private→speaker.
- **Rationale**: Classification by request (not by our reading) is what keeps audio out
  from under phone calls while leaving navigation prompts ducked; direction-of-travel
  (appearing vs disappearing device) separates deliberate acts from privacy events.
- **Alternatives considered**: single duck-everything policy (rejected — superseded by
  FR-013a: 20%-under-a-call is unintelligible and socially wrong); pause-everything
  (rejected — chops the document on every navigation prompt).

## R6. MediaSession as mirror + notification contract

- **Decision**: Rust is sole authority; `MediaSession.setPlaybackState()` reflects engine
  state, never computes position (FR-002/FR-003). `MediaStyle` notification shows title +
  current section title + progress only; `setOngoing(true)` while playing (FR-028e);
  transient reason surfaces expire in 3 s; resident paused surface becomes dismissable.
  Progress determinate only when the total is known at session start, else indeterminate
  for the whole session (FR-028a). Audible confirmation (FR-018a/SC-008): short earcon on
  every lock-screen press plus spoken state on resume/rate-change, emitted through the same
  Rust path the mirror reports; blind trials pass on earcon-or-speech, never on visual.
- **Rationale**: Two position authorities desynchronise notification and audio invisibly
  in tests and audibly in the car — the named defect FR-003 exists to prevent. Determinacy
  fixed at session start keeps a guess from ever presenting as fact.
- **Alternatives considered**: Media3-owned playback (rejected — constitution v1.2.0
  moved authority to Rust deliberately); teleprompter body text on lock screen
  (rejected — FR-028, driving hazard).

## R7. Position store: redb + blake3

- **Decision**: `redb` — `bookmarks` table (32-byte `blake3` content hash → offset,
  paused-at, sentence index); `settings` table (app-wide last rate, default 1.0x).
  Pause/close/kill/power-loss/network-loss preserve; Stop and document-end delete
  (FR-032/FR-032a/FR-032b); edited file = different hash = no resume (FR-036a).
- **Rationale**: ACID + crash-safe + pure Rust suits "survive process death" (FR-004);
  content hash survives move/rename and invalidates on edit by construction.
- **Alternatives considered**: `sled` (larger binary, less durability control); `rusqlite`
  (overkill for two KV tables, heavier on Android).

## R8. Rewind, snap, re-anchor, pre-roll

- **Decision**: Transient resume = cursor back `min(5% × paused-duration, 30 s)`, audio
  from that sentence's start, total backward movement hard-capped at 30 s (FR-015).
  Engine/voice change re-anchors to nearest sentence/node boundary, else document start
  (FR-038). Pre-roll ≤60 s ring, empty while idle, refilled on Play press; idle pause
  synthesises/bills nothing (FR-015a/FR-015c/FR-015b pipeline property).
- **Rationale**: Proportional term scales context with interruption length; sentence snap
  keeps resumes intelligible; the cap bounds both long pauses and long sentences; idle
  drain removes unbilled synthesis cost.
- **Alternatives considered**: fixed-second rewind (rejected — over-replays short ducks,
  under-replays long interruptions); continuous pre-roll while paused (rejected — bills
  per-character engines for unheard audio).

## R9. Rate ladder + pitch-preserving stretch

- **Decision**: Closed const ladder `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`, one step per
  press, saturate at ends (FR-019); stretch is an engine-contract operation the Rust
  module invokes, never platform/buffer stretching (FR-043); delivered rate invariant
  across rungs (SC-034); rate never shown on lock screen (FR-028).
- **Rationale**: Closed ladder is glance-operable without looking; engine-side stretch
  keeps the pitch guarantee (FR-019a) testable per engine and replaceable with the engine.
- **Alternatives considered**: free/continuous rate (out of scope — unglanceable);
  platform-layer stretching (rejected — untestable without an engine, wrong side of the
  boundary). Contingency: if a real engine cannot stretch, evaluate `soundtouch`
  (LGPL-2.1 → Exceptions Log entry) in the tasks phase; `rubato` is resampling only and
  `rubberband` bindings are too immature.

## R10. Diagnostic record

- **Decision**: Append-only JSONL in app-private storage, schema in
  `contracts/diagnostic-record.schema.json`; dual bound (30 days + 10 MB ≈ 100k entries),
  oldest-first trim emitted as a trim event; best-effort writes (storage-full drops a gap
  marker, never interrupts playback); readable via `adb`/tooling; no in-app surface
  (FR-039a deferral); never leaves the device (FR-040).
- **Rationale**: JSONL is directly tooling-readable with exact control over rotation and
  trim-visibility; private storage needs no permission and is unreadable by other apps
  (FR-039b); the record is the only evidence for the zero-occurrence criteria.
- **Alternatives considered**: `sled`/SQLite store (overkill, less directly readable);
  `log`-crate file logging (unstructured, no rotation/trim semantics); device-wide log
  (rejected — wider surface than entitled, readable by other apps).

## R11. Engine-failure recovery mechanism (SC-051 vs SC-044)

- **Decision**: Closed declared fallback order (remaining same-tier → higher tiers in
  registry order → Edge last, never default; consent-bounded — unconsented engines never
  selected, FR-023c); 3 s bounded offer naming failure + alternatives, then automatic
  fallback from last spoken position; no-alternative expires to paused-with-reason.
  Verified here against two stand-in engines honouring the handoff contract (SC-051);
  SC-044 re-verified on one-offline + one-cloud real hardware when synthesis lands.
- **Rationale**: Two engines is the smallest configuration where takeover is observable;
  stand-ins make FR-023a testable now instead of schedule-blocked on FR-024; closed order
  keeps unattended switches auditable and consent-bounded.
- **Alternatives considered**: single-engine reporting only (rejected — reduces recovery
  to failure display); three-engine provisioning (rejected — forced-second-failure gives
  the same evidence cheaper).
