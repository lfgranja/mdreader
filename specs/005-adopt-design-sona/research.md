# Research: Adotar o DESIGN-SONA (Option A subset)

**Feature**: `005-adopt-design-sona` | **Date**: 2026-10-24 | **Spec**: [spec.md](spec.md)
**Lineage**: `DESIGN-SONA/` v2.2.0 (tag `design-sona-v2.2.0` after Q2) · `specs/004-ressoa-dogfood-build/` (earcon grammar + surfaces contracts) · `specs/001-android-media-shell/` (shell) · this feature's 15 clarify notes (`research/clarify-{b1..b4,c1..c3,d1..d5,e1..e3}-*.md`) + 36 Q/A bullets in spec.md.

All spec clarifications resolved (0 NEEDS CLARIFICATION). This file records the plan-time decisions the spec defers to design: how each FR/SC becomes a check, a contract, or an edit, and what the alternatives were.

## R1: Single design version + pin (Q1/Q2 alignment task)

- **Decision**: Normalize the dir to ONE declared version — every audited header says `v2.2.0`; add `DESIGN-SONA/CHANGELOG.md` and `DESIGN-SONA/VERSION`; create the git tag `design-sona-v2.2.0`; repoint spec references; the Option-A subset-selection record in the spec stays as the scope authority. The alignment ALSO fixes the stale earcon/etwahl rows (`tokens.md:16/:28/:47/:65`, `components.md:17`, `DESIGN-SONA.md:60/:85`, `preview.html:238`), records the shipped failure grammar in the design rows (E4 ≈330 Hz square + minor-2nd ≈450 ms; the old DESIGN-SONA-noted D3 stays on record as superseded), sets the ONE banned green = `#5AD6A8` (contracts/surface.md `#5FD0A0` = erratum), and includes the yellow-dot «Notificação (ponto pulsante)» erratum (`tokens.md:48`, `components.md:18`).
- **Rationale**: Today no single artifact self-declares the design version (zero `CHANGELOG.md`/`VERSION`/tag on disk — verified), so «homologado e fixado» was unverifiable; a versioned + tagged dir gives the census a stable auditee and honors §VII (adding/removing a vocabulary member = coordinated edit + version bump + docs).
- **Alternatives considered**: multi-version prose (rejected — two greens contradiction is exactly what it caused); changelog-only (rejected — no machine-readable pin); tag-only (rejected — header drift possible); deferred Lente-scoped (rejected — Q1/Q2 own the task, and the census needs a stable auditee to be fail-closed).

## R2: Closed hex census + role exclusivity (SC-001 host core)

- **Decision**: One deterministic host check (new `verify.sh` step + cargo test) walking a CLOSED list of audited files (`status_view.html`, `status_view.rs` relevant constants, `MediaMirror.kt` constants block, `DESIGN-SONA/` files post-R1, in-app header asset) against the audited set of **10 hexes** — 8 `--sona-*` (`--sona-void #0B0F1A`, `--sona-stage #141B2D`, `--sona-voice #F2F0E6`, `--sona-etwahl #8FD8E8`, `--sona-hush #A9B8CE`, `--sona-harp` (ledger-defined, gold positional dot), `--sona-celeste #B9A6F5` (reserved, ZERO paint), `--sona-boundary #55688F`) + `--signal-caution #FFD400` + `--signal-failure #FF4D63` — with per-token paint rules out of `contracts/status-surface.md`. Violation classes: any hex outside the audited set; jade `#5AD6A8`/erratum `#5FD0A0` paint anywhere; celeste paint anywhere; `#8FD8E8` paint outside the `:focus-visible` ring class; `#FFD400` outside banner + in-app 3 s dot; `#FF4D63` outside permanent-failure; signal pair adjacency without the ≥2.25:1 separation rule. Census runs over BOTH progress states (R3) — the indeterminate state is reachable and census-able.
- **Rationale**: a hex census converts SC-001 «sem vazamento de cor» from inspection to a gate (Principle V). The existing purity test is key/value-based and chromatically blind (spec, Q-D2 session); values verified shipped: `status_view.html:29-30,179-180` (`--warning/--failure` to rename), `MediaMirror.kt:728/730-743` (slates/declarative accents), accent `#F2F0E6` (T044).
- **Alternatives considered**: manual inspection per trial (rejected — judgment, not a gate); keeping «progress white»/slates `#0f172a`/`#1e293b` (rejected — off-set hexes; C1 mutual-white 1.14:1); census over arbitrary glob (rejected — closed auditee list is the tamper detector).

## R3: Indeterminate pill + movement rule (C5/C1/E2)

- **Decision**: Determinate: fill `--sona-voice` over track `--sona-boundary`, single flat value both states, no gradient. Indeterminate: solid sliding pill `--sona-voice` over `--sona-boundary` (004's `linear-gradient` shimmer removed). Movement census (SC-001 item (v) host leg + SC-007 freeze leg): ONLY continuous motion = the pill; under `prefers-reduced-motion: reduce` it freezes at a visible track position (no disappear, no opacity trick); yellow pulse exists strictly inside the 3 s TTL; permanent failure = static `--signal-failure` (no pulse); reason banner entry one-shot ≤200 ms (`slideIn` kept); no other loop may exist (census counts `animation`/`transform` declarations inside the audited file, minus the freeze-guard exception).
- **Rationale**: C5 kills the uncertifiable shimmer gap; E2 closes 2.2.2 strictness (a perpetual pill would otherwise loop forever) via the sanctioned OS-setting mechanism; the shipped file has zero reduced-motion handling and a 1.5 s infinite `@keyframes indeterminate` (E2 dive finding) — both must change.
- **Alternatives considered**: gradient + registered exemption (rejected as governance debt per batch C5); freeze-to-invisible (rejected — a frozen-but-hidden track fails the visible-position requirement); pausing animation duration 0s (rejected — indeterminate must remain reachable/re-renderable; SC-001 census runs in both states).

## R4: Reduced-Lente stamps + closed element inventory (C3/C4/C6/B4/D4)

- **Decision**: The restyle applies to the EXISTING surface structure: keep + style ONLY the 004-contract elements (reason line with expiry, resume offer, progress with identity, transport, rate ladder) · add exactly: header Concha Harmônica "R" (`DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` — already veto-color-compliant; triad `logo-canonical.svg` restricted to host faces), `--sona-stage #141B2D` metadata card, positional gold rate dot on the 7-rung ladder · structurally REMOVE dead remainders (empty version footer, unrendered state fields, the numeric-rate bench leak «Taxa: 1.0×» which dies with the wiring). Single-column geometry preserved (the only real CSS grid today is the 3-col transport; C3: «grade 2-campo» is a non-normative alias, and C6 defers the 62%/38% signature grid to the future full Lente). Launcher icon unchanged (D4 exclusion — keeps `gen/android` regenerator hazards and Play assets out).
- **Rationale**: honors «Lente reduzida» at minimal cost; the closed inventory (SC-001 item (ii)) becomes checkable because the file lists exactly what may exist — no extra element (visible, hidden, or dead) passes.
- **Alternatives considered**: two-column signature grid now (rejected by batch C6 — restructure exceeds tokens-only restyle and injects a new geometry oracle); RMS strip/buffer ring (excluded from scope — feature future); adapting the triad for the header (rejected — jade/celeste colors are prohibited/reserved).

## R5: Serve the audited surface (packaging prerequisite, Assumptions #9)

- **Decision**: The packaged app must RENDER `status_view.html` as served UI: a build-time wiring copies/serves the audited file as the bundle entry (`src-tauri/tauri.conf.json` `frontendDist`, currently `../dist`), and a host check asserts the served `index.html` is byte-identical to `src-tauri/src/status_view.html`. The bench `dist/index.html` content may remain as a fallback only if it does not leave the audited surface unserved.
- **Rationale**: verified today: the packaged app serves a tokens-less bench (`frontendDist: "../dist"`), so device legs of SC-001/SC-006 cannot pass regardless of how good `status_view.html` becomes. Spec makes the wiring a prerequisite (line 168).
- **Alternatives considered**: Svelte app switch (rejected — Option A excludes the frontend toolchain); dual surfaces served by route (rejected — single in-app surface is normative); leaving serving implicit (rejected — reinvents the F2 mistrust of undocumented wiring; the check must be deterministic).

## R6: Earcon name→action map + 004 grammar authority (US2/FR-003)

- **Decision**: Mapping (names → confirmations, closed): play→Adagio (C5→E5 ascending, 120 ms), pause→Andante (E5→C5 descending, 120 ms), rate-step→Unison (880 Hz tick, 60 ms), skip-forward→Allegro (880→1108 Hz chirp), skip-back→Allegro descending (880→660 Hz), warning→Ostinato, failure→Sforzando (E4 ≈330 Hz square + minor-2nd ≈450 ms). The GRAMMAR is owned by `specs/004-ressoa-dogfood-build/contracts/earcon.md` + `crates/player/src/earcon.rs` — never re-specified here; the DESIGN-SONA §6 divergences (incl. the old «taxa→Sforzando» bug) are settled by the shipped code (clarif #93, #81) and aligned in R1's rows.
- **Rationale**: FR-003 already binds to the shipped contract; re-pitching would fork 004's vocabulary mid-feature and invalidate T051-04 procedures.
- **Alternatives considered**: re-pitching per DESIGN-SONA §6 (rejected — D5/D6 re-pitch explicitly rejected in clarif #93); adding Crescendo/Aria (out of FR-003 scope).

## R7: Burst policy + typed diagnostics (D3/B2/edge #72)

- **Decision**: In `crates/player/src/backend.rs`, `enqueue_earcon` truncates/substitutes: a user earcon in flight is cut (tail dropped pre-append) so the newest wins; the additive mix remains earcon↔narração ONLY (per-slot `mul_add`); system Ostinato/Sforzando interrupt any in-flight user confirmation and fire immediately at state change; pause drop-and-refire untouched (user-initiated set only); the single `short_mix` channel stays. Truncation is temporal, registered as a **typed diagnostic counter**; emission failure (synthesis error/short channel unavailable) → action proceeds, narration never interrupted, AT MOST one re-emission at the next render space, then a typed diagnostic event — replacing the current silent no-op (non-conformant per clarif #102). §VII: both are coordinated closed-set extensions (diag kind set), documented in `contracts/earcon-mapping.md` and released under §VII discipline.
- **Rationale**: D3 measured the FIFO serialization (`backend.rs:266,285-290`) breaks FR-010 under bursts; B2 measured silence-without-registration as the exact failure class the constitution bans.
- **Alternatives considered**: queue/serialize ears (rejected — violates ≤100 ms at burst); mix earcon↔earcon (rejected — additive mix is specced narration-only); drop-newest (rejected — «latest wins» is the accepted rule); unbounded re-emissions (rejected — one, then registration).

## R8: ≤100 ms latency oracle + warning-path exception clause

- **Decision**: FR-010 is verified host-side reusing 004's first-consumed-sample technique (T152 precedent) — the deck asserts the first short-mix sample emitted for the earcon arrives ≤100 ms from the trigger; the device leg (quickstart §3) re-checks the same bound on the reference phone. If measurement proves the 100 ms infeasible ONLY on the pause-warning path (system-initiated Ostinato), register a measured exception à la the FR-006 precedent — never silently, and only with the multi-session device data, at which point ≤200 ms enters the spec via amendment.
- **Rationale**: 005 fixed 100 ms at spec level with evidence (clarif #87; Nittono 2004/Schmid 2024 bounds); the constitution requires measured bounds be exercised by deterministic tests.
- **Alternatives considered**: <50 ms decode boundary (rejected — unjustifiable ease tax); 200 ms blanket (rejected — breaks action attribution).

## R9: Haptic disposition for the gate (E3, option A accepted)

- **Decision**: SC-002's attribution is SOUND-ONLY with app haptics inert: the packaged binary publishes `haptic=None` (`src-tauri/src/lib.rs:6047`) and nothing fires `MediaMirror.kt:540-583` — no change needed, but the disposition becomes an audited line of the trial sheet (rigid mount, hands off body, admin buzz-check at the pilot) — and if 004's haptic wiring EVER lands before the trial, this feature must provide its OWN deactivation route (build flag or publish `None`) because the system «haptic feedback» setting does NOT gate direct `VibratorManager` calls. The haptic half stays owned by 004's T051-04 sounds-off condition — never by this gate.
- **Rationale**: dive verified dormancy; the trial sheet's disposition line must be true regardless of later wiring (the confounding risk is the recorded E3 rationale).
- **Alternatives considered**: bimodal dual-session leg (rejected — crosses 004 boundary without mandate; restyle-only scope); wiring haptics in this feature (same rejection); no clause (rejected — leaves the sheet's meaning undefined the day wiring lands).

## R10: SC-002 trial architecture (7 types)

- **Decision**: Protocols designed rollout-first: randomized constant-stimulus over **7 types** (play, pausa, salto-frente, salto-trás, taxa, aviso, falha), **≥15 trials/type**, no two equal consecutively, no in-session feedback, 5–10% trap attempts, 5–10 min familiarization with feedback first, confusion matrix artifact + direction-of-skip sub-score, parked-car media-speaker channel (phone headphone secondary), **≥24 h replication**, plus the rapid-sequence sub-test (2–3 heterogeneous actions inside 100–150 ms, per-trial identification). Risky pairs measured (Allegro-back×Andante; Unison×Allegro-front); systematic confusion is MEASURED evidence authoring a 004-contract amendment — never a silent edit; 004's SC-004 + T051-04 procedure emend to split directions.
- **Rationale**: B1 upgraded the protocol with classical psychophysics (constant stimulus + traps + replication) recorded in `research/clarify-b1-blind-test-protocol.md`.
- **Alternatives considered**: 10/10-per-type (rejected — batch B1 explicitly raises it); single-stimulus protocol (rejected — cannot falsify the FR-009 overlap policy).

## R11: SC-007 aria/TalkBack plan

- **Decision**: SC-001 item (iv) (deterministic host inventory — each enumerated interactive control carries non-empty `aria-label`, state by attribute), `role="progressbar"` + `aria-valuenow`/`aria-valuetext`, rate value exposed ONLY via a visually-hidden live region (`role="status"`; dot strip stays `aria-hidden`; the reason banner is re-typed from the incompatible alert+polite pair to status), NEW SC-007 device leg: TalkBack announces 10/10 (labels, ladder step value «velocidade 1,25», progress state, 3 s reason + failure states) AND the reduced-motion freeze leg keeps announcements inalterable 10/10. No slider/select (APG false keyboard promise deferred).
- **Rationale**: D5 verified the value-blind channel currently doesn't exist (0 `aria-valuetext` in repo) and scored the dogfood quality decision to build it now despite no legal pressure.
- **Alternatives considered**: register the gap as TODO (rejected — D5 explicitly chose «verified»); slider (rejected — falsely promises keyboard); visible numeric (rejected — value-blind is normative through this feature).

## R12: Contrast verification (SC-001 item (i))

- **Decision**: SC-001 computes WCAG 2.1 AA contrast ratios for every painted pair over the two effective backgrounds (void `#0B0F1A`, stage `#141B2D`) — text ≥4.5:1, non-text ≥3:1 — parametrized by background so future light mode reuses the same calculator. Verified shipped margins: `--sona-aria` ~10.59 (post-recompute), `--sona-voice` fill 16.75/15.01 vs void/stage and 4.88 vs the boundary track, `--sona-boundary` 3.43 (void) / 3.07 (stage) — the track is a non-text non-threshold pair but still passes narrowly; any failing pair resolves via the ledger's own recipe (never an off-set hex).
- **Rationale**: batch Q1/static decision #103 — design-dir numbers are documentation, not authority; the calculator is the authority and lives in the host check.
- **Alternatives considered**: trusting tokens.md declared ratios (rejected — at least one drift found on recompute); adding new hexes to rescue a pair (rejected — closed set).

## R13: Verification-gate integration

- **Decision**: The new checks are wired into `scripts/verify.sh` (fresh gate steps, fail-closed naming per 004 precedent) and their test inventory is pinned in `scripts/expected-tests.txt` (two-way diff sync, T157 precedent). The top-level acceptance gates stay the nine constitution gates; nothing is a tenth gate (no amendment required).

- **Alternatives considered**: separate CI script (rejected — verify.sh is the single entrypoint); voluntary checks (rejected — a non-failing check is decoration).

Each R above names its dismissed alternatives; every decision traces to a recorded clarification or a verified dive.
