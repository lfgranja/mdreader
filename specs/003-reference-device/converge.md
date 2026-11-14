# Converge: 003 Reference Device Realignment

Status: **CONVERGED** — reference-device definition and cold-start bound reconciled to the maintainer’s Redmi 2201117TY (≤10 s), with cross-spec divergence closed.

## Implementation Summary

Documentation/governance realignment replaced the hypothetical A78-class reference with the possessed device (Redmi 2201117TY, Snapdragon 680 `bengal`, 4× A73 + 4× A53, ~5.8 GB RAM, Android 13/API 33) and reconciled device-coupled latency bounds in 001 to ≤10 s cold-start, consistent with the 002 amendment. Divergence notes were closed with preserved history (no rewrite of prior measurements).

### Evidence (verified)
- `specs/003-reference-device/tasks.md`: 12 checked / 0 unchecked (12 total) [verified in prior cycles].
- Git evidence: `1be84f43 docs(001): realign reference device to Redmi 2201117TY, cold-start bound to <=10s`, `03829646 docs(002,003): close reference-device divergence, land 003 amendment artifacts`, `d5d6b455 docs(spec): amend SC-009 bound 3s->10s, reference device to Redmi Note 11` [verified in prior cycles].
- Spec artifacts present (`spec.md`, `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`) documenting contract sites and amendment discipline [verified in prior cycles].
- ROADMAP alignment: CONVERGIDO (Protocolo e hardware de referência). ROADMAP notes 12/14 tasks; task ledger shows 12/12 checkbox tasks complete — counted per `tasks.md` [verified in prior cycles].

### Gate 9 (Android build)
- Not applicable (docs-only amendment; no app/runtime code changes).

### Non-Changes
- No code, dependency, or constitution changes introduced by this spec.
- Historical verdict measurements preserved; re-judgments recorded explicitly rather than by editing old rows.

### Unresolved / Deferred
- None identified within 003 scope.

### Acceptance Criteria Status
- [x] FR-001: 001 reference-device paragraph realigned to Redmi 2201117TY [verified in prior cycles].
- [x] FR-002: Device-coupled cold-start bounds reconciled to ≤10 s in 001 context [verified in prior cycles].
- [x] FR-003/FR-004: Amendment recorded and 002 divergence closed with preserved history [verified in prior cycles].
- [x] SC-001–SC-003 satisfied per spec/task evidence [verified in prior cycles].
