# Contract: Diagnostic Full Trace (bounded, private, prose-free)

FR-017. Extends `specs/001-android-media-shell/contracts/diagnostic-record.schema.json` (same `RecordEvent` shape, same 30-day + 10 MB oldest-first trim with `Trim` markers, same `Gap` on storage-full, same `scan.rs` 7 closed violations). Naming: Rust variants are PascalCase (`SessionStart`); wire strings are snake_case (`session_start`) per `serde(rename_all` in `crates/diag/src/writer.rs`. This doc uses Rust-variant names; the schema lists wire strings. No in-app surface; retrieval = maintainer-only adb pull from app-private storage; never transmitted.

## Mandatory coverage (every session must be able to emit each kind)

- `SessionStart` / `SessionEnd` (with `transport` + `session_end_reason` from the closed 4).
- `TransportChange` per play/pause/skip/stop/rate-step action.
- Per-section progress entries (section boundary crossed, with content-hash `detail` ref).
- Per-rate-change entries (old → new rung).
- Buffer-state entries (pre-roll remaining/depth; `MAX_PREROLL_SECS=60` (seconds; historical alias `MAX_PREROLL_MS=60000` — do not use ms name)).
- `FocusTransition` / `OutputDevice` per interruption with **measured** values in `detail`: play-to-audio latency ms, device-removal pause latency ms, actual replay seconds.
- `Failure` / `Refusal` with `error` name from the closed 8 (`FocusDenied`, `FocusDelayed`, `PermanentLoss`, `TransientPaused`, `DeviceRemoved`, `VoiceUnavailable`, `FormatUndeliverable`, `DecodeFailed`); `error` required on failure *and* refusal.

## Content rules

- Timestamps + content-hash refs only; **zero document prose** in any entry (`scan.rs` `ProseInDetail` predicate enforces).
- `detail` ≤200 chars identifier-pattern (`key=value`) or typed fields only.
- Trim: oldest-first; `Trim` markers flushed before retained entries, never silent.

## Schema delta (new required kinds vs 001)

`TransportChange`, per-section progress, per-rate, buffer-state, and measured-latency `detail` keys (`play_to_audio_ms`, `removal_pause_ms`, `replay_secs`) become required-to-support; storage/retention/scan semantics unchanged. See `diagnostic-record.schema.json` (copied from 001, unchanged: nine kinds; 004 per-section/per-rate/buffer/latency coverage rides as data inside those kinds via `detail` keys, not as new kinds — cf. plan.md Structure diag REUSE).
