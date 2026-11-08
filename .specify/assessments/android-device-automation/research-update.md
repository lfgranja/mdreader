# Android Device Automation — Research Update

## Research 2: Audio detection tools

Goal: tools for identifying audio playing on Android (Ressoa/mdreader context) — audio capture, MediaSession state polling, diagnostic JSONL, dumpsys media_session, RMS monitoring.

### 1. adb screenrecord with audio (Android 11+)

- Command: `adb shell screenrecord --audio <path>.mp4` (API 30+/Android 11+). Captures display + system audio in one MP4; `--audio-source` selects `output` (default) or `mic`. Max 3 min per recording.
- Pros: single artifact captures both screen and audio; no root needed; widely supported; good for post-hoc review and evidence.
- Cons: 3-minute cap; encoding overhead; not real-time feedback; audio quality/compression; cannot be queried mid-run; needs pull + offline analysis.
- Source: Android Developers "Android Debug Bridge (adb)" — https://developer.android.com/tools/adb ; adbshell.com screenrecord — https://adbshell.com/commands/adb-shell-screenrecord

### 2. MediaSession state polling

- `adb shell dumpsys media_session` prints active sessions, playback state (STATE_PLAYING/PAUSED/STOPPED), metadata (title/artist), position, and the owning package.
- Programmatic: `MediaController.getPlaybackState()` / `isActive()` via MediaSession token; python-androidtv exposes `CMD_AUDIO_STATE` patterns parsing dumpsys.
- Pros: authoritative playback state; identifies which app owns audio; low overhead; scriptable in CI.
- Cons: only apps that publish a MediaSession (media players); silent apps / raw AudioTrack won't appear; polling latency; parsing dumpsys is fragile across OEM skins.
- Source: Android Developers MediaSession API — https://developer.android.com/reference/android/media/session/MediaSession ; StackOverflow "Is there an ADB command to check a media is playing or not" — https://stackoverflow.com/questions/54241551 ; python-androidtv constants — https://github.com/JeffLIrion/python-androidtv

### 3. dumpsys media_session / media.audio_flinger

- `adb shell dumpsys media_session` — sessions + playback state.
- `adb shell dumpsys media.audio_flinger` — per-track AudioTrack/AudioRecord, mixer state, sample rates, buffer sizes; useful for detecting any audio output including non-MediaSession apps.
- `adb shell dumpsys audio` — audio policy, focus, volume.
- Pros: catches all audio paths (AudioTrack, ExoPlayer internal); diagnostic depth; no instrumentation needed.
- Cons: verbose, OEM-dependent format; not machine-stable; requires parsing; some fields hidden on production builds.
- Source: AOSP "Audio debugging" — https://source.android.com/docs/core/audio/debugging ; Android Developers dumpsys — https://developer.android.com/tools/dumpsys

### 4. Diagnostic JSONL logging

- Emit one JSON object per line from the test harness: timestamp, event (play/pause), package, playbackState, RMS dB, battery level, latency. JSONL is append-only and trivially parseable by jq/pandas.
- Pros: structured, diffable, streamable; integrates with existing observability; easy to assert in tests.
- Cons: requires instrumentation in harness; schema discipline; storage/rotation concerns on device.
- Source: JSONL for Logs — https://jsonl.help/use-cases/log-processing

### 5. RMS monitoring

- Compute RMS over captured PCM (from screenrecord audio or an AudioRecord tap) to detect silence vs. playback and quantify level changes; tools: ffmpeg `volumedetect`, Python `librosa`/`numpy`, audioscan (Rust) for loudness/silence windows.
- Pros: objective signal for "is audio playing" and level; can detect clipping/silence gaps; automatable assertions.
- Cons: threshold tuning per device/codec; background noise; requires decoded audio; offline step.
- Source: audioscan crate — https://crates.io/crates/audioscan ; AOSP audio debugging

### 6. Media Controller Test app (MCT)

- Google's reference app for exercising MediaSession; useful as a known-good client to validate session state reporting.
- Source: https://developer.android.com/media/optimize/mct

### Recommendation for Ressoa

- Primary signal: `dumpsys media_session` polling (authoritative, low cost) + RMS over `screenrecord --audio` capture for ground truth.
- Secondary: `dumpsys audio`/`audio_flinger` when debugging missing sessions.
- Emit diagnostic JSONL (timestamp, state, package, RMS dB, battery) for reproducibility and assertions.
- Acceptance: treat STATE_PLAYING + RMS above threshold as "audio playing"; require both to reduce false positives.

### Sources

1. Android Developers — adb: https://developer.android.com/tools/adb
2. adbshell.com — screenrecord: https://adbshell.com/commands/adb-shell-screenrecord
3. AOSP — Audio debugging: https://source.android.com/docs/core/audio/debugging
4. Android Developers — dumpsys: https://developer.android.com/tools/dumpsys
5. Android Developers — MediaSession: https://developer.android.com/reference/android/media/session/MediaSession
6. StackOverflow — adb media playing check: https://stackoverflow.com/questions/54241551
7. python-androidtv constants: https://github.com/JeffLIrion/python-androidtv
8. JSONL help: https://jsonl.help/use-cases/log-processing
9. audioscan: https://crates.io/crates/audioscan
10. Media Controller Test app: https://developer.android.com/media/optimize/mct
