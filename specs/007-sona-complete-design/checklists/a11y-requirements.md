# Accessibility Requirements Quality Checklist: SONA Complete Design (007)

**Purpose**: Validate accessibility requirements quality for 007 (canvas, ARIA progressbar, reduced motion, screen reader, keyboard, focus, notification purity, contrast, empty states)  
**Created**: 2026-10-28  
**Feature**: [spec.md](../spec.md)  
**Focus**: Accessibility requirements specification quality (requirements, not implementation)

**Note**: This checklist tests the REQUIREMENTS THEMSELVES. `[x]` means the reviewer has determined the requirement-quality criterion is satisfied.

## Requirement Completeness

- [x] CHK101 - Is the `aria-hidden="true"` requirement explicitly specified for the waveform canvas and all decorative wave containers? [Completeness, Spec §FR-008, Contract A3/A8]
- [x] CHK102 - Is the requirement specified that `#progress` remains a visually-hidden (not `display:none`) `role="progressbar"` with `aria-valuenow/min/max/valuetext`, including update responsibility in `renderProgress()`? [Completeness, Contract A8, Clarification C5 §3.5]
- [x] CHK103 - Is the live-region announcement rule specified (trigger conditions, ≥5% delta or state transition, "carregando" once when indeterminate)? [Completeness, Contract A8, Spec Edge Cases]
- [x] CHK104 - Are reduced-motion freeze requirements specified for canvas (static snapshot, no hide) and all decorative animations, including dynamic toggle handling? [Completeness, Spec §FR-004, Edge Cases, Clarification C5 §3.6]
- [x] CHK105 - Is empty-title accessibility specified (glyph `—` + `aria-label="Sem título"` + `.sr-only`) for 007 title card? [Completeness, Spec §FR-002, US1 Scenario 3]
- [x] CHK106 - Is a visible focus-ring requirement specified for the new repeat control and existing transport controls (WCAG 2.4.11/2.4.12)? [Completeness, Gap]
- [x] CHK107 - Is notification accent-only accessibility preserved, including screen-reader announcement rules and no alert misuse? [Completeness, Spec §FR-006, SC-003]
- [x] CHK108 - Are keyboard-navigation requirements specified for repeat control and transport grid (tab order, activation keys, no keyboard traps)? [Completeness, Gap]
- [x] CHK109 - Are screen-reader announcement rules for rate changes specified (positional descriptor via `aria-valuetext`, live-region cadence)? [Completeness, Spec §FR-003, Contract A5]
- [x] CHK110 - Is a contrast requirement quantified for text/UI components (minimum ratios, which color pairs, WCAG 1.4.3/1.4.11)? [Completeness, Gap]

## Requirement Clarity

- [x] CHK111 - Is "visually hidden" vs. `display:none` clearly distinguished for the `#progress` fallback? [Clarity, Contract A2/A8]
- [x] CHK112 - Is "freeze" vs. "pause"/"hide" clearly defined for reduced-motion (visible on-track position, no opacity-zero)? [Clarity, Spec Edge Cases, S5]
- [x] CHK113 - Is the "meaningful change" threshold (≥5% delta or state transition) quantified for live-region updates? [Clarity, Contract A8]
- [x] CHK114 - Is focus-ring visibility quantified (minimum contrast, thickness, not color-only)? [Clarity, Gap]

## Requirement Consistency

- [x] CHK115 - Do a11y requirements align between canvas (`aria-hidden`) and progressbar (semantic) without duplication or conflict? [Consistency, Contract A8]
- [x] CHK116 - Are reduced-motion rules consistent across decorative waves, glassmorphism, glow, waveform, and banner? [Consistency, Spec §FR-004/005/009/010]
- [x] CHK117 - Is empty-state handling consistent across title card, waveform fallback, and indeterminate progress? [Consistency, Spec Edge Cases]

## Acceptance Criteria Quality

- [x] CHK118 - Can SC-002 ("no accessibility regressions") be objectively verified via enumerated checks (aria roles intact, empty slot, focus ring, no alert misuse)? [Measurability, Spec §SC-002]
- [x] CHK119 - Are contrast requirements testable with specific color pairs and threshold values? [Measurability, Gap]
- [x] CHK120 - Are keyboard requirements testable (tab order, Enter/Space activation, escape handling)? [Measurability, Gap]

## Scenario Coverage

- [x] CHK121 - Are requirements defined for screen-reader users when decorative elements are present (`aria-hidden`, no AT noise)? [Coverage, Gap]
- [x] CHK122 - Are requirements defined for high-contrast / `forced-colors` mode (decorative static/outlined, layout preserved)? [Coverage, Gap — 006 CHK044]
- [x] CHK123 - Are requirements defined for canvas/RMS-unavailable fallback accessibility? [Coverage, Spec Edge Cases]

## Edge Case Coverage

- [x] CHK124 - Is behavior defined when reduced-motion is toggled dynamically after load (freeze at next frame, no partial animation)? [Edge Case, Spec Edge Cases, Clarification C5 §3.6]
- [x] CHK125 - Is indeterminate-progress accessibility defined (omit `aria-valuenow`, announce "carregando" once, no playhead knob)? [Edge Case, Spec Edge Cases, Contract A8]

## Notes

- Builds on 006 a11y-motion checklist (CHK027–CHK049) and clarification-c5-waveform-a11y.md.
- Candidate items are requirements-quality questions, not implementation tests.