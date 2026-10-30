# UX & Visual Requirements Quality Checklist: Lente Sinestésica Completa

**Purpose**: Validate completeness, clarity, and consistency of visual/UI requirements  
**Created**: 2025-10-27  
**Feature**: [spec.md](../spec.md)  
**Focus**: Visual hierarchy, proposal alignment, tokens, title card, waves, controls

**Note**: This checklist tests the REQUIREMENTS THEMSELVES, not implementation. `[x]` means the reviewer has determined the requirement quality criterion is satisfied.

## Requirement Completeness
- [x] CHK001 - Are all major visual elements from the complete proposal (waves, title card, visualization, refined controls) explicitly required or intentionally excluded? [Completeness, Spec §Overview]
- [x] CHK002 - Is the header requirement ("Concha Harmônica R") clearly specified with no prohibited colors (no jade `#5AD6A8`, no celeste `#B9A6F5`)? [Completeness, Spec §US1]
- [x] CHK003 - Are title card requirements fully specified (metadata source, filename fallback, empty state)? [Completeness, Spec §FR-002, §US1]
- [x] CHK004 - Is the decorative wave requirement defined (static preferred, animation allowance with constraints)? [Completeness, Spec §FR-001, §FR-004, §FR-005, Clarifications]
- [x] CHK005 - Are control bar requirements specified to preserve existing functionality while matching refined style? [Completeness, Spec §US2]

## Requirement Clarity
- [x] CHK006 - Is "complete Lente Sinestésica" unambiguously bounded (single StatusView only, no new surfaces)? [Clarity, Spec §Overview, §FR-007]
- [x] CHK007 - Is the title card data source unambiguously specified as upstream authority `metadata_title.unwrap_or(filename)`? [Clarity, Spec §FR-002]
- [x] CHK008 - Is "value-blind rate" clearly defined as positional indicator only with no painted numeric rate? [Clarity, Spec §FR-003, §US2]
- [x] CHK009 - Is "subtle CSS animations" quantified or bounded (what counts as subtle vs continuous loop)? [Clarity, Spec §FR-004, §FR-005, Clarifications]
- [x] CHK010 - Is "refined control layout" specific enough to be verifiable or does it need measurable criteria? [Ambiguity, Spec §FR-001]

## Requirement Consistency
- [x] CHK011 - Do visual requirements align with reduced Lente scope (expanding, not contradicting) from feature 005? [Consistency, Spec §Overview]
- [x] CHK012 - Are token usage requirements consistent with closed token set preference? [Consistency, Spec §Overview, §FR-001]
- [x] CHK013 - Do animation requirements (FR-004/FR-005) align with S5 constraints stated in plan? [Consistency, Plan §Constitution Check, Spec §FR-004/FR-005]
- [x] CHK014 - Are warning/failure visual requirements consistent with existing banner behavior (static failure, warning with TTL)? [Consistency, Spec §US2]

## Acceptance Criteria Quality
- [x] CHK015 - Can SC-001 ("visual matches proposal intent on device... without jade/celeste paint") be objectively verified? [Measurability, Spec §SC-001]
- [x] CHK016 - Are acceptance scenarios for empty title state testable (glyph `—` + `aria-label="Sem título"`)? [Measurability, Spec §US1 Scenario 3]
- [x] CHK017 - Is proposal alignment criteria measurable beyond "matches intent"? [Measurability, Gap, Spec §SC-001]

## Scenario Coverage
- [x] CHK018 - Are requirements defined for document loaded with metadata vs filename-only vs no title? [Coverage, Edge Case, Spec §FR-002, §US1]
- [ ] CHK019 - Are requirements defined for different control states during visual expansion? [Coverage, Spec §US2]
- [ ] CHK020 - Are requirements defined for failure/warning states with complete visuals applied? [Coverage, Spec §US2 Scenario 2]

## Edge Case Coverage
- [x] CHK021 - Is fallback behavior specified when decorative assets fail to load? [Edge Case, Gap]
- [x] CHK022 - Is behavior defined for very long titles/filenames in the title card? [Edge Case, Gap]
- [x] CHK023 - Are layout constraints specified if title card takes more vertical space in single-column layout? [Edge Case, Gap]

## Dependencies & Assumptions
- [ ] CHK024 - Is the assumption "title/section metadata available or filename fallback sufficient" validated/documented? [Assumption, Spec §Assumptions]
- [ ] CHK025 - Is the assumption "can adapt within single-column layout without breaking existing controls" validated? [Assumption, Spec §Assumptions]
- [ ] CHK026 - Is dependency on `metadata_title` from upstream documented? [Dependency, Gap]

## Notes
- Items marked [Gap] indicate missing requirements that should be addressed before planning/implementation
- Focus on requirements quality, not implementation correctness
