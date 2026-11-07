# Specification Quality Checklist: TTFA synthesis pipeline scheduling

**Purpose**: Validate specification completeness and quality before planning/implementation  
**Created**: 2026-11-07  
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] Focused on user/system value (fast first audio, continuous playback, preserved invariants)
- [x] Grounded in code/device/API evidence (no speculative mechanism claims)
- [x] All mandatory sections completed
- [x] Scope bounded (Option A only; Option B excluded with evidence)

## Requirement Completeness

- [x] No unresolved blocking ambiguity for planning Option A; measurement follow-ups are explicit validation tasks
- [x] Requirements testable (unit/contract/integration/device smoke criteria)
- [x] Success criteria measurable where feasible (TTFA baseline/after, underrun absence/regression, test pass)
- [x] Acceptance scenarios cover cold start, resume offset, cancelation, error/FR-001/005, chapter/directory impact
- [x] Edge cases identified (high offset, new document during synthesis, queue full, poisoned lock, chapter advance)
- [x] Dependencies/assumptions identified (sherpa-onnx 1.13.8, cpal 60s queue, Tauri/Tokio threading, device 2201117TY)

## Feature Readiness

- [x] Functional requirements map to trace evidence and assess decision
- [x] Scenarios preserve pre-roll/fingerprint/resume/SourceRange/medição
- [x] Risks/rollback constraints stated (no silent Playing, cancel tokens, bounded queue)

## Notes

- TTFA baselines sampled from device diagnostic record 2026-11-07: 1659–17309 ms common, 45878–120422 ms extreme (offset/high-position cases). Phase-level instrumentation is a follow-up validation task.

## Implementation/Convergence Status (2026-11-07, honest)

- [~] **FR-TTFA/RESUME/CANCEL/SAFE host evidence partial**: 013 generation/wake/producer refactor + generation-checked enqueue/start + hook-driven host proofs added. `cargo fmt` clean; `cargo clippy -p src-tauri --lib -D warnings` clean; `cargo nextest run -p src-tauri --lib ttfa_013_tests` **11 passed**; targeted watch/resume/play-to-audio regressions included in **16 passed** suite.
- [ ] **SC-TTFA-001 device proof**: `blocked:device_ttfa_smoke`. 013 debug APK installed on 2201117TY (2026-11-07 09:38:23). `DebugIngestActivity` delivered doc snapshot but did not start controlled playback; no fresh `play_to_audio_ms` captured. Manual controlled StartNow/high-offset/switch-cancel/5+ min smoke required.
- [ ] **MDMV R4 validate / R7 release**: not satisfied (device TTFA proof + full SC/regression evidence incomplete).
- [ ] **T232/T233**: unchanged, not complete.
