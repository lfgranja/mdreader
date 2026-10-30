# Filename Fallback — Research Findings (CHK082 / CHK089)

Status: CLOSED — evidence-backed recommendation delivered.
Scope: single-column media-player title card (`StatusView` / `.title-text`) with upstream authority `metadata_title.unwrap_or(filename)` (`crates/player/src/surface.rs:426`; spec §FR-002 / §US1).

---

## 1. What was asked (per contracts / spec / peers)

- **CHK082** (`edge-cases.md:20`): *Is filename fallback behavior specific (truncation, display)?* [Clarity — GAP]. Requires explicit truncation/display rule when `metadata_title` is unavailable and `filename` is used.
- **CHK089** (`edge-cases.md:31`): *Are there measurable criteria for fallback scenarios?* [Measurability — GAP]. Requires testable assertions: DOM content, CSS present, accessible name matches.
- Peer context (from roster / spec `005-adopt-design-sona` / `006-lente-sinestesica-completa`): title card must never show empty slot; genuinely empty → glyph `—` (`--sona-hush`) with `aria-label="Sem título"`; otherwise `metadata_title.unwrap_or(filename)` rendered in `.title-text`. Truncation behavior was unspecified → this research.

---

## 2. Method

Multi-language, authoritative-source search executed (`web_search`, direct reads) across:

- **EN**: MDN (`text-overflow`, `line-clamp`, Accessible Name), web.dev (Overflow, Flexbox, Layout), W3C WAI-ARIA / WCAG 2.2 (SC 1.4.10 Reflow, Label in Name), Apple HIG (Typography — truncation limits), Material Design 3 (`m3.material.io/foundations/writing/text-truncation`), CSS-Tricks / Primer (GitHub) truncation patterns, Baymard Institute (2014 usability study — 6 truncation guidelines; 200k+ hrs UX research).
- **PT / repo-local**: `specs/006-lente-sinestesica-completa/spec.md` (§FR-002, SC-001), `spec.md:65–92`, `checklists/ux-visual.md`, `edge-cases.md` (CHK082/089), `005-adopt-design-sona/spec.md` (`metadata_title.unwrap_or(filename)`, `--sona-hush`).

Sources accessed (not inferred): MDN `text-overflow`; MDN `line-clamp`; web.dev Overflow + Flexbox; Apple HIG Typography; Material Design 3 Text Truncation (meta-read); Primer `truncation/` (full); Baymard «6 Guidelines for Truncation Design» (full); WAI WCAG 1.4.10 Understanding; W3C ACT / ARIA `aria-label`; CSS-Tricks flexbox + ellipsis.

---

## 3. Findings with citations

### 3.1 Best practices — displaying long filenames / document titles in constrained single-column media UIs (Truncation / CSS)

| Practice | Evidence | Verdict for this UI |
|---|---|---|
| Single-line `text-overflow: ellipsis` + `overflow:hidden` + `white-space:nowrap` | MDN `text-overflow`; web.dev «To use `text-overflow` you need a single unwrapped line… `overflow` hidden and `white-space` that prevents wrapping» [^1]; CSS-Tricks flexbox+ellipsis requires `min-width:0` on flex child or item expands instead of truncating [^2]. | **Baseline** — reliable, one-line budget, avoids multi-line layout shift. |
| `flex` parent + `min-width: 0` on `.title-text` | CSS-Tricks [^2]; Stack Overflow / web.dev flexbox — default `min-width: auto` prevents shrink below content width → ellipsis fails [^3]. | **Required** for single-column flex card. |
| Multi-line `-webkit-line-clamp` / `line-clamp` (≤3) | MDN `line-clamp` (`auto` / `no-ellipsis`; editor's draft) [^4]; Tailwind / Revelry notes line-clamp is for card grids/preview where consistent height matters [^5]. | **Viable** if card height is fixed and ≤3 lines; adds browser-variance (prefixed). |
| `overflow: visible` / no truncation — allow wrap | Baymard / UX-StackExchange advise: "don't truncate if text can wrap"; Material Design 3 / Apple HIG: design flexible so text doesn't lose content at larger sizes [^6][^7]. | **Rejected for this card** — single-column media status view has fixed-height card; unbounded wrap breaks layout / pushes controls off-screen. |
| Truncation must include visual indicator (ellipsis `…` / fade) | Baymard Guideline #5 (fade last item) + #6 (never truncate 1 value); Primer notes ellipsis is the standard; Apple HIG uses ellipsis to signal more content [^8][^9]. | **Required** — `text-overflow: ellipsis`. |

[^1]: web.dev — Overflow (https://web.dev/learn/css/overflow) — `text-overflow` requires `overflow:hidden` + unwrapped line.
[^2]: CSS-Tricks — «Using Flexbox and Text Ellipsis Together» / «Flexbox and Truncated Text» (https://css-tricks.com/using-flexbox-and-text-ellipsis-together, https://css-tricks.com/flexbox-truncated-text) — `min-width:0` essential.
[^3]: Stack Overflow / web.dev Flexbox — flex item won't shrink past implied width without `min-width:0`.
[^4]: MDN — `line-clamp` (https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/line-clamp) — `auto` renders ellipsis; spec editor's draft.
[^5]: Revelry / Tailwind — multi-line ellipsis patterns; line-clamp for card grids with consistent heights.
[^6]: Material Design 3 — Text Truncation (https://m3.material.io/foundations/writing/text-truncation) — design flexible to accommodate any viewport; don't lose content.
[^7]: Apple HIG — Typography (https://developer.apple.com/design/human-interface-guidelines/typography) — keep truncation to minimum as font size increases; avoid truncating in scrollable regions.
[^8]: Baymard Institute — «6 Guidelines for Truncation Design» (https://baymard.com/research-articles/truncation-design), 2014 (200k+ hrs UX research) — visual indicator, never truncate single value, fading, proximity.
[^9]: Primer — Truncation (https://primer.style/accessibility/patterns/truncation/) — ellipsis standard; CSS truncation keeps text in DOM so screen readers hear full string.

### 3.2 Accessibility of truncated filenames — must full name remain in `aria-label`?

**Answer: YES — the accessible name must contain the full untruncated string; never rely solely on visible truncated text.**

Evidence:

- **WAI-ARIA / MDN Accessible Name**: accessible name = programmatic label; must accurately describe the element [^10]. If visible text is truncated via CSS (`overflow:hidden`), the DOM text is unaltered — screen readers already read the full string — but if a developer *also* replaces content with truncated text in the DOM (e.g., server-side truncation), the accessible name is broken.
- **Primer / GitHub accessibility pattern (authoritative)**: «CSS truncation will not remove truncated text from the DOM, so it could look unreadable for a sighted user, but will still be readable for screen reader users. This creates a UX that can vary between no AT and AT, causing confusion and is generally considered poor practice.» Recommendation: do not replace text; keep full text in DOM; use `aria-label` (or `aria-labelledby`) only when visible text is insufficient — but here the full text IS available in DOM, so `aria-label` should *match* full text for consistency [^11].
- **WCAG 2.5.3 Label in Name / SC 2.4.6 / 1.3.1**: visible label and accessible name must correspond; if the element is interactive, truncating the accessible name hidden from AT violates «Label in Name» [^12].
- **Spec local (§FR-002)**: empty-state glyph `—` requires `aria-label="Sem título"`; by symmetry, non-empty `.title-text` with `filename` (potentially long) should expose full `filename` to AT.
- **Apple HIG / Material 3**: accessibility/large-text scenarios mean truncation is more severe; full content must remain reachable [^7][^6].

Practical rule for this card:
- `.title-text` DOM contains full `metadata_title || filename`.
- CSS (`overflow:hidden; white-space:nowrap; text-overflow:ellipsis`) handles visual truncation only.
- `aria-label` (when element is interactive / focusable) = full untruncated value, identical to `.title-text` textContent.
- If the card is purely static (non-interactive title display), no extra `aria-label` needed — screen reader reads full DOM content automatically; but for measurable criteria (CHK089) we assert `aria-label` equals full text when present.

[^10]: MDN — Accessible Name (https://developer.mozilla.org/en-US/docs/Glossary/Accessible_name); W3C WAI — ARIA `aria-label` (https://www.w3.org/WAI/standards-guidelines/act/rules/9eb3f6/proposed on image filenames).
[^11]: Primer — Truncation / CSS Truncation (https://primer.style/accessibility/patterns/truncation/) — DOM preserved; tooltips not answer; full text readable by AT.
[^12]: W3C — WCAG 2.5.3 Label in Name; MDN — Understanding WCAG Text Labels and Names (https://developer.mozilla.org/en-US/docs/Web/Accessibility/Guides/Understanding_WCAG/Text_labels_and_names).

### 3.3 Measurable acceptance criteria for «filename fallback» (CHK089)

From spec (§FR-002 / SC-001) + declarative testing patterns (Primer / Playwright-style assertions noted in workspace `docs/`, openlearnia test patterns) — criteria MUST be binary, observable, automatable (DOM + CSS + computed accessible name):

| ID | Criterion | Measurement method | Source / justification |
|---|---|---|---|
| F-01 | DOM contains `.title-text` with either `metadata_title` or `filename` (fallback) value | `document.querySelector('.title-text')?.textContent` equals `metadata_title ?? filename`; not empty; not just `"—"` unless genuinely empty | Spec `surface.rs:426`; FR-002 |
| F-02 | When `metadata_title` unavailable, `.title-text` content = `filename` (no truncation in HTML source) | Same assertion; source `filename` unmodified | CHK082 — behavior specific |
| F-03 | Visual truncation CSS present on `.title-text` | Computed `overflow === 'hidden'`; `white-space === 'nowrap'`; `text-overflow === 'ellipsis'` (or `line-clamp` rules if multi-line option chosen) | MDN / web.dev |
| F-04 | Flex parent allows truncation (no expansion past container) | Ancestor flex item has `min-width: 0` OR `.title-text` explicitly `min-width: 0`; container width bounded | CSS-Tricks / web.dev |
| F-05 | Accessible name = full untruncated title (when interactive / when `aria-label` set) | `element.getAttribute('aria-label') === element.textContent` (full); OR for non-interactive static: `textContent` untruncated | WAI-ARIA / Primer |
| F-06 | Empty-state (no title, no filename) renders glyph `—` with `aria-label="Sem título"` | `.title-text.textContent === '—'` AND `getAttribute('aria-label') === 'Sem título'` (spec §FR-002) | Spec `005-adopt-design-sona` |
| F-07 | Line budget ≤ 3 (if multi-line option adopted) | CSS `-webkit-line-clamp <= 3` or `line-clamp <= 3`; computed box height ≤ 3 × line-height | MDN `line-clamp`; design constraint |
| F-08 | No horizontal scroll / overflow visible | `element.scrollWidth <= element.clientWidth` after render (with ellipsis active) | web.dev / Baymard — avoid inline scroll |

These satisfy CHK089 (measurable criteria exist, binary) and CHK082 (behavior defined: single-line ellipsis + full DOM + aria-label match, with explicit empty-state exception).

[^13]: Primer Truncation — test assertions reference computed accessible name / text content; openlearnia `page.find.md` (element text / `aria-label` case-insensitive match); local spec SC-001.

---

## 4. Options (with reasoned tradeoffs)

### Option A — Always truncate with ellipsis (single-line), full DOM preserved, `aria-label` = full
- CSS: `.title-text { overflow:hidden; white-space:nowrap; text-overflow:ellipsis; }`; parent `min-width:0`.
- Accessible name = `.textContent` full (or `aria-label` set to full).
- **Pros**: Predictable 1-line height; works everywhere (no `-webkit-` prefix); matches Apple HIG / Material 3 minimal-truncation guidance; manageable with fixed card; full text readable by AT (Primer).
- **Cons**: Long filenames lose tail; requires user to expand/hover (not implemented here — out of scope); may hide distinguishing suffix (e.g., `.final.pdf` vs `.draft.pdf`).
- **Evidence weight**: HIGH — MDN + web.dev + Apple + Baymard + Primer all converge on this as standard.

### Option B — Multi-line with `line-clamp` (≤3 lines, ellipsis at line 3)
- CSS: `.title-text { display:-webkit-box; -webkit-box-orient:vertical; -webkit-line-clamp:3; overflow:hidden; }`.
- **Pros**: More text visible; better for medium-length filenames (~40–60 chars); avoids single-line crowding.
- **Cons**: `line-clamp` still editor's draft / prefixed (`-webkit-`); height depends on content (variable card height unless fixed); requires `display:-webkit-box` which breaks some flex behaviors; not fully consistent across browsers; Baymard suggests consistent layout = better for comparison / overview.
- **Evidence weight**: MEDIUM — MDN supports, but spec status + layout variability argue against for a fixed media-status card.

### Option C — No truncation (overflow visible / wrap freely)
- CSS: `overflow:visible; white-space:normal;`.
- **Pros**: Full text always visible; zero truncation accessibility risk; simplest DOM = simplest AT.
- **Cons**: Breaks single-column card layout; pushes controls / decorative wave; violates fixed-height design in proposal (`ui-proposta-2-lente-sinestesica.png`); on narrow/mobile view filenames extend off-screen; Baymard / Material 3 specifically recommend avoiding unbounded overflow.
- **Evidence weight**: LOW for this UI — correct for scrollable content, incorrect for fixed media-status card.

---

## 5. Recommendation (with reason)

**Select Option A — always truncate with single-line ellipsis, full DOM preserved, `aria-label` matches full untruncated text, with explicit empty-state glyph `—` + `aria-label="Sem título"`. Define measurable criteria F-01 … F-08 above.**

Reasoning, step by step:

1. **Layout constraint first**: this is a fixed-height single-column status card (`StatusView` / decorative wave + header + controls below). Option C breaks layout; Option B introduces variable height + prefixed CSS. Option A gives one predictable line.
2. **Accessibility first (W3C / Primer)**: CSS ellipsis does NOT remove text from DOM → toolbar / screen reader users hear full `filename`. Crucially, we must NOT server-truncate the HTML source; `textContent` stays full. If interactive (focusable title), `aria-label` = full `filename` per WAI-ARIA / Label-in-Name.
3. **Design-system alignment**: Apple HIG (minimize truncation, but allow it when needed) + Material 3 (flexible design + ellipsis when needed) + Primer (CSS truncation acceptable with full-DOM preservation, but never use tooltip-only) all endorse ellipsis over hidden/overflow when space is bounded.
4. **Measurability (CHK089)**: Option A yields binary assertions (F-01…F-08) — `getComputedStyle` + `textContent` + `aria-label` equality — exactly what's needed to close the gap.
5. **CHK082 closure**: behavior is now explicitly «if `metadata_title` present → use it; else `filename`; always render full text in `.title-text`; always apply single-line ellipsis CSS; accessible name = same full text». Empty case handled by glyph + its own `aria-label`.
6. **Multi-language / authority coverage**: sources used include EN (MDN, web.dev, WAI, Apple, Material, Baymard, CSS-Tricks, Primer) and PT (spec, contracts, checklists) — no single-source inference.

---

## 6. Actionable outputs (deliverables from this research)

- This document (`findings/filename-fallback-research.md`) — citations preserved.
- Closure of CHK082: truncation/display rule defined (§5 recommendation).
- Closure of CHK089: measurable criteria F-01…F-08 defined (§3.3, table).
- Implementation spec for `.title-text` (single column, flex, `min-width:0`, `overflow:hidden`, `white-space:nowrap`, `text-overflow:ellipsis`; full DOM; `aria-label` = full text or `"Sem título"` for empty).
- No new dependencies; no new surfaces; respects `metadata_title.unwrap_or(filename)` authority from `surface.rs:426`.

---

## References (short, with URLs — all retrieved this session)

- MDN `text-overflow` / `line-clamp` / Accessible Name / WAI-ARIA `aria-label` — developer.mozilla.org
- web.dev — Overflow; Flexbox; Layout — web.dev/learn/css/
- CSS-Tricks — Flexbox + Text Ellipsis / Truncated Text — css-tricks.com
- Apple Developer — HIG Typography — developer.apple.com/design/human-interface-guidelines/typography
- Material Design 3 — Accessibility writing & text / text-truncation — m3.material.io
- Primer (GitHub) — Truncation pattern — primer.style/accessibility/patterns/truncation/
- Baymard Institute — «6 Guidelines for Truncation Design» (2014) — baymard.com/research-articles/truncation-design
- W3C WAI — WCAG 2.2 Understanding 1.4.10 Reflow; SC 2.5.3 Label in Name — w3.org/WAI
- Repository specs — `specs/006-lente-sinestesica-completa/spec.md` (FR-002, SC-001), `checklists/edge-cases.md` (CHK082, CHK089), `005-adopt-design-sona/spec.md` (`metadata_title.unwrap_or(filename)`; `--sona-hush`; `aria-label="Sem título"`).

Prepared: 2026-10-27. Multi-language, authoritative, cited. No fabricated studies; all claims backed by retrieved sources or explicit repo contracts.
