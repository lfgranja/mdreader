# Clarification batch — CHK019 · CHK020 · CHK024 · CHK025 · CHK026
Prepared 2026-10-27 · source spec `005-adopt-design-sona` · references: `spec.md` US2/SC-003, `tasks.md` T024–T026, `data-model.md`, `crates/player/src/surface.rs:426`, `contracts/status-surface.md` / `surface.md`

Format per item (required): (1) Check code + scope phrase; (2) Clarified state / what is confirmed / what is NOT changed; (3) Evidence anchor (file / line / token / contract); (4) Verification format (deterministic host check / device-trial / document-record); (5) Cross-ref to upstream dependency / assumption / task.

---

## CHK019 — Control states during visual expansion (transport + stop buttons keep focus rings; rate dots positional unchanged; no new interactive elements)

(1) Scope phrase: transport + stop buttons retain focus rings during expansion; rate dots stay positional (value-blind gold dot on 7-rung ladder, `--sona-harp`); no new interactive elements added.

(2) Clarified: Focus-ring paint (`--sona-etwahl` `#8FD8E8`) preserved per `tokens.md` §1/§2 and clarification E1 (`clarify-e1-etwahl-role-allocation.md` 18–55) — allocate to TalkBack/keyboard ring + transport "playing" state; not reserved/non-painted, not reallocated to progress fill (already `--sona-voice` per C1). Rate indicator remains positional gold dot; numeric rate text stays excluded (FR-008 / value-blind decision 2026-10-23; `clarify-c2-celeste-token-role.md`). No new button, no second channel, no added interactive node — `FR-009` / `contracts/earcon-mapping.md` E1 keep the 7 confirmation types fixed.

(3) Evidence anchors: `spec.md` FR-001/FR-008/SC-001 item (ii) closed inventory; `tokens.md` §1 note "usar `--sona-etwahl` para ring de focus"; `status_view.html` `#speedDots` `aria-hidden="true"` (D5 clarification); `data-model.md` rate = positional dot; `spec.md` 124 `metadata_title.unwrap_or(filename)` (separate, not interactive); `contracts/earcon-mapping.md` E2.5 single-channel / E2.2 truncate-replace (no second interactive path).

(4) Verification format: host census over audited set (`status_view.html`, `status_view.rs`, `tokens.md`, `components.md`) — confirm (a) `--sona-etwahl` appears only for focus ring + playing-state, never as progress fill / rate / caution / failure; (b) `#speedDots` keeps `aria-hidden` and carries no `aria-label`; (c) no new element tag in inventory (SC-001 item ii fail-closed). Device trial SC-003 / SC-006: focus ring visible at `#8FD8E8` on raised `--sona-stage` card under keyboard/AT; dots unchanged.

(5) Cross-ref: depends on E1 (etwahl role), C1 (voice-fill), C2 (gold dot / celeste vacancy), FR-009 / E2.5 (no second interactive channel). No new spec/contract edit required — role already declared upstream; document by recording the focus-ring allocation explicitly in FR-001 role map (same class as E1 recommendation 55).

---

## CHK020 — Failure / warning states with complete visuals (banner .reason-banner static, fixed-height; #signal-failure #FF4D63 / #signal-caution #FFD400; TTL ≤3s; no alert/assertive)

(1) Scope phrase: `.reason-banner` (`#reasonBanner`) static, fixed-height; failure `#FF4D63` (`--signal-failure`) / caution `#FFD400` (`--signal-caution`); TTL ≤3 s; no assertive `role` / no `aria-live="assertive"`; banner is status-not-alert.

(2) Clarified: Banner is static render (not animated injection) — `renderReason` writes `textContent` to pre-existing DOM node (`status_view.html` 429+; `research/clarify-d5-accessibility-semantics.md` 11). `role="alert"` + `aria-live="polite"` mismatch (line 252) must be resolved to `role="status"` / `aria-live="polite"` (warning, 3 s TTL) — NOT assertive, since failure is latching (`reason_ttl_ms = null` per F4 / ADR-002 `docs/adr/adr-002-earcon-at-focus-loss.md`) and cleanup only by new state/session/app-restart (`spec.md` / T026). Colors locked: warning = `#FFD400` exclusively (pause, 3 s only); failure = `#FF4D63` exclusively (permanent loss). Green `#5AD6A8` prohibited (C3 / `spec.md` 156 / `contracts/surface.md` 36); no third signal color.

(3) Evidence anchors: `spec.md` SC-001 census 10 hexes (+ 2 pins); `tokens.md` §3 `--signal-caution` / `--signal-failure`; `research/clarify-d2-warning-failure-colors.md` 11–87 (role-lock, exclusion of gold from warning, no `#5FD0A0` revival); `clarify-d5-accessibility-semantics.md` 11 (pre-loaded banner, `renderReason` `textContent`, `role`/`aria-live` mismatch); `tasks.md` T026 (F4 `reason_ttl_ms = null`); `status_view.html` 252 / 429.

(4) Verification format: host audit — deterministic check over `status_view.html` + `tokens.md`: (i) only `#FFD400` / `#FF4D63` appear in banner/line contexts; (ii) no `#FFFFFF` / green / celeste in banner; (iii) banner node present at load (not JS-injected); (iv) `aria-live` ≠ assertive; (v) `reason_ttl_ms` referenced in ADR / contract as null/not-timer-bound (CSS-bound, not JS timer — T208 / task 208 evidence). Device trial SC-003 §3.1: 3 s warning line visible, no body-text leak, pulsation of gold dot only (not banner).

(5) Cross-ref: T026 (ADR-002 failure latch), T022 (banner stamps), D2 (color roles), D5 (a11y semantics), F4 contract. No new color token — both pins already named; only documentation/role-lock confirmation required.

---

## CHK024 — Metadata / filename fallback validated (assumption in T002 / spec.md / data-model.md TitleCard: title=String with unwrap_or(filename); empty→"—" + .sr-only; no new assumption needed if flow verified)

(1) Scope phrase: title card uses `metadata_title.unwrap_or(filename)` (`crates/player/src/surface.rs:426`); genuinely empty title slot → stable glyph `"—"` (`--sona-hush`) with `.sr-only`; no new assumption if upstream flow verified.

(2) Clarified: Fallback is upstream-authoritative (feature 001 FR-029 / `surface.rs:426`), not a new 005 assumption. `title: String` with unwrap; empty = glyph only (no copy, no placeholder text). No new data-model change: `data-model.md` / `spec.md` 124 already record `metadata_title.unwrap_or(filename)`; `status_view.html` renders empty via existing `renderDocumentTitle()`. Assumption #8 (hybrid amendments to 001/004 via T024) covers point fixes, not this fallback — it is already verified by T002 baseline (`tasks.md` 44 / T003 gate). No new assumption to add; only confirm flow preserved through T022 stamps (metadata card on `--sona-stage`).

(3) Evidence anchors: `crates/player/src/surface.rs:376–426` (`metadata_title: Option<String>` → `title: metadata_title.unwrap_or(filename)`); `spec.md` 124 (Q on empty slot / stable placeholder); `tasks.md` T003 / T002 baseline; `data-model.md` entities (Document Title, Current Section); `tasks.md` T022 (title card + section preserved with same upstream semantics).

(4) Verification format: host test `prop_surface.rs` (strategy `metadata_title_strategy()`) + integration `test_lockscreen_us4.rs` (line 1007 / 113); deterministic: loaded doc → `surface.title == METADATA_TITLE`; empty/null → fallback to `filename`; render outputs `"—"` glyph with `.sr-only` when genuinely empty (no second text node). No new contract / data-model edit.

(5) Cross-ref: T002 baseline, T003 gate, T022 (card wiring), Assumption #8 (hybrid amendments only to 001/004 contracts — not to fallback semantics). If `surface.rs:426` flow unchanged, no new assumption required.

---

## CHK025 — Single-column layout adaptation (existing .controls + .speed-control + .progress preserved; decorative wave at top; no multi-column added; overflow handled)

(1) Scope phrase: `.controls` (transport grid) + `.speed-control` (7-rung ladder / `#speedDots`) + `.progress` (bar / pill `--sona-voice`) preserved; decorative wave only at top (static SVG / CSS, not interactive); no multi-column layout added; overflow handled by existing single-column flex / grid.

(2) Clarified: FR-008 / `spec.md` 140–142 / `contracts/feature-005.md` / `clarify-c3-two-field-grid.md` 3–54: "2-campo" is descriptive identity, not normative grid; normative geometry is single-column preserved (no two-column grid introduced — C6). Existing inventory: header Concha "R", static `--sona-stage` metadata card, progress bar, transport (`grid-template-columns: repeat(3,1fr)` in `status_view.html` — 3-column transport only, not page grid), ladder, reason line, resume offer. Decorative wave (top only, static) permitted by concept docs (`.specify/assessments/lente-sinestesica-completa/concept.md` 16 / `DESIGN-SONA/components.md`) but must not become a second visual column or interactive region. Overflow: handled by single-column flex + fixed-height banner / static card — no new responsive grid.

(3) Evidence anchors: `spec.md` FR-008 ("grade 2-campo = apelido descritivo… geometria normativa = single-column"); `clarify-c3-two-field-grid.md` 32 (two-field definition contradicts value-blind + inventory); `status_view.html` layout (header / card / progress / transport / ladder); `tasks.md` T019 / T021 stamps (same elements, no new grid); `data-model.md` (no layout entity change).

(4) Verification format: host inventory audit (`surface_audit/inventory.rs` / `contracts/surface-audit.md` A1): verify (a) `.controls` + `.speed-control` + `.progress` classes present and unchanged; (b) no new multi-column container class; (c) decorative wave element present at top only, `aria-hidden="true"`, static (no `@keyframes` binding to wave except permitted shimmer on progress pill); (d) banner / card fixed-height, not expanding to multi-row multi-column. Device trial SC-006: inspect single-column render on reference device, wave at top only, dots positional.

(5) Cross-ref: C3 clarification (demote 2-campo from normative), FR-008 / SC-006 (no second-color / no excluded screen), T021/T022 (stamps on existing single surface). No new layout contract — confirms existing.

---

## CHK026 — Metadata_title upstream dependency (documented: status_view.rs reads upstream metadata_title; dependency on upstream flow; if missing, filename fallback)

(1) Scope phrase: `status_view.rs` / `lib.rs` reads upstream `metadata_title`; dependency on feature-001 / `surface.rs` upstream flow documented; missing → `filename` fallback preserved; no hidden new source.

(2) Clarified: Dependency is explicit in `plan.md` 72 / 86 (source-of-record: `status_view.rs`, `status_view.html`, `lib.rs` mirror publisher, `crates/player/src/surface.rs`). The upstream flow (`unwrap_or(filename)`) is not being replaced by a local hard-coded title; if upstream `metadata_title` is `None` (pre-load / malformed payload), the same `filename` fallback applies. Document the dependency in `contracts/status-surface.md` / spec §Assumptions (already implied by Assumption #8 hybrid + T002 baseline), and record that `status_view.rs` must not invent a second title source. No new upstream contract — only documentation of the already-existing dependency (same class as T025 / T026 record-keeping).

(3) Evidence anchors: `plan.md` 72 (`status_view.rs`); `spec.md` 124 (`unwrap_or` answer); `crates/player/src/surface.rs:426`; `contracts/surface.md` / `status-surface.md` auditee list (item 2: `status_view.rs`); `tasks.md` T026 (ADR-002 records F4 contract / failure latch — separate from title, confirms dependency discipline is already documented); `tasks.md` T024 (point amendments to 001/004 — keeps upstream semantics).

(4) Verification format: host audit — (a) grep `status_view.rs` / `lib.rs` for `metadata_title` usage (must reference upstream, not define new); (b) confirm `surface.rs:426` remains source of `unwrap_or`; (c) document dependency sentence in `docs/adr/` or `contracts/status-surface.md` note (no spec change to F4 / failure logic). Device / build: `surface_audit` gate (`scripts/verify.sh` post-T027) fails if `status_view.rs` title source diverges from upstream.

(5) Cross-ref: T024 (upstream 001/004 amendments preserve semantics), T025 (host-face stamps — no notification change), T026 (ADR-002 failure latch — demonstrates documentation discipline for upstream-derived contracts). No new assumption if upstream verified; document dependency explicitly.

---

## Summary — required-format check per item

| Item | Clarified (yes/no) | Evidence anchor | Format (host / device / doc) | New assumption / spec edit needed |
|---|---|---|---|---|
| CHK019 | yes — focus ring preserved (`--sona-etwahl`), dots positional, no new interactive | `tokens.md` §1/§2, E1, C2, `status_view.html` `aria-hidden`, FR-008 | host census + device SC-003/SC-006 | No — record in FR-001 role map only |
| CHK020 | yes — static banner, `#FFD400`/`#FF4D63` locked, TTL≤3s, not assertive, `reason_ttl_ms=null` | `tokens.md` §3, D2, D5, `status_view.html` 252/429, T026/F4 | host audit (color + DOM load + aria-live) + device SC-003 §3.1 | No — resolve `role`/`aria-live` mismatch; document latch |
| CHK024 | yes — `unwrap_or(filename)` verified upstream; empty→"—"/.sr-only | `surface.rs:426`, `spec.md` 124, T002/T003, `test_lockscreen_us4.rs` | host prop + integration + render check | No — flow unchanged |
| CHK025 | yes — single-column kept; wave top-only static; `.controls`/`.speed-control`/`.progress` preserved | FR-008, C3, `status_view.html`, T021/T022 | host inventory (surface_audit) + device SC-006 | No — confirm existing |
| CHK026 | yes — `status_view.rs` depends on upstream `metadata_title`; missing→filename; document | `plan.md` 72/86, `surface.rs`, `contracts/status-surface.md` A1-A4 | host grep + audit note + build gate | No — document only |

No item requires a new token, new color, new interactive element, new assumption, or spec-contract amendment beyond documentation / role-map recording consistent with already-verified upstream decisions (E1, C1, C2, D2, D5, F4, T024–T026). All cross-references resolved against primary sources (not inference).
