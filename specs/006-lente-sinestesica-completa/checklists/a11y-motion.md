# Accessibility & Motion Requirements Quality Checklist: Lente Sinestésica Completa

**Purpose**: Validate accessibility and motion requirements for reduced-motion compliance and screen reader support  
**Created**: 2025-10-27  
**Feature**: [spec.md](../spec.md)  
**Focus**: Reduced-motion (S5), ARIA, empty states, eyes-free usage

**Note**: This checklist tests the REQUIREMENTS THEMSELVES, not implementation. `[x]` means the reviewer has determined the requirement quality criterion is satisfied.

## Requirement Completeness
- [x] CHK027 - Are reduced-motion requirements fully specified (when to pause/freeze, what animations affected)? [Completeness, Spec §FR-004, §US1 Scenario 2]
- [x] CHK028 - Are ARIA requirements specified for all new elements (title card, waves container)? [Completeness, Gap]
- [x] CHK029 - Is empty title state accessibility specified (glyph `—` with `aria-label="Sem título"`)? [Completeness, Spec §FR-002, §US1 Scenario 3]
- [x] CHK030 - Are progressbar ARIA requirements preserved/extended? [Completeness, Spec §SC-002]
- [x] CHK031 - Are keyboard/screen reader considerations addressed for eyes-free usage context? [Completeness, Gap]

## Requirement Clarity
- [x] CHK032 - Is "reduced-motion guard" explicitly defined (what triggers it, what happens)? [Clarity, Spec §FR-004, Clarifications]
- [x] CHK033 - Is "pause/freeze" vs "remove" clearly distinguished for decorative animations? [Clarity, Spec §FR-004]
- [x] CHK034 - Is "no alert misuse" quantified/specifically defined (what constitutes misuse)? [Clarity, Spec §SC-002, Clarifications line 15]
- [x] CHK035 - Are animation loop constraints measurable (≤1 continuous loop, what counts as loop)? [Clarity, Spec §FR-004, Plan §S5]

## Requirement Consistency
- [x] CHK036 - Do motion requirements align with S5 constraints (≤1 continuous loop, reduced-motion guard, yellow pulse ≤3s, no pulse on failure)? [Consistency, Plan §Constitution Check]
- [x] CHK037 - Are a11y requirements consistent between status view and notification (notification must remain accessible)? [Consistency, Spec §FR-006, §US3]
- [x] CHK038 - Do empty state requirements align across all scenarios (glyph + aria-label)? [Consistency, Spec §FR-002, §US1]

## Acceptance Criteria Quality
- [ ] CHK039 - Can SC-002 ("no accessibility regressions... progressbar aria intact, empty slot aria-label, no alert misuse") be objectively verified? [Measurability, Spec §SC-002]
- [x] CHK040 - Can reduced-motion compliance be tested objectively? [Measurability, Gap, Spec §SC-004]
- [ ] CHK041 - Are there specific acceptance criteria for title card accessibility? [Measurability, Gap]

## Scenario Coverage
- [x] CHK042 - Are requirements defined for reduced-motion preference enabled? [Coverage, Spec §US1 Scenario 2]
- [x] CHK043 - Are requirements defined for screen readers with decorative elements present? [Coverage, Gap]
- [x] CHK044 - Are requirements defined for high contrast mode if applicable? [Coverage, Gap — defined per clarification #2: minimal structural guard (`forced-colors: active` → decorative static/outlined, layout preserved; title/bar text via CurrentColor; no color-only signaling; reference SC-006 device trial if supported). ResearchHighContrast confirms Option B.

## Edge Case Coverage
- [x] CHK045 - Is behavior defined if reduced-motion is toggled dynamically? [Edge Case, Gap]
- [ ] CHK046 - Is behavior defined for animations that would violate loop constraint? [Edge Case, Gap]
- [x] CHK047 - Are focus management requirements specified for any interactive elements added? [Edge Case, Gap — resolved: SC-006 "no new interactive elements"; decorative non-focusable per Clarifications; focus order unchanged. If any future interactive added, apply WCAG 2.4.12 visible focus ring]

## Dependencies & Assumptions
- [x] CHK048 - Is CSS `prefers-reduced-motion` support assumed/documented? [Assumption, Gap — mechanism documented (Clarifications line 14: `@media (prefers-reduced-motion: reduce)` + `matchMedia`; FR-004; SC-006). Add explicit entry to spec.md Assumptions §89-94 for completeness]
- [x] CHK049 - Is the assumption about browser/OS reduced-motion behavior validated? [Assumption, Gap]

## Notes
- Eyes-free usage is critical per project context (driving) - a11y requirements must be explicit
