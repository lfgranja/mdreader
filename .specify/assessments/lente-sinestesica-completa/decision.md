# Decision: Implementar a proposta COMPLETA de design SONA

- **Slug**: lente-sinestesica-completa
- **Created**: 2025-10-27
- **Decision**: APPROVE (with constraints)
- **Chosen option**: Option B — Extender reduzida para full com limites rígidos
- **Appetite**: medium
- **Inputs used**: intake.md, research.md, problem.md, concept.md

## Rationale

The full proposal fills a visual richness gap vs the current reduced Lente (feature 005). Option B is feasible because:
- Stays on single StatusView (no new surface) — respects S1
- Can leverage existing tokens where possible; closed set limits additions
- Can use static decoration/SVG + CSS respecting reduced-motion (S5)
- Device already verified for reduced scope; build pipeline works

Option A (do nothing) leaves gap vs design intent. Option C risks contract violations.

## Rejected Options

- **Option A** — Keep reduced: rejected because design/proposal shows full Lente; user explicitly requested full implementation.
- **Option C** — New mode/rewrite: rejected due to surface-count constraints and large scope/risk.

## Constraints / Guardrails

- **Surface**: StatusView only (no notification redesign)
- **Tokens**: Use existing closed set where possible; any new tokens require coordinated contract change
- **Layout**: Prefer single-column adaptation; avoid full 2-column restructure
- **Motion**: Respect S5 (≤1 continuous loop, reduced-motion guard)
- **Assets**: Static SVG preferred; no scripts/foreignObject
- **Data**: Use existing metadata/filename fallback if possible
- **Contracts**: Update surface-audit/status-surface if needed, following §VII
- **Verification**: Update device trials for expanded UI

## Next Action

**Proceed to specification**: `/speckit.specify slug=lente-sinestesica-completa` (create spec.md/plan.md/tasks.md for full Lente implementation under new feature scope or extend 005 with approved changes)

## Open Questions to Resolve During Specify/Plan

- Exact title card data source and fallback
- Static vs animated waves (respect reduced-motion)
- Token additions vs reuse
- Layout adaptation strategy
- Contract changes required
- Device trial criteria
