# Implementation Plan: Ressoa Dogfood Build

**Branch**: `004-ressoa-dogfood-build` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-ressoa-dogfood-build/spec.md` (assessment verdict `go`, Option A only: maintainer-only sideload, single-voice, screen-off commute loop, eyes-free interruptions, content-keyed resume, on-device measurements; full release scope excluded)

## Summary

Integrate the proven pieces (001 shell + 002 `sherpa-onnx`/`faber` synthesis + 003 reference phone) into one installable dogfood app: system file-picker ingest (`.md`/`.txt` → BLAKE3-keyed transient snapshot), screen-off playback with MediaStyle lock-screen (title + section + content-offset progress only), section-boundary skip (±10 s fallback), 7-rung pitch-preserving rate ladder (in-app stepper + lock-screen rate-step custom action without value display; no rate leak — FR-011 purity holds), classified interruptions split by focus class (`MAY_DUCK`→duck to 0.20 keep playing; non-duckable `TRANSIENT`/call/alarm→pause + 3 s reason, never auto-resume), procedural Rust earcons + Kotlin haptics for blind confirmation, tiered durable position (20 s fixed heartbeat + event writes, last-spoken atomic tx), single minimal Svelte in-app screen (title/section/progress/speed/transport + Stop, never prose) in the DESIGN-SONA appearance (palette tokens canonically defined in contracts/surface.md — sibling agent edits that section only, no parallel list), and a bounded private full-trace JSONL record (30-day + 10 MB, adb-pull only). Fixed at plan time: voice `pt_BR-faber-medium`, durable interval 20 s fixed (within FR-007 15–30 s envelope), failure E4 transposition (~330 Hz square + minor-2nd dissonance, in-band 300–2500 Hz), progress = content-offset fraction. Pure offline, zero network voice code, nothing published.

## Technical Context

**Language/Version**: Rust, Edition 2024, MSRV 1.85 (constitution-fixed); Kotlin for the thin platform shell only

**Primary Dependencies**: `tauri` 2.12.1 (stable 2.x); `cpal` + AAudio via `ndk` (mono f32 playback, Rust-owned); `sherpa-onnx` 1.13.8 + Piper `pt_BR-faber-medium` (CC0, sole voice, `ort` 2.0.0-rc.13 NNAPI pre-recorded exception); `pulldown-cmark` 0.13.x (section parse) + `unicode-segmentation` (sentence guard); `redb` (bookmarks + settings); `blake3` (content identity); `serde`/`serde_json`, `thiserror` (closed sets), `tracing`, `clap` (CLI per Principle II). Kotlin side: `ACTION_OPEN_DOCUMENT` picker, `Service` + `MediaSession` + `MediaStyle` mirror, `AudioFocusRequest` (`AUDIOFOCUS_GAIN`), `Vibrator`/`VibrationEffect` haptics — mirror only, zero computation (SC-035)

**Storage**: `redb` — `bookmarks` (blake3 → `{offset_secs, first_yield_ms, end_kind}` atomic single-row tx; `stopped|completed` delete) + `settings.last_rate` (default `1.0x`); transient snapshots `filesDir/snapshots/<hex>.{md,txt}` (synthesis reads only this); diagnostic JSONL append-only in app-private `/data/data/<pkg>/files/`, 30-day + 10 MB oldest-first trim with `Trim`/`Gap` markers, best-effort never blocks playback

**Testing**: `cargo nextest run --workspace` (host: rewind/snap math, ladder, BLAKE3, content-offset progress, earcon-duration/band asserts, trim bounds, closed-set exhaustiveness, schema conformance; property tests via workspace `proptest` dev-dependency, latest stable at implementation time (re-verified on crates.io per tasks.md T002c, never hardcoded here), crates.io-only, locked, `deny`+`audit` gates); new public APIs require rustdoc (`missing_docs` deny, gate 3); `scripts/expected-tests.txt` lists the expected suite inventory and gate 2 fails closed when it diverges; `check-manifest.sh` (SC-006) + `check-shell-purity.sh` (SC-035); committed Portuguese fixture (~2 h at 1.0x, headings + numbers/currency); on-device 10/10 trials per SC-001…SC-010 on the reference phone only

**Target Platform**: Android 10 (API 29) min through API 34 FGS-start rules; `aarch64-linux-android`; reference phone Redmi 2201117TY (Note 11, Snapdragon 680, Android 13/API 33) normative for all time bounds; headless Linux CI + gate 9 (`cargo tauri android build --target aarch64`); sideload install only

**Project Type**: Mobile app (Tauri) on a Cargo-workspace modular monolith of focused library crates (`player`, `session`, `diag`, `engine-core`, `doc-structure`, `adapter-synthesis`), each CLI-drivable per Principles I–II; Tauri app one consumer

**Performance Goals**: First audible speech ≤3 s after Play, 10/10 cold-start (SC-002); `MAY_DUCK` nav transient → duck to 0.20 + restore without losing place (continuous, no reason line); non-duckable `TRANSIENT`/call/alarm/permanent loss → pause immediately (+ 3 s reason, never auto-resume); device removal → pause ≤1 s; interruption resume replay ≤30 s at sentence-start + 5–10 s cue; kill/power-loss replay ≤60 s (20 s fixed durable heartbeat worst-case). Declared implementation target, informative and non-gating: routing decision 250 ms within the normative ≤1000 ms FR-006 bound (mirrored by tasks.md T024). Informative targets (not acceptance-gated, no FR/SC parent): rate change effective ≤2 s (SC-007 gates persistence + unpitched only); pre-roll buffer cap ≤60 s (`MAX_PREROLL_SECS` — the single `player::position::PREROLL_CAP_SECS`; the bound is the **player-side resident-queue** bound in `crates/player/src/backend.rs` (`ResidentQueue`), not an engine-side one: the resident audio is the pre-roll window ahead of the play head, the audio beyond the window is DROPPED and the audio at the play head never is (the reading stays audibly contiguous from the play head), the drop moves no position (the audible clock stays the only advance path, FR-002), and the resident depth is observable through `CpalBackend::buffer_state()` — the source of the diag buffer-state entries per `contracts/diagnostic-trace.md`; not a latency bound)

**Constraints**: Pure offline (zero network voice code ships); exactly one CC0 voice; closed sets everywhere (8 product reasons, 7 rate rungs, 3 transports (playing|paused|stopped — STATE; The Stop CONTROL (the transport Stop that clears the stored position) is in-app only; the media lock-screen surface has no Stop CONTROL; the resident-paused surface (`contracts/surface.md` §Resident-paused notification surface) carries the resident-stop, which ends residency and PRESERVES the stored position (FR-002)), 4 end-kinds; no `Other`/`Unknown`, no `_ =>`, gate 7); Rust rigor lints (`unsafe` deny, pedantic clippy deny, no unwrap/expect/println); mono-f32-22050 handoff (mismatch = named `EngineError`, never silent); lock-screen + in-app never show prose/times/rate/voice (rate-step is a lock-screen custom action without value display — stepping reachable without unlock, no numeric leak); rate memory app-wide via `settings.last_rate`; diag never transmitted, no in-app surface, zero prose, `error` on failure *and* refusal; dismissal refused while playing; Ressoa-external / mdreader-technical naming boundary

**Scale/Scope**: Single-document sessions (no queue/playlist/library); one stored position per content hash; one resident component + one notification; fixture ~2 h at 1.0x; oversize-document byte cap `MAX_DOC_BYTES` = fixture_bytes × 1.5 (fixed by T001b after fixture measurement, FormatUndeliverable refusal per FR-018); 30-min acceptance session valid at every speed incl. 4.0x; diag ~100k entries max; maintainer-only distribution (no store/listing/support)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I (Library-First)**: PASS — no new crates; `earcon.rs` + ingest/persistence wiring live in existing `player`/`session`/`src-tauri`; Tauri one consumer; every behavior CLI-drivable.
- **II (Text I/O)**: PASS — `player earcon …`, `session ingest …`, existing `get/put/clear/rate` + `diag scan` CLIs with `--format json`; IPC calls the same functions; 2026-10-13 U2 decision: player progress verb required, no derived-view-only exemption.
- **III (Test-First)**: PASS — host-testable (progress fraction, snap math, ladder, fingerprint, earcon band/duration, trim, exhaustiveness) red-green; on-device 10/10 trials for the rest; no implementation before failing tests in tasks phase.
- **IV (Stable + supply-chain)**: PASS — no new runtime dependencies (procedural earcons = no assets; `Vibrator` = platform API); property tests reuse the existing workspace `proptest` dev-dependency at latest stable at implementation time (re-verified on crates.io per tasks.md T002c, never hardcoded here) (crates.io-only, `Cargo.lock` committed, `cargo deny` + `cargo audit` gates); `faber` + `ort` RC covered by existing registry/exception; `cargo deny` + `cargo audit` + `--locked` gates unchanged.
- **V (Deterministic gates)**: PASS — manifest + shell-purity checks retained; missing input fails, never skips; `verify.sh` ordering kept.
- **VI (Engine-neutral)**: PASS — no new engine/tier; single `OfflineDefault` selectable; rate stretch stays engine-contract op; capability declaration untouched.
- **VII (Closed sets)**: PASS — 8 product reasons map N:1 onto untouched closed taxonomies (wire `EngineError` stays 7 members, no semver bump); ladder/session/progress/dismissal/earcon enums closed; gate 7 holds.
- **VIII (Driving-first)**: PASS — the feature IS this principle: screen-off commute loop, lock-screen everything including rate-step without value display (custom action, no numeric leak — FR-011 purity holds; per constitution Driving-Safety, rate changes are reachable without unlocking), earcon + haptic blind confirmation (10/10 per type, sounds-on and -off), 3 s bounded offers, no look-demanding interaction.
- **IX (Offline-first)**: PASS — pure offline; snapshot synthesis survives revocation; no network to start/continue; paused-idle synthesises nothing.
- **X (Free-tier budgeted)**: PASS — no paid tier; idle pause bills nothing; byte-budget chunking unchanged (adapter-side ≤5000 B).
- **XI (Sovereignty)**: PASS — no secrets; no second voice/cloud/credentials/consent surface; diag device-local private, never transmitted, adb-pull only.
- **Tauri Platform Contract**: PASS — Kotlin confined to picker intent + Service/MediaSession/notification/focus-request + haptics (forward-only mirror driven by the Rust reason — `Vibrator`/`VibrationEffect` fire patterns selected by Rust, zero computation — Kotlin renders pattern-id verbatim, selects nothing (SC-035 holds)); Rust single playback authority; required permissions `FOREGROUND_SERVICE*` + `VIBRATE` + `POST_NOTIFICATIONS`; manual `<service mediaPlayback>` manifest edit re-verified every Tauri upgrade (re-verify covers the `VIBRATE` + `POST_NOTIFICATIONS` permissions too).
- **Audio Pipeline Contracts**: PASS — sentence/node granularity as designed precision; content-offset progress uses parsed units, audio-duration estimates never feed it; degradation observable.
- **Driving-Safety / Licence**: PASS — no new licences (CC0 `faber` already admitted; no `soundtouch` needed — engine stretches); transient duck ~20% per ADR-001 deviation stands.

No violations. No Complexity Tracking entry required.

## Project Structure

### Documentation (this feature)

```text
specs/004-ressoa-dogfood-build/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── ingest.md
│   ├── surface.md
│   ├── earcon.md
│   ├── diagnostic-trace.md
│   ├── resume-algorithm.md
│   └── diagnostic-record.schema.json
├── checklists/          # Reviewer-owned requirements-quality reviews
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── player/src/         # ADD: earcon.rs (procedural PCM grammar + duck-exempt bypass descriptor)
│                       # MODIFY: backend.rs (short-mix bypass), surface.rs (content-offset
│                       #   Progress{consumed,total,k,n} + transientReason wiring), position.rs /
│                       #   rewind.rs / resume.rs / recovery.rs (20 s heartbeat hookup points),
│                       #   focus.rs / routing.rs (reuse: 8-reason mapping rides their
│                       #   untouched closed taxonomies), interruption.rs (NEW closed 8-reason
│                       #   surface/diag enum mapping N:1 onto focus/routing/engine taxonomies),
│                       #   skip.rs (section ±10 s
│                       #   fallback), rate.rs (reuse ladder), cli.rs (earcon + progress verbs; progress verb per tasks.md T030b (2026-10-13 remediation U2: no exemption taken — progress CLI required for Principle II CLI-drivability))
├── session/src/        # MODIFY: store.rs / fingerprint.rs / offer.rs reuse; cli.rs gains
│                       #   `ingest` verb (hash + lookup → resume decision JSON)
├── diag/src/           # REUSE: writer.rs / scan.rs / cli.rs as-is (full-trace kinds are
│                       #   data, not schema change — nine EventKind values unchanged, per-section/per-rate/buffer/latency ride as detail data); new measured-latency detail keys only
├── engine-core/src/    # UNTOUCHED (7-member EngineError stable; DecodeFailed rides Unreachable)
├── doc-structure/src/  # REUSE: headings_before / nearest-boundary / reanchor for skip +
│                       #   progress totals; TXT empty-section path
└── adapter-synthesis/  # REUSE: faber bundling + chunk::split_document + format_check +
                        #   failure mapping as-is; no new voice/tier

src-tauri/src/          # MODIFY: lib.rs (open_document(bytes) reusing play path + 20 s
│                       #   fixed heartbeat TIMER WIRING ONLY — cadence LOGIC lives in the
│                       #   `player`/`session` library (CLI-drivable); lib.rs only ticks and
│                       #   calls `BookmarkStore::put` + event-durable puts), surface.rs / failure.rs (≤8-word
│                       #   8-reason lines per contracts/surface.md reason-line table; Focus3s = reason_ttl_ms=3000, FailurePersistent = reason_ttl_ms=null (aliases retained here only)), status_view.rs
│                       #   + status_view.html (single minimal screen markup — .html extension by existing convention, Svelte 5 Runes frontend per Technical Context; section + progress
│                       #   + stepper + Stop; owns no state, renders surface.rs/failure.rs strings only)
src-tauri/tests/        # NEW: host contract/purity/noscope suites — test_surface_us1.rs,
│                       #   test_purity.rs (no prose/time/rate/voice leak), test_noscope.rs
│                       #   test_progress_cli.rs (T040b progress CLI contract)
└── gen/android/app/src/main/
    ├── AndroidManifest.xml  # MODIFY-if-missing (requires service mediaPlayback + FOREGROUND_SERVICE* + VIBRATE + POST_NOTIFICATIONS; if VIBRATE/POST_NOTIFICATIONS absent, edit manifest directly and re-verify on every Tauri upgrade per Tauri Platform Contract)
    └── assets/voices/pt_BR-faber-medium/  # SOLE voice (no addition)

android-shell/src/main/kotlin/app/mdreader/
│                       # ADD: DocIngest.kt picker intent (ACTION_OPEN_DOCUMENT + persistable grant +
│                       #   ContentResolver → filesDir/snapshots/<hex>.{md,txt} write); MODIFY: PlaybackService.kt (existing foreground residency
│                       #   holder; assert screen-off continuity; dismissal refused
│                       #   while playing); MODIFY: MediaMirror.kt
│                       #   (haptic patterns in publish()/buildSurface() (render-only) + VIBRATE permission)
│                       #   — forward-only, zero playback logic (SC-035 holds)

tests/device/           # ADD: 004 trial procedures (commute/blind/resume/purity/rate/record
│                       #   per quickstart §3); documentation only, no cargo target
└── fixtures/
    └── fixture-pt-2h.md  # NEW: committed Portuguese Markdown (~2 h at 1.0x, headings +
                          #   numbers/currency) for SC-001 at every speed incl. 4.0x

scripts/                # REUSE: verify.sh (gates 1–9) + check-manifest.sh + check-shell-purity.sh
│                       #   + expected-tests.txt (expected suite inventory; gate 2 fails closed on divergence)
```

**Structure Decision**: Cargo-workspace modular monolith (constitution-fixed). Zero new crates: 004 is an integration build that wires the 001 shell, 002 synthesis, and 003 device record together plus four narrow deltas (picker→snapshot ingest, content-offset progress, procedural earcons + haptics, single minimal in-app screen) inside the crates that own the behavior. Kotlin stays a forward-only mirror; Tauri stays a delegating consumer; every new behavior is CLI-drivable and host-testable except the on-device 10/10 trials.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — table intentionally empty.

## Post-Design Re-check (Phase 1 complete)

- All Phase 1 artifacts (`data-model.md`, `contracts/ingest.md`, `contracts/surface.md`, `contracts/earcon.md`, `contracts/diagnostic-trace.md` + `contracts/resume-algorithm.md` + `diagnostic-record.schema.json`, `quickstart.md`) resolve into the crates above without moving logic into the Kotlin shell or the Tauri app layer; `engine-core` untouched (no semver bump); no new dependency, licence, or tier.
- Voice fixed `pt_BR-faber-medium` (CC0, already bundled); durable interval fixed 20 s (worst loss inside 30 s cap); failure E4 transposition (~330 Hz square + minor-2nd dissonance) keeps FR-016 300–2500 Hz (abstract-dissonant, in-band); progress uses parsed units (audio-duration never feeds it); `DecodeFailed` rides `Unreachable` on the wire with `decode_failed` in diag (revisit only if trials show confusion).
- Gates: no ERROR, one ownership clarification (2026-10-13 remediation I4): constitution gate 8 (byte-budget + SSML + sentence-alignment) was DEFERRED for 001-shell per Exceptions Log 8 (owner=synthesis, SC-044). 004 integrates 002 faber synthesis, so T048 MUST attempt gate-8-full discharge with evidence; only if synthesis SC-044 remains open per specs/002 status does T048 record `gate-8-full: DEFERRED (owner=synthesis, reason=SC-044-open)` — never claim constitution gate 8 PASS without the full discharge. Proceed to /speckit.tasks (implement T048 as written 2026-10-13).
