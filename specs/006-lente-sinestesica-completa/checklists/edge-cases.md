# Edge Cases & Error Handling Requirements Quality Checklist: Lente Sinestésica Completa

**Purpose**: Validate edge case coverage and error/empty state requirements  
**Created**: 2025-10-27  
**Feature**: [spec.md](../spec.md)  
**Focus**: Empty states, fallbacks, failures, metadata variations

**Note**: This checklist tests the REQUIREMENTS THEMSELVES, not implementation. `[x]` means the reviewer has determined the requirement quality criterion is satisfied.

## Requirement Completeness
- [x] CHK075 - Are all empty/missing data states specified (no title, no section, no metadata)? [Completeness, Spec §FR-002, §US1]
- [x] CHK076 - Are filename fallback requirements specified when metadata unavailable? [Completeness, Spec §FR-002 — `metadata_title.unwrap_or(filename)` defined]
- [x] CHK077 - Are failure state visual requirements complete (static failure banner, no pulse)? [Completeness, Spec §US2 Scenario 2]
- [x] CHK078 - Are warning state requirements specified? [Completeness, Spec §US2 Scenario 2]
- [ ] CHK079 - Are asset loading failure requirements specified? [Completeness, Gap]

## Requirement Clarity
- [x] CHK080 - Is the empty state glyph `—` with `aria-label="Sem título"` unambiguous (when exactly to show)? [Clarity, Spec §FR-002, §US1]
- [x] CHK081 - Is "static failure banner" vs warning banner behavior clearly distinguished? [Clarity, Spec §US2 Scenario 2]
- [x] CHK082 - Is filename fallback behavior specific (truncation, display)? [Clarity, Gap — defined: single-line `overflow:hidden` + `text-overflow:ellipsis` + `white-space:nowrap`; flex `min-width:0`; vertical budget ≤3 lines if line-clamp; full name preserved in `aria-label`; empty-state `—` when both missing. ResearchFilenameFallback §3.1/§4.
- [ ] CHK083 - Are error conditions for title card rendering defined? [Clarity, Gap]

## Requirement Consistency
- [x] CHK084 - Do empty state requirements align across all UI states? [Consistency, Spec §FR-002]
- [x] CHK085 - Are failure/warning behaviors consistent with feature 005 reduced Lente? [Consistency, Spec §US2 Scenario 2]
- [x] CHK086 - Do fallback requirements align with upstream authority principle? [Consistency, Spec §FR-002 — upstream authority explicitly `metadata_title.unwrap_or(filename)`]

## Acceptance Criteria Quality
- [x] CHK087 - Can empty state be objectively verified (glyph + aria-label present)? [Measurability, Spec §US1 Scenario 3]
- [x] CHK088 - Can failure banner behavior be objectively verified (static, no animation)? [Measurability, Spec §US2 Scenario 2]
- [x] CHK089 - Are there measurable criteria for fallback scenarios? [Measurability, Gap — defined: F-01 DOM `.title-text` = `metadata_title||filename`; F-02 CSS `overflow:hidden/nowrap/ellipsis`; F-03 flex `min-width:0`; F-04 `aria-label===full`; F-05 empty-state `—` + `aria-label="Sem título"`; F-06 line budget ≤3; F-07 no scroll; F-08 binary pass/fail. ResearchFilenameFallback §3.3.]

## Scenario Coverage
- [x] CHK090 - Are requirements defined for no document loaded? [Coverage, Edge Case]
- [x] CHK091 - Are requirements defined for metadata present vs absent vs partial? [Coverage, Edge Case, Spec §FR-002]
- [x] CHK092 - Are requirements defined for very long filenames/titles causing layout issues? [Coverage, Edge Case, Gap]
- [ ] CHK093 - Are requirements defined for network/asset loading failures? [Coverage, Edge Case, Gap]
- [x] CHK094 - Are requirements defined for reduced-motion with animations present? [Coverage, Edge Case, Spec §FR-004]

## Edge Case Coverage
- [ ] CHK095 - Is behavior specified when decorative SVG fails to load (graceful degradation)? [Edge Case, Gap]
- [ ] CHK096 - Is behavior specified for CSS animation failure/fallback? [Edge Case, Gap]
- [x] CHK097 - Is behavior specified for title card overflow in constrained layout? [Edge Case, Gap]
- [ ] CHK098 - Are requirements defined for concurrent state changes (failure during animation)? [Edge Case, Gap]

## Dependencies & Assumptions
- [ ] CHK099 - Is assumption about metadata availability validated? [Assumption, Spec §Assumptions]
- [x] CHK100 - Is fallback strategy clearly specified as "filename" (what if filename also missing)? [Assumption, Gap — defined: strategy = `metadata_title.unwrap_or(filename)` (FR-002); if both null/empty → glyph `—` + `aria-label="Sem título"` (US1 Sc3 / SC-001/006). Authority preserved; empty-state covers missing-filename case. ResearchFilenameFallback §3.1/§4.

## Notes
- Edge cases are critical for eyes-free usage and resilience
