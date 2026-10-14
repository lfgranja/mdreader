# Contract: Earcon + Haptic Confirmation (no-look channel)

FR-016. Every driving-time control confirms without looking; fully operable with sounds off (state-change + haptic path).

## Grammar (procedural PCM, mono f32 @ 22050 Hz, `crates/player/src/earcon.rs`)

| Action | Shape | Length |
|---|---|---|
| Play | Adagio caricature C5→E5, sine, 120 ms/note | ≤500 ms |
| Pause | Andante caricature E5→C5, sine, 120 ms/note | ≤500 ms |
| Skip fwd/back | Allegro tick 880 Hz base +40 ms directional contour (fwd: rising 880→1108 Hz; back: falling 880→660 Hz) | ≤500 ms |
| Warning (pause-demanding interruption) | Ostinato abstract double-tick A5, 60+80 ms, dissonant | ≤500 ms |
| Failure (undeliverable voice) | E4 ~330 Hz square + minor-2nd, 200 ms + repeat | ≤500 ms |
| Rate step | Unison 880 Hz / 60 ms | ≤500 ms |

All in 300–2500 Hz; play/pause/skip differ in timbre + rhythm + contour from the warning/failure pair. No audio assets (no `.wav`, no `SoundPool`, no `ToneGenerator`).

## Routing

- Same stream/same clock as speech via `CpalBackend`; earcon PCM takes a **duck-exempt short-mix bypass** (skips `SampleQueue::gain` scaling) so a nav-duck never quiets the confirmation.
- Files no `AudioFocusRequest`; never pre-empts navigation; ≤500 ms at full gain even while ducked; dropped/re-fired post-`GAIN` if a pause stands.
- Must not advance the position bookmark (clock counts speech samples only).
- Trigger + lock-screen mirror publish is one Rust call (impossible to sound without showing).

## Haptic + silent path (Kotlin `MediaMirror.publish()`)

- `Vibrator`/`VibrationEffect.createOneShot` (API 26; `VibratorManager` API 31), `VIBRATE` permission; distinct pattern per action (e.g. single 30 ms play vs double 20+40 ms warning vs 80 ms failure). Acceptance mapping: single 30ms=play vs double 20+40ms=warning vs 80ms=failure (illustrative; exact patterns fixed at implementation, pairwise-distinguishable by duration/count, verified 10/10 blind per SC-004 sounds-on and -off).
- Rides outside `STREAM_MUSIC` → works muted/DND; fully-disabled-haptic still covered by the giant transport glyph + progress + banner state change.
- Blind acceptance: 10/10 correct per confirmation type (play, pause, skip, rate-step, warning, failure) without seeing the screen (SC-004).

## CLI surface (Principle II)

`player` binary: `earcon <play|pause|skip-fwd|skip-back|warning|failure|rate> [--format json]` emits the PCM descriptor + duration assertion; Tauri IPC calls the same constructor.

## Named contract constants

| Constant | Value | Meaning |
|---|---|---|
| `DUCK_GAIN` | `0.20 ± 0.02` | Duck gain factor for `AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK` (FR-004); applied to speech stream, never to earcon short-mix |
| `PITCH_TOLERANCE_CENTS` | `50` | Maximum permitted pitch deviation in cents at any rate rung (SC-007); engine-contract stretch, tested blind |
