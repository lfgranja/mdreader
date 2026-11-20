# Idea Research: Ressoa Complete Design

- **Slug**: ressoa-complete-design
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: high

## Users & Demand

- Ressoa listeners want the exact proposal look in the driving/peripheral-gaze use case: wave ribbons, frosted title card, live waveform, numeric rate and complete transport — [source: `specs/007-ressoa-complete-design/spec.md:23-46`] (confidence: high)
- Spec 006 delivered the Lente Sinestésica but excluded precisely these elements under the older constitution; 007 exists to remove those exclusions — [source: `spec.md:15`] (confidence: high)

## Prior Art

- 005/006 established the single StatusView surface, accent-only notification and token governance; 007 builds on them rather than forking a new surface — [source: `spec.md`, `plan.md`] (confidence: high)
- RMS is already produced in the CPAL audio callback; the plan pins an in-memory `AtomicF32` + `tauri::ipc::Channel` at ~20 Hz with last-wins coalescing and a 30 fps canvas gate (33.33 ms) — [source: `plan.md:Technical Context/Performance Goals`, `spec.md:67` FR-008] (confidence: high)
- Waveform rendering pattern: 5 mirrored cubic-bezier ribbons, per-layer opacity 0.15–0.35, teal→jade→violet, white 1 px center line, 6 px playhead, `aria-hidden="true"` canvas with parallel sr-only progressbar/live region — [source: `spec.md:67` FR-008] (confidence: high)

## Market & Context

- Cost of doing nothing: the shipped UI diverges from the approved proposal; the P1 visual/controls story stays unfulfilled even though the constitution now permits it.

## Data & Constraints

- Constitution v2.0.0 §VII allows design-token expansion with coordinated edit + justification + census update (jade `#5AD6A8`, celeste `#B9A6F5`); §VIII permits supplementary visuals but forbids making any required interaction depend on them — [source: `spec.md:60-70`, `plan.md:Constitution Check`] (confidence: high)
- Invariants: notification accent-only; surface count = 2; element inventory 9→10; `display:none` corpse ban; reduced-motion freeze (not hide); glow on ≤2 elements, alpha ≤0.30; pure-offline static assets, no scripts/`foreignObject` — [source: `spec.md:66-70`, `plan.md:Constraints`, FR-004..FR-011] (confidence: high)
- Ledger: T001–T029 all checked; ROADMAP §2.2 records 007 as CONVERGED (29/29); T028 closed the initial-RMS-payload/screenshot defect, T029 recorded formal §VII/§VIII checklist exemptions — [source: `specs/007-ressoa-complete-design/converge.md`, `tasks.md`] (confidence: high)

## Evidence Against the Idea

- Extra canvas/glow work risks a11y-count regressions, a new continuous animation loop and token-drift. Mitigation: structural host census (C-H1 chroma, C-H4 inventory, C-H5 aria), reduced-motion guard and §VII sanctioned scopes. — [source: `plan.md:Constitution Check`, FR-004/FR-007] (confidence: medium)

## Gaps & Open Questions

- None blocking; device visual trial remains [NV] in local deterministic validation (SC-006).

## Sources

- `specs/007-ressoa-complete-design/{spec.md,plan.md,tasks.md,converge.md}` (host: local repo, policy: allowlisted path)
- `DESIGN-RESSOA/assets/ui/ui-proposta-2-lente-sinestesica.png` (host: local repo, proposal of record)
- `.specify/memory/constitution.md` v2.0.0 (host: local repo, policy: allowlisted path)
