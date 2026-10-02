# Research: Android Media Shell — PCM Playback Owned by Rust

**Feature**: `specs/001-android-media-shell`
**Date**: 2026-10-01
**Scope**: Research only — no code changes.

---

## 1. PCM Playback: cpal 0.18.x AAudio vs oboe 0.6.x

### Decision: cpal 0.18.2 with AAudio backend (primary path)

| Aspect | cpal 0.18.2 | oboe-rs (katyo/oboe-rs) |
|---|---|---|
| Crate | `cpal 0.18.2` | `oboe 0.6.x` via `oboe-rs` |
| Android backend | AAudio via `ndk 0.9` (`ndk::audio::AudioStream`) | Oboe C++ lib (AAudio + OpenSL ES fallback) |
| Supported target | `aarch64-linux-android` — confirmed | `aarch64` precompiled static lib included |
| Sample formats | `I16`, `F32` — `AudioFormat::PCM_Float` | `Float`, `I16`, `I24`, `I32` |
| Mono f32 output | ✅ Configure channel count=1, format=F32 | ✅ `set_channel_count::<Mono>()`, `set_format::<f32>()` |
| Device format conversion | ❌ cpal does NOT convert — caller must supply data in device-native format | ✅ `setFormatConversionAllowed(true)` + `setSampleRateConversionQuality()` |
| Buffer tuning | Dynamic underrun-based tuning in cpal's AAudio backend | Oboe auto-tunes via `PerformanceMode::LowLatency` |
| Maturity | Part of RustAudio ecosystem, widely used | 76★, 30 forks, last updated Apr 2026 |
| Thread safety | `Arc<Mutex<AudioStream>>`, implements Send+Sync | Oboe callback runs on high-priority thread |
| Error handling | `AudioError` → `cpal::Error` with `ErrorKind` mapping | `Result` enum, `convertResultToText()` |

### Rationale

cpal 0.18.2 already has a production-quality AAudio backend that uses the NDK's `AudioStream` API directly (via `ndk 0.9`). It supports mono f32 output (`SampleFormat::F32` → `ndk::audio::AudioFormat::PCM_Float`). The constitution's audio pipeline contract specifies that the engine delivers mono f32 at a fixed rate, and the Rust playback module converts to the device format — cpal's `supported_output_configs()` exposes the device's native format, so the conversion happens in Rust before the data callback.

oboe-rs offers built-in format/sample-rate conversion, which could simplify the device-format conversion step. However, it adds a C++ dependency (Oboe C++ library) that cpal already wraps through the `ndk` crate, and oboe-rs is less mature (76★ vs cpal's widespread adoption). The conversion logic in Rust is already a defined responsibility (FR-033/FR-034), so cpal's lack of conversion is not a gap — it is a contract boundary.

### Alternatives

- **oboe-rs 0.6.x**: Use if dynamic format conversion in Rust proves complex. Oboe's `setFormatConversionAllowed(true)` handles I16↔Float and rate conversion automatically. Crate: `oboe`, version `0.6.1` (10.8M downloads).
- **cpal with manual conversion**: The default path. Rust playback module reads device config via `device.supported_output_configs()`, converts mono f32 → device format in the callback.

### Key API Names (cpal AAudio)

- `cpal::platform::Host::default_host()` → `HostId::AAudio` on Android
- `Device::supported_output_configs()` → device-native formats
- `build_output_stream_raw(config, SampleFormat::F32, ...)` → `AudioFormat::PCM_Float`
- `AudioStreamBuilder::direction(AudioDirection::Output)`, `.channel_count(1)`, `.format(AudioFormat::PCM_Float)`
- `Stream::start()`, `Stream::pause()`, `Stream::stop()`
- `AudioStreamState::Started`, `Paused`, `Stopping`, `Disconnected`
- `AudioError::Disconnected` → `ErrorKind::DeviceNotAvailable`
- `AudioManager::get_mixer_bursts()` → buffer tuning

---

## 2. Audio Focus: AudioFocusRequest, Ducking, HEADSET_UNPLUG

### Decision: AudioFocusRequest.Builder (API 26+) with distinct transient/permanent handling

| Focus Event | Constant | Action per Spec |
|---|---|---|
| Transient + may duck | `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK` | Duck to ~20% volume, keep playing |
| Transient + no duck | `AUDIOFOCUS_GAIN_TRANSIENT` | Pause |
| Permanent loss | `AUDIOFOCUS_LOSS` | Pause, do not auto-resume |
| Transient loss | `AUDIOFOCUS_LOSS_TRANSIENT` | Pause |
| Transient + can duck | `AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK` | Duck |

### API Names

- `AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)` — API 26+
- `.setAudioAttributes(playbackAttributes)` — `AudioAttributes.Builder().setUsage(USAGE_MEDIA).setContentType(CONTENT_TYPE_SPEECH)`
- `.setAcceptsDelayedFocusGain(true)` — deferred focus
- `AudioManager.requestAudioFocus(AudioFocusRequest)` — API 26+
- `AudioManager.abandonAudioFocusRequest(AudioFocusRequest)` — API 26+

### Ducking Level

Spec: ~20% of normal volume (FR-013). The `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK` event tells the system the app permits ducking; the system applies the duck. The app must adjust its volume to 0.2f when it receives the duck callback and restore to 1.0f on focus regain.

### ACTION_AUDIO_BECOMING_NOISY (headphone unplug)

- Intent action: `AudioManager.ACTION_AUDIO_BECOMING_NOISY` (`"android.media.AUDIO_BECOMING_NOISY"`)
- Register `BroadcastReceiver` for this action
- On receipt: pause playback within 1 second (FR-016)
- This is a privacy measure — audio must not continue through speaker when headphones disconnect

### Output Device Switch (follow vs pause)

- `AudioManager.getDevices()` — API 23+ → `Array<AudioDeviceInfo>`
- `AudioDeviceInfo.getType()` → `TYPE_BUILTIN_SPEAKER`, `TYPE_WIRED_HEADPHONES`, `TYPE_BLUETOOTH_A2DP`, etc.
- Register `OnAudioDevicesChangedListener` — API 23+
- **Follow switch**: private→private (headphones↔Bluetooth), keep position
- **Pause**: private→speaker (built-in), privacy violation risk
- The spec distinguishes device switch from interruption: device switch is NOT an audio-focus event, it is a routing event

---

## 3. Kotlin MediaSession as State Mirror

### Decision: MediaSession mirrors Rust state; Rust is sole authority

| Role | Owner |
|---|---|
| Playback position | Rust (engine) |
| Playback state (playing/paused) | Rust |
| Transport controls (play/pause/skip/rate) | Rust via `PluginHandle::run_mobile_plugin` |
| MediaSession state reflection | Kotlin — calls `setPlaybackState()` only |
| Notification | Kotlin — non-dismissable while playing |
| Lock-screen controls | Kotlin — via `MediaSession.Callback` |

### API Names

- `MediaSession(Context, String)` — create
- `MediaSession.setCallback(MediaSession.Callback)` — transport callbacks
- `MediaSession.Callback.onPlay()`, `onPause()`, `onSkipToNext()`, `onSkipToPrevious()`
- `MediaSession.setPlaybackState(PlaybackState)` — reflect Rust state
- `PlaybackState.Builder.setState(state, position, speed)` — `STATE_PLAYING`, `STATE_PAUSED`
- `PlaybackState.PLAYBACK_POSITION_UNKNOWN` — indeterminate progress
- `MediaSession.setFlags(FLAG_HANDLES_MEDIA_BUTTONS | FLAG_HANDLES_TRANSPORT_CONTROLS)`
- `MediaSession.setActive(true)`
- `NotificationCompat.MediaStyle.setMediaSession(mediaSession)` — lock-screen style
- `setOngoing(true)` — non-dismissable while playing
- `setDeleteIntent(pausePendingIntent)` — pause on dismiss, not stop

### Lock-Screen Transport Controls

- Standard actions: `ACTION_PLAY`, `ACTION_PAUSE`, `ACTION_SKIP_TO_NEXT`, `ACTION_SKIP_TO_PREVIOUS`
- Skip: ±5 seconds (FR-018) — implemented via `setPosition()` adjustment in Rust
- Rate ladder: custom actions `ACTION_FAST_FORWARD`/`ACTION_REWIND` per step, or `setRatingType()` — spec says rate shown in app only, not on lock screen (FR-028)
- Progress: determinate bar once total spoken length known; indeterminate line until then; never switch mid-session (FR-028a)

### Single Playback Authority Rule

- Rust owns playback state and is the sole authority for position, buffering, and transport
- Kotlin `MediaSession` mirrors that state via `setPlaybackState()` — never computes its own position
- Transport callbacks from MediaSession go to Rust via `PluginHandle::run_mobile_plugin`
- Two authorities over playback position = desynchronisation between notification and audio (the failure this rule prevents)

### Non-Dismissable Notification

- `NotificationCompat.Builder.setOngoing(true)` while playing
- On pause: transient notification with reason, dismissable after 3 seconds (FR-041)
- Resident notification persists between sessions in paused state (FR-009a)

---

## Summary of Key Crate Versions and API Names

| Component | Crate/Version | API |
|---|---|---|
| Audio I/O | `cpal 0.18.2` | `HostId::AAudio`, `SampleFormat::F32`, `AudioFormat::PCM_Float` |
| NDK bindings | `ndk 0.9.0` | `ndk::audio::AudioStream`, `AudioStreamBuilder`, `AudioDirection::Output` |
| Oboe (alternative) | `oboe 0.6.1` via `oboe-rs` | `AudioOutputStream`, `AudioStreamBuilder`, `setFormatConversionAllowed` |
| Audio focus | Android framework | `AudioFocusRequest.Builder`, `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK`, `AUDIOFOCUS_LOSS` |
| Device routing | Android framework | `AudioManager.getDevices()`, `OnAudioDevicesChangedListener`, `ACTION_AUDIO_BECOMING_NOISY` |
| Media session | Android framework | `MediaSession`, `MediaSession.Callback`, `PlaybackState`, `NotificationCompat.MediaStyle` |