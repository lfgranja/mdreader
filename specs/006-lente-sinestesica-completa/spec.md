# Feature Specification: Lente Sinestésica Completa

**Feature Branch**: `006-lente-sinestesica-completa`  
**Created**: 2025-10-27  
**Status**: Draft  
**Input**: User description: "Implementar agora a proposta COMPLETA de design SONA, com TODOS os elementos de UI/UX propostos" (design proposal image: `DESIGN-SONA/assets/ui/ui-proposta-2-lente-sinestesica.png`)

## Clarifications

### Session 2025-10-27
- Q: Should the decorative waves be strictly static SVGs, or may they include CSS animations (while still respecting reduced motion)? → A: Allow subtle CSS animations with reduced-motion guard
### Session 2026-10-27 (Clarifications accepted — all recommendations Option A / Option C)
- Q: Subtle CSS animations quantified? → A: Option C — multi-axis definition (allowed props: opacity / translate≤10px / scale≤1.05; duration ≤250ms/≤400ms; no infinite loops; area≤1/3 viewport; mandatory `prefers-reduced-motion` guard with `animation:none` under reduce). Evidence: `motion-research-report.md` (WCAG 2.2/2.3, MDN, PubMed >35% vestibular by 40).
- Q: Reduced-motion guard (freeze vs hide)? → A: Freeze motion (`animation:none` / static), keep decorative visible; trigger `@media (prefers-reduced-motion: reduce)` + `matchMedia`; no opacity tricks. Evidence: W3C C39 + Apple Dev docs + `CHK032_CHK034_CHK035_evidence_report.md`.
- Q: Alert misuse quantified? → A: `assertive`/`alert` ONLY for critical/time-sensitive errors; `polite`/`status` for status/progress; never on interactive elements, never auto-disappearing, never frequent/repeating. Evidence: W3C ARIA APG Alert Pattern + A11Y Collective.
- Q: Loop measurable? → A: Loop = repeating >5s or `infinite`; ≤1 continuous loop permitted; gated under `reduce`; bounded ≤5s escapes 2.2.2 (G152). Evidence: `CHK032_CHK034_CHK035_evidence_report.md` §4 / W3C 2.2.2.
- Q: Decorative ARIA / keyboard / screen-reader? → A: Waves → `aria-hidden="true"`; title card → `role="region"` + `aria-label` (populated) / `aria-label="Sem título"` (empty); keyboard unchanged, decorative non-focusable. Evidence: `a11y-motion-review-report.md` (CHK028/031/043).
- Q: Graceful degradation / overflow / long filenames? → A: Decorative failure → `animation:none`, layout preserved, no broken-image/alert; long title → `overflow:hidden; text-overflow:ellipsis; white-space:nowrap` with flex `min-width:0`; vertical budget ≤3 lines; banner fixed-height; focus ring visible per WCAG 2.4.12. Evidence: `LAYOUTBANNER_RETRY_REPORT.md` (4-dp/16/24 spacing, Fitts/Thumb-Zone, overflow patterns, 10-hex audited set).
- Q: Measurable layout / proposal alignment? → A: SC-001 purely objective (presence/absence of named elements + C-H1 hex census + C-H3 rate-blind + C-H4 no dead + C-H5 aria); decorative wave NOT continuous loop; proposal "intent" removed or made objective. Evidence: `UxVisualReview` + `review_contracts.md` + `motion-research-report.md`.
- Q: Coordinated edit / token addition / failure mode? → A: 3-file workflow (contract + `data-model.md` §SonaToken + source); justification record (commit message + `DESIGN-SONA/tokens.md`); SC-007 audit (C-H1 census, binary pass/fail); fail-closed block / rollback; dead-CSS removal via C-H4 + byte-identical S2 check. Evidence: `ContractGovernanceRetry` (W3C DTCG v2025.10 + Apple HIG audit-observable failure) + `review_contracts.md`.
- Q: Dynamic reduced-motion / OS validation? → A: SC-006 / device trial verifies `reduce` at load + after dynamic toggle (freeze decorative/progress/static banner); OS-level validation (Android `Settings→Accessibility→Remove animations`; iOS `Reduce Motion`; Windows `Show animations`) + `adb shell settings put global animator_duration_scale 0`. Evidence: `MotionAccessibilityResearch` + `web.dev` + `a11y-motion-review-report.md` (CHK045/049).

## Overview

Expand the reduced Lente Sinestésica (feature 005, token restyle + stamps only) to the complete design shown in the proposal: add all visual/UI elements including decorative wave graphics, title/section card using document metadata, wave visualization, and refined control layout — while keeping the single StatusView surface and respecting project constraints (closed token set where possible, reduced-motion, static assets).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver a Lente Sinestésica completa na tela em uso (Priority: P1)

Como usuário em uso (condução/olhar periférico), quero ver o cabeçalho, cartão de título/seção, visualização e controles conforme a proposta completa, com tokens SONA corretos e sem texto do corpo do documento.

**Why this priority**: Main visual experience of the complete design.

**Independent Test**: Open app, verify header "Concha Harmônica R", title card with document metadata/filename fallback, decorative elements, controls, no numeric rate painted.

**Acceptance Scenarios**:

1. **Given** app on StatusView with document loaded, **When** viewed, **Then** header shows Concha Harmônica R (no jade `#5AD6A8`, no celeste `#B9A6F5` paint), title card shows metadata or filename.
2. **Given** reduced motion preference, **When** viewed, **Then** decorative animations are paused/frozen per S5; no new continuous loops.
3. **Given** no title/section available, **When** viewed, **Then** glyph `—` with `aria-label="Sem título"` used (not "Seção: —").

### User Story 2 - Controles e progresso permanecem funcionais e eyes-free (Priority: P1)

Como usuário em movimento, quero que todos os controles permaneçam operáveis com feedback auditivo/háptico existentes; o visual completo não deve prejudicar discernibilidade ou adicionar texto numérico ao rate.

**Acceptance Scenarios**:

1. **Given** complete visual applied, **When** transport/rate changed, **Then** earcon confirmation works as before; rate remains value-blind (positional indicator only).
2. **Given** warning/failure states, **When** triggered, **Then** banner behavior unchanged (static failure, warning with TTL); no new alert/obtrusive elements.

### User Story 3 - Sem regressão na puridade de notificação/contratos (Priority: P1)

Como sistema, deve manter invariante: notificação permanece accent-only, sem body text/rate/voice/mark; closed sets preservados onde possível.

**Acceptance Scenarios**:

1. **Given** complete UI applied, **When** notification checked, **Then** only `--sona-voice` accent, no body/rate/voice/mark (SC-003).
2. **Given** surface inventory, **When** audited, **Then** no dead `display:none` corpses left (fix C-H4 if present); accessibility preserved.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Apply complete Lente Sinestésica visuals to single StatusView: decorative waves (static preferred), title card with metadata/filename fallback, refined controls, respecting closed token set where feasible.
- **FR-002**: Title card must use upstream authority `metadata_title.unwrap_or(filename)`; empty state glyph `—` with `aria-label="Sem título"`.
- **FR-003**: Rate remains value-blind (positional only), no painted numeric rate.
- **FR-004**: Respect reduced-motion: provide reduced-motion guard for decorative animations; maintain S5 (≤1 continuous loop). Subtle CSS animations allowed only if guarded.
- **FR-005**: Static assets preferred (SVG/CSS); CSS animations allowed only with reduced-motion guard and no scripts/foreignObject; must not create new continuous loops beyond S5 limits.
- **FR-006**: Notification remains accent-only (no body/rate/voice/mark).
- **FR-007**: Update contracts if tokens/layout change; maintain surface count = 2.

### Key Entities

- **LenteCompleta**: Visual configuration for complete design (waves, title card, layout) on StatusView.
- **TitleCard**: Metadata-driven card with fallback to filename; empty state handling.

## Success Criteria *(mandatory)*

### Measurable Outcomes

**SC-001** (objective — split sub-criteria):
- Presence / elements: header "Concha Harmônica R" (correct asset); decorative wave pseudo-element (`CSS ::before` on `.card`); title card (`metadata_title.unwrap_or(filename)` or glyph `—` + `.sr-only` "Sem título"); controls (transport + stop).
- Hex census (`C-H1`): audited hex values match `SonaToken` ledger (`--sona-void #0B0F1A`, `--sona-voice #F2F0E6`, `--sona-hush #A9B8CE`, `--sona-etwahl #8FD8E8`, `--signal-caution #FFD400`, `--signal-failure #FF4D63`, `--sona-stage #141B2D`, `--sona-boundary #55688F`, `--sona-harp #E8C46A`); banned: jade `#5AD6A8` + `#5FD0A0` (both variants, reconciled per D5 / §VII) and celeste `#B9A6F5` absent.
- Rate-blind (`C-H3`): no numeric rate value painted (`1.25×`, `2.0`) in `#progressLabel`; positional indicator permitted.
- Notification purity (`C-H3`): accent-only (`--sona-voice`); no body / rate / voice / mark (SC-003).
- Proposal reference non-normative: `DESIGN-SONA/assets/ui/ui-propuesta-2-lente-sinestesica.png`; unreconciled proposal elements explicitly excluded (numeric rate `1.25×`, extra interactive controls, green/violet wave colors outside token set).
**C-H3** (audit definition): rate-blind = no painted numeric rate value in `#progressLabel` / display; notification-purity = notification surface carries only `--sona-voice` accent, zero body / rate / voice identity / mark. Verified by host census (`C-H1` hex audit) + content scan of rendered surface.
- **SC-002**: No accessibility regressions (progressbar aria intact, empty slot aria-label, no alert misuse).
- **SC-003**: Notification purity unchanged (accent-only).
- **SC-004**: Reduced-motion respected; no new continuous loops beyond allowed.
- **SC-005**: Surface inventory valid; no `display:none` corpses (C-H4 passes); aria inventory correct (C-H5 passes).
**SC-006** (objective): Device trial verifies: visual elements match objective criteria from `SC-001`; decorative pseudo-element renders correctly on device; reduced-motion guard activates (`prefers-reduced-motion: reduce`) — wave animation disabled (`animation: none`), progress bar static (`#progress-fill`), banner animation disabled (`.reason-banner.warning`); title card displays correctly with document metadata or filename; section title shows correct value or empty-state (`.sr-only` announces "Sem título"); no new interactive elements added; focus order unchanged; no dead `display:none` elements (`C-H4`); surface inventory unchanged (`C-H4`); notification remains accent-only (`SC-003`).

## Assumptions

- Decorative waves can be static SVG or CSS-based without RMS instrumentation.
- Title/section metadata available or filename fallback sufficient.
- Can adapt within single-column layout without breaking existing controls.
- Token set may need minimal coordinated additions only if unavoidable; prefer reuse.
## Edge Cases (derived from clarifications.md Q7/Q8/Q9)
- Decorative wave CSS failure: `animation:none`, layout preserved (`::before` pseudo-element remains), no broken-image / alert; graceful degradation per clarification Q9.
- Long filenames / empty title: `overflow:hidden; text-overflow:ellipsis; white-space:nowrap; flex:min-width:0`; vertical budget ≤3 lines; banner fixed-height; empty-state glyph `—` with `.sr-only` "Semítulo" per FR-002 / clarification Q8.
- Reduced-motion dynamic toggle: freeze decorative / progress / banner (`animation:none`) at load and after OS-level toggle (`prefers-reduced-motion: reduce` + `matchMedia`); no new loops after freeze (SC-004 / clarification Q3).
- Alert misuse guard: `assertive`/`alert` ONLY for critical/time-sensitive errors; never on interactive elements; never auto-disappearing; never frequent/repeating (clarification Q5 / SC-002).
