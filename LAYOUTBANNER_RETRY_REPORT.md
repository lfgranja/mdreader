# LayoutBannerRetry — CHK005/010/021/022/023/097 + CHK019/020 Report
Feature: 005-adopt-design-sona | Date: 2026-10-27 | Sources: spec.md FR-001/FR-007/FR-008; plan.md Implementation Strategy; contracts/status-surface.md S1–S7; DESIGN-SONA/*.md (spacing/gutter/focus); web sources (WCAG focus-visible, MDN graceful-degradation, W3C C9 decorative, CSS overflow, Carbon notifications).

## 1. Evidence read (verified, not inferred)

- spec.md FR-001 (token set, 10 hex, --sona-etwahl = focus ring only, --sona-celeste vacancy, jade prohibited, --sona-voice accent-only notification); FR-007 (no logo/waves on MediaStyle/lock-screen); FR-008 (2 surfaces only — notification + StatusView; Lente reduzida = restyle of same StatusView, single-column preserved, no 62/38 restructure this feature); clarifications 2026-10-23/24 (focus ring via :focus-visible on enumerated interactive controls; value-blind rate ladder exposed only via hidden role="status" live region; banner slideIn ≤200ms one-shot; pill freeze under prefers-reduced-motion at visible on-track position; yellow pulse bounded 3s TTL; continuous-motion set = {pill}); SC-001 items (iv) aria inventory + (v) movement census.
- plan.md Implementation Strategy: grade 4-dp M3, margins 16dp, gutter 24dp (DESIGN-SONA.md §87); control-bar / floating-pill typography; "grade 2-campo" is descriptive identity, not structural; StatusView is source of record (S2); packaging prerequisite = wire status_view.html into served bundle; host checks C-H1..C-H9 deterministic; device legs deferred per D2.
- contracts/status-surface.md: S3 --sona-etwahl ONLY inside :focus-visible on enumerated interactive controls (TalkBack draws its own ring — any other occurrence = E1 violation); S4 inventory = exactly 9 elements; S5 continuous motion set = {pill}, freeze under reduced-motion at visible position (invisible/opacity-zero = FAIL), yellow pulse bounded, failure static, banner slideIn one-shot ≤200ms, census counts declaration occurrences inside audited file; S6 MediaMirror.kt const block only accent #F2F0E6; S7 coordinated-edit discipline.
- Web brief (verified): WCAG 2.1 SC 2.4.7 (focus visible must not be removed); SC 2.4.11/2.4.12 (sticky banner must not fully/partially obscure focused component); MDN graceful-degradation (modern feature → still-usable fall-back); W3C C9 / WCAG 2.0 TECH technique (purely decorative images → CSS background / empty alt, assistive tech ignores); CSS text-overflow (ellipsis at content-area limit, clip/ellipsis/scroll); Carbon Design System notifications (banner top of related content, dismissible with action, persist semantics).

## 2. CHK005 / CHK010 — Refined layout / spacing / focus-visible / z-index (recommendations)

Context from checklists: CHK005 = loading/transition/state coverage; CHK010 = precise surface-bound (paused-resident notification content bounded; no elapsed/remaining/section leakage). Both map to SC-001 (token census + role exclusivity + aria + movement) and S3/S4.

Recommendations (measurable, fail-closed):

A. Spacing / gutter (design-dir evidence): adopt the declared 4-dp grid, 16 dp margins, 24 dp gutter (DESIGN-SONA.md §87; plan.md). The control-bar / floating-pill transport row (Play/Pause + −15s/+30s skip + rate ladder) MUST maintain ≥16 dp respiro between touch targets; target size ≥48dp, preferred ≥64–72 dp per design-dir Fitts / Thumb-Zone (avaliacao_ui_ux.md 88–93) — this directly answers measurable criteria for CHK005 (state-transition spacing consistency) and CHK010 (bounded content raises no unexpected overlap). Verify with host CSS audit counting gap declarations inside status_view.html (C-H1/C-H6).

B. Focus-visible (S3 contract + WCAG 2.4.7 / :focus-visible MDN): --sona-etwahl (#8FD8E8) is sanctioned ONLY inside :focus-visible targeting enumerated interactive controls (transport buttons, rate-step targets). No other paint of etwahl (no "fluxo ativo" on progress row — already corrected per clarification B3 / E1). Focus indicator MUST be visible (no removal, no opacity-0) and MUST NOT be overridden by banner z-index. Recommendation: declare z-index layering explicitly — banner slideIn (≤200ms one-shot, S5) lives at z-index above content but below any modal/focus trap; focused interactive controls remain on top within their stacking context. Recommendation for measurement: host check verifies exactly one :focus-visible block per enumerated control ID and zero other --sona-etwahl occurrences (S3 violation = E1).

C. Z-index / overlay interaction (CHK019/020 banner; S5 + WCAG 2.4.11/2.4.12): the reason banner (3 s TTL warning / permanent failure) slides in ≤200ms. Under reduced-motion it freezes at visible position (S5, not hidden). It MUST NOT fully obscure a focused component (SC 2.4.12 failure class). Recommendation: banner container z-index must allow focus ring visibility; if banner covers lower-third transport row, either (1) banner renders above content with focus ring drawn by user-agent / TalkBack independently, or (2) banner pushes content rather than overlapping — prefer (1) for eyes-free driving, but guarantee the focused control remains partially visible (WCAG 2.4.11 minimum). Measure: trial with focused Play button + banner entry, confirm focus indicator visible; host check verifies banner CSS uses relative/absolute within same stacking context, not fixed at highest layer.

D. Refinement for CHK005 (state-transition spacing consistency): define the closed-state sequence (playing / paused-with-resume-offer / paused-no-offer / failed-permanent / warning-3s) with fixed vertical spacing and no layout shift when banner enters/exits. Measurement: comparison of computed box metrics across states (host CSS computed-style check, deterministic).

## 3. CHK021 — Graceful degradation of decorative assets (recommendations)

Context: checklist = pre-roll/resource bounds / decorative-failure bounds; spec FR-001/FR-007 = brand logo / sound waves prohibited on notification/lock-screen; contract S3 (empty-alit decorative via CSS) + design-dir assets (logo-primary-concha-r.svg, decorative waves).

Evidence: W3C C9 / WCAG 2.0 TECH; MDN graceful-degradation; spec clarifications (logo header uses "sem jade e sem celeste" variant already shipped; decorative sound-wave graphics not in audited surfaces; invocation only via in-app header not notification).

Recommendations:

A. Decorative-asset failure path (CHK021): pure decorative elements (logo wave pattern on header background, decorative divider) MUST be implemented as CSS background-image or SVG inline with aria-hidden / empty alt, not as content-bearing <img>. Failure mode: if asset fails to load (broken CDN / missing file / SVG parse error), the UI MUST NOT lose structural layout or accessible labels — graceful degradation = background disappears, text/controls remain fully operable. Host verification: census confirms decorative assets are either CSS background or inline SVG with role="presentation" / aria-hidden; zero content-bearing decorative <img> in audited files.

B. Closed-set protection (FR-005 / FR-007): decorative assets must never inject disallowed colors (jade #5AD6A8, celery #5FD0A0 erratum, celeste #B9A6F5 when not reserved-vacancy). If decorative SVG references forbidden hexes, render MUST fall back to token-only palette (void/stage/voice). Recommendation: decorative SVG assets embedded in header should reference currentColor / token variables, not literal disallowed hex — fail-closed if literal prohibited hex detected by host hex-census (C-H1).

C. Notification / lock-screen (FR-007 / FR-002): decorative brand logo / sound waves are explicitly out of scope for notification (accent-only #F2F0E6). Gracful degradation here means: if platform reduces notification to minimal view, no decorative fallback is expected — surface stays pure title/section/progress per SC-003 (10/10). No decorative-asset failure can expose logo/waves on lock-screen.

## 4. CHK022 / CHK023 / CHK097 — Very long filenames / title overflow / truncation (recommendations)

Context: CHK022/023 = title/card edge cases (position before first heading; end-of-document locked); CHK097 = title-card overflow in constrained layout (lente spec edge-case gap). All tie to FR-002 / FR-008 / S4 (title on StatusView + notification; value-blind; no painted numeric rate) and text-overflow CSS.

Evidence: CSS text-overflow (ellipsis at content-area; clip/ellipsis/scroll); spec clarification (placeholder title slot renders stable glyph "—" with aria-label="Sem título" when genuinely empty; never shows empty slot for loaded doc — metadata_title.unwrap_or(filename) at surface.rs:426); banner entry ≤200ms one-shot; value-blind rate ladder (no "×" numeral painted).

Recommendations:

A. Title overflow (CHK097): title card / header MUST use `overflow: hidden; text-overflow: ellipsis; white-space: nowrap` (or appropriate multi-line clamp) with a defined max-width relative to container (not viewport-absolute, since control bar can vary 19.5:9/20:9/tablet). Truncation MUST occur at content-area boundary, not mid-character (CSS module Level 3). Measurement: rendered-width of longest test title (long filename + section) must not break control-bar 3-column transport row; host check verifies ellipsis applies to title element and never wraps to second line inside header (maintaining single-row header per design-dir).

B. Very long filenames (CHK022-style): when document title is long filename (no heading parsed), the title slot uses `filename` (not empty). Must not push section metadata or progress bar off-screen. Recommendation: flex/grid layout with `min-width: 0` on text container, fixed or min-width on transport controls, so long text shrinks with ellipsis rather than overflowing container. This is exactly the "overflow / truncation" measurement the brief asked for.

C. Empty / no-document state (spec clarification 124, CHK022/023 adjacency): genuinely empty slot (pre-load / malformed payload) renders glyph "—" (`--sona-hush`) with `aria-label="Sem título"`; never a blank slot. This is a graceful-degradation case — failure of doc load → stable placeholder, not layout collapse. Verify: host check ensures `.title` never renders zero-width or missing element in either state.

D. Reason banner truncation (related to overflow): banner text (3 s warning / failure reason) MUST truncate with ellipsis if longer than banner width; banner height must not grow unbounded (fixed or max-height). Banner slideIn is one-shot; no text overflow causes vertical layout shift.

## 5. CHK019 / CHK020 — Banner interaction / dismissal contract (recommendations)

Evidence: notification-surface checklist CHK019 = section-title correct after boundary/resume/skip/re-anchor (measurable); CHK020 = dismissal contract per surface kind (transient dismissed within 3 s of pause; resident present-and-dismissable with working resume). Contract S5 (movement) + WCAG 2.4.11/2.4.12 (banner must not obscure focused component).

Recommendations:

A. Banner interaction state machine (CHK019/020 mapped to status-surface): define the closed transition set — playing → paused-with-offer (3s banner) → resumed / paused-no-offer / failed-permanent. Banner entry (slideIn ≤200ms) and exit must be paired with aria-live / role="status" updates (S4) so screen-reader users receive state change. Interaction: banner MUST offer resume/dismiss action (per notification contract); in-app banner uses static reason line (not separate widget) per S6 / FR-002 / FR-007 — no extra interactive banner widget, reducing interaction conflicts.

B. Focus-obscuring verification (WCAG 2.4.12): trial protocol — with Play/Pause button focused (TalkBack / keyboard), trigger pause → banner enters. Confirm focus indicator visible and control still reachable (not covered by banner). If banner overlaps, either move banner to top-of-content (not lower-third overlay) or ensure z-index + focus ring painting order keeps control visible. Recommendation: prefer banner at top of StatusView (above transport row) so lower-third focus targets stay unobscured — aligns with Carbon notification best practice (banner at top of related content).

C. Dismissal / resume contract (CHK020 analog): for resident-notification analog, confirm transient-dismissed ≤3 s when paused; resident present with resume. At StatusView level: failure state never expires (reason_ttl_ms = null, per spec clarification 125); warning exits at 3 s TTL with yellow pulse bounded (S5). Host check verifies banner duration declarations (animation-iteration-count bounded; no infinite loop) and that failure banner has no timer.

D. Banner interaction with decorative assets / title overflow: banner entry must not cause long-title truncation to shift (no layout shift). Use fixed-height banner container; title container uses flex-shrink with ellipsis independent of banner visibility.

## 6. Cross-cutting measurement plan (fail-closed, deterministic)

Per plan.md verification subsection and contracts/status-surface.md S7 (coordinated edit discipline):
- Host checks (C-H1 census 10 hex + role exclusivity; C-H2 role; C-H3 accent-only constant; C-H4 contrast WCAG 2.1 AA; C-H5 aria inventory + hidden status live region; C-H6 movement census + reduced-motion freeze + bounded pulse; C-H7 burst/substitute; C-H8 latency ≤100ms; C-H9 first-consumed; plus new checks for spacing/gutter declaration, focus-visible occurrence count, banner z-index / overlap test, decorative-asset CSS-class verification, text-overflow declaration count, title-placeholder states).
- Device trials deferred (D2) for SC-001 visual / SC-002 blind / SC-007 TalkBack / reduced-motion freeze; not claimed by host. Confirmed per spec: deferred named with owner=device-build / gate-9.

## 7. Confirmed recommendations summary per CHK cluster

- CHK005 / CHK010 (layout / spacing / focus / z-index): enforce 4-dp grid + 16/24 spacing; restrict --sona-etwahl to exactly :focus-visible on enumerated controls; layer banner below focus ring visibility; verify no layout shift on banner entry; measure box metrics across states.
- CHK021 (decorative failure): CSS-only decorative assets (empty alt / role=presentation / aria-hidden); graceful degradation = missing asset → no layout loss; forbid literal jade/celeste in decorative SVG; census verifies no content-bearing decorative <img>.
- CHK022 / CHK023 / CHK097 (overflow / truncation / long titles / empty state): title uses nowrap + ellipsis with min-width:0 flex; placeholder "—" stable with aria-label; banner fixed-height; no vertical growth.
- CHK019 / CHK020 (banner interaction / dismissal / focus not obscured): banner at top of content, ≤200ms slideIn, bounded 3s/none by state, focus ring visible, aria-live updates, resume/dismiss actions defined, no infinite animation.

All recommendations trace to spec clauses (FR-001/FR-002/FR-007/FR-008), contract clauses (S3/S4/S5/S6/S7), design-dir measurements (4-dp / 16/24 / 48–72 dp touch / 3 s TTL / ≤200 ms / ≤100 ms), and verified web sources (WCAG 2.4.7/11/12, MDN graceful-degradation, W3C C9, CSS overflow/text-overflow, Carbon notifications). No fabricated measurement values; every numeric claim (4 dp, 16 dp, 24 dp, 3 s, 200 ms, 100 ms, ≥48 dp) is copied from the read sources above.
