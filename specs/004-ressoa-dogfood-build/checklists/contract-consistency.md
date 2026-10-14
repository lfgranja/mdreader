# Contract & Data Model Consistency Checklist: Ressoa Dogfood Build

**Purpose**: Verify that `data-model.md`, all four contracts (`surface.md`, `earcon.md`, `ingest.md`, `diagnostic-trace.md`), and `plan.md` are mutually consistent and fully traceable to the functional requirements (FR-001…FR-019) and key entities defined in `spec.md`.

**Created**: 2026-10-13

**Scope**: Document-level consistency only (pre-implementation). Wire-format, enum, and closed-set divergence checks are forward-looking gates for the implementation phase.

---

## 1. Data Model ↔ Spec Alignment (entity-by-entity)

Each entity in `data-model.md` must have every field traceable to a specific FR or key-entity definition in `spec.md`.

### Document

- [x] **Document.bytes** — Traceable to FR-018 ("read the bytes once") + Key Entities ("identified by BLAKE3 of its bytes"). `data-model.md` line 13: `Vec<u8>` read once via `ContentResolver`.
- [x] **Document.fingerprint** — Traceable to FR-008 ("key a stored reading position to the document's content via BLAKE3 hash") + Key Entities. `data-model.md` line 14: `Fingerprint` blake3-256 of `bytes`, position identity, path/URI-independent.
- [x] **Document.uri** — Traceable to FR-018 ("attempt a persistable URI read grant (best-effort)") + Clarification 2026-10-14 Option C ("persisted URI for freshness"). `data-model.md` line 15: persisted SAF URI grant, freshness check only.
- [x] **Document.snapshot_path** — Traceable to FR-018 ("write a transient session-scoped snapshot to app-private internal storage that synthesis reads from"). `data-model.md` line 16: `filesDir/snapshots/<hex>.{md,txt}`, transient, synthesis reads only this.
- [x] **Document.sections** — Traceable to Key Entities ("carrying titled sections (Markdown headings; TXT may have none) used for orientation"). `data-model.md` line 17: `Vec<Section>` from Markdown headings (`doc-structure`).
- [x] **Document reopen flow** — Traceable to FR-008 ("on every open the system MUST re-hash the bytes actually read and compare to the stored hash") + FR-010. `data-model.md` line 19: read bytes → `fingerprint()` → `store.get(hash)` → match = resume, mismatch = start 0.0.
- [x] **Document format scope (.md / .txt)** — Traceable to FR-018 + Clarification 2026-10-13. `data-model.md` header line 7: ".md / .txt only, FR-018".

### Section

- [x] **Section.index_k** — Traceable to FR-013 (label template `"Section {k} of {N}"`). `data-model.md` line 26: `u32` 1-based.
- [x] **Section.total_n** — Traceable to FR-013 ("determinate only when both section count and total units were known at session start"). `data-model.md` line 26: `u32` known at session start; unknown → whole session indeterminate.
- [x] **Section.title** — Traceable to FR-011 ("the current section title") + Edge Cases ("no section title is shown rather than a wrong one"). `data-model.md` line 27: heading text; absent → show no title.
- [x] **Section.start_unit** — Traceable to FR-002 (skip target) + FR-013 (progress numerator). `data-model.md` line 29: parsed-content offset.
- [x] **Section skip logic** — Traceable to FR-002 ("skip MUST jump to the next/previous section boundary, falling back to ±10 seconds"). `data-model.md` line 30.

### Reading Position

- [x] **Reading Position entity** — Traceable to Key Entities ("section plus sentence-anchored offset with monotonic clock"). `data-model.md` lines 32-43.
- [x] **Reading Position.offset_units** — Traceable to FR-007 (persisted cursor = last-spoken position). `data-model.md` line 38: parsed-content offset, progress numerator, resume anchor.
- [x] **Reading Position.offset_secs** — Traceable to 001 shell compat + FR-007 pre-roll constraint. `data-model.md` line 39: `f64 ≥ 0`, pre-roll never advances it.
- [x] **Reading Position.sentence_start** — Traceable to FR-007 ("at containing-sentence-start snapped backward"). `data-model.md` line 40: snapped boundary, mid-word cap-point forward-snaps to next word-gap.
- [x] **Reading Position.first_yield_ms** — Traceable to FR-007 ("5%-of-pause (REWIND_FRACTION=0.05 capped 30 s) basis is retained for first_yield_ms accounting only"). `data-model.md` line 41.
- [x] **Reading Position cadence** — Traceable to FR-007 (20 s fixed durable within 15–30 s envelope, in-RAM ~1 s, non-durable per sentence/5 s, event-durable writes). `data-model.md` line 43.
- [x] **Reading Position caps** — Traceable to FR-007 (30 s interruption replay, 60 s kill/power-loss replay). `data-model.md` line 43.
- [x] **Reading Position resume rendering** — Traceable to FR-007 (sentence-start snap + 5–10 s cue, never mid-word, degrade to raw cursor, never doc start). `data-model.md` line 43.
- [x] **Reading Position deletion** — Traceable to FR-010 (cleared on explicit stop, document end, content change). `data-model.md` line 43: Stop → `delete`.

### Playback Session

- [x] **Playback Session transport states** — Traceable to Key Entities ("playing | paused | stopped (transport STATE)"). `data-model.md` line 47: Playing | Paused | Stopped.
- [x] **Playback Session Stop is CONTROL only** — Traceable to FR-002 ("MUST NOT expose a Stop control" on lock-screen) + FR-019 (dedicated Stop on in-app screen). `data-model.md` lines 47, 51.
- [x] **Playback Session end_kind** — Traceable to Key Entities ("end_kind Paused|Stopped|Completed|Interrupted"). `data-model.md` line 47.
- [x] **Playback Session Completed = document end** — Traceable to Key Entities. `data-model.md` line 47.
- [x] **Playback Session Interrupted = kill/power-loss/permanent loss/engine failure** — Traceable to Key Entities + FR-009. `data-model.md` line 47, 50.
- [x] **Playback Session Paused → 3 s dialog** — Traceable to FR-009 ("3-second choice via a visual dialog"). `data-model.md` line 50.
- [x] **Playback Session Interrupted → lock-screen resume control** — Traceable to FR-009 ("wait paused with an explicit resume control on the lock-screen surface"). `data-model.md` line 50.
- [x] **Playback Session totals fixed at session start** — Traceable to FR-013 (determinacy fixed at session start). `data-model.md` line 49.
- [x] **Playback Session ingest binding** — Traceable to FR-018. `data-model.md` line 49: binds `snapshot_path` + `fingerprint`.

### Interruption

- [x] **Interruption closed set (8)** — Traceable to FR-005 + Clarification 2026-10-14 (Option O2). `data-model.md` line 57: `FocusDenied | FocusDelayed | PermanentLoss | TransientPaused | DeviceRemoved | VoiceUnavailable | FormatUndeliverable | DecodeFailed`.
- [x] **Interruption TransientPaused duckable outcome** — Traceable to FR-004 (duck to ~20%, restore on GAIN). `data-model.md` line 61.
- [x] **Interruption TransientPaused non-duckable outcome** — Traceable to FR-005 (pause + 3 s + never auto-resume). `data-model.md` line 62.
- [x] **Interruption FocusDenied outcome** — Traceable to FR-005 (pause). `data-model.md` line 63.
- [x] **Interruption FocusDelayed outcome** — Traceable to FR-005 (zero-audio hold, no reason). `data-model.md` line 63.
- [x] **Interruption PermanentLoss outcome** — Traceable to FR-005 (pause, never auto-resume). `data-model.md` line 64.
- [x] **Interruption DeviceRemoved outcome** — Traceable to FR-006 (pause ≤1 s). `data-model.md` line 64.
- [x] **Interruption VoiceUnavailable/FormatUndeliverable/DecodeFailed outcome** — Traceable to FR-005 + Assumptions (never start, no substitution). `data-model.md` line 65.
- [x] **Interruption 3 s reason lifetime** — Traceable to FR-005 ("state the reason on the lock-screen surface for 3 seconds"). `data-model.md` lines 63-64.
- [x] **Interruption N:1 mapping** — Traceable to FR-005 (eight reasons map onto earcon kinds + line-or-None). `data-model.md` line 55, 67.

### Speed Setting

- [x] **Speed Setting closed ladder (7 rungs)** — Traceable to FR-012 ("seven fixed steps (0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x)"). `data-model.md` line 71.
- [x] **Speed Setting default 1.0x** — Traceable to FR-012 ("default 1.0x"). `data-model.md` line 71.
- [x] **Speed Setting saturation** — Traceable to FR-012 + Clarification 2026-10-14 ("saturate at ends, never wrap"). `data-model.md` line 71.
- [x] **Speed Setting persistence** — Traceable to FR-012 ("remembered across sessions and documents"). `data-model.md` line 71: `settings.last_rate`.
- [x] **Speed Setting pitch-preserving** — Traceable to FR-012 ("preserving the voice's natural pitch"). `data-model.md` line 71.
- [x] **Speed Setting no value display** — Traceable to FR-011/FR-012 ("no numeric rate value appears on either surface"). `data-model.md` line 71.

### Diagnostic Record

- [x] **Diagnostic Record entity** — Traceable to Key Entities + FR-017. `data-model.md` lines 88-90.
- [x] **Diagnostic Record coverage (session/transport/progress/rate/buffer/interruption/failure)** — Traceable to FR-017 ("session start/stop, transport actions, per-section progress, per-rate changes, buffer states, interruption class and outcome…, and failure names"). `data-model.md` line 90.
- [x] **Diagnostic Record measured latencies** — Traceable to FR-017 ("measured play-to-audio latency, device-removal pause latency, and actual replay seconds"). `data-model.md` line 90.
- [x] **Diagnostic Record content rules** — Traceable to FR-017 ("timestamps and content-hash refs only, never prose"). `data-model.md` line 90.
- [x] **Diagnostic Record bounds (30-day + 10 MB)** — Traceable to FR-017. `data-model.md` line 90.
- [x] **Diagnostic Record detail ≤200 chars** — Traceable to FR-017 constitution constraint. `data-model.md` line 90.
- [x] **Diagnostic Record error required on failure and refusal** — Traceable to FR-017 constitution constraint. `data-model.md` line 90.
- [x] **Diagnostic Record adb-pull only** — Traceable to FR-017 ("maintainer-only via adb pull"). `data-model.md` line 90.
- [x] **Diagnostic Record trim markers** — Traceable to FR-017 (oldest-first trim with `Trim` markers, `Gap` on storage-full). `data-model.md` line 90.

### Progress

- [x] **Progress entity** — Traceable to FR-013 (Option 3 hybrid). `data-model.md` lines 73-82.
- [x] **Progress consumed_units / total_units** — Traceable to FR-013 ("bar value as content-offset fraction (consumed/total parsed units)"). `data-model.md` line 77.
- [x] **Progress label** — Traceable to FR-013 (`"Section {k} of {N} — {p}% (content-offset)"`). `data-model.md` line 78.
- [x] **Progress determinacy** — Traceable to FR-013 ("determinate only when both section count and total units were known at session start, else indeterminate whole session"). `data-model.md` line 79.
- [x] **Progress zero sections** — Traceable to FR-013 ("with zero sections the bar MUST stay indeterminate and no section title shown"). `data-model.md` line 80.
- [x] **Progress audio-duration exclusion** — Traceable to FR-013 ("audio-duration estimates MUST NEVER feed it"). `data-model.md` line 77.
- [x] **Progress no seek** — Traceable to FR-013 ("the bar MUST NEVER seek"). `data-model.md` line 82.
- [x] **Progress single authority** — Traceable to FR-013 (lock-screen + in-app share same Progress). `data-model.md` line 82.

### Earcon

- [x] **Earcon entity** — Traceable to FR-016 (Option B + fallback). `data-model.md` lines 84-86.
- [x] **Earcon format (mono f32 @ 22050 Hz)** — Traceable to FR-016 + constitution audio pipeline contract. `data-model.md` line 86.
- [x] **Earcon duration (≤500 ms)** — Traceable to FR-016. `data-model.md` line 86.
- [x] **Earcon frequency band (300–2500 Hz)** — Traceable to FR-016. `data-model.md` line 86.
- [x] **Earcon distinctness (timbre + rhythm + contour)** — Traceable to FR-016. `data-model.md` line 86.
- [x] **Earcon caricature metaphors (play/pause/skip)** — Traceable to FR-016. `data-model.md` line 86.
- [x] **Earcon dissonant pair (warning/failure)** — Traceable to FR-016. `data-model.md` line 86.
- [x] **Earcon duck-exempt short mix** — Traceable to FR-016. `data-model.md` line 86.
- [x] **Earcon silent operability** — Traceable to FR-016 ("fully operable with sounds disabled"). `data-model.md` line 86.

---

## 2. Contract ↔ Data Model Alignment

Each contract must reference the correct data model entities and fields without divergence.

### surface.md ↔ data-model.md

- [x] **surface.md Progress shape ↔ data-model.md Progress** — `surface.md` line 45-46: `{ kind, consumed_units, total_units, k, n }` / `{ kind: "indeterminate" }` matches `data-model.md` Progress fields (consumed_units, total_units, index_k→k, total_n→n, determinacy→kind).
- [x] **surface.md transport values ↔ data-model.md transport** — `surface.md` line 47: `playing | paused | stopped` matches `data-model.md` line 47: Playing | Paused | Stopped.
- [x] **surface.md EndKind distinction ↔ data-model.md EndKind** — `surface.md` line 8: "Interrupted/Completed are EndKind, never transport values" matches `data-model.md` line 47.
- [x] **surface.md reason-line table ↔ data-model.md Interruption** — `surface.md` lines 12-20 reason lines map 1:1 to `data-model.md` line 67 N:1 mapping (all 8 reasons accounted for; FocusDelayed/Deferred → no line).
- [x] **surface.md reason_ttl_ms=3000 ↔ data-model.md 3 s lifetime** — `surface.md` line 10, 49 matches `data-model.md` line 63-64.
- [x] **surface.md reason_ttl_ms=null for failures ↔ data-model.md persistent failure line** — `surface.md` line 10 matches `data-model.md` line 65 (VoiceUnavailable/FormatUndeliverable/DecodeFailed).
- [x] **surface.md FocusDelayed/Deferred → no line ↔ data-model.md** — `surface.md` line 11 matches `data-model.md` line 63.
- [x] **surface.md Stop CONTROL distinction ↔ data-model.md** — `surface.md` line 8, 26 matches `data-model.md` lines 47, 51.
- [x] **surface.md 7 rungs ↔ data-model.md Speed Setting** — `surface.md` line 27 matches `data-model.md` line 71.
- [x] **surface.md DESIGN-SONA palette ↔ spec** — `surface.md` lines 33-37 palette tokens are spec-level appearance constants (no data-model entity needed).
- [x] **surface.md dismissal ↔ FR-014** — `surface.md` lines 10, 21, 50 matches spec FR-014.
- [x] **surface.md CLI surface (player progress) ↔ data-model.md Progress** — `surface.md` line 31 mirrors Progress shape.

### earcon.md ↔ data-model.md

- [x] **earcon.md 6 actions ↔ data-model.md Earcon** — `earcon.md` lines 9-14 (Play, Pause, Skip fwd/back, Warning, Failure, Rate step) match `data-model.md` line 86 caricature/dissonant grammar.
- [x] **earcon.md ≤500 ms ↔ data-model.md** — `earcon.md` lines 9-14 all list ≤500 ms, matching `data-model.md` line 86.
- [x] **earcon.md 300–2500 Hz ↔ data-model.md** — `earcon.md` line 16 matches `data-model.md` line 86.
- [x] **earcon.md mono f32 @ 22050 Hz ↔ data-model.md** — `earcon.md` line 5 matches `data-model.md` line 86.
- [x] **earcon.md duck-exempt bypass ↔ data-model.md** — `earcon.md` line 20 matches `data-model.md` line 86.
- [x] **earcon.md haptic + silent path ↔ data-model.md** — `earcon.md` lines 27-29 matches `data-model.md` line 86.
- [x] **earcon.md no audio assets ↔ data-model.md** — `earcon.md` line 16 matches `data-model.md` line 86 (procedural, no assets).

### ingest.md ↔ data-model.md

- [x] **ingest.md ACTION_OPEN_DOCUMENT ↔ data-model.md Document** — `ingest.md` line 7 matches `data-model.md` line 13 (bytes read via ContentResolver).
- [x] **ingest.md BLAKE3 fingerprint ↔ data-model.md Document.fingerprint** — `ingest.md` line 9 matches `data-model.md` line 14.
- [x] **ingest.md snapshot_path ↔ data-model.md Document.snapshot_path** — `ingest.md` line 9 matches `data-model.md` line 16.
- [x] **ingest.md uri grant ↔ data-model.md Document.uri** — `ingest.md` line 8 matches `data-model.md` line 15.
- [x] **ingest.md FormatUndeliverable ↔ data-model.md Interruption** — `ingest.md` line 17 matches `data-model.md` line 65.
- [x] **ingest.md trim on new pick/Stop/doc-end ↔ data-model.md Document** — `ingest.md` line 11 matches `data-model.md` line 16.
- [x] **ingest.md resume decision ↔ data-model.md Reading Position + Playback Session** — `ingest.md` line 10 matches `data-model.md` lines 19, 50.

### diagnostic-trace.md ↔ data-model.md

- [x] **diagnostic-trace.md SessionStart/SessionEnd ↔ data-model.md Diagnostic Record** — `diagnostic-trace.md` line 7 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md 8 failure names ↔ data-model.md Interruption** — `diagnostic-trace.md` line 13 lists all 8, matching `data-model.md` line 57.
- [x] **diagnostic-trace.md measured latencies ↔ data-model.md Diagnostic Record** — `diagnostic-trace.md` line 12 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md 30-day + 10 MB ↔ data-model.md** — `diagnostic-trace.md` line 3 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md prose-free ↔ data-model.md** — `diagnostic-trace.md` line 17 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md Trim/Gap markers ↔ data-model.md** — `diagnostic-trace.md` line 3, 19 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md detail ≤200 chars ↔ data-model.md** — `diagnostic-trace.md` line 18 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md error required on failure and refusal ↔ data-model.md** — `diagnostic-trace.md` line 13 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md adb-pull only ↔ data-model.md** — `diagnostic-trace.md` line 3 matches `data-model.md` line 90.
- [x] **diagnostic-trace.md schema delta (no new kinds) ↔ data-model.md** — `diagnostic-trace.md` line 23 matches `data-model.md` line 90 (rides as data inside existing kinds).

---

## 3. Closed Set Consistency

All closed sets must be identically represented across spec, data model, and all contracts.

### 8 Failure Reasons

- [x] **Spec defines 8** — `spec.md` line 138, 159: `FocusDenied, FocusDelayed, PermanentLoss, TransientPaused, DeviceRemoved, VoiceUnavailable, FormatUndeliverable, DecodeFailed`.
- [x] **Data model lists identical 8** — `data-model.md` line 57: same 8, same spelling.
- [x] **surface.md reason-line table covers all 8** — `surface.md` lines 12-20: 7 reasons with lines + FocusDelayed noted as no-line = all 8 accounted for.
- [x] **diagnostic-trace.md lists identical 8** — `diagnostic-trace.md` line 13: same 8, same spelling.
- [x] **Plan references 8 product reasons** — `plan.md` line 27, 41: "8 product reasons".
- [x] **No `Other`/`Unknown` variant anywhere** — All sources explicitly exclude (spec line 138, data-model line 5, plan line 27).

### 7 Rate Rungs

- [x] **Spec defines 7** — `spec.md` line 145: `0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x`.
- [x] **Data model lists identical 7** — `data-model.md` line 71: same 7, same order, same default.
- [x] **surface.md references 7 rungs** — `surface.md` line 27: "7 closed rungs 0.75x–4.0x (default 1.0x)" — abbreviated but consistent.
- [x] **Plan references ladder** — `plan.md` line 9, 27: "7-rung pitch-preserving rate ladder".

### 3 Transport States

- [x] **Spec defines 3** — `spec.md` line 158: `playing | paused | stopped`.
- [x] **Data model lists identical 3** — `data-model.md` line 47: Playing | Paused | Stopped.
- [x] **surface.md lists identical 3** — `surface.md` line 8, 47: `playing|paused|stopped`.
- [x] **Plan lists identical 3** — `plan.md` line 27: "3 transports (playing|paused|stopped — STATE)".

### 4 End Kinds

- [x] **Spec defines 4** — `spec.md` line 158: `Paused | Stopped | Completed | Interrupted`.
- [x] **Data model lists identical 4** — `data-model.md` line 47: Paused | Stopped | Completed | Interrupted.
- [x] **surface.md references EndKind** — `surface.md` line 8: "Interrupted/Completed are EndKind, never transport values".
- [x] **Plan lists 4 end-kinds** — `plan.md` line 27: "4 end-kinds".

---

## 4. Enum Value Consistency

Exact enum values must be identical (no typos, no variations) across spec, data model, and all contracts.

### Failure Reason Enum Values

- [x] **FocusDenied** — Identical across spec line 138, data-model line 57, surface.md line 14, diagnostic-trace.md line 13. ✓
- [x] **FocusDelayed** — Identical across spec line 138, data-model line 57, surface.md line 11, diagnostic-trace.md line 13. ✓
- [x] **PermanentLoss** — Identical across spec line 138, data-model line 57, surface.md line 16, diagnostic-trace.md line 13. ✓
- [x] **TransientPaused** — Identical across spec line 138, data-model line 57, surface.md line 15, diagnostic-trace.md line 13. ✓
- [x] **DeviceRemoved** — Identical across spec line 138, data-model line 57, surface.md line 17, diagnostic-trace.md line 13. ✓
- [x] **VoiceUnavailable** — Identical across spec line 138, data-model line 57, surface.md line 18, diagnostic-trace.md line 13. ✓
- [x] **FormatUndeliverable** — Identical across spec line 138, data-model line 57, surface.md line 19, ingest.md line 17, diagnostic-trace.md line 13. ✓
- [x] **DecodeFailed** — Identical across spec line 138, data-model line 57, surface.md line 20, diagnostic-trace.md line 13. ✓

### Rate Rung Values

- [x] **0.75x** — Identical across spec line 145, data-model line 71, surface.md line 27. ✓
- [x] **1.0x** — Identical across spec line 145, data-model line 71, surface.md line 27. ✓
- [x] **1.25x** — Identical across spec line 145, data-model line 71. ✓
- [x] **1.5x** — Identical across spec line 145, data-model line 71. ✓
- [x] **1.75x** — Identical across spec line 145, data-model line 71. ✓
- [x] **2.0x** — Identical across spec line 145, data-model line 71. ✓
- [x] **4.0x** — Identical across spec line 145, data-model line 71, surface.md line 27. ✓

### Transport State Values

- [x] **playing** — Identical across spec line 158, data-model line 47, surface.md line 8/47, plan line 27. ✓
- [x] **paused** — Identical across spec line 158, data-model line 47, surface.md line 8/47, plan line 27. ✓
- [x] **stopped** — Identical across spec line 158, data-model line 47, surface.md line 8/47, plan line 27. ✓

### End Kind Values

- [x] **Paused** — Identical across spec line 158, data-model line 47, surface.md line 8. ✓
- [x] **Stopped** — Identical across spec line 158, data-model line 47, surface.md line 8. ✓
- [x] **Completed** — Identical across spec line 158, data-model line 47, surface.md line 8. ✓
- [x] **Interrupted** — Identical across spec line 158, data-model line 47, surface.md line 8. ✓

---

## 5. Plan ↔ Contract Alignment

The plan's Project Structure section must reference all contracts and data model entities, and the crate layout must cover all contract surfaces.

- [x] **Plan references contracts/surface.md** — `plan.md` line 77 (surface.rs), line 99 (surface.rs/failure.rs), line 103 (test_surface_us1.rs, test_purity.rs).
- [x] **Plan references contracts/earcon.md** — `plan.md` line 77 (earcon.rs procedural PCM grammar).
- [x] **Plan references contracts/ingest.md** — `plan.md` line 86 (session ingest verb), line 111 (DocIngest.kt).
- [x] **Plan references contracts/diagnostic-trace.md** — `plan.md` line 89 (diag writer.rs/scan.rs reuse), line 25 (diag buffer-state entries).
- [x] **Plan references data-model.md** — `plan.md` lines 77-94 map every data-model entity to a crate file.
- [x] **Plan crate layout covers surface.md** — `src-tauri/src/surface.rs` + `crates/player/src/surface.rs` + `status_view.rs/html`.
- [x] **Plan crate layout covers earcon.md** — `crates/player/src/earcon.rs` + `backend.rs` short-mix bypass + `MediaMirror.kt` haptic.
- [x] **Plan crate layout covers ingest.md** — `crates/session/src/cli.rs` (ingest verb) + `DocIngest.kt` + `adapter-synthesis` (format_check).
- [x] **Plan crate layout covers diagnostic-trace.md** — `crates/diag/src/writer.rs` + `scan.rs` (reuse, no new kinds).
- [x] **Plan covers all 8 data-model entities** — Document (session/store.rs), Section (doc-structure), Reading Position (player/position.rs + rewind.rs + resume.rs), Playback Session (player/session), Interruption (player/interruption.rs + focus.rs + routing.rs), Speed Setting (player/rate.rs + session/settings), Diagnostic Record (diag), Progress (player/surface.rs), Earcon (player/earcon.rs).
- [x] **Plan Contract Structure lists all contract files** — `plan.md` lines 62-67: ingest.md, surface.md, earcon.md, diagnostic-trace.md, diagnostic-record.schema.json.
- [x] **Plan does not create new crates** — All contract surfaces live in existing crates (player, session, diag, engine-core, doc-structure, adapter-synthesis).

---

## 6. Wire Format Consistency

The surface.md payload JSON must match the data model's Progress and Session shapes exactly (field names and types).

- [x] **Payload `title` (string) ↔ Document metadata** — `surface.md` line 43 matches data-model Document (title is metadata, not first-sentence).
- [x] **Payload `section_title` (string | null) ↔ Section.title** — `surface.md` line 44 matches data-model Section.title (null before first / zero sections).
- [x] **Payload `progress.kind` ("determinate" | "indeterminate") ↔ Progress.determinacy** — `surface.md` line 45-46 matches data-model Progress determinacy field.
- [x] **Payload `progress.consumed_units` (int) ↔ Progress.consumed_units** — `surface.md` line 45 matches data-model Progress.consumed_units.
- [x] **Payload `progress.total_units` (int) ↔ Progress.total_units** — `surface.md` line 45 matches data-model Progress.total_units.
- [x] **Payload `progress.k` (int) ↔ Section.index_k** — `surface.md` line 45 maps data-model Section.index_k → wire `k`.
- [x] **Payload `progress.n` (int) ↔ Section.total_n** — `surface.md` line 45 maps data-model Section.total_n → wire `n`.
- [x] **Payload `transport` ("playing | paused | stopped") ↔ Playback Session transport** — `surface.md` line 47 matches data-model Playback Session transport.
- [x] **Payload `reason` (string | null) ↔ Interruption reason line** — `surface.md` line 48 matches data-model Interruption N:1 mapping (null = no line).
- [x] **Payload `reason_ttl_ms` (3000 | null) ↔ Interruption 3 s lifetime** — `surface.md` line 49 matches data-model Interruption reason lifetime.
- [x] **Payload `dismissal` ("non_dismissable_while_playing | dismissable") ↔ FR-014** — `surface.md` line 50 matches spec FR-014.
- [x] **Wire format exhaustive match (gate 7)** — `surface.md` line 56: exhaustive match on `progress.kind`, `transport`, `dismissal` with no `_ =>`.
- [x] **Progress wire shape closed union** — `surface.md` lines 45-46: determinate variant carries exactly {kind, consumed_units, total_units, k, n}; indeterminate variant carries exactly {kind}. No extra or missing fields.
- [x] **Transport wire values are lowercase** — `surface.md` line 47 uses lowercase `playing|paused|stopped` (wire format), data-model uses PascalCase `Playing|Paused|Stopped` (Rust variants). serde rename handles this — consistent with diagnostic-trace.md line 3 convention.

---

## 7. Cross-Cutting Invariants

- [x] **No document prose anywhere** — Enforced across surface.md (line 7), diagnostic-trace.md (line 17), data-model.md (line 90), plan (line 27).
- [x] **No rate value on any surface** — surface.md lines 7, 27, 54; data-model.md line 71; plan line 27.
- [x] **No elapsed/remaining time on lock-screen** — surface.md line 7; data-model.md Progress (no time fields).
- [x] **No voice identity on lock-screen** — surface.md line 7.
- [x] **pt-BR for all user-visible strings** — spec lines 20, 144, 152; surface.md reason lines (lines 14-20) are pt-BR per FR-011.
- [x] **Closed sets have no catch-all** — spec line 138; data-model line 5; surface.md line 56; plan line 27.
- [x] **EngineError taxonomy untouched** — data-model.md line 55 (N:1 onto untouched taxonomies); plan line 41, 90 (7-member EngineError stable, no semver bump).
- [x] **DecodeFailed rides Unreachable** — plan line 139; data-model.md line 65 (DecodeFailed in closed set).
- [x] **Focus3s / FailurePersistent aliases** — plan line 99; surface.md line 10 (reason_ttl_ms=3000 / null).
- [x] **20 s fixed durable heartbeat** — spec FR-007; data-model.md line 43; plan line 9, 97.
- [x] **Duck gain = 0.20×** — spec FR-004; data-model.md line 61; plan line 25.
- [x] **Pre-roll cap = 60 s** — data-model.md Reading Position (offset_secs, pre-roll); diagnostic-trace.md line 11 (MAX_PREROLL_SECS=60).
- [x] **Fixture voice = pt_BR-faber-medium** — plan line 9, 108; spec Assumptions line 178.
- [x] **Single authority: Rust owns playback** — surface.md line 3; plan line 46; SC-035.

---

## Notes

| Category | Pass | Fail | Total |
|---|---|---|---|
| 1. Data Model ↔ Spec Alignment | 72 | 0 | 72 |
| 2. Contract ↔ Data Model Alignment | 36 | 0 | 36 |
| 3. Closed Set Consistency | 18 | 0 | 18 |
| 4. Enum Value Consistency | 22 | 0 | 22 |
| 5. Plan ↔ Contract Alignment | 12 | 0 | 12 |
| 6. Wire Format Consistency | 14 | 0 | 14 |
| 7. Cross-Cutting Invariants | 14 | 0 | 14 |
| **Total** | **188** | **0** | **188** |

**Result**: All 154 checks pass. The data model, all four contracts, and the plan are mutually consistent and fully traceable to the spec's FR-001…FR-019. No enum divergence, no closed-set mismatch, no wire-format discrepancy, and no missing contract-to-crate mapping was found.

**Forward-looking gates** (to be verified at implementation, not document review):
- Gate 7 (no `_ =>` in `crates/*/src`) — verify during implementation.
- Gate 9 (`cargo tauri android build`) — verify during implementation.
- Schema conformance (`diagnostic-record.schema.json`) — verify during implementation.
- SC-004 (10/10 blind earcon identification) — verify during on-device trials.
- SC-008 (diag prose-free scan) — verify during on-device trials.
