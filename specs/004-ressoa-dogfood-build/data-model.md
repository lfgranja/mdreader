# Data Model: Ressoa Dogfood Build

**Feature**: `004-ressoa-dogfood-build` | **Source**: spec Key Entities + FRs | **Extends**: `specs/001-android-media-shell/data-model.md` (shell), `specs/002-fr-024-synthesis-adapter/data-model.md` (synthesis)

All enums are closed sets (Principle VII): no `Other`/`Unknown`, no `_ =>`. Adding a variant is a breaking change (semver bump + docs). Reused 001 shapes are referenced, not redefined; only 004 deltas are fully specified here.

## Document (.md / .txt only, FR-018)

A Markdown or plain-text document the maintainer listens to. Other formats out of scope.

| Field | Type | Rule |
|---|---|---|
| `bytes` | `Vec<u8>` read once via `ContentResolver` | source of truth for identity + parse |
| `fingerprint` | `Fingerprint` blake3-256 of `bytes` | position identity; path/URI-independent (FR-008) |
| `uri` | persisted SAF URI grant (best-effort) | freshness check only, never synthesis input |
| `snapshot_path` | `filesDir/snapshots/<hex>.{md,txt}` | transient session-scoped copy; synthesis reads only this file |
| `sections` | `Vec<Section>` from Markdown headings (`doc-structure`); TXT may be empty | orientation only |

Reopen flow: read bytes → `fingerprint()` → `store.get(hash)` → match = resume offer path, mismatch = start `0.0` no offer. `SecurityException`/`FileNotFoundException` on re-read → re-prompt picker → hash-match restore (SC-005).

## Section

| Field | Type | Rule |
|---|---|---|
| `index_k` | `u32` 1-based | label `Section k of N` |
| `total_n` | `u32` known at session start | determinacy input; unknown → whole session indeterminate |
| `title` | heading text | lock-screen section line; absent (zero sections / before first heading) → show no title, never a wrong one |
| `start_unit` | parsed-content offset | skip target; progress numerator input |

Skip (FR-002): next/previous section boundary; fallback ±10 s when zero sections.

## Reading Position (FR-007 + FR-008)

Listener's place in content: section + sentence-anchored offset + monotonic clock. Persisted cursor = **last-spoken** (never buffered-ahead) in a single atomic redb tx.

| Field | Type | Rule |
|---|---|---|
| `offset_units` | parsed-content offset | progress numerator; resume anchor |
| `offset_secs` | `f64 ≥ 0` audible position | compat with 001 `PositionClock`; pre-roll never advances it |
| `sentence_start` | snapped boundary at/before cursor | snap-back target; mid-word cap-point forward-snaps to next word-gap |
| `first_yield_ms` | `u64` unix-ms cumulative | 5%-of-pause (REWIND_FRACTION=0.05 capped 30 s) accounting basis only, never reset at supersede; the 004 cue distance is 5–10 s further-backward from containing-sentence-start per FR-007 |

Cadence (Option C): in-RAM tick ~1 s; non-durable commit per sentence-end/5 s (snap-back source only); **durable every 20 s fixed** (within the FR-007 15–30 s envelope) while Playing + on every pause, focus-loss, device-removal, section-change, Stop (then `delete`), app-background, `DocumentEnd`. Worst-case loss = one interval (20 s); caps: 30 s interruption replay, 60 s kill/power-loss replay. Resume rendering: containing-sentence-start snap, then further backward 5–10 s cue, never mid-word, total backward ≤ cap; degrade to raw last-spoken cursor, never doc start.

## Playback Session (extends 001 `PlaybackSession`)

Transport STATE (single authority): Playing | Paused | Stopped. Stop is an in-app CONTROL only, never a lock-screen control. EndKind (separate closed enum): Paused | Stopped | Completed | Interrupted (Completed=document end; Interrupted=kill/power-loss/permanent loss/engine failure). Stopped and Completed delete the stored position. Deltas for 004:

- Ingest binds `snapshot_path` + `fingerprint` at session start; totals (`total_n`, `total_units`) fixed at session start for progress determinacy.
- `end_kind`: `Paused | Stopped | Completed | Interrupted` — `Stopped` (in-app Stop only) and `Completed` delete the stored position; `Paused` → 3 s visual-dialog offer (silence = continue from stored); `Interrupted` (kill/power-loss/permanent loss/engine failure) → wait paused + explicit lock-screen resume control, never auto-start.
- Stop control exists **only** on the single in-app screen; lock-screen has no Stop (FR-002/FR-019).

## Interruption (closed 8, FR-005 + FR-004 + FR-006)

Surface/diag-level reason enum mapping N:1 onto the untouched `FocusOutcome`/`FocusGrant`/`RoutingDecision`/`EngineError` taxonomies:

`FocusDenied | FocusDelayed | PermanentLoss | TransientPaused | DeviceRemoved | VoiceUnavailable | FormatUndeliverable | DecodeFailed`

| Variant | Outcome | Reason lifetime |
|---|---|---|
| `TransientPaused` (`AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK`, duckable) | duck to 0.20 keep reading, restore on `GAIN`, no rewind owed | no reason line (continuous) |
| `TransientPaused` (non-duckable `TRANSIENT` / call-alarm silence-demanding) | pause immediately, never auto-resume | 3 s lock-screen line + warning earcon |
| `FocusDenied` / `FocusDelayed` | pause (`Refused`) / zero-audio hold (`Deferred`, no reason) | 3 s / none |
| `PermanentLoss` / `DeviceRemoved` | pause ≤1 s (routing), never auto-resume, latest-wins | 3 s + warning earcon |
| `VoiceUnavailable` / `FormatUndeliverable` / `DecodeFailed` | never start, no substitution | persistent failure line + failure earcon |

Each variant maps 1:1 to a ≤8-word lock-screen line + distinct earcon + logged `error` name. No-look channel = earcon (no spoken announcement).

## Speed Setting (FR-012)

Closed ladder `0.75x | 1.0x | 1.25x | 1.5x | 1.75x | 2.0x | 4.0x`, default `1.0x`, one step per action, in-app stepper + lock-screen step action (custom action, no value display — purity = no numeric rate display), app-wide `settings.last_rate` remembered across sessions/documents, pitch-preserving engine stretch at every rung.

## Progress (FR-013, Option 3 hybrid)

| Field | Type | Rule |
|---|---|---|
| `consumed_units` / `total_units` | parsed-content offsets | bar = fraction; audio-duration estimates never feed it |
| `label` | `Section k of N + position %` | text alongside bar |
| `determinacy` | fixed at session start | determinate only when `total_n` + `total_units` both known then; else indeterminate whole session; no flip, no backward jump; freeze-then-drop on re-parse |
| zero sections | — | bar indeterminate + no section title |

Lock-screen + in-app share the same `Progress` value (single authority). No seek without true random-access offset.

## Earcon (FR-016, Option B + fallback)

Procedural mono f32 @ 22050 Hz, ≤500 ms, 300–2500 Hz, distinct in timbre + rhythm + contour; play/pause/skip = caricature metaphors, warning/failure = abstract dissonant pair (failure transposed to E4 ~330 Hz square + minor-2nd dissonance to stay in-band 300–2500 Hz); duck-exempt short mix (bypasses speech gain, same stream/clock, never moves bookmark, never pre-empts nav, re-fires post-gain if missed); each trigger rides the same Rust publish as its lock-screen state change + distinct Kotlin haptic (silent/DND path). Fully operable with sounds disabled (state-change + haptic path).

## Diagnostic Record (FR-017 full trace)

Extends 001 `DiagnosticRecord` with mandatory per-event coverage: session start/stop, transport actions, **per-section progress, per-rate changes, buffer states**, interruption class + outcome (**measured** play-to-audio latency, device-removal pause latency, actual replay seconds), failure names — timestamps + content-hash refs only, never prose (`detail` ≤200 ch key=value identifier-pattern or typed fields; `error` required on failure *and* refusal). Bounds: 30-day window + 10 MB cap, oldest-first trim with `Trim` markers, `Gap` on storage-full, never transmitted, no in-app surface, adb-pull only. Schema: `contracts/diagnostic-record.schema.json` (+ delta doc `contracts/diagnostic-trace.md`).
