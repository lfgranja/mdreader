# Performance & Non-Functional Requirements Quality Checklist: SONA Complete Design

**Purpose**: Validate performance and non-functional requirements quality for the canvas waveform visualizer (frame rate, IPC cadence, battery/thermal, DPR, visibility, reduced motion, memory, graceful degradation, offline) before planning
**Created**: 2026-10-28
**Feature**: [spec.md](../spec.md)
**Focus**: Performance & non-functional requirements quality (NFR) for canvas waveform viz in mobile webviews

**Note**: This checklist tests the REQUIREMENTS THEMSELVES, not implementation. `[ ]` means the reviewer must determine the requirement quality criterion is satisfied.

## Requirement Completeness

- [x] CHK001 - Are frame-rate requirements completely bounded: cap 30 fps via 33.3 ms gate, with measurable preconditions (mobile webview device class) and a defined measurement point (IPC event to painted frame)? [Completeness, Spec §FR-008, Clarifications (third pass)]
- [x] CHK002 - Are IPC cadence requirements completely specified: ~20 Hz throttled channel, coalescing/backpressure rule, and behavior when IPC events arrive faster than the 30 fps gate? [Completeness, Spec §FR-008, Assumptions]
- [x] CHK003 - Are visibility-handling requirements defined for both hiding and returning: pause on `visibilitychange` (hidden), resume with one fresh frame on visible, and no redraw while hidden? [Completeness, Spec §Edge Cases, Clarifications (third pass)]
- [x] CHK004 - Are DPR requirements specified: backing store capped at 2×, handling of `devicePixelRatio` changes (rotation/zoom), and fallback when DPR unavailable? [Completeness, Spec §FR-008, Edge Cases]
- [x] CHK005 - Are battery/thermal requirements specified as testable NFRs: no continuous rAF loop, pause on hidden, freeze under reduced motion, and any adaptive throttling under thermal pressure? [Completeness, Spec §FR-008, Assumptions]
- [x] CHK006 - Are memory bounds specified: canvas backing-store size cap, no allocation on hot path, cleanup on unmount/navigation, and bounded RMS buffer lifetime? [Completeness, Spec §FR-008, Edge Cases]
- [x] CHK007 - Are graceful-degradation requirements defined for every failure mode: RMS unavailable → static placeholder, canvas unavailable → `#progress` bar, `backdrop-filter` unsupported → solid `--sona-stage`? [Completeness, Spec §Edge Cases, Clarifications]
- [x] CHK008 - Are offline requirements defined: waveform viz and StatusView function with no network, no remote asset dependency, and no network-triggered redraw? [Completeness, Spec §Assumptions, Spec §FR-008]

## Requirement Clarity

- [x] CHK009 - Is "no self-running `requestAnimationFrame` loop" stated as a verifiable absence (no rAF scheduling except the single IPC-driven redraw path) rather than an aspiration? [Clarity, Spec §FR-008, Clarifications (third pass)]
- [x] CHK010 - Is "no allocation on hot path" defined with a measurable criterion (zero heap allocations per redraw frame) and a stated measurement method (profiler/DevTools)? [Clarity, Measurability, Spec §FR-008]
- [x] CHK011 - Is the 33.3 ms gate defined precisely (timestamp comparison, inclusive/exclusive boundary, behavior on exactly 33.3 ms)? [Clarity, Spec §FR-008]
- [x] CHK012 - Is "pause on `visibilitychange`" scoped to `document.visibilityState === 'hidden'` (and `freeze`/`pagehide` where applicable) with a defined resume trigger? [Clarity, Spec §FR-008, Edge Cases]
- [x] CHK013 - Is the DPR 2× cap stated as `min(devicePixelRatio, 2)` with a defined fallback to 1 when `devicePixelRatio` is unavailable or non-finite? [Clarity, Spec §FR-008]
- [x] CHK014 - Is the static placeholder fallback specified with concrete content (SVG/CSS asset, dimensions, color) and a defined trigger condition (RMS unavailable OR canvas unsupported)? [Clarity, Spec §Edge Cases]

## Requirement Consistency

- [x] CHK015 - Do the 30 fps cap and ~20 Hz IPC cadence align without contradiction (IPC slower than frame cap, so the gate never drops IPC updates that are within budget)? [Consistency, Spec §FR-008]
- [x] CHK016 - Are reduced-motion requirements consistent with performance requirements: freeze (static snapshot) under `prefers-reduced-motion: reduce` must not introduce a new continuous loop or rAF? [Consistency, Spec §FR-004, Spec §FR-008, Clarifications]
- [x] CHK017 - Are visibility-pause requirements consistent with reduced-motion freeze (both must leave a static snapshot; neither may tear down canvas state)? [Consistency, Spec §FR-008, Edge Cases]

## Acceptance Criteria Quality

- [x] CHK018 - Can the 30 fps cap be objectively verified (frame counter / DevTools FPS meter) in 10/10 trials on the stated device class? [Measurability, Spec §SC-001, Spec §FR-008]
- [x] CHK019 - Can "pause on `visibilitychange`" be objectively verified (zero canvas redraws while hidden, measured via instrumentation)? [Measurability, Spec §SC-004, Edge Cases]
- [x] CHK020 - Can "no rAF loop" be objectively verified via static analysis / runtime (zero `requestAnimationFrame` calls outside the sanctioned path)? [Measurability, Spec §FR-008]
- [x] CHK021 - Can graceful degradation be objectively verified (placeholder rendered when RMS unavailable; `#progress` bar shown when canvas unavailable)? [Measurability, Spec §Edge Cases, SC-002]

## Scenario & Edge Coverage

- [x] CHK022 - Are requirements defined for the dynamic reduced-motion toggle (freeze at load AND after OS-level toggle) without a page reload, and is the DPR>2 / devicePixelRatio-change mid-session behavior specified? [Coverage, Edge Case, Spec §Edge Cases, Spec §FR-008]

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation
- Research inputs: web.dev canvas performance, MDN Page Visibility, Chrome Page Lifecycle, Android WebView performance, prefers-reduced-motion (WCAG C39/SCR40), High DPI canvas (web.dev)
- Items are numbered sequentially for easy reference