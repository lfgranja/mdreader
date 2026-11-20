# Concept: Adotar o DESIGN-RESSOA

- **Slug**: adopt-design-ressoa
- **Created**: 2026-11-20 (backfilled)
- **Recommended option**: Option A

## Options

### Option A — Notification-first chromatic restyle + earcon mapping + double brand (recommended)
- **Sketch**: Keep the two existing surfaces; apply the closed 10-hex set (FR-001), Concha R header in-app and Ressonador Canônico host-diagnostics-only (FR-004), map 7 confirmations to 6 acoustic names using 004's grammar (FR-003), enforce notification purity (FR-002/FR-007), additive duck-exempt earcon channel with ≤100 ms (FR-009/FR-010), reduced-motion + TalkBack (SC-007). Host census via surface-audit C-H1..C-H8.
- **Appetite**: medium-large; host-verifiable, device legs deferred.
- **Trade-offs**: Delivers auditable identity without a new UI framework; risk confined to closed-set/aria regression, pinned by deterministic tests and bundle parity.

### Option B — Full Svelte UI rewrite with complete Lente
- **Sketch**: Replace StatusView with a new framework-based surface and the full proposal.
- **Appetite**: very large.
- **Trade-offs**: Out of accepted Option A scope, new surfaces break S1 and bundle-parity invariants; rejected (covered separately by spec 006).

### Option C — Cosmetic-only palette swap
- **Sketch**: Change hex values only; skip earcons, brand discipline and audits.
- **Appetite**: small.
- **Trade-offs**: No distinctiveness or accessibility evidence; rejected.

## Recommendation

Option A. It matches the accepted handoff and is the only option with a deterministic host oracle and explicit device deferral.

## Out of Scope (recommended option)

- Svelte UI, new surfaces, new tokens, numeric rate, brand/waves on notification or lock-screen, device trials (deferred).

## Assumptions to Validate

- 004 earcon grammar is reusable as-is (verified: FR-003 points to `contracts/earcon-mapping.md`).
- Single-column geometry survives restyle (FR-008; host structural test pending at backfill).
- `dist/index.html` stays byte-identical to `src-tauri/src/status_view.html` (bundle parity test T031/T038/T055).
