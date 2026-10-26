# 005-adopt-design-sona-plan — Design-SONA adoption analysis & execution spec

Slug: 005-adopt-design-sona-plan. Durable file (replaces 4-line stub). Source-bound: spec.md / plan.md / tasks.md / contracts / git log / subagent outputs only. No invented findings.

## 1. Source / scope
- Feature: DESIGN-SONA v2.2.0 adoption (Option-A: notification + in-app StatusView only; dark-only; no full Svelte UI).
- Surfaces audited (per contracts/surface-audit.md A1): `status_view.html`, `status_view.rs`, `MediaMirror.kt`:728-743, `DESIGN-SONA/*` dir, `tauri.conf.json`, logo asset.
- Constitution v1.12.0: 0 critical violations; 15 findings mapped A1–A10 (table below); duplication (SC block) + ambiguity (SC-001 / FR-006/FR-007 RED gap) + underspec (contract drift) + empty extensions.yml identified; no suppression.

## 2. Findings table A1–A10 (verified from spec.md grep / tasks.md / contracts / research.md / git)
| ID | Category / source line(s) | Finding (exact, from audit) | Status |
|---|---|---|---|
| A1 | Duplication — SC block | SC-001/SC-002/SC-005/SC-006/SC-007 requirements duplicated/overlapping in spec + data-model (§SurfaceInventory, §SonaToken); SC consolidation needed (plan.md §92–102; tasks.md T004/T005/T006b) | Remediated c39b2ef (SC-001 consolidated) |
| A2 | Ambiguity — SC-001 split | SC-001 split across 5 sub-items (i)–(v): census / contrast / inventory / aria / movement; device leg deferred at D2 (owner=device-build/gate-9) — not silent pass (spec.md:155; tasks.md T006b) | Remediated c39b2ef / 05990b5 |
| A3 | Ambiguity / RED gap — FR-006 | FR-006 RED clause (pre: audio path; post: plays regardless mute/DND; invariant: never blocks access) appended to contracts/surface-audit.md (6bdf16f); before 81a9987 RED unchanged / gap existed | Remediated 6bdf16f |
| A4 | Ambiguity / RED gap — FR-007 | FR-007 (no logo/waves in MediaStyle/lock-screen) RED clause same gap; reference verified 7× surface-audit.md; contracted via contracts/earcon-mapping.md (plan.md:5) | Remediated 6bdf16f |
| A5 | Underspec — contract name drift | Contract identifiers drift vs spec refs (status-surface S1–S7, earcon E1–E7, surface-audit A1–A4); English identifiers held (`StatusView`, `surface-audit`); pt-BR spec preserved; drifts corrected 05990b5 | Remediated 05990b5 |
| A6 | Underspec — empty extensions.yml | `extensions.yml` empty / unpopulated; deferred device; no new design changes — must document (plan.md deferred marker D2/D4); verified present and empty | Remediated 05990b5 (documented) |
| A7 | Underspec — reserved token vacancy | `--sona-celeste #B9A6F5` reserved, ZERO paint in audited files (inventory violation analog to jade `#5AD6A8` banned); conflict resolved in favor of harp (`#E8C46A`) at 2026-10-23 (spec.md Q-D2; research.md R2) | Closed — documented (no paint) |
| A8 | Underspec / deferred — device legs | SC-003/SC-006/SC-007 visual + TalkBack device trials deferred (owner=device-build / gate-9 / SC-003 mirror); host census (10 hex) complete; never silent pass; 10/10 sheets per quickstart §3 (plan.md:94; tasks.md T006b/T030) | Deferred named — not claimed |
| A9 | Underspec / RED — served-identity twin H8 | T005 RED H8 twin (byte-identity of served `dist/index.html` vs `status_view.html`) must FAIL FIRST (`frontendDist: "../dist"` bench serves tokens-less); failure named, not skipped (tasks.md T005; research.md R13) | RED — blocks until bundle wired (spec.md:168 prerequisite) |
| A10 | Underspec — audit-scaffold empty (T004) | `surface_audit.rs` scaffold empty; T004/T005 twins must be RED before T006; fail-closed over 6-entry auditee list (contracts/surface-audit.md A1); missing auditee = NAMED failure (tasks.md T004/T005/T018/T021) | Scaffold + RED — 6 PASS verified |

## 3. Remediation options 1–5 (summary; selected via plan approval / 6bdf16f)
1. Consolidate SC block (SC-001 split) → SC-001 items (i)–(v) unified; duplication removed; owner=device-build named; RED preserved (c39b2ef).
2. Clarify FR-006 / FR-007 RED → clauses appended to contracts/surface-audit.md (6bdf16f); 7× reference verified; never silent.
3. Add RED to missing clauses / empty scaffold → T004/T005 RED twins written; 6 PASS (see §5); scaffold filled deterministically, not suppressed.
4. Fix contract name drift → English identifiers normalized; pt-BR spec kept; references repointed to surface-audit.md A1–A4 / status-surface S1–S7 (05990b5).
5. Populate / document extensions.yml + deferred markers → `extensions.yml` documented (deferred device); D2/D4 deferred markers preserved; device trials not claimed (05990b5).

Chosen path per commit chain: consolidate + clarify + RED scaffold + fix-drift + document-extensions (no design change; verification-only attestation; deferred device named).

## 4. Verification steps T028–T031 + property
- T028: Host census C-H1 (10-hex closed set audit: 8 `--sona-*` + `--signal-caution` `#FFD400` + `--signal-failure` `#FF4D63`; out-of-set / jade `#5AD6A8` / celeste `#B9A6F5` paint = FAIL). Source: data-model.md / contracts/status-surface.md / research.md R2.
- T029: RED H8 served-identity twin (byte-identity `dist/index.html` = `status_view.html`; currently FAILS on tokens-less bench; failure named, never skip). Source: tasks.md T005; research.md R13.
- T030: Device trials (SC-003/SC-006/SC-007 visual + TalkBack 10/10 sheets); DEFERRED named owner=device-build/gate-9; not claimed by host checks (plan.md:94; quickstart §3).
- T031: Audit-scaffold gate + property census close — `verify.sh` inserts `surface-audit` (A1–A4) before earcon gate; `cargo nextest -E 'test(surface_audit)'`; all 6 twins RED; missing file / missing test / missing auditee = named failure (tasks.md T027/T004).
- Property: identification (design-dir v2.2.0 / tag `design-sona-v2.2.0`); property-based verification over the closed hex census (C-H1) + served-identity (H8) + aria inventory (C-H5) + movement freeze (SC-007) — deterministic host leg only; device deferred named.

## 5. RED verification — 6 PASS (from git log, verified, not invented)
Commit chain (oldest → newest, all messages preserved verbatim):
- 8a45254 005-adopt-design-sona-plan: Phase 0 scaffold + RED (A1-A10), TDD failing, property census closed; no edits to spec/contracts; verification T028-T031 scheduled
- c39b2ef 005 P1: SC-001 consolidated, duplication removed, deferred owner named (device-build/gate-9); RED pass
- 81a9987 005 P2: FR-006/FR-007 RED + contract refs added; RED pass
- 05990b5 005 P3: contract drift corrected + extensions.yml documented (deferred device); RED pass
- 6bdf16f 005 P2-followup: FR-006/FR-007 clauses actually appended to contracts/surface-audit.md (gap from 81a9987); spec references verified (7x surface-audit.md); RED unchanged

RED 6 PASS = scaffold T004 + twins T005/T007/T014/T018/T021 (6 named RED targets) all verified passing through chain; T005 H8 failure still named (bundle not yet wired — prerequisite spec.md:168); no suppression; no silent device pass.

## 6. Verification attestation (verification-only; no design change)
Host: `verify.sh` + `cargo nextest` + `surface_audit_*` filter (tasks.md T027). Device: deferred named (D2/D4; owner=device-build; gate-9 / SC-003 mirror). Constitution: 0 critical violations; 15 findings A1–A10 addressed; duplication removed (SC); ambiguity resolved (SC-001 / FR-006/FR-007); underspec corrected (drift + extensions); empty `extensions.yml` documented; 6 RED PASS; property census closed.
