# Data Model: Android Media Shell

**Feature**: `001-android-media-shell` | **Source**: spec Key Entities + FRs

All enums are closed sets (Principle VII): no `Other`/`Unknown`, no `_ =>`. Adding a
variant is a breaking change (semver bump + docs).

## PlaybackSession

One continuous reading of one document. The single source of truth (FR-002).

| Field | Type | Rule |
|---|---|---|
| `document_fingerprint` | `Fingerprint` (blake3-256) | identifies the document (FR-036) |
| `position_secs` | `f64 ≥ 0` | authoritative time-offset; pre-roll never advances it |
| `transport` | `TransportState` | `Playing \| Paused \| Stopped \| Completed \| Interrupted` |
| `rate` | `PlaybackRate` | one of 7 closed rungs; app-wide memory, default 1.0x |
| `end_kind` | `SessionEnd` | `Paused | Stopped | Completed | Interrupted` — the FR-044 closed set; unexpected ends (kill, power loss, permanent focus loss, engine failure) are `Interrupted` — decides resume offer vs clean start vs teardown |

Transitions: `Playing → Paused` (pause, transient-silence demand, BECOMING_NOISY,
device loss); `Playing → Interrupted` (permanent loss, engine failure pending offer);
`Interrupted → Playing` (offer choice or fallback takeover, same position);
`Paused → Playing` (resume); any → `Stopped` (Stop press); `Playing → Completed`
(document end → clear stored position + tear down resident, FR-032b/FR-032f).
Never: `Interrupted → Playing` without user action except the FR-023a/FR-032c bounded
expiries; never any transition that reports playing while silent (FR-005).

## PlaybackPosition

`f64` seconds from spoken-audio start. Engine-independent narration point; crossing
voices re-anchors via `session` to the nearest sentence/node boundary (FR-038), falling
back to `0.0` when no boundary exists at/before it. Persisted on every pause-path;
replay after kill ≤30 s (FR-004).

## StoredPosition (redb `bookmarks`)

| Field | Type | Rule |
|---|---|---|
| key `fingerprint` | `[u8; 32]` | blake3 of document bytes |
| `offset_secs` | `f64` | last spoken position |
| `paused_at_ms` | `u64` unix-ms | basis for 5%-of-pause rewind (FR-015) |
| `sentence_idx` | `u32` | snap target hint |

Cleared only by Stop or document end. Unreachable hash (edited content) ⇒ no offer.

## ResumeOffer

Shown on reopen with stored position. Two disjoint forms: **deliberate-pause** (visible
3 s, continue-default + start-over alternative, auto-starts stored position on expiry only
while the application is in the foreground and otherwise converts to explicit-resume,
FR-032c) and **unexpected-end** (paused state + explicit resume control, never
auto-starts, FR-031/FR-032d). Start-over ⇒ begin at `0.0`, discard stored position.

## AlwaysResident

The playback `Service` itself retained paused between sessions (FR-009a). Invariant:
≤1 resident component, ≤1 ongoing notification. Paused-resident notification asserts no
elapsed/remaining time and no section title. Torn down at document end (FR-032f); a kill
while nothing plays is not an interrupted session.

## PlaybackRate (closed)

`0.75x | 1.0x | 1.25x | 1.5x | 1.75x | 2.0x | 4.0x`. One step per press, saturate at ends
(FR-019). App-wide `settings.last_rate`; new sessions open at it (FR-019b). Applied by
engine time-stretch op (FR-043); pitch/voice invariant across all rungs incl. 2.0x→4.0x.

## FocusTransition (closed)

Classified by the interrupter's request: `DuckableTransient` (→ volume ~20%, keep
reading, FR-013), `SilenceDemanding` call/alarm (→ pause + FR-041 reason, FR-013a),
`PermanentLoss` (→ pause, no auto-resume, FR-014). Carries `paused_duration` for FR-015
rewind: cursor back `min(5% × duration, 30 s)`; audio from that sentence start; total
backward movement ≤30 s even for long sentences.

## PauseReason vs EngineFailure (disjoint, FR-037a)

- `PauseReason`: `"Leitura pausada — outro app usando o áudio"` + document title only;
  scrolling lock-screen text; expires everywhere after 3 s (FR-041).
- `EngineFailure` (closed `EngineError`: `Unreachable | InitFailed | RateMismatch |
  ChannelMismatch | DeviceRefused | DeviceLost | StartRefused`): short lock-screen
  statement + full reason persisted in-app indefinitely (FR-037); leads to `EngineOffer`.

## EngineOffer

Names failure + consent-bounded alternatives in declared order (FR-023c); 3 s window;
choice honoured or fallback auto-continues from last spoken position; no alternative ⇒
paused-with-reason. Repeated failure re-offers over remaining engines (FR-023a).

## PreRollBuffer

Ring ≤60 s of engine-delivered audio; never advances position; empty while idle; refilled
on Play; abandoned at session end (FR-015a/FR-015c). Pipeline-side bound on handed-over
audio (FR-015b).

## NotificationSurface

Title (metadata, never first-sentence, FR-029) + most-recent-passed section title
(none before first heading / no headings, FR-028c, re-resolved on boundary-cross/resume/
skip/re-anchor, FR-028d) + progress (determinate iff total known at session start, else
indeterminate whole session, FR-028a). Non-dismissable while playing; pause is the route
to dismissal; transient-reason surfaces auto-dismiss at 3 s, resident becomes
dismissable (FR-028e). Never: body text, elapsed/remaining, rate, voice.

## DiagnosticRecord

Append-only events (`SessionStart | SessionEnd | TransportChange | FocusTransition |
OutputDevice | Failure | Refusal | Trim | Gap`), each with position + timestamp; 30-day +
10 MB oldest-first bounds; gap/trim markers instead of silent gaps (`Gap` marks entries
dropped under storage exhaustion, emitted per the FR-039 edge case); private storage;
never transmitted; no app surface. Schema: `contracts/diagnostic-record.schema.json`.

## DeclaredFormat

Single declared sample rate (22050 Hz, FR-033) + mono-f32 shape; engine resamples, Rust module converts
device format only. Rate/channel/width mismatch ⇒ `EngineError`, never silent adaptation.
