# Contract: Diagnostic Full Trace (bounded, private, prose-free)

FR-017. Extends `specs/001-android-media-shell/contracts/diagnostic-record.schema.json` (same `RecordEvent` shape, same 30-day + 10 MB oldest-first trim with `Trim` markers, same `Gap` on storage-full, same `scan.rs` 7 closed violations). Naming: Rust variants are PascalCase (`SessionStart`); wire strings are snake_case (`session_start`) per `serde(rename_all` in `crates/diag/src/writer.rs`. This doc uses Rust-variant names; the schema lists wire strings. No in-app surface; retrieval = maintainer-only adb pull from app-private storage; never transmitted.

## Mandatory coverage (every session must be able to emit each kind)

- `SessionStart` / `SessionEnd` (with `transport` + `session_end_reason` from the closed 4).
- `TransportChange` per play/pause/skip/stop/rate-step action.
- Per-section progress entries (section boundary crossed, with content-hash `detail` ref).
- Per-rate-change entries (old → new rung).
- Buffer-state entries (pre-roll remaining/depth; `MAX_PREROLL_SECS=60` (seconds; historical alias `MAX_PREROLL_MS=60000` — do not use ms name)).
- `FocusTransition` / `OutputDevice` per interruption with **measured** values in `detail`: play-to-audio latency ms, device-removal pause latency ms, actual replay seconds.
- `Failure` / `Refusal` with `error` naming a member of the closed **seven**-value engine wire taxonomy — the schema's `error.enum` (`engine_unreachable`, `engine_init_failed`, `rate_mismatch`, `channel_mismatch`, `device_refused`, `device_lost`, `start_refused`); `error` required on failure *and* refusal. The eight product reasons are the SURFACE vocabulary and map N:1 onto these seven — see "The `error` set" below.

## Content rules

- Timestamps + content-hash refs only; **zero document prose** in any entry (`scan.rs` `ProseInDetail` predicate enforces).
- `detail` ≤200 chars identifier-pattern (`key=value`) or typed fields only.
- Trim: oldest-first; `Trim` markers flushed before retained entries, never silent. Design estimate: ~100k entries max within the 30-day/10 MB bounds (informative, not a hard limit).

## Schema delta (new required kinds vs 001)

`TransportChange`, per-section progress, per-rate, buffer-state, and measured-latency `detail` keys (`play_to_audio_ms`, `removal_pause_ms`, `replay_secs`) become required-to-support; storage/retention/scan semantics unchanged. See `diagnostic-record.schema.json` (copied from 001, unchanged: nine kinds; 004 per-section/per-rate/buffer/latency coverage rides as data inside those kinds via `detail` keys, not as new kinds — cf. plan.md Structure diag REUSE).

## The `error` set: the eight product reasons over the seven wire values

Two closed sets meet in the record's `error` field, and they are **not** the same set. Each is named here so neither is confused for the other:

- the **eight product reasons** — `FocusDenied`, `FocusDelayed`, `PermanentLoss`, `TransientPaused`, `DeviceRemoved`, `VoiceUnavailable`, `FormatUndeliverable`, `DecodeFailed` — are the **surface** vocabulary: `player::interruption::InterruptionReason`, the reason the lock screen states in pt-BR (FR-005, `contracts/surface.md`). They are what a reader sees.
- the **seven wire values** — `engine_unreachable`, `engine_init_failed`, `rate_mismatch`, `channel_mismatch`, `device_refused`, `device_lost`, `start_refused` — are the **engine** taxonomy: `engine_core::closed::EngineError` (seven members), re-exported verbatim as `crates/diag`'s `ErrorValue` (`crates/diag/src/writer.rs`) and written to the record as `error`. This is the schema's `error.enum`, unchanged from 001.

The eight map **N:1** onto the seven (research.md R5): each product reason rides one wire value, and where two product reasons ride the same wire value the `detail` key `cause=<identifier>` names which one it was, so no two distinct product reasons are indistinguishable on the wire. `DecodeFailed` is the case research.md R5 names explicitly: it rides `engine_unreachable` on the wire, and the distinguishing `decode_failed` rides in `detail` only.

| product reason (surface vocabulary) | wire `error` (engine taxonomy) | distinguishing `detail` `cause` |
| --- | --- | --- |
| `FocusDenied` | `start_refused` | `permission_refused` |
| `FocusDelayed` | — (the zero-audio hold fails nothing) | — |
| `PermanentLoss` | — (the focus loss rides the `focus` field of a `focus_transition` entry) | — |
| `TransientPaused` | — (the focus loss rides the `focus` field of a `focus_transition` entry) | — |
| `DeviceRemoved` | `device_lost` | `device_removed` |
| `VoiceUnavailable` (the stored row is unreadable) | `engine_init_failed` | `store` |
| `VoiceUnavailable` (the engine could not produce the audio) | `engine_unreachable` | `synthesis` |
| `FormatUndeliverable` (the delivered shape cannot be read) | `channel_mismatch` | `empty_document` / `invalid_utf8` / `no_segments` |
| `FormatUndeliverable` (the delivered rate/width deviates) | `rate_mismatch` | `rate_deviation` / `width_deviation` / `pitch_tolerance_violation` |
| `DecodeFailed` | `engine_unreachable` | `decode_failed` |

The engine taxonomy has members the eight product reasons do not name: the audio device's refusal of the playback request (`device_refused`, `cause=backend`) and the refused playback start (`start_refused`, `cause=no_start` — the same wire value `FocusDenied` rides, told apart by the `cause`). The `cause` identifiers are the closed identifier set of `src-tauri/src/lib.rs`'s `diag_error_cause` (T097) and `adapter-synthesis`'s `FailureCondition::detail()`; they are identifiers, never prose and never document text.

The `error` field is the engine-failure taxonomy and it is set on `failure` and `refusal` entries only — the schema requires it there and nowhere else. A focus-side interruption is recorded by the interruption entry it is: a `focus_transition` entry carries the four-value `focus` taxonomy (`duckable_transient`, `silence_demanding`, `permanent_loss`, `regained`) and an `output_device` entry carries the `device` field. Neither carries an `error`.

The mapping is declared once, not twice: the seven values live in `engine_core::closed::EngineError` (FR-044 — a second list would drift), and the record's projection of them is implemented once, in `src-tauri/src/lib.rs`'s `diag_error_of` (T097). This contract restates that mapping; it declares no third copy of either closed set.
