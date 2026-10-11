# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

FR-024 — synthesis adapter for `001-android-media-shell`: integrate `sherpa-onnx` + Piper pt-BR (`faber`) as `offline_default` engine; deliver mono f32 @ 22050 Hz; adapter splits Markdown into bounded chunks (≤5000 bytes, paragraph/heading boundary); declares `(SsmlLevel::None, AlignmentGranularity::None)` against existing `engine-core` contract shapes (`format.rs`, `closed.rs`); preserves `SC-051` invariant; defers `SC-044`, consent acquisition, and active fallback switch to v2. Option A (no cloud, no second engine, no SSML). Branch `002-fr-024-synthesis-adapter`.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust, Edition 2024, MSRV 1.85 (constitution Technology Stack; `AGENTS.md`).

**Primary Dependencies**: `sherpa-onnx` 1.13.8 + Piper pt-BR (`faber` CC0) — `offline_default` (constitution §TTS Engine Registry); `ort` 2.0.0-rc.13 (NNAPI, exception 4); `pulldown-cmark` 0.13.x (not yet workspace dependency — integration-stage confirmation required); `redb` + `blake3` unchanged.

**Storage**: `redb` bookmark store (`session`); adapter does not alter position (`Assumption 5`).

**Testing**: `cargo nextest` workspace; adapter fixtures on `standin_a`/`standin_b`; byte-budget test under 5000 bytes / chunk; `SC-009` benchmark on reference device (`mid-range arm64`, 4× A78, 6 GB RAM).

**Target Platform**: Android (Tauri 2.12.1 pinned — not 3.0.0-alpha); Kotlin `Service` (manifest `foregroundServiceType="mediaPlayback"` + `FOREGROUND_SERVICE*`) + `MediaSession` mirror; `register_android_plugin` bridge; no `registerService` (does not exist in Tauri 2 — `AGENTS.md`).

**Project Type**: Cargo workspace modular monolith (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`); adapter at edge (design phase — crate not yet created).

**Performance Goals**: First audible chunk ≤3 s (`SC-009` hard); pre-roll ≤60 s; chunk ≤5000 bytes (byte-measured, `constitution.md:353`); rewind ≤5% / 30 s cap (`FR-015` / `SC-027`); mono f32 @ 22050 Hz (`FR-024` / `DeclaredFormat`).

**Constraints**: Closed-set `EngineError` 7 members (Principle VII); adapter owns chunk split (`FR-024a`); format = mono f32 @ 22050 Hz or named failure (`FR-024b` / `SC-035`); no second authority over playback (`constitution.md:82-86`); byte budget adapter-side; `pulldown-cmark` reachability deferred to integration; `T016` trait not built — declare `(none,none)` against existing `format.rs`/`closed.rs` shapes (`Assumption 3` corrected).

**Scale/Scope**: Single adapter crate; one registered tier (`OfflineDefault`); no cloud / consent / SSML / secondary engine / diagnostic viewer / picker — bounded by `spec.md` Non-Goals (`spec.md:137`).

**Unknowns (all resolved — see `research.md`)**: chunk/budget (adapter split); Piper boundaries (none / deferred); `SC-051` (invariant preserved — active switch deferred until second engine); failure names + condition table (7 members mapped at `spec.md:85`); `SC-044` deferred (`DEFERRED`, `test_double` unreachable per `001/spec.md:89`); resume (`None` / paragraph-heading); declaration (`(none,none)`); bad Markdown layer (document/parse, not `EngineError`; `diag` unchanged at v1); `pulldown-cmark` readiness (not in workspace — design assumes, integration confirms). No unconfirmed dependency blocks Phase 1.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status (pre-Phase-0)**: PASS — all 9 gates evaluated; no unjustified violations; only pre-recorded exceptions (4: `ort` RC; 8: gate split) apply. See full gate evaluation above in Technical Context (`Constitution Check` subsection). No new deviation proposed; adapter does not weaken any principle (`T016` trait not built is a design boundary, not a violation — declaration unconditional against existing shapes, `Assumption 3` corrected).

**Post-Phase-1 re-check criteria** (to confirm after design artifacts):
- Adapter contract shapes match `crates/engine-core/src/format.rs` + `closed.rs` (verified in `research.md` §2).
- `SC-051` mechanism passes with adapter registered in `offline_default` (`standin_a`/`standin_b` not selectable per `001/spec.md:89` / `closed.rs:398-406`).
- No new `AdapterFailed` variant; bad Markdown remains document/parse layer (`spec.md:85` / C4-A); `diag` unchanged.
- `CLARIFICATION_BATCH.md` mapping (`C1–C4`) preserved at `spec.md:85` (condition→variant table); `spec.md:138` fixed from stale `133`; `SC-044` stays `DEFERRED` (`constitution.md:643`).
- Byte-budget test can be written against adapter split logic (≤5000 bytes / chunk — `constitution.md:353` / gate 8 shell).
- `cargo fmt`, `cargo clippy -D warnings`, `cargo deny` pass with no new suppressions; `[lints] workspace = true` in adapter crate.

**No complexity-tracking violations** ( Complexity Tracking section left empty — no unjustified deviations from constitution required; adapter fits existing workspace and registry without new crates, new tiers, or new authority sources).

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root — adapter crate design only)

```text
crates/
├── engine-core/           # existing (closed, format, standin_a, standin_b; no trait_.rs)
├── adapter-synthesis/     # NEW — design phase; crate structure below
│   ├── src/
│   │   ├── lib.rs         # declared_capabilities(), adapter registration
│   │   ├── chunk.rs       # pulldown-cmark split (paragraph/heading; ≤5000 bytes)
│   │   ├── synth.rs       # sherpa-onnx + Piper pt-BR pipeline wrapper
│   │   ├── format_check.rs # mono f32 @ 22050 Hz verification against DeclaredFormat
│   │   ├── failure.rs     # condition → EngineError 7-member mapping (spec.md:85)
│   │   └── capabilities.rs # (SsmlLevel::None, AlignmentGranularity::None) declaration
│   └── Cargo.toml         # sherpa-onnx, piper bindings, ort (RC exception), pulldown-cmark (integration)
├── player/                # existing (position / buffering; converts adapter output to device)
├── session/               # existing (redb bookmark; blake3 fingerprint; unchanged by adapter)
├── diag/                  # existing (JSONL; no parse-kind expansion at v1; unchanged)
└── ... (other workspace members)

android-shell/
└── src/main/java/.../Service.kt  # foregroundServiceType="mediaPlayback"; MediaSession mirror

tests/
└── adapter-synthesis/     # byte-budget test; format-check test; SC-051 mechanism drill
```

**Structure Decision**: One new edge crate (`adapter-synthesis`) in existing Cargo workspace; no new workspace; no new authority source (adapter registers `OfflineDefault`; `player` remains sole playback authority per `constitution.md:82-86`); Kotlin `Service` stays thin mirror per Tauri Platform Contract; `pulldown-cmark` introduction deferred to `003-contract-edit` integration stage (design assumes; build confirms).

## Complexity Tracking

> **No unjustified deviations registered.** All design choices sit within existing workspace (one new edge crate, no new authority source, no new tier, no new principle). The only structural additions are: (a) adapter crate at edge (`crates/adapter-synthesis/` — one crate, not four); (b) `pulldown-cmark` as integration dependency (deferred to `003-contract-edit`, not blocking design); (c) `declared_capabilities()` declaration against existing `format.rs`/`closed.rs` (no phantom trait file). These are not violations — they are feature-level scope, documented in `constitution.md` §TTS Engine Registry and §Audio Pipeline Contracts.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| (none) | — | — |
