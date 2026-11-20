# Concept: Lente Sinestésica completa vs reduzida

- **Slug**: lente-sinestesica-completa
- **Created**: 2025-10-27
- **Recommended option**: Option B — Extender reduzida para full com limites rígidos

## Options

### Option A — Manter reduzida (fazer nada)
- **Sketch**: Keep current reduced Lente (token restyle + stamps only) as implemented in feature 005. No decorative waves, no title card, no layout change.
- **Appetite**: small
- **Trade-offs**: Wins: no scope creep, passes all existing contracts (closed tokens, single-column, movement limits), already device-verified. Sacrifices: doesn't match full proposal; less rich visuals.
- **Rabbit holes**: None significant; maintenance burden low.

### Option B — Extender reduzida para full com limites rígidos
- **Sketch**: Add full proposal elements (decorative waves as static SVG, title card using existing metadata/filename fallback, expanded control styling) while staying on single StatusView, respecting closed tokens/movement/reduced-motion, no RMS instrumentation, no new surfaces.
- **Appetite**: medium
- **Trade-offs**: Wins: matches design proposal more closely; improves visual richness. Sacrifices: more assets/CSS/JS; risk of contract drift (tokens/layout/animations); needs careful scope control.
- **Rabbit holes**: Layout restructuring (2-column temptation), adding tokens/colors violating closed set, animation loops violating S5, RMS/visualizer data dependencies, device trial expansion.

### Option C — Reescrever completo em novo modo
- **Sketch**: Separate "full Lente" mode with new layout/visuals, toggleable or replacing entirely.
- **Appetite**: large
- **Trade-offs**: Wins: clean separation, easier to match proposal. Sacrifices: much larger scope, likely new surfaces/routes, higher risk, contradicts "single surface" constraint from clarifications.
- **Rabbit holes**: Navigation changes, state management, contract conflicts (S1 two surfaces only), maintenance complexity.

## Recommendation

**Option B — Extender reduzida para full com limites rígidos**. It directly addresses the problem (gap between reduced and full proposal) while respecting constraints documented in research (single surface, closed tokens, reduced motion, no RMS). Option A leaves gap; Option C risks violating contracts and is too large. However, this requires explicit validation of assumptions (title data availability, token additions, animation constraints).

## Out of Scope (for the recommended option)

- New surfaces/notification redesign
- RMS-reactive audio visualization (static decoration only if at all)
- Breaking closed token set without coordinated contract change
- Full layout restructure to multi-column (adapt within single column)
- Unbounded animations (respect ≤1 continuous loop, reduced-motion)

## Assumptions to Validate

- Title/section metadata sufficient for title card (or use filename fallback like current behavior)
- Decorative elements can be implemented with static SVG + CSS tokens (no scripts)
- Animation budget fits S5 (≤1 continuous loop, reduced-motion guard)
- No need for new tokens; can use existing palette
- Device trials for expanded visuals can reuse SC-001/SC-006 with adjustments
- Contract changes (if any) are minimal and coordinated
