# Structured Review: a11y-motion.md (CHK027–CHK049)

**Source files read:**
- `specs/006-lente-sinestesica-completa/checklists/a11y-motion.md`
- `specs/006-lente-sinestesica-completa/spec.md` (FR-001..FR-007, US1-US3, SC-001..SC-006)
- `specs/006-lente-sinestesica-completa/plan.md` (Constitution Check §S5, S1, S3; Phase 0, Phase 1)
- `.specify/memory/constitution.md` (amendment records; references S5 via plan/research citations)
- `specs/005-adopt-design-sona/contracts/status-surface.md` (S1-S7, esp. S4 aria contract, S5 movement contract)
- `contracts/surface.md` (surfaces, notification purity, value-blindness, FR-002, FR-028/029)

**Note:** The checklist item references (e.g. Spec §FR-004, Plan §S5) are taken verbatim from `a11y-motion.md`. Evidence below quotes from `spec.md`, `plan.md`, `contracts/status-surface.md`, and `contracts/surface.md`.

---

## Requirement Completeness

### CHK027 — PASS (with clarification note)
**Statement:** Are reduced-motion requirements fully specified (when to pause/freeze, what animations affected)? [Completeness, Spec §FR-004, §US1 Scenario 2]
**Evidence:**
- `spec.md` FR-004: "provide reduced-motion guard for decorative animations; maintain S5 (≤1 continuous loop)"; FR-005: "CSS animations allowed only with reduced-motion guard and no scripts/foreignObject; must not create new continuous loops beyond S5 limits."
- `plan.md` Constitution Check §S5: "≤1 continuous loop, reduced-motion guard, yellow pulse ≤3s, no pulse on failure, banner ≤200ms."
- `spec.md` US1 Scenario 2: decorative animations paused/frozen per S5; no new continuous loops.
- `spec.md` SC-006: guard activates (`prefers-reduced-motion: reduce`) — wave animation disabled (`animation: none`), progress bar static (`#progress-fill`), banner animation disabled (`.reason-banner.warning`).
**Assessment:** PASS — the trigger (`prefers-reduced-motion`) and effects (freeze/disable listed animations) are covered, though the spec relies on `plan.md`/contract for the full animation inventory (waves, pill, banner, progress fill). Not a gap, but clarification helps.
**Clarification question:** Should FR-004 enumerate the affected decorative animations explicitly (wave pseudo-element, progress fill, banner, pill) rather than relying solely on the S5 reference?

### CHK028 — GAP
**Statement:** Are ARIA requirements specified for all new elements (title card, waves container)? [Completeness, Gap]
**Evidence:**
- `spec.md` FR-002 defines title card data model (`metadata_title.unwrap_or(filename)`) and empty-state glyph `—` with `aria-label="Sem título"`. No `role`, `aria-label` for the title card when present, and no `aria-hidden`/label for decorative waves.
- `spec.md` SC-001 mentions `.sr-only` accessible label for empty state; SC-006 mentions `.sr-only` announces "Sem título" for empty title card. No other ARIA for title card.
- `specs/005-adopt-design-sona/contracts/status-surface.md` S4 covers interactive controls + progressbar + reason banner; does not cover new wave/title card elements.
- `plan.md` Phase 1 design mentions TitleCard data model but no ARIA contract extension.
**Assessment:** GAP — waves container has zero accessibility specification; title card only has an empty-state label, not a full ARIA contract.
**Clarification question:** Should the decorative wave container carry `aria-hidden="true"` (or a descriptive `aria-label`)? Should the title card have `aria-label` when populated, and a `role` (e.g., `region`, `group`)?

### CHK029 — PASS
**Statement:** Is empty title state accessibility specified (glyph `—` with `aria-label="Sem título"`)? [Completeness, Spec §FR-002, §US1 Scenario 3]
**Evidence:**
- `spec.md` FR-002: "Title card must use upstream authority `metadata_title.unwrap_or(filename)`; empty state glyph `—` with `aria-label="Sem título"`."
- `spec.md` US1 Scenario 3: "Given no title/section available, When viewed, Then glyph `—` with `aria-label="Sem título"` used (not 'Seção: —')."
- `spec.md` SC-001: "empty-state glyph `—` with `.sr-only` accessible label"; SC-006: ".sr-only announces 'Sem título'."
**Assessment:** PASS — unambiguous, consistent across FR, US, SC.

### CHK030 — PASS (preserved; no extension required)
**Statement:** Are progressbar ARIA requirements preserved/extended? [Completeness, Spec §SC-002]
**Evidence:**
- `spec.md` SC-002: "No accessibility regressions (progressbar aria intact, empty slot aria-label, no alert misuse)."
- `contracts/status-surface.md` S4: progress = `role="progressbar"` + `aria-valuenow`/`aria-valuetext`; no change proposed in 006 spec.
- `spec.md` SC-001 references progressbar aria; SC-006 verifies it unchanged.
**Assessment:** PASS — preserved; no new progressbar requirements are added by this feature, so "extended" is not required. If extension is expected, it is a minor gap (see clarification).
**Clarification question:** Should SC-002 explicitly confirm no new `aria-valuetext`/label changes, or does "intact" suffice for preservation only?

### CHK031 — GAP
**Statement:** Are keyboard/screen reader considerations addressed for eyes-free usage context? [Completeness, Gap]
**Evidence:**
- `spec.md` US2 (eyes-free) covers audio/haptic feedback and value-blind rate; no keyboard/screen-reader requirements for the new title card or decorative elements.
- `spec.md` SC-006 says "focus order unchanged" and "no new interactive elements added"; does not specify keyboard behavior for existing controls under the expanded visual layout.
- `contracts/status-surface.md` S4 covers interactive controls' `aria-label` + state, but does not extend to eyes-free usage beyond notification contracts.
**Assessment:** GAP — eyes-free context is acknowledged (US2, SC-006) but keyboard/tab-focus and screen-reader announcements for the full layout (title card, decorative elements, refined controls) are not explicitly required.
**Clarification question:** Should US2 or a new requirement specify keyboard navigation (tab order), screen-reader announcements for the expanded layout, and haptic feedback consistency for the full visual?

---

## Requirement Clarity

### CHK032 — PASS
**Statement:** Is "reduced-motion guard" explicitly defined (what triggers it, what happens)? [Clarity, Spec §FR-004, Clarifications]
**Evidence:**
- `spec.md` FR-004: "provide reduced-motion guard for decorative animations"; FR-005: "CSS animations allowed only with reduced-motion guard".
- `spec.md` SC-006 defines the trigger (`prefers-reduced-motion: reduce`) and effects (wave `animation: none`, progress bar static, banner disabled).
- `contracts/status-surface.md` S5: guard freezes pill at visible on-track position (`animation-play-state: paused` + static offset); invisible/opacity-zero is FAIL.
**Assessment:** PASS — trigger (`prefers-reduced-motion`) and actions (freeze/disable specific animations) are defined in acceptance criteria and contract.

### CHK033 — PASS
**Statement:** Is "pause/freeze" vs "remove" clearly distinguished for decorative animations? [Clarity, Spec §FR-004]
**Evidence:**
- `spec.md` US1 Scenario 2: "decorative animations are paused/frozen per S5; no new continuous loops" (freeze, not removal).
- `spec.md` SC-006: `animation: none` (disables animation) while keeping element visible/static; `#progress-fill` static; banner disabled.
- `contracts/status-surface.md` S5: freeze at visible on-track position; invisible/opacity-zero = FAIL, distinguishing freeze from removal/hiding.
**Assessment:** PASS — distinction is explicit: freeze = visible/static; removal/hiding = failure.

### CHK034 — UNCLEAR / GAP
**Statement:** Is "no alert misuse" quantified/specifically defined (what constitutes misuse)? [Clarity, Spec §SC-002]
**Evidence:**
- `spec.md` SC-002: "no alert misuse" — no definition of misuse, no count or role restriction specified.
- `contracts/status-surface.md` S4 mentions a single `role="status"` for reason banner and removal of the incompatible alert+polite pair; this implies misuse = adding extra `role="alert"` or duplicating banner roles. But `spec.md` does not restate this.
**Assessment:** UNCLEAR — the phrase is unquantified in `spec.md`.
**Clarification question:** Should SC-002 define "alert misuse" explicitly (e.g., no extra `role="alert"` elements, only one `role="status"` banner, count = 1)?

### CHK035 — PASS
**Statement:** Are animation loop constraints measurable (≤1 continuous loop, what counts as loop)? [Clarity, Spec §FR-004, Plan §S5]
**Evidence:**
- `spec.md` FR-004/FR-005: "≤1 continuous loop"; "must not create new continuous loops beyond S5 limits."
- `plan.md` §S5: same constraint.
- `contracts/status-surface.md` S5: continuous motion set = `{pill}`; census counts declaration occurrences inside audited file; fails on any loop outside the closed set. This makes it measurable.
**Assessment:** PASS — loop is defined (pill only allowed), measurement method is census in contract.

---

## Requirement Consistency

### CHK036 — UNCLEAR (spec relies on contract/plan for full S5 alignment; not all S5 rules restated in spec)
**Statement:** Do motion requirements align with S5 constraints (≤1 continuous loop, reduced-motion guard, yellow pulse ≤3s, no pulse on failure)? [Consistency, Plan §Constitution Check]
**Evidence:**
- `spec.md` FR-004 references S5; US1 Scenario 2 references S5; SC-004 says reduced-motion respected, no new loops.
- `plan.md` Constitution Check §S5 lists all constraints: ≤1 loop, reduced-motion guard, yellow pulse ≤3s, no pulse on failure, banner ≤200ms.
- `spec.md` does not mention yellow pulse or failure pulse rules; SC-001/SC-006 mention no pulse on failure indirectly (banner static, no new loops), but yellow pulse ≤3s is absent from the 006 spec.
**Assessment:** UNCLEAR — the spec aligns by reference but does not restate all S5 sub-constraints, making direct spec-level verification harder.
**Clarification question:** Should the spec restate the yellow-pulse (≤3s) and no-pulse-on-failure rules explicitly for feature 006, or is the S5 contract reference sufficient?

### CHK037 — PASS
**Statement:** Are a11y requirements consistent between status view and notification (notification must remain accessible)? [Consistency, Spec §FR-006, §US3]
**Evidence:**
- `spec.md` FR-006: notification remains accent-only (`--sona-voice` only, no body/rate/voice/mark).
- `spec.md` SC-003: notification purity unchanged (accent-only).
- `spec.md` US3: notification behavior unchanged; SC-006 verifies notification remains accent-only.
- `contracts/status-surface.md` S6: notification stays declarative mirror, no structural change; accessibility preserved by not changing the surface.
**Assessment:** PASS — consistent; notification not altered, so accessibility preserved.

### CHK038 — PASS
**Statement:** Do empty state requirements align across all scenarios (glyph + aria-label)? [Consistency, Spec §FR-002, §US1]
**Evidence:**
- `spec.md` FR-002: empty glyph `—` + `aria-label="Sem título"`.
- `spec.md` US1 Scenario 3: same glyph + label (not "Seção: —").
- `spec.md` SC-001/SC-006: `.sr-only` label announces "Sem título"; no conflicting definition elsewhere.
**Assessment:** PASS — single definition, consistent across FR/US/SC.

---

## Acceptance Criteria Quality

### CHK039 — UNCLEAR (partially measurable; "alert misuse" undefined blocks full verification)
**Statement:** Can SC-002 ("no accessibility regressions... progressbar aria intact, empty slot aria-label, no alert misuse") be objectively verified? [Measurability, Spec §SC-002]
**Evidence:**
- `spec.md` SC-002: three parts: (1) progressbar aria intact — verifiable via DOM/HTML inspection; (2) empty slot aria-label — verifiable (`aria-label="Sem título"`); (3) no alert misuse — undefined (see CHK034).
**Assessment:** UNCLEAR — two parts are objective; the third is not quantifiable without a definition of misuse.
**Clarification question:** Should SC-002 split "no alert misuse" into a measurable rule (e.g., exactly one `role="status"` banner, zero `role="alert"` outside it) and reference `contracts/status-surface.md` S4 for the exact count?

### CHK040 — PASS
**Statement:** Can reduced-motion compliance be tested objectively? [Measurability, Gap, Spec §SC-004]
**Evidence:**
- `spec.md` SC-004: "Reduced-motion respected; no new continuous loops beyond allowed."
- `spec.md` SC-006: explicit objective test — `prefers-reduced-motion: reduce` activates; wave `animation: none`; progress fill static; banner disabled; focus unchanged; surface inventory unchanged.
- `plan.md` §S5 and `contracts/status-surface.md` S5 provide census/count rules.
**Assessment:** PASS — SC-006 gives concrete observable conditions for verification.

### CHK041 — UNCLEAR (partial gap in criteria for title card accessibility)
**Statement:** Are there specific acceptance criteria for title card accessibility? [Measurability, Gap]
**Evidence:**
- `spec.md` SC-001: title card shows metadata/filename or empty-state `.sr-only` label.
- `spec.md` SC-006: title card displays correctly with document metadata/filename; empty-state `.sr-only` announces "Sem título"; no new interactive elements.
- No criteria specify `aria-label`, `role`, or screen-reader behavior for the non-empty title card; no keyboard focus criteria.
**Assessment:** UNCLEAR / partial GAP — criteria cover display and empty-state label, not full accessibility.
**Clarification question:** Should SC-006 include specific accessibility criteria for the title card (e.g., `aria-label` when title present, container `role`, or screen-reader announcement behavior)?

---

## Scenario Coverage

### CHK042 — PASS
**Statement:** Are requirements defined for reduced-motion preference enabled? [Coverage, Spec §US1 Scenario 2]
**Evidence:**
- `spec.md` US1 Scenario 2: "Given reduced motion preference, When viewed, Then decorative animations paused/frozen per S5; no new continuous loops."
- `spec.md` SC-006: guard activates under `prefers-reduced-motion: reduce`.
**Assessment:** PASS — covered by US scenario and SC verification.

### CHK043 — GAP
**Statement:** Are requirements defined for screen readers with decorative elements present? [Coverage, Gap]
**Evidence:**
- `spec.md` FR-005 mentions decorative waves (CSS pseudo-element `::before` on `.card`); no accessibility treatment defined for these decorative elements.
- No `aria-hidden`, `role="presentation"`, or screen-reader exclusion specified.
**Assessment:** GAP — decorative elements are not addressed for assistive technology.
**Clarification question:** Should decorative waves be marked `aria-hidden="true"` (or `role="presentation"`) for screen readers, or should they carry a brief descriptive label?

### CHK044 — GAP
**Statement:** Are requirements defined for high contrast mode if applicable? [Coverage, Gap]
**Evidence:**
- No mention of `prefers-contrast`, Windows High Contrast Mode, or high-contrast adjustments in `spec.md`, `plan.md`, or contracts.
- The feature relies on CSS tokens; no requirement addresses token visibility under high contrast.
**Assessment:** GAP.
**Clarification question:** Should high-contrast mode be addressed explicitly (e.g., decorative elements must remain visible, tokens must not rely solely on color for meaning)?

---

## Edge Case Coverage

### CHK045 — GAP
**Statement:** Is behavior defined if reduced-motion is toggled dynamically? [Edge Case, Gap]
**Evidence:**
- `spec.md` SC-006 tests static `prefers-reduced-motion: reduce` at load; no runtime toggle behavior defined.
- No media-query listener or dynamic freeze/unfreeze behavior specified.
**Assessment:** GAP.
**Clarification question:** Should the spec define behavior when `prefers-reduced-motion` changes at runtime (e.g., freeze/unfreeze animations dynamically, or only apply at initial render)?

### CHK046 — UNCLEAR / GAP
**Statement:** Is behavior defined for animations that would violate loop constraint? [Edge Case, Gap]
**Evidence:**
- `spec.md` FR-005 says "must not create new continuous loops beyond S5 limits."
- `contracts/status-surface.md` S5 says census fails on any loop outside closed set; does not specify a runtime failure/recovery behavior (e.g., disable animation, freeze at initial state, log error).
**Assessment:** UNCLEAR — prevention is defined (must not create), but the failure/recovery behavior is not.
**Clarification question:** Should the spec define a failure/recovery action when an animation violates the loop constraint (e.g., disable the animation, freeze at initial state, or fail verification)?

### CHK047 — GAP
**Statement:** Are focus management requirements specified for any interactive elements added? [Edge Case, Gap]
**Evidence:**
- `spec.md` SC-006 says "focus order unchanged" and "no new interactive elements added"; does not specify focus rules for existing elements in the expanded layout.
- `contracts/status-surface.md` S4 covers interactive control `aria-label` + state, but not focus management.
**Assessment:** GAP — since no new interactive elements are added, the gap is low-impact, but focus management is not explicitly required.
**Clarification question:** Should focus management be explicitly stated (e.g., "focus remains on existing interactive controls; decorative/title elements are not focusable; no new tab stops")?

---

## Dependencies & Assumptions

### CHK048 — UNCLEAR (assumption implied, not documented in spec)
**Statement:** Is CSS `prefers-reduced-motion` support assumed/documented? [Assumption, Gap]
**Evidence:**
- `spec.md` SC-006 uses `@media (prefers-reduced-motion: reduce)`; `plan.md` §S5 and `contracts/status-surface.md` S5 assume the media query mechanism.
- `spec.md` does not document the assumption that browsers/OS support this media query.
**Assessment:** UNCLEAR — mechanism used but assumption not stated as an assumption in the spec.
**Clarification question:** Should the spec document the assumption that browsers/OS support `prefers-reduced-motion`, and note any fallback if unsupported?

### CHK049 — GAP
**Statement:** Is the assumption about browser/OS reduced-motion behavior validated? [Assumption, Gap]
**Evidence:**
- No validation/test mentioned in `spec.md` or `plan.md` confirming OS-level reduced-motion settings (e.g., Android/iOS accessibility toggles, Windows "Show animations" setting) are respected.
- SC-006 tests the CSS media query only; no device-level or OS-level validation requirement.
**Assessment:** GAP.
**Clarification question:** Should the spec include a validation/test confirming OS-level reduced-motion settings are respected (e.g., device trial under Android/iOS reduced-motion accessibility settings)?

---

## Summary Table

| ID | Verdict | Evidence Key | Clarification Question (UNCLEAR/GAP) |
|---|---|---|---|
| CHK027 | PASS | FR-004, FR-005, US1 Sc2, SC-006, Plan §S5 | Should FR-004 enumerate affected animations explicitly? |
| CHK028 | GAP | FR-002 (empty-state only), SC-001/SC-006 (empty-state only), no wave ARIA | Wave `aria-hidden`? Title card `role`/label when present? |
| CHK029 | PASS | FR-002, US1 Sc3, SC-001, SC-006 | — |
| CHK030 | PASS (preserved) | SC-002, SC-001, SC-006, contracts/status-surface.md S4 | Does "intact" cover preservation only, not extension? |
| CHK031 | GAP | US2 (audio/haptic), SC-006 (focus unchanged), no keyboard/screen-reader spec for new layout | Add keyboard/tab and screen-reader requirements for expanded layout? |
| CHK032 | PASS | FR-004, FR-005, SC-006, contracts/status-surface.md S5 | — |
| CHK033 | PASS | US1 Sc2, SC-006, contracts/status-surface.md S5 | — |
| CHK034 | UNCLEAR / GAP | SC-002 only says "no alert misuse"; contracts/status-surface.md S4 implies misuse = extra `role="alert"`/duplicate banners | Define misuse explicitly (e.g., exactly one `role="status"`, zero extra `alert`)? |
| CHK035 | PASS | FR-004/FR-005, Plan §S5, contracts/status-surface.md S5 | — |
| CHK036 | UNCLEAR | FR-004 references S5, but yellow pulse ≤3s and failure pulse rules absent from spec text | Restate yellow-pulse and no-pulse-on-failure rules in spec? |
| CHK037 | PASS | FR-006, SC-003, US3, contracts/status-surface.md S4/S6 | — |
| CHK038 | PASS | FR-002, US1 Sc3, SC-001, SC-006 | — |
| CHK039 | UNCLEAR | SC-002 (two parts measurable; "alert misuse" undefined) | Split into measurable rule referencing S4 banner contract? |
| CHK040 | PASS | SC-004, SC-006, contracts/status-surface.md S5 | — |
| CHK041 | UNCLEAR | SC-001/SC-006 cover display and empty-state label only | Add accessibility criteria for non-empty title card? |
| CHK042 | PASS | US1 Sc2, SC-006 | — |
| CHK043 | GAP | FR-005 (decorative waves mentioned, no screen-reader spec) | `aria-hidden="true"` or descriptive label for waves? |
| CHK044 | GAP | No high-contrast references in spec, plan, contracts | Address `prefers-contrast` or high-contrast token visibility? |
| CHK045 | GAP | SC-006 tests static `prefers-reduced-motion`; no runtime toggle spec | Define dynamic toggle behavior? |
| CHK046 | UNCLEAR / GAP | FR-005 prevents violation; contracts/status-surface.md S5 defines census failure but not runtime recovery | Define failure/recovery action when loop constraint violated? |
| CHK047 | GAP | SC-006: focus unchanged; no explicit focus-management spec for existing/new elements | Explicitly state focus rules (no new focusable elements, no tab-order change)? |
| CHK048 | UNCLEAR | Media query used in SC-006; assumption not documented in spec | Document assumption about `prefers-reduced-motion` support? |
| CHK049 | GAP | No OS-level reduced-motion validation/test in spec or SC-006 | Add device-level validation for OS reduced-motion settings? |

---

## Overall Assessment for Main Agent

- **PASS:** CHK027, CHK029, CHK032, CHK033, CHK035, CHK037, CHK038, CHK040, CHK042.
- **UNCLEAR (needs clarification to become PASS):** CHK030 (preservation vs extension of progressbar ARIA), CHK034 (define alert misuse), CHK036 (restating full S5 rules in spec), CHK039 (measurable criteria blocked by undefined misuse), CHK041 (title card accessibility criteria), CHK046 (loop violation recovery), CHK048 (document media-query assumption).
- **GAP (spec missing coverage):** CHK028 (ARIA for waves / full title card), CHK031 (keyboard/screen-reader for eyes-free full layout), CHK043 (screen readers with decorative elements), CHK044 (high contrast), CHK045 (dynamic reduced-motion toggle), CHK047 (focus management specification), CHK049 (OS-level reduced-motion validation).

**Recommendation:** Before closing the checklist, resolve the GAP/UNCLEAR items via clarification questions (listed per row above). The most impactful gaps are CHK028 (ARIA for new decorative/title elements), CHK031 (eyes-free keyboard/screen-reader), CHK034/CHK039 (quantify alert misuse), CHK043 (decorative elements for screen readers), and CHK044/CHK045/CHK049 (high contrast, dynamic toggle, OS validation). Once clarification answers are incorporated, the checklist can be completed.
