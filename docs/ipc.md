# IPC: the Rust ↔ shell boundary

Grounded in `specs/001-android-media-shell/contracts/shell-ipc.md` (the authority
for this page), the crates under `crates/`, and `src-tauri/`. Every requirement
token (FR-### / SC-###) refers to `specs/001-android-media-shell/spec.md`.

**Direction: every decision flows Rust → shell.** The Kotlin surface and the
Tauri layer carry no playback logic (FR-042, enforced by the SC-035 check):
they own the `Service`, the `MediaSession` (a mirror — it renders the state
Rust publishes, it never computes position or transport), the notification,
and the `AudioFocusRequest`. The Kotlin `PlaybackService` self-bootstraps from
the Application/Activity and reaches Rust through the Tauri bridge
(`register_android_plugin` + `run_mobile_plugin`, `src-tauri/src/lib.rs`);
the Android manifest is edited directly for the media-playback foreground
service (the `mobile::update_manifest` API does not exist in Tauri 2).

## The closed verb set

The contract's closed verb set (extension follows the FR-044 coordinated-edit
plus documentation rule):

`Play | Pause | Stop | SkipForward5s | SkipBack5s | RateUp | RateDown |
CycleVoiceNext | CycleVoicePrev | ResumeAtStored | StartOver | ChooseEngine(id)`

`CycleVoice*` cycles one voice per press in the order supplied by the
voice-selection interface (FR-024, FR-018b); `ChooseVoice(id)` is intentionally
absent — picking from a list needs eyes and violates FR-018a. Rapid repeated
presses are each honoured or explicitly rejected — never a phantom playing state
(FR-005): an illegal press is rejected with
`PlayerError::IllegalTransition { current, verb }` and the session does not move
(the T019 double-press convention — a driver may press a verb twice; the second
press is discarded).

The Rust-side enumeration is `player::cli::ShellVerb` — fifteen members: the
thirteen transport/session verbs plus the two voice-cycle verbs. It maps onto
the contract's twelve as follows: `SkipForward5s`/`SkipBack5s` are
`SkipForward`/`SkipBackward` (the skip displacement is exactly ±5.0 s,
FR-018), and the three members the contract does not list are `Complete`
(document completed), `Interrupt` (e.g. focus loss), and `ExternalStart` (the
externally-started-service trial). The closed transition function is an
exhaustive match over the 5 × 15 product of transport state × verb — five
states (`Playing | Paused | Stopped | Completed | Interrupted`) × fifteen
verbs, no catch-all arm (Principle VII / FR-044 / gate 7): a new state or
verb is a compile error until it has a table entry. `ChooseEngine` is a
selection, not a transition — legal from every state, changing no transport or
position state, so it can never report playing while silent (FR-005).

The same functions serve every driver (Principle II). Verified CLI
invocations (workspace `target/debug/player`; note this binary prints the
Rust `Debug` forms — `"Playing"`, `R1_0` — while the canonical wire form per
the case-mapping table below is snake_case, which the Tauri bridge emits via
`Display`/serde):

```sh
player play              # → {"transport":"Playing","position_secs":0,"rate":R1_0,"engine_id":"rust-player"}
player pause             # → {"transport":"Paused","position_secs":0,"rate":R1_0,"engine_id":"rust-player"}
player rate-up           # → {"transport":"Paused","position_secs":0,"rate":R1_25,"engine_id":"rust-player"}
player skip-back         # → {"transport":"Paused","position_secs":0,"rate":R1_0,"engine_id":"rust-player"} (clamped at the 0.0 floor, FR-018)
player stop              # → {"transport":"Stopped","position_secs":0,"rate":R1_0,"engine_id":"rust-player"} (discards the stored position)
player resume-at-stored  # → {"transport":"Playing","position_secs":0,"rate":R1_0,"engine_id":"rust-player"}
player position          # → {"transport":"Paused","position_secs":0}     (read-only query, not a verb)
player state             # → {"transport":"Paused"}                        (read-only query)
player engine-tier       # → {"engine_tier":"offline_default"}             (read-only query)
```

Each invocation is a fresh session: `PlaybackSession::new()` starts at position
0.0, rate `R1_0`, transport `Paused` (FR-019b), with no duration knowledge
(FR-028c). The read-only queries (`position`, `state`, `engine-tier`) are
handled separately and are not part of the closed-verb exhaustiveness.

## The state mirror (Rust → MediaSession / notification)

On every transport/position/focus event Rust publishes: `transport`,
`position_secs`, `section_title`, `progress (determinate value | indeterminate)`,
`transient_reason?`. The shell renders; it never computes position.
`setOngoing(true)` while playing.

Three mirror payloads exist, each with an exact field contract:

- `player::surface::MirrorState` — the view the `MediaSession` mirror consumes:
  exactly `transport`, `position_secs`, `progress`, all read from the
  session's authoritative snapshot. The mirror is derived, never a second
  clock (SC-035 / FR-003).
- `player::surface::SurfaceSnapshot` — the notification surface (FR-028
  series / FR-029): exactly `title` + `section_title` + `progress`, with
  `section_title` **absent** (not `null`) when no heading has been passed
  yet (FR-028c). Never document body text, elapsed/remaining time, rate, or
  voice. The title is the document metadata title, or the filename as the
  stable placeholder (FR-029).
- `player::cli::SessionSnapshot` — the session's authoritative state and
  nothing else (SC-035): exactly the four snake_case keys `transport`,
  `position_secs`, `rate`, `engine_id`.

The progress indication is fixed at session start and never switches
mid-session (FR-028a): a session whose total spoken length was known at start
is determinate at every position — the determinate total being the marked
document extent, never an estimate — and a session whose total was unknown
stays indeterminate for the whole session. Wire form (externally tagged):
`{"determinate": {"position_secs": …, "duration_secs": …}}` or
`"indeterminate"` (the indeterminate variant carries no duration field).

Dismissal policy (FR-028e): a playing document is non-dismissable — pausing is
the only route to dismissal while a document plays; a transient-reason surface
(a failure reason under FR-037 or a focus-loss pause reason under FR-041)
auto-dismisses after `TRANSIENT_DISMISS_MS` (exactly 3 s); a resident pause
becomes dismissable without being dismissed automatically, so its paused state
and resume control survive until the reader swipes it away, stops the resident
component, or the document ends (FR-032f).

## Focus & routing intents (Kotlin → Rust, classified at the boundary)

The closed nine-member intent set (`player::focus::FocusIntent` — no `Other`,
fieldless; device identity travels alongside the intent in the diagnostic
record, never inside it). Each intent classifies into outcome + transport +
gain (`focus::classify`, nine explicit arms, never a wildcard):

| Intent | Outcome | Transport | Rule |
|---|---|---|---|
| `DuckableTransient` | `duckable_transient` | `playing` | Duck to ~20% (`DUCK_GAIN`, bounds 15–25%) and keep reading (FR-013, ADR-001) |
| `SilenceDemanding` | `silence_demanding` | `paused` | Pause immediately, no ducking — call/alarm (FR-013a) |
| `PermanentLoss` | `permanent_loss` | `paused` | Pause immediately; reason expires after 3 s, never auto-resume (FR-014) |
| `FocusRegained` (wire `regained`) | — (follow-only) | `playing` | Authorizes resume via the rewind engine (FR-015; resume math lives in `player::rewind`) |
| `BecomingNoisy` | `silence_demanding` | `paused` | Headphone removal; the ≤1 s bound is owned by the routing machine (FR-016, T046) |
| `DeviceAppeared(id)` | — (follow-only) | `playing` | Follow at unchanged position, never private→speaker without a pause-first (FR-017a, T046) |
| `DeviceDisappeared(id)` | `permanent_loss` | `paused` | Pause first, position preserved, explicit resume only (FR-016 / FR-017a) |
| `AppForegrounded` | — (latch-only) | — | No playback decision; latches the foreground state consulted at resume-offer expiry (FR-032c) and engine-offer expiry (FR-023a); a foregrounded offer is never converted early |
| `AppBackgrounded` | — (latch-only) | — | No playback decision; latches the same state (FR-032c) |

The two lifecycle intents are reachable from the CLI as `foreground on|off`
(`session --db <path> foreground on|off`), so the SC-020
foreground/background split is drivable in CI without an emulator.

Outcome escalation follows latest-event-wins supersede (FR-014): a permanent
loss superseding any transient clears both the resume flag and the
duck-restore flag (`focus::supersede_flags`, a closed 3 × 3 product); a late
transient after a permanent loss authorizes nothing.

## Residency intents (Kotlin → Rust)

`ResidentStop | SwipeAway | DocumentEnd` — wire strings `resident_stop |
swipe_away | document_end`:

- `resident_stop` ends residency while **preserving** the stored bookmark (the
  session stays resumable) — distinct from the transport `Stop`, which
  discards it (FR-009a against FR-032).
- `swipe_away` pauses first under the FR-028e fallback when audio was still
  playing, preserves the bookmark, and ends residency.
- `document_end` tears residency down at the document's end (FR-032f): a
  completed document has no resumable position, so nothing remains resident
  and no resume is offered (FR-032e, SC-045, SC-049).

An unrecognised platform lifecycle value classifies as the diagnostic
`lifecycle_unmapped`, never absorbed.

## Permission snapshots (T030b)

The Kotlin shell reads the platform permissions and forwards the answer through
the `update_permissions` Tauri command as a `PermissionSnapshot`
(`src-tauri/src/permissions.rs`) — mirror-only, no decision:

```json
{"foreground_service_media_playback":"denied","post_notifications":"denied"}
```

Each field is the closed three-member `PermissionState` — exactly
`granted | denied | not_requested` (wire form: the snake_case state name).
`PermissionSnapshot::default()` is the all-granted host/CI baseline; a real
device pushes the platform answer.

Two pure gates consume the snapshot and nothing else (never the manifest
text — the manifest declaration itself is gated statically by
`scripts/check-manifest.sh`, T013/T081):

- `check_startup` — runs once at boot (the `run` setup closure). Fails
  closed when the foreground-service permission is `denied` **or**
  `not_requested` (both are "absent"): the cold start is refused with
  `PermissionFailure::ForegroundServiceMissing` (reason string
  `foreground_service_missing`), and no playing state may be presented —
  playback begins only after the user opens the application (FR-008/FR-010).
- `check_pre_play` — runs before every Play-class verb (`Play`,
  `ExternalStart`, `ResumeAtStored`, `StartOver` — the verbs that can reach
  `Playing`), so a refused start can never produce a playing state
  (FR-010). Same fail-closed rule as the startup gate.

A denied notification permission is deliberately **not** a playback blocker:
`check_pre_play` passes with it, playback may be started, paused, and
stopped from inside the application (SC-028), and `notification_statement`
returns the in-app statement `lock_screen_controls_unavailable` (FR-021) —
surfaced inside the application, because the denied permission is precisely
what removes the lock-screen surface it would otherwise appear on.

## Reason surfaces (T065 / T078)

One closed vocabulary of pause and failure reasons, rendered in the two
presentations every surface states: the short lock-screen statement
(`ShortReason` through `src-tauri/src/surface.rs::short_reason`) and the full
in-app reason (`FullReason` through `full_reason`). The lock screen, the
in-app status view (T065b), and the failure surface (T078) all render
through these getters — there is no second definition of any rendered string
(Principle II). The two enums are parallel — the same ten variants under the
same `snake_case` wire forms — so one wire value deserializes into either
presentation.

| Variant (wire form) | Origin | Expiry |
|---|---|---|
| `focus_silence_demanding` | Focus-loss pause: a transient loss demanded silence (FR-013a) | `focus_3s` |
| `focus_permanent_loss` | Focus-loss pause: permanent loss (FR-014); where a transient is superseded, the 3 s window restarts at the permanent-loss callback and this is the authoritative reason | `focus_3s` |
| `engine_unreachable` | Failure: engine unreachable at the transport level (FR-023) | `failure_persistent` |
| `engine_init_failed` | Failure: engine failed to initialize (FR-023) | `failure_persistent` |
| `engine_rate_mismatch` | Failure: delivered rate deviates from the declared handoff shape, including a pitch-tolerance violation at 2.0× (FR-035) | `failure_persistent` |
| `engine_channel_mismatch` | Failure: delivered channel count deviates from the declared mono shape (FR-033) | `failure_persistent` |
| `engine_device_refused` | Failure: the audio device refused the playback request (FR-022) | `failure_persistent` |
| `engine_device_lost` | Failure: the audio device was lost during playback (FR-022) | `failure_persistent` |
| `engine_start_refused` | Failure: playback start was refused (FR-013b) | `failure_persistent` |
| `notification_denied` | Denied-permission statement (FR-021): lock-screen controls unavailable; renders identically through both getters | persists while the permission is denied |

The expiry classification (`ReasonExpiry`) is the boundary FR-037a draws: a
focus-loss pause reason expires 3 s after the pause on **both** surfaces —
the application shows it only if the reader opens it within that window, and
only the paused state and its resume control remain afterwards (FR-030,
FR-041) — while a failure reason persists in the application until it is
read (FR-037). The two sets of reasons are disjoint (FR-013a, FR-037a,
FR-041): a focus loss is not a malfunction, and its expiry never applies to
a failure. No focus-loss reason ever promises a resume (FR-014).

Statement language is pt-BR (the language of the offline voices the
constitution's engine registry admits). The focus-loss statement is the
specification's canonical Pause Reason (FR-041: "Leitura pausada — outro app
usando o áudio"); the failure and permission statements follow the
identifier-style stated-reason shape the permission gate already renders. A
rendered reason never quotes document content (FR-037, FR-041): only the
focus-loss Pause Reason names the document, and it names the title and
nothing else from the document.

## The Tauri IPC surface

Three `#[command]`s in `src-tauri/src/lib.rs`:

- `register_android_plugin()` — binds the native Android plugin surface; the
  Kotlin `PlaybackService` self-bootstraps from the Application/Activity
  and reaches Rust through `run_mobile_plugin`.
- `update_permissions(snapshot: PermissionSnapshot) -> Result<(), String>` —
  stores the pushed platform permission answer (T030b; see above).
- `run_mobile_plugin(action: String, document: Option<String>) ->
  Result<serde_json::Value, String>` — executes one playback action over IPC.
  Actions:
  - `play` — the pre-play permission gate runs **first** (T030b, FR-008/
    FR-010): a refusal returns `{"ok": false, "error": "foreground_service_missing"}`
    and executes nothing, so no playing state can ever be reported from a
    refused start (FR-010). Otherwise the document is split
    (`adapter_synthesis::chunk::split_document` — the adapter owns the chunk
    boundary; the engine receives bounded chunks only), synthesized
    (`adapter_synthesis::synth::synthesize`), enqueued as PCM on the `CpalBackend`,
    and `ShellVerb::Play` is executed, returning:
    `{"status":"playing","segments":N,"pcm_samples":N,"duration_secs":N,"position_secs":N,"transport":"playing"}`
    (`duration_secs` = `pcm_samples / 22050`, the declared handoff rate).
  - `pause` / `stop` — act on the live session and backend, returning
    `{"status":"paused","position_secs":N,"transport":"paused"}` /
    `{"status":"stopped","position_secs":N,"transport":"stopped"}`.
  - `diagnose` — read-only report of the packaged voice-asset state (the
    resolved resource dir and whether each required file is present); never
    touches playback.
  - anything else — `Err("unknown action: {action}")`.

IPC transport (contract): `PluginHandle::run_mobile_plugin` for command
calls; `tauri::ipc::Channel` for Rust→frontend streaming. Payloads are JSON
per the `diagnostic-record.schema.json` conventions (snake_case, closed
string enums) — an unknown wire value never deserializes, because there is no
variant for it to land in.

## Case mapping (normative)

Rust (PascalCase) → wire JSON (snake_case) → verb/intent. Wire strings are
what the diagnostic record and IPC payloads carry; Rust names never appear on
the wire (contracts/shell-ipc.md, "Case mapping"):

| Rust enum | Wire string | Verb / intent |
|---|---|---|
| `Playing\|Paused\|Stopped\|Completed\|Interrupted` | `playing\|paused\|stopped\|completed\|interrupted` | `Play\|Pause\|Stop` (+ `Completed`/`Interrupted` terminal, no verb) |
| session-end `Paused\|Stopped\|Completed\|Interrupted` | same four wire strings in `session_end_reason` (never `playing`) | — (FR-044) |
| `DuckableTransient\|SilenceDemanding\|PermanentLoss` + `Regained` event | `duckable_transient\|silence_demanding\|permanent_loss\|regained` | intents `DuckableTransient\|SilenceDemanding\|PermanentLoss\|FocusRegained\|BecomingNoisy\|DeviceAppeared\|DeviceDisappeared` |
| `Unreachable\|InitFailed\|RateMismatch\|ChannelMismatch\|DeviceRefused\|DeviceLost\|StartRefused` | `engine_unreachable\|engine_init_failed\|rate_mismatch\|channel_mismatch\|device_refused\|device_lost\|start_refused` | — (named `error` on `failure` + `refusal`) |
| `PlaybackRate` 7 rungs | `0.75\|1.0\|1.25\|1.5\|1.75\|2.0\|4.0` numbers | `RateUp\|RateDown` (+ `CycleVoiceNext\|CycleVoicePrev` per FR-018b) |
| — | `offline_default\|cloud_default\|cloud_escalation\|test_double\|evaluated_fallback` in `engine_tier` | — (closed tier order, FR-023c) |
