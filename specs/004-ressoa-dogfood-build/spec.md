# Feature Specification: Ressoa Dogfood Build

**Feature Branch**: `004-ressoa-dogfood-build`

**Created**: 2026-10-12

**Status**: Draft

**Input**: User description: "@.specify/assessments/ressoa-mobile-app/ — assessment verdict `go` for Option A only (dogfoodable single-voice sideload build): a maintainer-only installable app on the reference phone that reads long documents aloud with the screen off, shows only title + current section + progress on the lock-screen surface in the DESIGN-SONA appearance, resolves interruptions into states graspable without looking, resumes by content-derived position, and produces the missing on-device measurements. Nothing is published; full release scope is explicitly excluded."

**Assessment lineage**: `.specify/assessments/ressoa-mobile-app/{intake,research,problem,concept,decision}.md` (verdict: go for Option A scope; evidence confidence medium). Prior related work: `specs/001-android-media-shell/` (playback shell, host-verified), `specs/002-fr-024-synthesis-adapter/` (real-audio proof, SC-044 deferred), `specs/003-reference-device/` (reference phone).

## Clarifications

### Session 2026-10-14

- Q: When the maintainer picks a second document while a first is playing or paused, what must the app do with the first document's position and audio? → A: Auto-save + switch, never auto-start second — durable put of first position (last-spoken, Paused/Interrupted) before teardown, backend stop, new snapshot then keep-current-only trim, new doc through the normal resume offer starting at 0.0 when no stored position.
- Q: When the lock-screen or in-app rate-step action is invoked past the top (4.0x) or bottom (0.75x) rung, must it saturate or wrap? → A: Saturate at ends, never wrap or fail — R4_0→R4_0, R0_75→R0_75, idempotent no-op.
- Q: After a pause caused by device removal or silence-demanding focus loss, must a later device-reconnect or AUDIOFOCUS_GAIN auto-resume, or must it always wait for explicit Play? → A: Duck-restore automatic, everything else explicit-Play — MAY_DUCK gain restore stays automatic; DeviceRemoved/SilenceDemanding/PermanentLoss/FocusDenied plus any GAIN-after-pause requires explicit Play; FocusDelayed/Deferred zero-audio hold unchanged.
- Q: In which language must all user-visible strings appear (lock-screen ≤8-word reason lines, in-app screen, transport buttons)? → A: pt-BR everywhere user-visible, ≤8 words including any title — reason lines, buttons, and in-app strings in pt-BR; English identifiers stay diag-only and are never rendered.
- Q: What is the largest document the dogfood build must handle, and what must happen when a document exceeds it? → A: Fixture-scale bound + named refusal, streaming deferred — support up to ~2–3h at 1.0x (exact byte cap fixed at plan from the measured fixture); larger docs refuse before synthesis with closed-set FormatUndeliverable, never OOM/Unreachable.

### Session 2026-10-13

- Q: How does the maintainer get a document into the app to start listening? → A: Via the system file picker — Markdown and plain-text documents from device storage at listen time (narrowed by the format-scope clarification below); no library, no import step.
- Q: What in-app surface exists beyond the lock-screen surface in this build? → A: A single minimal screen — title, section, progress, speed control, and transport only; never document prose.
- Q: How long and how much of the diagnostic record is kept on the device? → A: 30-day window and 10 MB cap, oldest entries trimmed first.
- Q: How much already-heard audio may repeat after the phone is killed or loses power mid-sentence? → A: At most 60 seconds of already-heard audio may repeat (aligned to the pre-roll bound; interruption replay under FR-007 stays capped at 30 seconds).
- Q: On which phones must the lock-screen surface acceptance (SC-006) be verified? → A: Reference phone only — the cross-manufacturer matrix is deferred to the later distribution decision.
- Q: Which document formats must the dogfood build be able to open and read aloud? → A: Markdown (.md) and plain text (.txt) only; other formats are out of scope for this build.
- Q: What does each lock-screen skip forward/back control do when pressed? → A: Skip jumps to the previous/next section boundary, falling back to ±10 seconds when the document has no sections.
- Q: Which seven reading-speed steps must the app offer, and which one is the default? → A: The prior 7-rung ladder (0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x) with 1.0x default.
- Q: Which events and measurements must the on-device diagnostic record capture? → A: Full trace — session, transport, per-section progress, per-rate change, buffer states, interruption class+outcome with measured latencies/replay, and failure names; timestamps and content-hash refs only, never prose.
- Q: How must each driving-time control confirm itself without requiring a look? → A: Short distinct earcons (tones) per action plus lock-screen state change; operable with sounds on or off.
- Q: Which single offline voice must the dogfood build bundle for all listening in this build? → A: Any single CC0 voice at builder discretion, fixed at plan time (still exactly one voice in the build).
- Q: What pass bar makes the blind no-look confirmation check (SC-004) testable? → A: 10 out of 10 correct per confirmation type (play, pause, skip, rate-step, warning, failure).
- Q: How is the 3-second resume choice presented when a deliberately paused document is reopened? → A: Visual dialog for deliberate pause; lock-screen resume control after kill/power-loss.
- Q: Does any network-dependent voice code ship in this dogfood build? → A: Pure offline, zero network voice code in this build.
- Q: What document fixture sustains the 30-minute acceptance session at every speed including 4.0x? → A: Single fixed fixture (~2h at 1.0x), committed to repo.
- Q: How does the maintainer explicitly stop a document (clearing its stored position)? → A: Dedicated Stop control on the single in-app screen only; lock-screen surface has no Stop.
- Q: How is the on-device diagnostic record retrieved for acceptance measurements? → A: Maintainer-only adb pull from app-private storage; no transmission, no in-app surface.
- Q: How is a pause-demanding interruption reason made graspable without looking? → A: Distinct warning earcon plus the 3-second lock-screen reason text; no spoken announcement.
- Q: What language and content must the fixed acceptance fixture use? → A: Committed Portuguese Markdown with headings plus numbers/currency (exercises pt-BR voice, section skip, and spoken normalization).
--- *Non-normative annotation* ---
- Q: When is playback progress honestly determinate? → A: Summary only — FR-013 + Q39 normative: determinate only when both section count and total units were known at session start, else indeterminate for the whole session with no mid-session flip or backward jump; audio-duration estimates never feed it (see FR-013 and Q39; this bullet is non-normative). This is informational only; does not change the normative requirement above.
- Q: How should the app retain document access and bytes after the system file picker returns? → A: Hybrid — persisted URI for freshness plus transient internal snapshot for offline synthesis, both BLAKE3-keyed; synthesize from snapshot, re-pick on move/revoke (Option C).
- Q: How often must the reading position be durably persisted to meet the replay bounds? → A: Tiered hybrid — in-RAM ~1s plus non-durable per sentence/5s plus durable every 15–30s and on every interruption/lifecycle transition, sentence-start snap-back capped (Option C).
- Q: How is the determinate progress value computed when sections vary in length or are absent? → A: Hybrid — bar is content-offset fraction, label is Section k of N plus position percent; determinate only when both totals known at session start, else indeterminate for the whole session with no mid-session flip or backward jump (Option 3).
- Q: Which closed failure/reason enumeration must the build implement for audio denials, losses, and undeliverable voice? → A: Medium closed 7±1 — FocusDenied, FocusDelayed, PermanentLoss, TransientPaused, DeviceRemoved, VoiceUnavailable, FormatUndeliverable, DecodeFailed; no Other/Unknown, no catch-all (Option O2).
- Q: Which earcon scheme must confirm every driving-time control blind, and how does it stay operable with sounds off? → A: Hybrid auditory-icon plus earcon — play/pause/skip as caricature metaphors, warning/failure as abstract dissonant pair, all ≤500ms in 300–2500 Hz with distinct timbre+rhythm+contour, each mirrored 1:1 on lock-screen plus haptic for silent operation (Option B plus fallback).
--- *Non-normative annotation* (informational only; does not change the normative FR-007 requirement above) ---
- Q: Is the REWIND_FRACTION=0.05 constant in FR-007 a normative requirement that must be tested, or an internal accounting detail for the first_yield_ms diagnostic field? → A: Accounting detail — removed from FR-007 (normative content: fixed 5 s cue, 30 s interruption / 60 s kill caps), kept in plan/contract as implementation note. **(Non-normative; see `data-model.md` annotation at `first_yield_ms`)**.
- Q: Is the progress label template "Section {k} of {N} — {p}% (content-offset)" in FR-013 a literal user-visible string rendered as-is, or an English identifier that gets translated to pt-BR before rendering? → A: English identifier — rendered string is the pt-BR translation.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Listen through a whole commute with the screen off (Priority: P1)

The maintainer opens a long document via the system file picker, presses Play, locks the phone, and listens for the duration of a commute without ever unlocking. Playback continues with the screen off and the device locked; transport controls (play, pause, skip forward/back) stay reachable from the lock-screen surface.

**Why this priority**: This is the reason the product exists. Every other story is worthless if playback dies when the screen turns off, because the primary context is a moving vehicle.

**Independent Test**: Install the build on the reference phone, start the fixed fixture document, lock the device, and confirm continuous audible reading for a 30-minute session with lock-screen controls functional throughout.

**Acceptance Scenarios**:

1. **Given** a document is reading aloud, **When** the screen turns off or is manually locked, **Then** playback continues uninterrupted until the user pauses or the document ends.
2. **Given** playback continues with the screen off, **When** the user opens the lock-screen audio surface without unlocking, **Then** play, pause, and skip controls are visible and functional.
3. **Given** the user presses Play, **When** the system starts reading, **Then** the first audible speech arrives within 3 seconds.

---

### User Story 2 - Survive interruptions without looking (Priority: P1)

While listening, the driver is interrupted: a navigation prompt speaks, a call or alarm demands silence, or the headphones are unplugged. Each interruption resolves into a state the driver can grasp by ear and by glance — quieter-but-continuing under a navigation prompt, paused-with-stated-reason for a call/alarm or headphone loss — and never resumes audio on its own after a pause that demands an explicit choice.

**Why this priority**: Interruptions are where driving gets dangerous. A reader that talks over a turn prompt teaches the driver to turn the app off; a reader that blasts from the speaker when headphones come out is a privacy and safety failure.

**Independent Test**: During playback on the reference phone, trigger each interruption class (navigation-style transient, call-style demanding, headphone removal, permanent loss) and confirm the resulting audible state, the stated reason, and the resume behavior — all without unlocking the phone.

**Acceptance Scenarios**:

1. **Given** audio is playing, **When** a navigation-style transient interruption arrives, **Then** the reading gets clearly quieter but keeps going, and returns to full volume afterwards without losing the listener's place.
2. **Given** audio is playing, **When** a call- or alarm-style interruption arrives, **Then** reading pauses immediately, the lock-screen surface states the reason briefly, and playback never resumes on its own.
3. **Given** audio is playing through headphones, **When** the headphones are unplugged, **Then** reading pauses within 1 second rather than playing aloud.
4. **Given** reading paused for an interruption, **When** the driver presses Play again, **Then** reading resumes at or just before the point of interruption — never mid-word, never far back (at most 30 s after interruption / 60 s after kill per FR-007: containing-sentence-start + 5–10 s cue, never mid-word).

---

### User Story 3 - Never lose your place (Priority: P2)

The maintainer reopens a document days later — possibly after the file was moved or renamed — and is offered a 3-second choice to continue where they left off or start over; silence (no choice) continues from the stored position for a deliberate pause. If the document's content changed since, no stale resume is offered. If the maintainer explicitly stopped the document, the next open starts from the beginning.

**Why this priority**: Losing position in a long document is the core pain for technical readers; a resume that points at the wrong place after an edit is worse than starting over.

**Independent Test**: Pause mid-document, move and rename the file, reopen it, and confirm the continue-from-position offer; edit the file's content and confirm no resume is offered; explicitly stop and confirm the next open starts from the beginning.

**Acceptance Scenarios**:

1. **Given** the reader paused a document, **When** they reopen the same content (even moved or renamed), **Then** they are offered a 3-second choice to continue from the stored position or start over, and silence continues from the stored position for a deliberate pause.
2. **Given** the document's content changed since the stored position was saved, **When** the reader reopens it, **Then** no resume is offered and reading starts from the beginning.
3. **Given** the reader explicitly stopped a document, **When** they open it again, **Then** reading starts from the beginning with no resume offer.

---

### User Story 4 - A lock screen that respects attention (Priority: P2)

While driving, the lock-screen surface shows only what orients without demanding reading: the document title, the current section title, and a progress indication. It never shows document prose, elapsed or remaining time, playback rate, or voice identity. The reading speed follows a small fixed set of steps adjustable in the app plus a value-blind rate-step action on the lock screen (no rate value ever shown there), persists across sessions, and keeps the voice's natural pitch at every speed.

**Why this priority**: A lock screen that invites reading while driving is a hazard; a surface limited to title, section, and progress gives orientation at a glance without demanding eyes.

**Independent Test**: During a commute-length session, inspect the lock-screen surface at multiple points (playing, paused, section change, rate change) and confirm only the permitted fields ever appear; change the rate in the app and confirm it persists into the next session with unchanged voice character.

**Acceptance Scenarios**:

1. **Given** any playback state, **When** the lock-screen surface is shown, **Then** it displays at most the document title, the current section title, and a progress indication — never document prose, times, rate, or voice.
2. **Given** the reader changes the reading speed in the app, **When** they reopen any document later, **Then** the chosen speed is still in effect and the voice sounds quicker or slower, not higher- or lower-pitched.
3. **Given** the document's total length is not yet known, **When** progress is shown, **Then** the indication is honestly indeterminate — never a guessed or estimated total.
4. **Given** the device is locked, **When** the driver invokes the lock-screen rate-step action, **Then** the speed advances one step with earcon+haptic confirmation only and the lock-screen still displays no rate value.

---

### Edge Cases

- What happens when the system denies or delays the audio request at Play time? The reader sees the closed-set reason (FocusDenied or FocusDelayed), never silence presented as playback.
- What happens when a transient interruption is immediately followed by a permanent one? The latest event wins: the final state is paused with no auto-resume, and the stated reason reflects the latest cause.
- What happens when a permanent loss is followed by a transient interruption? Permanent loss is a sticky state (non-normative summary): the transient is a no-op (the app is already paused), and any subsequent AUDIOFOCUS_GAIN waits for explicit Play — never auto-resumes (FR-005 normative).
- What happens when the lock-screen surface is dismissed while audio plays? Dismissal is refused while playing; pausing is the route to dismissal.
--- *Non-normative annotation* (informational only; does not change the normative FR-007 requirement above) ---
- What happens when the phone is killed or loses power mid-sentence? Summary only — FR-007 normative: At most 60 seconds of already-heard audio repeats on resume per FR-007 (30s interruption cap, 60s kill/power-loss cap; guaranteed by the FR-007 20s fixed durable heartbeat within the 15–30s envelope plus event-durable writes; worst-case loss is one interval) (see FR-007; this bullet is non-normative).
- What happens when a second document is picked while a first is playing or paused? The first position is durably saved before teardown, current audio stops, and the new document goes through the normal resume offer without auto-starting; the first bookmark is never deleted by the switch.
- What happens when the phone's own lock-screen privacy setting hides notification contents? The app obeys the platform setting and does not work around it.
- What happens when the requested voice or audio format cannot be delivered? Reading does not start; the closed-set failure (VoiceUnavailable, FormatUndeliverable, or DecodeFailed) is shown instead of substituted or silent output.
- What happens when the document has no sections or the position precedes the first section? No section title is shown rather than a wrong one.

## Normative Contract References

The following contract files are the normative sources for the algorithms, grammars, and flows referenced in this spec; this spec, `plan.md`, and `tasks.md` reference them rather than restating them:

- `contracts/resume-algorithm.md` — FR-007 resume/snap-back/replay algorithm (normative for A1).
- `contracts/earcon.md` — FR-016 earcon grammar + duck-exempt bypass (normative for A2).
- `contracts/ingest.md` — FR-018 document ingest flow (normative for A3).
- `contracts/surface.md` — FR-011/FR-019 surface rules + progress label.
- `contracts/diagnostic-trace.md` — FR-017 diagnostic record shape.

When a contract and this spec appear to diverge, the contract is authoritative; updates must be coordinated (see plan.md Complexity Tracking rule).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST continue reading aloud with the screen off and the device locked until the user pauses or the document ends.
- **FR-002**: The system MUST expose play, pause, and skip-forward/back controls on the lock-screen surface during playback; each skip MUST jump to the next/previous section boundary, falling back to ±10 seconds when the document has no sections. The **media playback** lock-screen surface MUST NOT expose a Stop control. The always-resident service's paused notification (the resident-paused surface, `contracts/surface.md` §Resident-paused notification surface) is a distinct surface outside that prohibition: it exposes the resident-stop action, which ends residency while PRESERVING the stored position and never clears it. The Stop CONTROL — the transport Stop that clears the stored position — remains in-app only (FR-010, FR-019), so a document ended by the resident-stop is still offered for resume, and a document ended by the Stop CONTROL is not.
- **FR-003**: The system MUST produce the first audible speech within 3 seconds of a Play action on the reference phone.
- **FR-004**: The system MUST make reading clearly quieter (to 0.20× gain (±0.02, ~20%)) while continuing under a navigation-style transient interruption (TransientPaused duckable outcome keyed to AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK), and restore full volume afterwards without losing the listener's place.
- **FR-005**: The system MUST pause within 1s (same bound as FR-006 routing) on a silence-demanding interruption (call, alarm) or permanent audio loss, state the reason on the lock-screen surface for 3 seconds, and never resume audio on its own afterwards. TransientPaused has two mapped outcomes keyed by focus class: AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK → duck per FR-004; AUDIOFOCUS_LOSS_TRANSIENT (non-duckable)/call/alarm → pause + 3s reason + never auto-resume. Reasons MUST come from the closed set (FocusDenied, FocusDelayed, PermanentLoss, TransientPaused, DeviceRemoved, VoiceUnavailable, FormatUndeliverable, DecodeFailed — no Other/Unknown); each pause-demanding variant maps 1:1 to a ≤8-word pt-BR lock-screen line (word count includes any appended title), a distinct warning earcon, and a logged failure name; `FocusDelayed` (zero-audio hold) and `TransientPaused`-duckable (continuous, no interruption of playback) map to no line and no earcon. Duck-gain restore after a duckable transient (MAY_DUCK) is automatic; every other paused outcome (DeviceRemoved, non-duckable TransientPaused, PermanentLoss, FocusDenied, and any AUDIOFOCUS_GAIN arriving after a pause) MUST wait for explicit Play and MUST never auto-resume, and device-reconnect MUST only latch the route, never change transport. The no-look channel for the reason is a distinct warning earcon (no spoken announcement). FR-006 is the timing bound for the DeviceRemoved subset of FR-005.
- **FR-006**: The system MUST pause within 1 second when the audio output device is removed (headphones unplugged or wireless route lost) rather than playing aloud. FR-006 is the timing bound for the DeviceRemoved subset of FR-005.
- **FR-007**: The system MUST resume after an interruption at or just before the point of interruption — at containing-sentence-start snapped backward, then further backward by a fixed 5 s context cue, total backward ≤30 s interruption / ≤60 s kill, never mid-word, never doc start; the persisted cursor MUST be the last-spoken position (never buffered-ahead) in a single atomic transaction, durably written every 20s fixed (within 15-30s envelope; worst-case loss one interval) and on every pause, focus-loss, device-removal, section-change, stop, app-background, DocumentEnd event, with in-RAM tracking ~1s and non-durable commits per sentence/5s between durable writes. Degrade falls back to the raw last-spoken cursor, never doc start.
- **FR-008**: The system MUST key a stored reading position to the document's content via BLAKE3 hash (not its file name, location, or URI) so a moved or renamed document still offers resume; on every open the system MUST re-hash the bytes actually read and compare to the stored hash (match = resume offer, mismatch = no resume).
- **FR-009**: The system MUST offer a 3-second choice via a visual dialog on the single in-app screen (continue vs. start over) when reopening a deliberately paused document, continuing from the stored position when no choice is made; after an unexpected end (system kill, power loss, permanent audio loss, or engine failure) it MUST wait paused with an explicit resume control on the lock-screen surface instead.
- **FR-010**: The system MUST NOT offer resume for content that changed since the position was stored, nor after an explicit stop (via the in-app Stop control) or a finished document.
- **FR-011**: The lock-screen surface MUST show at most the document title (original language — proper noun, exempt from pt-BR; source: first Markdown H1 heading for `.md`, filename without extension for `.txt`, fallback: filename), the current section title, and a progress indication; it MUST NEVER show document prose, elapsed/remaining time, playback rate, or voice identity. Note: lock-screen MUST NOT display rate/voice values; a rate-step affordance without value display is permitted. All user-visible strings on the lock-screen surface (reason lines, state labels) MUST be pt-BR; English identifiers stay diag-only and are never rendered.
- **FR-012**: The system MUST offer reading speed as seven fixed steps (0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x, default 1.0x), one step per action saturating at the ends (step-up at 4.0x stays 4.0x, step-down at 0.75x stays 0.75x, never wraps or fails), adjustable inside the app and via lock-screen rate-step action without displaying rate value (earcon+haptic confirm only), remembered across sessions and documents, preserving the voice's natural pitch at every speed. No numeric rate value appears on either surface (lock-screen or in-app); the in-app stepper orients by position + earcon/haptic only.
- **FR-013**: The progress indication MUST be determinate from parsed content once known — bar value as content-offset fraction (consumed/total parsed units, where a parsed unit = one sentence from `adapter-synthesis::chunk::split_document`) with exact text template `"Section {k} of {N} — {p}% (content-offset)"` (English identifier; rendered string is the pt-BR translation per FR-011) — determinate only when both section count and total units were known at session start, and honestly indeterminate for the whole session otherwise (e.g., parse incomplete or position precedes the first section), never flipping mid-session and never jumping backward on re-parse (freeze then drop to indeterminate); with zero sections the bar MUST stay indeterminate and no section title shown; audio-duration estimates MUST NEVER feed it and the bar MUST NEVER seek (no seek control exists in this build).
- **FR-014**: The system MUST refuse lock-screen dismissal while audio plays; pausing MUST be the route to dismissal.
- **FR-015**: The system MUST produce audible speech with no network connectivity; this build is pure offline with zero network-dependent voice code — any future network voice is a later opt-in, never the sole path.
- **FR-016**: The system MUST confirm every driving-time control with a short distinct earcon per action — hybrid auditory-icon plus earcon (play/pause/skip as caricature metaphors, warning/failure as abstract dissonant pair, each ≤500ms in 300–2500 Hz differing in timbre plus rhythm plus contour, duck-exempt short mix that never pre-empts navigation and dropped during duck/pause, re-fired post-GAIN once Playing is restored) — plus a glanceable lock-screen state change (giant transport glyph, progress, warning/failure banner) plus distinct haptic so each action is verifiable without looking; the interface MUST remain fully operable when confirmation sounds are disabled or DND/muted (via the state-change plus haptic path).
- **FR-017**: The system MUST keep a private on-device diagnostic record of session start/stop, transport actions, per-section progress, per-rate changes, buffer states, interruption class and outcome (with measured play-to-audio latency, device-removal pause latency, and actual replay seconds), and failure names with timestamps and content-hash refs only, retained for a 30-day window and 10 MB cap with oldest-first trimming, never transmitted anywhere, with no in-app surface in this scope and no document prose in any entry. Retrieval for acceptance trials is maintainer-only via adb pull from app-private storage.
- **FR-018**: The system MUST let the reader open Markdown (.md) and plain-text (.txt) documents from device storage via the system file picker (`ACTION_OPEN_DOCUMENT`) at listen time, with no user-visible library or import step; on pick the system MUST attempt a persistable URI read grant (best-effort, failure re-prompts), read the bytes once, record BLAKE3, and write a transient session-scoped snapshot to app-private internal storage that synthesis reads from; reopening a moved or renamed document MUST go through the same picker path with hash-match restore. Other formats are out of scope for this build. Snapshot retention is keep-current-only: trim all but the current snapshot on new pick / Stop / document-end. Picking a second document while a first is playing or paused MUST auto-save the first position (durable put of last-spoken with Paused/Interrupted end-kind) before teardown, stop current audio, write the new snapshot then trim keep-current-only, and route the new document through the normal resume offer (stored Paused → 3-second dialog, Interrupted → lock-screen resume control, otherwise start at 0.0 with no offer); the second document MUST never auto-start while the first is playing, and switching MUST never delete the first bookmark. Oversize documents beyond the fixture-scale bound (see Assumptions) MUST refuse before synthesis with closed-set FormatUndeliverable, never OOM or generic failure.

**FR-018 Acceptance Scenarios**:
1. **Given** the maintainer picks a Markdown document via the system file picker, **When** the document is opened, **Then** the bytes are read once, BLAKE3 recorded, and a transient snapshot is written to app-private storage.
2. **Given** a document is open and playing, **When** the maintainer picks a second document, **Then** the first position is durably saved, audio stops, the new snapshot is written, and the new document goes through the normal resume offer without auto-starting.
3. **Given** a document exceeds the fixture-scale bound, **When** the maintainer picks it, **Then** reading refuses before synthesis with FormatUndeliverable, never OOM or generic failure.
4. **Given** a document is open, **When** the maintainer explicitly stops it, **Then** the current snapshot is trimmed and the stored position is cleared.
- **FR-019**: The system MUST provide a single minimal in-app screen carrying only the document title (original language — proper noun, exempt from pt-BR), current section title, progress indication, speed control, and transport (play, pause, skip-forward/back, and a dedicated Stop that clears the stored position); it MUST NEVER show document prose. All user-visible strings on this screen (labels, transport names, status lines) MUST be pt-BR; English identifiers stay diag-only and are never rendered.

### Key Entities

- **Document**: A Markdown (.md) or plain-text (.txt) document the user listens to; identified by BLAKE3 of its bytes for position purposes, carrying titled sections (Markdown headings; TXT may have none) used for orientation; synthesis reads only the transient app-private snapshot taken at pick time.
- **Reading Position**: The listener's place in a document's content (section plus sentence-anchored offset with monotonic clock), stored per BLAKE3 content identity under the FR-007 cadence and cleared on explicit stop, document end, or content change.
- **Playback Session**: One listening episode for a document — playing | paused | stopped (transport STATE; The Stop CONTROL (the transport Stop that clears the stored position) is in-app only; the media lock-screen surface has no Stop CONTROL; the resident-paused surface (`contracts/surface.md` §Resident-paused notification surface) carries the resident-stop, which ends residency and PRESERVES the stored position (FR-002)) with end_kind Paused|Stopped|Completed|Interrupted (Completed=document end; Interrupted=kill/power-loss/permanent loss/engine failure) — with its interruption history and resume offer behavior. Note: "change voice" (Constitution VIII) is not applicable in this single-voice build — the operation does not exist.
- **Interruption**: A classified external audio event from the closed set (TransientPaused with two mapped outcomes keyed by focus class: AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK → duck ~20% keep playing per FR-004; AUDIOFOCUS_LOSS_TRANSIENT (non-duckable)/call/alarm → pause + 3s + never auto-resume per FR-005; FocusDenied/FocusDelayed/PermanentLoss/DeviceRemoved for pause-demanding and loss, VoiceUnavailable/FormatUndeliverable/DecodeFailed for undeliverable voice) each with its mandated outcome and 3-second reason lifetime.
- **Speed Setting**: A single app-wide reading-speed step from the fixed set, remembered across sessions and documents.
- **Diagnostic Record**: The bounded, private, on-device full-trace event log (session, transport, per-section progress, rate changes, buffer states, interruptions with measured latencies, failures) used to diagnose failures after the fact; never containing document prose.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The maintainer completes a 30-minute commute-length session (per speed, 30 min at each rung; see quickstart §3.1) on the reference phone using a single fixed fixture (≥2 h of material at 1.0x, sufficient to sustain a 30-min session at 4.0x; exact duration recorded in fixture README, committed to the repo: Portuguese Markdown with headings plus numbers/currency) with the screen off and locked, using only lock-screen controls, with zero unlocks required to keep listening — valid at every speed including 4.0x.
- **SC-002**: First audible speech arrives within 3 seconds of Play in 10 out of 10 trials on the reference phone from a cold start (process-killed, fixture unopened, reference phone, radios off).
- **SC-003**: In 10 out of 10 trials per interruption class (navigation transient, call/alarm, headphone removal, permanent loss), the observed outcome matches the mandated one with no unlocking and no spontaneous resume after pauses that forbid it.
- **SC-004**: In blind (no-look) trials, the listener correctly identifies play, pause, skip, rate-step, warning, and failure confirmations without seeing the screen in 10 out of 10 trials per confirmation type.
- **SC-005**: After moving and renaming a paused document, 10 out of 10 reopens offer continue-from-position; after editing its content, 10 out of 10 reopens offer no resume; after explicit stop (via the in-app Stop control), the next open starts from the beginning; the deliberate-pause offer window is 3 s with silence continuing from the stored position.
- **SC-006**: Across a full session including section changes, rate changes, pauses, and interruptions, the lock-screen surface is observed showing only title, section, and progress in 10 out of 10 inspections on the reference phone — zero prose, time, rate, or voice leaks; dismissal attempted while playing is refused in 10 out of 10 attempts (pausing is the route to dismissal).
- **SC-007**: The chosen reading speed persists across sessions and documents, voice character stays unpitched at every step (normative: pitch deviation within PITCH_TOLERANCE_CENTS=50 — implementation reference constant, not a spec-level metric; engine-contract stretch; implemented/attested by T039 — spec does not depend on the task ID for its meaning) [canonical file: `crates/player/src/rate.rs` / contract `contracts/surface.md`]; and listeners judge the voice quicker/slower, not higher-/lower-pitched, in 10/10 trials per rung (same 10/10 discipline as SC-002…SC-006): persistence holds, pitch deviation within the engine's stated tolerance, blind listener judges quicker/slower not higher/lower-pitched.
- **SC-008**: The diagnostic record is verified retained within its 30-day window and 10 MB cap on device with zero entries containing document prose.
- **SC-009**: With all radios off (airplane mode enabled, Wi-Fi and cellular disabled), the maintainer completes a 15-minute session on the reference phone using the fixed fixture with zero network-dependent voice errors — playback continues uninterrupted from start to end with no connectivity, in 10 out of 10 trials.
- **SC-010**: The single in-app screen is observed showing only the document title, current section title, progress indication, speed control, and transport (play, pause, skip-forward/back, Stop) in 10 out of 10 inspections — zero document prose, elapsed/remaining time, rate value, or voice identity; the Stop control clears the stored position in 10 out of 10 trials; all user-visible strings on this screen are verified pt-BR.

## Assumptions

- The single offline voice included in this build (any one CC0 Piper voice — faber, cadu, or jeff — fixed at plan time, still exactly one voice in the build) is an acceptable listening experience for commute use; cabin intelligibility is measured during acceptance, not assumed.
- Sideload installation on the reference phone by the maintainer is the distribution form; no reviewer, listing, changelog, or support channel is involved.
- The reference phone defined by the project's reference-device record is the normative measurement device for all time-bound criteria.
- Documents consumed are Markdown (.md) and plain-text (.txt) documents with titled sections (Markdown headings); documents without sections simply show no section title rather than a wrong one. Supported size is fixture-scale (~2–3h of material at 1.0x; the exact byte cap is fixed at plan time from the measured fixture); larger documents refuse before synthesis with FormatUndeliverable rather than streaming, truncating, or failing generically.
- The public naming boundary holds: user-visible strings use the Ressoa product identity while the technical workspace name stays out of the user surface.
- No second voice, cloud voice, credential handling, or unattended engine switching is in scope; a voice that cannot be delivered is a closed-set failure (VoiceUnavailable, FormatUndeliverable, or DecodeFailed), not a silent substitution.
- Measurements that fail their bounds stop the line for any later distribution decision rather than renegotiating the bounds.
