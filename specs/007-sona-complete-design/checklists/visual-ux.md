# Visual/UX Requirements Quality Checklist: SONA Complete Design (007)

**Purpose**: Validate completeness, clarity, consistency, and measurability of visual/UI requirements (wave ribbons, glassmorphism card, waveform visualizer, numeric rate, repeat, glow, layout, overflow, reduced motion).  
**Created**: 2026-10-28  
**Feature**: [spec.md](../spec.md)  
**Focus**: Visual/UX requirements quality (WCAG 2.2, Apple HIG, Material, web.dev, design-system token discipline)

**Note**: This checklist tests the REQUIREMENTS THEMSELVES, not implementation. `[x]` means the reviewer has determined the requirement quality criterion is satisfied.

## Requirement Completeness

- [x] CHK001 - Are decorative wave ribbon requirements fully specified (teal→jade→violet stops, layer count, opacity, animation duration ≤400 ms, one-shot vs. continuous)? [Completeness, Spec §FR-005, §FR-001, Clarifications]
- [x] CHK002 - Is the glassmorphism title card requirement specified with exact blur radius (24 px), translucent overlay opacity, verbatim token base color, and solid `--sona-stage` fallback? [Completeness, Spec §FR-009, Contract A4]
- [x] CHK003 - Is the waveform visualizer requirement fully specified (RMS source, ~20 Hz IPC, 4–6 smooth bezier layers, mirrored axis, white progress line + circular playhead, `aria-hidden`, parallel progressbar)? [Completeness, Spec §FR-008, Contract A7, A8]
- [x] CHK004 - Is the numeric rate display requirement specified with exact format (`1.25×`), placement in speed-control area, and `aria-valuetext` retaining the positional descriptor? [Completeness, Spec §FR-003, Contract A5]
- [x] CHK005 - Is the repeat control requirement specified (restarts current section, reuses skip/section logic, reuses skip-back earcon, inventory update)? [Completeness, Spec §FR-011, Contract A2]
- [x] CHK006 - Is the glow/neon requirement specified with exact `box-shadow` values, the two sanctioned elements, and reduced-motion guard? [Completeness, Spec §FR-010, Contract A1]
- [x] CHK007 - Are missing states (empty title, loading/indeterminate progress, waveform unavailable) explicitly specified for the title card and waveform? [Completeness, Gap, Spec §Edge Cases, Contract A8]

## Requirement Clarity

- [x] CHK008 - Is "subtle" glow/neon quantified (blur radius, opacity, max 2 elements) rather than left subjective? [Clarity, Spec §FR-010]
- [x] CHK009 - Is "frosted" glassmorphism quantified (blur(24 px), opacity, fallback) rather than left subjective? [Clarity, Spec §FR-009]
- [x] CHK011 - Is "smooth ribbons" quantified (layer count 4–6, bezier control points, opacity per layer, mirror axis) or ambiguous? [Clarity, Spec §FR-008, Clarifications]
- [x] CHK012 - Is the reduced-motion guard explicitly defined (trigger, affected animations, freeze vs. remove, dynamic toggle)? [Clarity, Spec §FR-004, §Edge Cases]

## Requirement Consistency

- [x] CHK013 - Are token usage requirements consistent with sanctioned scopes (jade/celeste only in waveform + decorative wave; rate uses `--sona-harp`)? [Consistency, Spec §FR-007, Contract A1, C-H1, C-H11]
- [x] CHK014 - Do glassmorphism alpha variants comply with amended S3 (base colors verbatim SonaToken members)? [Consistency, Contract A1, C-H1]
- [x] CHK015 - Are glow colors consistent with the token set (no jade/celeste in glow elements)? [Consistency, Contract A1, C-H11]
- [x] CHK016 - Do visual requirements align with the single-surface constraint (no new surfaces beyond StatusView + notification)? [Consistency, Spec §FR-007, §US3]

## Acceptance Criteria Quality

- [x] CHK017 - Can SC-001 ("all proposal elements present") be objectively verified against an enumerated element list rather than "matches intent"? [Measurability, Spec §SC-001, Contract A2]
- [x] CHK018 - Is the 30 fps cap (33.3 ms gate) and ~20 Hz IPC measurable and testable? [Measurability, Spec §FR-008, Plan §Performance Goals]
- [x] CHK019 - Is reduced-motion compliance measurable (freeze to static snapshot, no new continuous loops beyond S5)? [Measurability, Spec §SC-004, C-H6]

## Scenario Coverage

 - [x] CHK021 - Are requirements defined for glassmorphism fallback when `backdrop-filter` is unsupported (solid `--sona-stage`)? [Coverage, Spec §Edge Cases, Contract A4]
 - [x] CHK020 - Are requirements defined for waveform behavior when RMS data is unavailable (static placeholder or graceful hide, never an empty region)? [Coverage, Spec §Edge Cases, Contract A7]
- [x] CHK022 - Are requirements defined for indeterminate progress (no playhead knob, omit `aria-valuenow`, announce "carregando" once)? [Coverage, Contract A8, Spec §Edge Cases]
- [x] CHK023 - Are requirements defined for long filenames / empty title overflow (ellipsis, `flex: min-width: 0`, `white-space: nowrap`)? [Coverage, Spec §Edge Cases, Contract A4]
## Edge Case Coverage

- [x] CHK024 - Is dynamic reduced-motion toggle behavior specified (freeze at load and after OS-level toggle, no re-animation)? [Edge Case, Spec §Edge Cases, Clarifications]
- [x] CHK025 - Is behavior defined for decorative wave CSS/asset failure (`animation: none`, layout preserved, no broken image or alert)? [Edge Case, Spec §Edge Cases]
- [x] CHK026 - Is the ARIA update gap addressed (`renderProgress()` sets `aria-valuenow`/`aria-valuetext`; live region populated on meaningful changes ≥5% delta or state transition)? [Edge Case, Spec §Edge Cases, Contract A8, C-H5]

## Dependencies & Assumptions

## Notes

- Items marked [Gap] indicate missing or underspecified requirements that should be addressed before planning/implementation.
- Focus on requirements quality, not implementation correctness.
- Cross-references: Constitution v2.0.0 Principles VII (closed sets/token expansion) and VIII (supplementary visuals, movement budget ≤1 continuous loop, reduced-motion guard); C-H1, C-H4, C-H5, C-H6, C-H10, C-H11.