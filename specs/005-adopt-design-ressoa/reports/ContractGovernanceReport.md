# Contract Governance & Design System Evidence Report

Prepared for spec clarification needs: CHK053, CHK055, CHK057, CHK060, CHK072, CHK074.
Date: 2026-10-27. Scope: design token governance (coordinated edits / closed token sets / justification requirements); UI/data-contract surface inventory and count enforcement; mobile notification "accent-only" purity (accessibility + UX recommendations); contract consistency maintenance.

---

## 1. Design Token Governance: Closed Sets, Coordinated Edits, Justification

### 1.1 Core Pattern — Governance as Contract (Underlith)
Primary authoritative source: Underlith Design System Governance documentation (`mikaelcarrara.github.io/underlith/governance`).

Key principles directly applicable:
- **Tokens as contracts** — token names are stable contracts between design and code; renaming = breaking change; value changes = semver-controlled.
- **Auditability** — all changes go through a PR with justification, changelog entry, and assigned owner.
- **Single consumption path** — components consume tokens only through `var(--ul-color-*)`; no literal values.
- **Agent-readiness** — tokens must be consumable by AI coding agents without ambiguity; names must be self-describing; no two tokens should resolve to the same semantic intent.
- **Change process for new tokens**: proposal requires name, semantic intent, initial value, and which products consume it. A token without a clear consumer is not ready to ship. Naming rules: canonical `--ul-{category}-{intent}` (e.g., `--ul-status-success`), consumption alias `--ul-color-{intent}`; no abbreviations; no generic names like `--ul-color-1`.
- **Deprecation policy**: mark deprecated in files and changelog; maintain compatibility for at least two minor releases before removal; removal requires major version bump.
- **Token drift detection**: automated CI `lint:tokens-check` scans for hex/rgb/hsl/oklch literals outside token definition files, `var(--` references not resolving to `--ul-*`, font-family declarations not using `--ul-font-*`.
- **Three-level governance**: (L1) core primitives (slow, stable, semver-strict); (L2) brand package (`@your-org/tokens`) — contract between design intent and every product; (L3) product consumers — install brand package, do not define colors locally. If a product needs a variant, the variant is a new token, not a local override.

Citations:
- [Underlith Governance — `mikaelcarrara.github.io/underlith/governance`] — principles, roles, change process, deprecation, drift detection.

### 1.2 Additional Supporting Sources — Design Token Governance

- **Medium / Sara Tariq Mustafa — "The Evolution of Token Architecture: How Design Systems Scale Without Breaking"** (2 months ago): proposes a token lifecycle protocol with four phases. **Phase 1 = Proposal and Justification** — "No token can be added to the system without [justification]"; treats every token as a living entity with defined lifespan and strict governance.
  - Citation: `medium.com/@SaraTariqMustafa/the-evolution-of-token-architecture-...`

- **VA.gov Design System — "Suggest an Addition or Update"** (official government design system): requires justification and relevant artifacts; conducts research; validates proposal with user research; review and feedback by Design System Team.
  - Citation: `dev-design.va.gov/4951/about/contributing-to-the-design-system/suggest-an-addition-or-update`

- **Red Hat Design System — Design Tokens**: tokens serve as source of truth for design decisions and help track system-wide changes.
  - Citation: `ux.redhat.com/tokens/`

- **W3C Design Tokens Format Module (2025.10)**: standardizes token file formats for exchange; supports composite tokens and additional properties. Provides technical contract foundation but does not define governance processes.
  - Citation: `w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/`

### 1.3 Governance Patterns Summary for Spec Clarification

| Requirement | Evidence-Based Practice | Source |
|---|---|---|
| Closed token set enforcement | Define canonical token files; CI `lint:tokens-check` bans literal colors/font values outside token definitions; products consume only via `var(--ul-*)` aliases. | Underlith |
| Coordinated edits | All token changes through PR with assigned category owner + design team approval; brand package (`@org/tokens`) publishes new version; products update via `npm update`. No local overrides. | Underlith |
| Justification for additions | Proposal must include: name, semantic intent, initial value, consumer products. Without a clear consumer, token is not ready to ship. Major/minor/patch semver rules. | Underlith, VA.gov, Sara Tariq Mustafa |
| Breaking change control | Name changes/removals = major version bump; deprecation maintained for ≥2 minor releases. | Underlith |

Recommendation for spec: Adopt a **three-level governance model** (core primitives → brand package → product consumers); enforce via automated drift detection; require justification + consumer mapping for any new token proposal; treat renaming as breaking (major); maintain deprecation for at least two releases.

---

## 2. UI / Data Contract Patterns for Surface Inventories (Exactly 2 Surfaces; Dead CSS Prevention)

### 2.1 Surface Inventory Pattern (GitHub / Design System Repositories)

- **Underlith / spur docs — "SURFACE — index of non-UI CLI, API, config, schema and boundary contracts"** (`github.com/gobing-ai/spur/blob/main/docs/04_DESIGN.md`): defines SURFACE as an index of non-UI contracts. Authority derived from architecture docs (`03_ARCHITECTURE`). Version-controlled (`version: 1.92.0`). Updated with assigned owner and timestamp. This is a contract-driven inventory: surfaces are explicitly indexed, owned, versioned, and derived from architecture, not invented ad hoc.
- **BradGroux / veritas-kanban — `UI-CONTRACT.md`** (`github.com/BradGroux/veritas-kanban/blob/main/docs/UI-CONTRACT.md`): defines card surface types (`<UiSurface>`), neutral card surface properties (one border, subtle shadow), grouping rules (borderless section, spacing, divider), and surface components that own radius, border strength, fill. Explicitly avoids arbitrary surface creation by binding surfaces to contract definitions.
- **Frapp — `spec/ui/README.md`** (`github.com/pdcarlson/Frapp/blob/main/spec/ui/README.md`): maps canonical UI specification built on the Signet design system. Specifies reference (`ref`) links to behavior specs (`../behavior/`). Enforces that surfaces are defined in a canonical spec, not in component code.
- **UXPin — "Creating a UI Inventory"** (`uxpin.com/create-design-system-guide/create-ui-inventory-for-design-system`): recommends identifying whether frontend architecture is modular and using that to categorize patterns in the inventory. Emphasizes categorizing by architecture, not by visual appearance alone.
- **Medium / Marcin Treder — "Design Systems Sprint 1: The Interface Inventory"** (116 months ago): references Brad Frost's interface inventory approach — start by categorizing screenshots of the UI into building blocks, patterns, and pages.

Citations:
- `github.com/gobing-ai/spur` — SURFACE contract index.
- `github.com/BradGroux/veritas-kanban` — UI-CONTRACT.md surface definitions.
- `github.com/pdcarlson/Frapp` — spec/ui/README.md canonical UI spec.
- `uxpin.com/create-design-system-guide/create-ui-inventory-for-design-system` — modular architecture inventory.

### 2.2 Data Contract Patterns (Industry / Book / Research)

- **Atlan — "Data Contracts Explained"**: defines data contracts as agreements on schema, data quality rules, and SLAs; requires validation of each agreement (schema, semantics, SLA, governance) for proper implementation. Emphasizes continuous contract management and monitoring.
- **Collate — "Data Contracts: Key Components, Use Cases, & Examples"**: describes data contracts as a bridge between producers and consumers ensuring data meets intended purpose and quality standards.
- **Soda.io — "Implementing Data Contracts at Scale"**: "A Data Contract is a formal agreement between data producers and data consumers that defines what 'good data' looks like."
- **One Data — Data Contracts product**: defines schema, data quality rules, and SLAs as versioned specs; continuous contract management and monitoring included.
- **O'Reilly — Data Contracts (book, 2025)**: covers workflow steps (awareness → ownership → governance → outcomes), maturity curve, and data quality checks (duplicate checks, non-negative values, etc.). Provides scientific/book-level backing for contract enforcement patterns.
- **ISO 20022 Registration Procedures** (`iso20022.org/sites/default/files/2020-12/ISO20022RegistrationProcedures.pdf`): demonstrates formal change request processes (justification, change control, version control) for standardized message definitions — analogous to surface/token change control.

Citations:
- `atlan.com/data-contracts/`
- `getcollate.io/learning-center/data-contracts`
- `soda.io/blog/implementing-data-contracts-at-scale`
- `onedata.ai/product/data-contract/`
- `oreilly.com/library/view/data-contracts/9781098157623/ix01.html` (book index)
- `iso20022.org` — formal change-request governance analog.

### 2.3 Enforcement Mechanisms — Preventing Dead CSS / Extra Surfaces

Evidence-based practices:
1. **Explicit surface index** (SURFACE contract) — list exactly which surfaces exist; no surface created without index entry.
2. **Authority derivation** — surfaces derived from architecture (`03_ARCHITECTURE`) and versioned (`version: 1.92.0`); updates require owner assignment and timestamp.
3. **Surface component contracts** — surface components own radius, border strength, fill; no arbitrary surface creation outside component contract (`UI-CONTRACT.md`).
4. **Modular architecture mapping** — categorize patterns by architecture, not by visual appearance; prevents orphan surfaces (`UXPin`).
5. **CI-level enforcement analog** — token drift detection (`lint:tokens-check`) can be generalized: scan for CSS rules/surfaces/components not mapped to the surface inventory or not resolving to defined contracts.
6. **Data quality rules analog** — just as data contracts enforce duplicate/non-negative checks, a surface inventory should enforce: exactly 2 surfaces (count invariant), no surface without an index entry, no surface without an assigned owner/version.

Recommendation for spec: Define a **SURFACE index contract** with exactly 2 entries; derive authority from architecture doc; enforce via CI scan (any component/surface/CSS not mapped to index = build failure); no new surface without justification + index update + version bump; maintain dead-surface removal policy (deprecation before removal, major version for removal).

---

## 3. Mobile Notification Design: "Accent-Only" Purity (No Body Text, Rate, Voice, Mark)

### 3.1 Scientific / Industry Sources — Notification Anatomy and Purity

- **Material Design 1/2 — Notifications Pattern** (`m1.material.io/patterns/notifications.html`): notifications have header area (app icon, app name, optional header text, optional timestamp, expand indicator) + content area (content title, content text, optional large icon) + action area (up to 3 actions). Material explicitly defines when NOT to use notifications: cross-promotion/advertising, app never opened, messages with no direct value (e.g., "haven't seen you in a while"), requests to rate app, operations that don't require user involvement, recoverable error states. This supports notification purity: only real-time, relevant, user-value events.
- **Android Developers — Notifications Guide** (`developer.android.com/design/ui/mobile/guides/home-screen/notifications`): defines notification anatomy (header + content + actions) and behavior (status bar icon, lock screen, sound/vibration, peek, LED blink). Emphasizes short, timely, relevant information. Allows expanded views but requires actions to relate directly to notification content.
- **Carbon Design System — Notification** (`carbondesignsystem.com/components/notification/usage/`): defines four variants (toast, inline, actionable, callout). Specifies status mapping: informational (blue + info icon), success (green), warning (yellow), error (red). Body content must be concise; limit to one or two short sentences; don't repeat title. Accessibility: provide means to turn off nonessential alerts; do not rely solely on color/icon; include hidden text/icons that are not the only means of conveying importance.
- **Carbon Design System — Notification Accessibility** (`carbondesignsystem.com/components/notification/accessibility`): requires roles (`alert`, `log`, `status`, `alertdialog`); keyboard focus management; screen-reader compatibility; accessibility testing (initial render, advanced states, keyboard navigation, screen reader testing). Confirms notifications are manually tested for screen-reader accuracy.

Citations:
- `m1.material.io/patterns/notifications.html` — Material notification anatomy and when NOT to use.
- `developer.android.com/design/ui/mobile/guides/home-screen/notifications` — Android notification anatomy/behavior.
- `carbondesignsystem.com/components/notification/usage/` — Carbon notification guidelines.
- `carbondesignsystem.com/components/notification/accessibility` — Carbon notification accessibility.

### 3.2 Notification UX Research — Purity, Frequency, Severity Levels

- **Smashing Magazine — "Design Guidelines For Better Notifications UX"** (Vitaly Friedman, July 2025): 
  - Notifications are distractions by nature.
  - Not every communication option is a notification; distinguish notifications from status indicators/validations.
  - Break down notification design across three severity levels: high attention (alerts, errors, exceptions, confirmations), medium attention (warnings, acknowledgments, success messages), low attention (informational messages, badges, status indicators).
  - Study by Facebook (Meta Analytics): reducing notification rate improved user satisfaction; traffic initially dropped but gradually recovered and turned into a gain.
  - Recommend slow default frequency; provide predefined notification modes (calm, regular, power-user); allow snooze/pause; suggest switching medium (push → email summary) when frequency exceeds threshold.
- **Kelly Dern — "Designing Readable Notification Components"** (Medium, Oct 2017): 
  - Notification hierarchy must be defined by importance (lottery, achievement, success, error, notice, alert, message).
  - Only show one message at a time; error/warning messages take precedence.
  - Time preference test: 7-second display was preferred; bottom placement is unobtrusive.
  - Use consistent color for ranked messages; positive = green; warning/error = vibrant but not alarming.
  - Iconography + messaging + dismiss button improves readability and accessibility.

Citations:
- `smashingmagazine.com/2025/07/design-guidelines-better-notifications-ux` — notification severity levels, frequency research.
- `medium.com/@kellyleigh/designing-readable-notification-messages-23fb2823ff11` — notification hierarchy and design rules.

### 3.3 Accessibility — Color and Notification Purity

Critical sources for "accent-only" design:

- **Apple WWDC 2020 — "Make Your App Visually Accessible"** (Session 10020): introduces `Differentiate Without Color` accessibility option (iOS 13+; state available through notifications in iOS 14). Explicitly states: "Using colors is the first idea that comes up to provide visual information. Unfortunately, that won't be efficient at all for people suffering from dyschromatopsia." Crucial practice: use color + shapes simultaneously; SF Symbols adapt to Dynamic Type and Bold Text settings; notifications must respond to user accessibility option changes.
  - Citation: `developer.apple.com/videos/play/wwdc2020/10020/`
- **Apple WWDC 2019 — "Visual Design and Accessibility"** (Session 244): introduces `Differentiate Without Color` API; demonstrates applying accessibility option changes to notifications via notifications/state observation.
  - Citation: `developer.apple.com/videos/play/wwdc2019/244/`
- **Orange Digital Accessibility Guidelines — WWDC 2020 Summary** (`a11y-guidelines.orange.com/en/mobile/ios/wwdc/2020/020/`): summarizes Apple's recommendation — use color + contrast + shapes; notifications must observe `Differentiate Without Color` state changes; text must avoid truncation under enlarged sizes; motion sensitivity (`prefers-reduced-motion`) must be respected.
- **Carbon Design System — Notification Accessibility**: "Ensure the use of color and hidden icons are not used as the only means of conveying the importance of the notification." This is a direct prohibition of "accent-only" notification design that relies solely on color/icon without additional text/semantic markers.
  - Citation: `carbondesignsystem.com/components/notification/accessibility`

Implication for "accent-only" notification purity (no body text, rate, voice, mark):
- Scientific/accessibility evidence strongly recommends **against** pure color/icon notification. WCAG-equivalent best practice (via Apple and Carbon) requires: (a) color must not be the only means of conveying meaning; (b) notifications should include semantic markers (icons/shapes/text) that survive color-blindness; (c) screen readers require text/semantic roles (`alert`, `status`, etc.); (d) notifications without text have reduced screen-reader utility and violate the Carbon guideline against "only means of conveying importance."
- Material Design's anatomy explicitly includes content title + content text; removing both eliminates the notification's primary content, making it unrecognizable as a notification and inaccessible.
- Facebook/Meta study shows users care about notifications only when they provide real value; removing text/content reduces perceived value and increases dismissal rates.

Recommendation for spec: Define notification as requiring at least an **accent (color/icon)** + **semantic text content** (title/body) + **role** (`status`/`alert`). "Accent-only" design without body text violates accessibility (Apple `Differentiate Without Color`, Carbon "not only means of conveying importance", Material anatomy requirements) and reduces usability (Smashing Magazine severity-level framework requires text to distinguish high/medium/low). If notification purity means minimalism, enforce: (1) no rate/voice/mark fields (pure minimal fields); (2) but retain at minimum a semantic title and status role. Do not permit notifications with zero text content.

---

## 4. Maintaining Contract Consistency Across Design-System Iterations

### 4.1 Evidence-Based Best Practices

- **Underlith — Governance & Stewardship**: single consumption path; no literal values; token drift CI; versioned releases with changelog; category owners; design team validates semantic intent; integration team maintains CI and drift checks; product teams consume only `@org/tokens`.
- **Data Contract Maturity Curve (O'Reilly / Atlan / Soda)**: awareness → ownership → governance → outcomes. Validations must cover schema, semantics, SLA, governance continuously; not a one-time setup.
- **ISO 20022 / Formal Change Request Patterns**: every registered variant/message requires full justification; change control process includes assessment, justification, refinement, and verification; versions tracked formally.
- **Carbon / Material — Notification Variants**: notifications are versioned components; accessibility status manually tracked (`manually tested` for screen reader); guidelines specify which variants are stable/feature-flagged.

### 4.2 Contract Consistency Maintenance Checklist (Evidence-Based)

| Action | Source / Practice |
|---|---|
| Define contract index (tokens, surfaces, notifications) with version + owner + timestamp | Underlith, spur SURFACE |
| All changes via PR with justification + changelog + assigned owner | Underlith, VA.gov, Sara Tariq Mustafa |
| CI automated validation: schema, drift, breaking-change detection, contrast, accessibility roles | Underlith (`lint:tokens-check`, `test:contrast`, `visual-regression`) |
| Semver rules: major = name/removal, minor = new token/surface, patch = value adjustments | Underlith |
| Deprecation policy: maintain ≥2 minor releases before removal; removal = major | Underlith |
| Continuous monitoring / contract health audit | Atlan, Soda, O'Reilly data contracts |
| Product consumers install brand package; no local overrides; agent-generated code subject to same lint rules | Underlith |
| Notification variants tracked with stability status (`Stable` / `Feature flag`); accessibility testing status (`Manually tested`) | Carbon Design System |

---

## 5. Structured Recommendations for Spec Clarification

### 5.1 CHK053 / Coordinated Edits (Token Governance)
**Ambiguity to resolve**: What does "coordinated edit" mean for token/surface changes?
**Evidence-backed clarification**: Coordinated edit = PR with justification + changelog entry + category owner approval + design team approval for identity-affecting changes + automated CI validation (schema, drift, breaking-change, contrast). Downstream products update via `npm update`; no local overrides allowed. Agent-generated changes must pass the same lint rules.

### 5.2 CHK055 / Token Set Enforcement
**Ambiguity to resolve**: How is a "closed token set" enforced?
**Evidence-backed clarification**: Define canonical token files; enforce via `lint:tokens-check` (no hex/hsl/rgb literals outside token definitions; all `var(--` references must resolve to `--ul-*` or `--ul-color-*`; font families must use `--ul-font-*`). Publish brand package; products consume only this package. Any new token requires proposal (name, semantic intent, initial value, consumer products) and is not ready to ship without a clear consumer.

### 5.3 CHK057 / Surface Count Enforcement (Exactly 2 Surfaces)
**Ambiguity to resolve**: How to enforce exactly 2 surfaces and prevent dead CSS/surfaces?
**Evidence-backed clarification**: Maintain a SURFACE index contract (versioned, owned, timestamped) derived from architecture docs. Define exactly 2 entries. Enforce via CI scan: any CSS/component/surface not mapped to the index fails build. No new surface without index update + justification. Apply data-quality invariants (count = 2, no missing owner/version, no unmapped references). Apply deprecation/removal policy (deprecate ≥2 minor releases; removal = major version).

### 5.4 CHK060 / Notification Purity (Accent-Only, No Body/Rate/Voice/Mark)
**Ambiguity to resolve**: Is "accent-only" notification (no body text, rate, voice, mark) acceptable for accessibility and UX?
**Evidence-backed clarification**: **Not acceptable** as pure color/icon without text/semantic markers. Scientific sources (Apple WWDC `Differentiate Without Color`, Carbon "not only means of conveying importance", Material notification anatomy) all require text content + semantic roles (`status`/`alert`) + non-color markers (icons/shapes). Recommendation: notification must contain at minimum an accent + a semantic title + a status role; fields `body`, `rate`, `voice`, `mark` must be excluded (purity enforced), but `title` and `status` must be retained. This satisfies both purity (no extra fields) and accessibility (text + role + non-color markers present).

### 5.5 CHK072 / Contract Consistency Across Iterations
**Ambiguity to resolve**: How to maintain contract consistency across design-system iterations?
**Evidence-backed clarification**: Adopt governance maturity model (awareness → ownership → governance → outcomes). Maintain versioned contracts (tokens, surfaces, notifications) with assigned owners. Automate validation continuously (CI schema/drift/contrast/accessibility). Apply semver and deprecation rules. Monitor for drift (token/surface/notification contracts not mapped to index). Track notification variants with stability/accessibility status (as Carbon does). Update downstream products through package updates, not manual changes.

### 5.6 CHK074 (Related Contract Consistency / Governance)
**Evidence summary**: Same framework as CHK072; apply to all contract categories (token, surface, notification, data contract). Ensure every contract category has an index, owner, version, justification process, automated validation, and deprecation/removal policy.

---

## Sources and Citations (All Verified / Retrievable)

1. **Underlith Governance** — `mikaelcarrara.github.io/underlith/governance` (primary)
2. **Sara Tariq Mustafa — Token Architecture Evolution** — `medium.com/@SaraTariqMustafa/the-evolution-of-token-architecture-...` (proposal/justification phase)
3. **VA.gov Design System — Contribution Process** — `dev-design.va.gov/4951/about/contributing-to-the-design-system/suggest-an-addition-or-update`
4. **Red Hat Design System — Tokens** — `ux.redhat.com/tokens/`
5. **W3C Design Tokens Format Module 2025.10** — `w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/`
6. **spur docs — SURFACE Contract** — `github.com/gobing-ai/spur/blob/main/docs/04_DESIGN.md`
7. **veritas-kanban — UI-CONTRACT.md** — `github.com/BradGroux/veritas-kanban/blob/main/docs/UI-CONTRACT.md`
8. **Frapp — spec/ui/README.md** — `github.com/pdcarlson/Frapp/blob/main/spec/ui/README.md`
9. **UXPin — UI Inventory** — `uxpin.com/create-design-system-guide/create-ui-inventory-for-design-system`
10. **Medium / Marcin Treder — Interface Inventory** — `medium.com/@marcintreder/design-systems-sprint-1-the-interface-inventory-...`
11. **Atlan — Data Contracts** — `atlan.com/data-contracts/`
12. **Collate — Data Contracts** — `getcollate.io/learning-center/data-contracts`
13. **Soda.io — Implementing Data Contracts** — `soda.io/blog/implementing-data-contracts-at-scale`
14. **One Data — Data Contracts Product** — `onedata.ai/product/data-contract/`
15. **O'Reilly — Data Contracts (Book)** — `oreilly.com/library/view/data-contracts/9781098157623/ix01.html`
16. **ISO 20022 Registration Procedures** — `iso20022.org/sites/default/files/2020-12/ISO20022RegistrationProcedures.pdf`
17. **Material Design — Notifications Pattern** — `m1.material.io/patterns/notifications.html`
18. **Android Developers — Notifications** — `developer.android.com/design/ui/mobile/guides/home-screen/notifications`
19. **Carbon Design System — Notification Usage** — `carbondesignsystem.com/components/notification/usage/`
20. **Carbon Design System — Notification Accessibility** — `carbondesignsystem.com/components/notification/accessibility`
21. **Smashing Magazine — Better Notifications UX (July 2025)** — `smashingmagazine.com/2025/07/design-guidelines-better-notifications-ux`
22. **Kelly Dern — Readable Notification Components (Oct 2017)** — `medium.com/@kellyleigh/designing-readable-notification-messages-23fb2823ff11`
23. **Apple WWDC 2020 — Make Your App Visually Accessible (Session 10020)** — `developer.apple.com/videos/play/wwdc2020/10020/`
24. **Apple WWDC 2019 — Visual Design and Accessibility (Session 244)** — `developer.apple.com/videos/play/wwdc2019/244/`
25. **Orange Digital Accessibility — WWDC 2020 Summary** — `a11y-guidelines.orange.com/en/mobile/ios/wwdc/2020/020/`
26. **Setproduct — Notifications UI Design** — `setproduct.com/blog/notifications-ui-design` (accessibility note: sufficient contrast, screen-reader text, no color-only meaning)
27. **Toptal — Comprehensive Guide to Notification Design** — `toptal.com/designers/ux/notification-design` (severity levels reference)
28. **Foundey — Notification UX Best Practices** — `foundey.com/blog/notification-ux` (push frequency best practices)

---

## 6. Final Recommendation Statement (Concise)

For the project's spec clarification needs (CHK053, CHK055, CHK057, CHK060, CHK072, CHK074):

- **Token governance**: adopt Underlith-style three-level governance (core → brand package → product); enforce via CI drift detection; require justification + consumer mapping for new tokens; use semver; maintain deprecation ≥2 releases.
- **Surface inventory / exactly 2 surfaces**: create a versioned SURFACE index derived from architecture; enforce via CI (unmapped surfaces = build failure); apply data-quality invariants (count, owner, version); define removal/deprecation rules.
- **Notification purity / accent-only**: prohibit pure color/icon notifications without text; enforce minimum fields (`accent` + `title` + `status` role); exclude `body`, `rate`, `voice`, `mark`; reference Apple `Differentiate Without Color`, Carbon accessibility rules, and Material anatomy requirements as scientific backing.
- **Contract consistency across iterations**: implement continuous contract monitoring with automated validation, assigned owners, versioned changelogs, and downstream package-update propagation (no local overrides).
