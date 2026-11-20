# Problem Definition: Implementar a proposta COMPLETA de design SONA

- **Slug**: lente-sinestesica-completa
- **Created**: 2025-10-27
- **Inputs used**: intake.md, research.md

## Problem Statement

Users/maintainers want the full SONA Lente Sinestésica experience (decorative waves, title card, wave visualization, expanded controls/layout) shown in the complete proposal, but the current implementation (feature 005 reduced Lente) only provides token-restyle + stamps on the existing single-column StatusView — creating a gap between the desired visual design and what is actually implemented.

## Affected Users & Stakeholders

- **Users**: Listeners/agents using the mdreader app on mobile — affected by less rich visual feedback vs the full proposal.
- **Stakeholders**: Design/maintainers — need to decide scope (replace reduced with full vs extend vs keep); impacted by added complexity/assets/tests.

## Goals

- Clarify whether to expand to full Lente or keep reduced scope.
- If expanding, define which elements are in scope and constraints.
- Ensure any expansion aligns with design system principles and existing contracts.

## Non-Goals

- [ASSUMPTION] Redesigning notification surface (must stay accent-only per S6/S1).
- [ASSUMPTION] Adding RMS-reactive instrumentation if not required — reduced scope explicitly avoided it.
- [ASSUMPTION] Breaking existing contracts (closed token set, surface count, movement constraints) without coordinated change.

## Success Metrics

- Decision clarity: explicit go/no-go with rationale (qualitative).
- If go: acceptance criteria defined, contracts updated as needed, device trials pass for full scope.
- If no-go: reduced Lente remains sufficient with documented rationale.

## Cost of Inaction

Current reduced Lente is functional but less visually rich than the full proposal; stakeholders may want more polish, but adding complexity without clear requirements risks scope creep and contract violations.

## Open Questions

- [NEEDS CLARIFICATION: Replace reduced Lente entirely or add as separate mode/flag?]
- [NEEDS CLARIFICATION: Which elements in scope (waves/visualization/title card/layout)?]
- [NEEDS CLARIFICATION: New data sources needed (document title/section for card)?]
- [NEEDS CLARIFICATION: Layout change to multi-column or adapt to single-column?]
- [NEEDS CLARIFICATION: Static decoration vs animated (respect reduced-motion)?]
- [NEEDS CLARIFICATION: Token additions needed or use existing closed set?]
- [NEEDS CLARIFICATION: Impacts on contracts (status-surface/surface-audit)?]
- [NEEDS CLARIFICATION: Updated device trials required?]
- [NEEDS CLARIFICATION: Is this feature 005 follow-up or new feature?]
