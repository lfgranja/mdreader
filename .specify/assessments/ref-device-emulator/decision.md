# Decision: Reference-Device Verification — Emulador Structural Layer

- **Verdict**: `go` (confirmed 2026-10-29 — user resolved all 6 blocking gaps G1–G6: AVD / sub-layer VERIF-EMULATOR / independent SC-044 / doc VERIF-EMULATOR.md / isolated AVD / assessment kept; Option A small confirmed; physical `gate-8-full` preserved; `SC-044` deferred; source unedited; `cargo check` deferred to T002 per `tasks.md` protocol).
## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | adequate | `003-reference-device` has fully-specified pipeline (`spec.md`/`plan.md`/`tasks.md`/`checklists/`/`contracts/`/`data-model.md`) but zero verified implementation/convergence; `verify.sh` not confirmed on current tree (`bg_14` interrupted); `gate-8-full` deferred; only historical `9501bba` green. Problem clearly stated in `problem.md`: structural verification gap + physical-only bottleneck. |
| Evidence strength | medium | High on emulator capability/limitations (`developer.android.com` + StackOverflow + spec-kit workflow); medium on whether this accelerates 003 converge (no external demand, no prior attempt, `cleanroom-core` empty); low on divergence/maintenance risk (assumption, not measured). 6 open questions from `research.md` remain. Not weak (sources verified) not strong (no build/test proof of layer). |
| Value vs. inaction | adequate | Doing nothing (status quo) means `003` stays unverified, `001` `gate-8-full` blocked, `004` remains only historical green; documented `VERIF-EMULATOR` structural layer could accelerate 003 without claiming false equivalence — value is moderate, not transformative. Cost of layer is small (docs/script only, no crate, no engine change). |
| Feasibility / appetite | adequate | Option A (small) exists: Android SDK AVD free/local, aligns with manifest rules (`SC-035`), respects `clippy`/`expect`/closed-set rules, uses existing patterns. No new dependencies. Appetite matches situation (51 dirty files, interrupted build, active 005 branch — large build not feasible now). |
| Strategic fit | adequate | Aligns with `speckit.assess` pipeline; supports constitution (gates, supply-chain, Tauri 2.12.1); does not conflict with `005-adopt-design-sona` or `002-synthesis-adapter`; addresses a verified gap in `003`. No strategic contradiction found. |
| Risk posture | adequate | Key risks identified in `problem.md`/`concept.md`: false-equivalence claim (mitigated by explicit `VERIF-EMULATOR` label + non-goals); maintenance divergence from `verify.sh`; cleanroom-core empty; 6 gaps unresolved. Not unmitigated — boundary is explicit. Not strong because gaps aren't resolved. |

## Verdict & Rationale

**Verdict: `needs-clarification`** — not `go`, not `kill`.

The evidence is real (emulator works structurally; physical still required for gate-8-full; 5 evidence-against points documented in `research.md`). The concept is shaped (Option A with strict non-goals; Option B status-quo fallback; Option C too large). The problem is clearly defined (`problem.md`). However, a `go` requires **both** evidence adequate+ **and** a credible path with unknowns resolved. Here, 6 load-bearing unknowns from `research.md` are still open:

1. Which exact emulator (AVD vs Genymotion vs BlueStack vs Waydroid)?
2. Sub-layer (`VERIF-EMULATOR`) or replacement for `gate-8-full`? (Critical — determines boundary and risk.)
3. Interaction with `SC-044` (consent/cloud TTS, deferred in `002`)?
4. Success criteria / attestation format (`verify-emulator.sh`? separate doc? benchmark `SC-009` impossible on AVD?)
5. CI / hardware / environment constraints (memory, GPU, disk, time) for AVD in current dirty/branch state?
6. Assessment (`ref-device-emulator`) vs. spec-note inside `003-spec`?

These are not trivial; they determine whether Option A is a 2-day documentation task or a multi-week integration with false-equivalence risk. The honest call is to resolve them before `go` — returning to `research` (clarify gaps) and then possibly `define` (if gaps change problem) or directly to `shape`/`specify` if the answers are straightforward (e.g., "AVD, structural only, document in 003/plan.md, no CI").

**Not `kill`:** The problem is real (verified structural gap), the evidence supports structural feasibility, the concept is bounded, and the cost of Option A is small. Killing would discard verified information; the right action is to resolve the gaps, then proceed.

## If needs-clarification — Blocking Questions

- [NEEDS CLARIFICATION: Which emulator?] Android SDK AVD is the default (public docs, SDK, free), but must be confirmed by verification maintainer / CI environment.
- [NEEDS CLARIFICATION: Sub-layer or replacement?] Must confirm `VERIF-EMULATOR` = structural-only; `gate-8-full` physical remains mandatory; no claim of equivalence.
- [NEEDS CLARIFICATION: SC-044 interaction?] Does `002` consent acquisition affect `003` verification timing, or is `003` independent of `SC-044`? (`problem.md` notes 002 deferred; no direct dependency, but 001 gate-8-full depends on 002.)
- [NEEDS CLARIFICATION: Success criteria / attestation?] `VERIF-EMULATOR` needs explicit definition: `specs/003-reference-device/verification/emulator.md`? `scripts/verify-emulator.sh`? No `SC-009` benchmark claimed.
- [NEEDS CLARIFICATION: CI / hardware / environment?] Can the current dirty branch (`005-adopt-design-sona`, 51 dirty, interrupted `cargo check`) support an AVD run? What resources available?
- [NEEDS CLARIFICATION: Assessment identity?] Keep `ref-device-emulator` as separate assessment or demote to `003` note? Affects tracking and documentation path.

## Revisit Stage

Revisit `/speckit.assess.research slug=ref-device-emulator` (and possibly `/speckit.assess.define` if answers change the problem's scope, or `/speckit.assess.shape` if answers allow shaping). Once the 6 gaps are answered, the verdict can be upgraded to `go` (if answers confirm Option A with strict boundary) or downgraded to `kill` (if answers reveal hidden costs — e.g., CI unavailable, false-equivalence unavoidable, or external QA rejects emulator layer). Until then: hold; do not proceed to `specify`.

## If go — Handoff to `/speckit.specify` (for after clarification)

- **Problem**: `003-reference-device` verification blocked by physical-only bottleneck; structural verification path needed but must not claim `gate-8-full` equivalence.
- **Chosen approach**: Option A — `VERIF-EMULATOR` structural layer (small), with boundary documented.
- **In scope**: Documentation/script for structural verification of manifest/service/notification/focus/diag/resume/rate; `VERIF-EMULATOR` label; physical device remains `gate-8-full`.
- **Out of scope**: Audio-output verification, `SC-009` benchmark, Bluetooth/car routing, 10/10 on-device, `SC-044` change, new crate, engine change.
- **Success metrics**: `003-reference-device` has documented structural-verification path; `gate-8-full` explicitly deferred to physical; no false-equivalence claim.
- **Carried-forward open questions**: Same 6 gaps — must be resolved before implementation begins; do not start `speckit.specify` until clarifications complete.

- **Verdict upgrade (2026-10-29)**: `needs-clarification` → `go` — user resolved all 6 blocking gaps (G1 AVD / G2 sub-layer VERIF-EMULATOR / G3 independent SC-044 / G4 doc VERIF-EMULATOR.md / G5 isolated AVD / G6 assessment kept as cross-spec layer). Option A (small) confirmed; no source edited; `cargo check` still deferred to T002; `gate-8-full` physical preserved; `SC-044` deferred. Handoff to `/speckit.specify` may proceed with problem statement, Option A boundary, success metrics, and carried open questions (none blocking if G1-G6 remain confirmed). If G1-G6 change, revert to `needs-clarification`.
