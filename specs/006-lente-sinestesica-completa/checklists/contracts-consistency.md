# Contracts & Consistency Requirements Quality Checklist: Lente Sinestésica Completa

**Purpose**: Validate contract compliance, token consistency, and surface invariants  
**Created**: 2025-10-27  
**Feature**: [spec.md](../spec.md)  
**Focus**: S1/S3/S5/S6, closed sets, surface inventory, notification purity, coordinated changes

**Note**: This checklist tests the REQUIREMENTS THEMSELVES, not implementation. `[x]` means the reviewer has determined the requirement quality criterion is satisfied.

## Requirement Completeness
- [x] CHK050 - Are surface count requirements explicit (exactly 2 surfaces)? [Completeness, Spec §FR-007, Plan §S1]
- [x] CHK051 - Are notification purity requirements fully specified (accent-only, no body/rate/voice/mark)? [Completeness, Spec §FR-006, §US3, Plan §S6]
- [ ] CHK052 - Is closed token set requirement specified with guidance on additions? [Completeness, Spec §FR-001, Plan §S3]
- [x] CHK053 - Are contract update requirements specified when tokens/layout change? [Completeness, Spec §FR-007]
- [x] CHK054 - Are surface inventory requirements specified (A1-A4, no dead display:none)? [Completeness, Spec §SC-005]

## Requirement Clarity
- [x] CHK055 - Is "coordinate changes" requirement unambiguous (what requires coordinated edit)? [Clarity, Plan §§VII Coordinated edit]
- [x] CHK056 - Is "prefer reuse" vs "additions need justification" clearly defined for tokens? [Clarity, Plan §S3]
- [x] CHK057 - Are the 10 token members or closed set definition referenced? [Clarity, Gap]
- [x] CHK058 - Is "no new surfaces" unambiguous (StatusView expansion vs new surface)? [Clarity, Spec §Overview]

## Requirement Consistency
- [x] CHK059 - Do all requirements align with feature 005 contracts (status-surface.md, surface-audit.md)? [Consistency, Plan §Source Code references]
- [x] CHK060 - Are notification requirements consistent across spec and plan (accent-only invariant)? [Consistency, Spec §FR-006, Plan §S6]
- [ ] CHK061 - Do movement/animation requirements align with S5 in constitution? [Consistency, Plan §S5]
- [x] CHK062 - Are rate requirements consistent (value-blind, positional only) across all stories? [Consistency, Spec §FR-003, §US2]

## Acceptance Criteria Quality
- [x] CHK063 - Can SC-003 ("Notification purity unchanged - accent-only") be objectively verified? [Measurability, Spec §SC-003]
- [x] CHK064 - Can SC-005 ("Surface inventory valid; no dead display:none corpses (C-H4 passes); aria inventory correct (C-H5 passes)") be objectively verified? [Measurability, Spec §SC-005]
- [x] CHK065 - Are contract compliance criteria defined for any token additions? [Measurability, Gap]

## Scenario Coverage
- [ ] CHK066 - Are requirements defined for contract update workflow if changes needed? [Coverage, Spec §FR-007]
- [x] CHK067 - Are requirements defined for notification during all StatusView states with complete UI? [Coverage, Spec §US3]
- [x] CHK068 - Are requirements defined for surface audit after changes? [Coverage, Plan §Implementation Strategy]

## Edge Case Coverage
- [x] CHK069 - Is behavior defined if token additions would violate closed set? [Edge Case, Gap]
- [x] CHK070 - Is behavior defined if layout change would affect surface count? [Edge Case, Gap]
- [x] CHK071 - Are requirements for removing dead CSS (if any) specified? [Edge Case, Gap]

## Dependencies & Assumptions
- [x] CHK072 - Is dependency on existing contracts (005-adopt-design-sona/contracts) documented? [Dependency, Plan §Source Code]
- [ ] CHK073 - Is assumption "token set may need minimal coordinated additions only if unavoidable" tracked? [Assumption, Spec §Assumptions]
- [ ] CHK074 - Is the coordinated edit process requirement clear (follows §VII)? [Dependency, Plan §§VII]

## Notes
- Any contract/token changes require coordinated edit per constitution §VII
