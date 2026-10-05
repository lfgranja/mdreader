# Contract: Shell IPC (Rust backend ↔ platform surface)

**Direction**: all decisions flow Rust → shell. The Kotlin surface and Tauri layer carry
no playback logic (FR-042, enforced by SC-035 check). FR-002/FR-003/FR-011/FR-018/FR-018b/FR-019/
FR-028 series/FR-041.

## Commands (notification / MediaSession → Rust; same code as any other driver, FR-042)

Closed verb set (single enumeration; extension follows the FR-044 coordinated-edit plus documentation rule): `Play | Pause | Stop | SkipForward5s | SkipBack5s | RateUp | RateDown |
CycleVoiceNext | CycleVoicePrev | ResumeAtStored | StartOver | ChooseEngine(id)`.
`CycleVoice*` cycles one voice per press in the order supplied by the voice-selection
interface (FR-024, FR-018b); `ChooseVoice(id)` is intentionally absent — picking from a
list needs eyes and violates FR-018a. Rapid repeated presses are each honoured
or explicitly rejected — never a phantom playing state (edge case, FR-005).

## State mirror (Rust → MediaSession/notification; mirror only, FR-003)

On every transport/position/focus event Rust publishes: `transport`, `position_secs`,
`section_title`, `progress (determinate value | indeterminate)`, `transient_reason?`.
The shell renders; it never computes position. `setOngoing(true)` while playing.

## Focus & routing intents (Kotlin → Rust, classified at the boundary)

`DuckableTransient | SilenceDemanding | PermanentLoss | FocusRegained |
BecomingNoisy | DeviceAppeared(id) | DeviceDisappeared(id) | AppForegrounded |
AppBackgrounded` → Rust applies FR-013/FR-013a/FR-014/FR-015/FR-016/FR-017a and replies with the
new mirror state.

`AppForegrounded`/`AppBackgrounded` carry no playback decision of their own. They latch the
application-lifecycle state consulted at resume-offer expiry (FR-032c) and at engine-offer
expiry (FR-023a); a foregrounded offer is never converted early, so FR-032c's 3-second
visibility MUST is unchanged. Reachable from the CLI as `foreground on|off` (Principle II), so
the SC-020 foreground/background split is drivable in CI without an emulator.

## Failure/refusal statements (Rust → shell)

Short lock-screen string + full in-app reason; document-title-only, no body text
(FR-037/FR-041). Focus-loss reasons expire at 3 s on both surfaces; failure reasons
persist in-app (FR-037a).

## IPC transport

`PluginHandle::run_mobile_plugin` for command calls; `tauri::ipc::Channel` for
Rust→frontend streaming. Payloads are JSON per `diagnostic-record.schema.json`
conventions (snake_case, closed string enums).

## Case mapping (normative)

Rust (PascalCase) → wire JSON (snake_case) → verb/intent (if any). Wire strings are what
the diagnostic record and IPC payloads carry; Rust names never appear on the wire.

| Rust enum | Wire string | Verb / intent |
|---|---|---|
| `Playing\|Paused\|Stopped\|Completed\|Interrupted` | `playing\|paused\|stopped\|completed\|interrupted` | `Play\|Pause\|Stop` (+ `Completed`/`Interrupted` terminal, no verb) |
| session-end `Paused\|Stopped\|Completed\|Interrupted` | same four wire strings in `session_end_reason` (never `playing`) | — (FR-044) |
| `DuckableTransient\|SilenceDemanding\|PermanentLoss` + `Regained` event | `duckable_transient\|silence_demanding\|permanent_loss\|regained` | intents `DuckableTransient\|SilenceDemanding\|PermanentLoss\|FocusRegained\|BecomingNoisy\|DeviceAppeared\|DeviceDisappeared` |
| `Unreachable\|InitFailed\|RateMismatch\|ChannelMismatch\|DeviceRefused\|DeviceLost\|StartRefused` | `engine_unreachable\|engine_init_failed\|rate_mismatch\|channel_mismatch\|device_refused\|device_lost\|start_refused` | — (named `error` on `failure` + `refusal`) |
| `PlaybackRate` 7 rungs | `0.75\|1.0\|1.25\|1.5\|1.75\|2.0\|4.0` numbers | `RateUp\|RateDown` (+ `CycleVoiceNext\|CycleVoicePrev` per FR-018b) |
| `offline_default\|cloud_default\|cloud_escalation\|test_double\|evaluated_fallback` | same five strings in `engine_tier` | — (closed tier order, FR-023c) |
