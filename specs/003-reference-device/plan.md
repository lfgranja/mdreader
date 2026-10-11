# Implementation Plan: Reference Device Realignment

**Branch**: `003-reference-device` | **Date**: 2026-10-11 | **Spec**: `specs/003-reference-device/spec.md`

**Input**: Feature specification from `specs/003-reference-device/spec.md`

## Summary

Replace the hypothetical mid-range reference-device class (4× Cortex-A78, 6 GB) in `specs/001-android-media-shell/plan.md` with the maintainer's possessed phone (Redmi 2201117TY, Snapdragon 680 `bengal`, 4× A73 + 4× A53, ~5.8 GB RAM, API 33), reconcile the two device-coupled numeric bounds (10/10-trial latency, "Audible audio after Play") from ≤3 s to ≤10 s cold-start per maintainer choice Q1:B, and close the `002/spec.md:117` "Divergence noted" annotation with a follow-up note. Docs-only amendment; no code, no new dependencies, no constitution change. Precedent: 002 amendment `d5d6b45`.

## Technical Context

**Language/Version**: N/A — Markdown documentation amendment; no code written or compiled.

**Primary Dependencies**: None. No crate added, removed, or bumped.

**Storage**: N/A. Past verdicts (e.g. SC-009 9 s × 5 FAIL-under-≤3 s in `SC-009-GATE-RECORD.md`) are preserved as history and re-judged only by explicit re-judgment statement, never by editing old measurements (spec Edge Cases).

**Testing**: Doc-consistency checks, all deterministic and host-runnable without a device:
- `grep` for exactly one reference-device definition across `001/plan.md` and `002/spec.md`, both naming Redmi 2201117TY (SC-001).
- `grep` that no `A78` reference-device text remains in `001/plan.md` (SC-003).
- `grep` that the reconciled bound reads ≤10 s cold-start in both touched paragraphs (SC-002 input).
- Re-judgment verdict of the 9 s × 5 measurement recorded un-asterisked (SC-002).
- No `cargo test` / `verify.sh` gate changes: docs edits cannot break gates 1–9; `tasks.md` will carry a gate-sanity check only.

**Target Platform**: N/A for the edit itself. The *named* reference for all on-device verdicts becomes Redmi 2201117TY (API 33, `aarch64`); the build target (`aarch64-linux-android`, API 29–34) is unchanged.

**Project Type**: Documentation amendment under spec-driven governance (same shape as 002 edit 5 / `d5d6b45`).

**Performance Goals**: The reconciled bound being written in: ≤10 s cold-start press→first-audible at transducer on the reference phone (consistent with amended 002 SC-009). Pre-roll becomes a future improvement, not a gate obligation. Device-independent bounds (headphone-pull ≤1 s, skip ±5 s, rate-change ≤2 s) are out of scope and untouched.

**Constraints**:
- Append-only history: amendment recorded with maintainer acceptance, date, rationale; prior text superseded by note, not rewritten (FR-003, FR-004).
- Amendment discipline mirrors the 002 change (`d5d6b45`): same acceptance shape, same "Divergence" closure style.
- Constitution needs no amendment (it fixes no device class and no numeric latency bound).

**Scale/Scope**: 3 edits across 2 files — `001/plan.md` Reference-device paragraph (FR-001), `001/plan.md` two device-coupled bound sites (FR-002), `002/spec.md:117` divergence-closure note (FR-004). Plus the amendment record itself (FR-003).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I (Library-First)**: PASS — no crate created, removed, or repurposed; no organiser-only artifact.
- **II (CLI-First)**: N/A — no capability added; no IPC/CLI surface touched.
- **III (Test-First)**: PASS — verification is the doc-consistency checks + `quickstart.md` scenarios (SC-001/002/003 each have a runnable check); docs amendments carry checks, not unit tests, per precedent `d5d6b45`.
- **IV (Latest Stable / Supply-Chain)**: PASS — zero dependency changes; gates 5–6 unaffected.
- **V (Deterministic Gates)**: PASS — all checks are `grep`/read assertions with exact expected strings; no judgment input.
- **VI (Engine-Neutral Core)**: PASS — untouched; no adapter or trait surface changed.
- **VII (Closed Sets)**: PASS — no enum touched; no `_ =>` / `Other` introduced.
- **VIII (Driving-First)**: PASS (supports) — meaningful on-device gates serve the driving context; no interaction surface changed.
- **IX (Offline-First)**: PASS — untouched; offline path unchanged.
- **X (Free-Tier Budgeted)**: PASS — untouched; no quota-affecting change.
- **XI (Credential/Content Sovereignty)**: PASS — no egress, no secret, no telemetry touched.

No violations. No Complexity Tracking entries required.

*Post-design re-check (Phase 1 complete): no design artifact introduces code, deps, enums, or surfaces — all gates above hold as stated.*

## Project Structure

### Documentation (this feature)

```text
specs/003-reference-device/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── amendment-record.md  # Exact edit sites + preservation rules
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

Docs-only amendment. No source tree changes. The touched files live outside this feature directory and are named here as edit targets, not as structure:

```text
specs/001-android-media-shell/plan.md   # FR-001 (Reference-device paragraph) + FR-002 (2 bound sites)
specs/002-fr-024-synthesis-adapter/spec.md  # FR-004 (divergence-closure note at :117)
specs/002-fr-024-synthesis-adapter/SC-009-GATE-RECORD.md  # read-only history source for SC-002 re-judgment
```

**Structure Decision**: No new source layout. This feature delivers edited paragraphs plus its own `specs/003-reference-device/` design artifacts; implementation (`/speckit.tasks` + `/speckit.implement`) performs the text edits and records the amendment commit.

## Complexity Tracking

> No constitution violations to justify — section intentionally empty.
