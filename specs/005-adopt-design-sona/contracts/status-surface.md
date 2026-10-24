# Contract — Status Surface (in-app StatusView + MediaStyle accent)

**Feature**: `005-adopt-design-sona` | **Date**: 2026-10-24 | Authority chain: `spec.md` (FR-001, FR-002, FR-005, FR-008) → this contract → implementation. No parallel lists: `data-model.md` §SonaToken/§SurfaceInventory are THE tables; this document adds binding rules only.

## S1. Surfaces

Exactly TWO styled surfaces exist: (1) MediaStyle notification — accent-only; (2) the single in-app StatusView (driving surface). No third surface enters this feature (Lente reduzida = restyle stage on the StatusView, batch C3/94). Excluded screens must be unreachable by code inspection (no route/link/hidden handler in the audited surface).

## S2. Packaging invariant (prerequisite)

`src-tauri/src/status_view.html` is the **source of record**. The packaged app's served HTML MUST BE byte-identical to it at acceptance (host check). Today the bundle serves `../dist` (tokens-less bench): the wiring is this feature's first implementation task. The check fails closed if the audited file is missing or if the served copy differs in one byte.

## S3. Token binding rules

- Every painted color inside the audited surface is a member of `data-model.md` §SonaToken — verbatim values, no rgb()/derived variants, no `#FFFFFF` (mutual white over the track is 1.14:1). The ledger (`DESIGN-SONA/tokens.md`, post-v2.2.0 alignment) is the ONLY hex authority for `--sona-harp`.
- Slates `#0f172a` / `#1e293b` (shipped backgrounds) are REPLACED by `--sona-void`/`--sona-stage`.
- `--sona-etwahl` appears ONLY inside `:focus-visible` rules targeting the enumerated interactive controls (TalkBack draws its own ring). Any other occurrence — including legacy «fluxo ativo»/progress rows — is the E1 violation class.
- `--sona-celeste` (vacancy semantics, batch C2): the token STAYS in the lexicon but paint of it is a census violation — the sanctioned scope is empty, not the token.

## S4. Element inventory + semantics

- The StatusView contains EXACTLY `data-model.md` §SurfaceInventory's 9 elements; extra/hidden/dead = failure. Dead removals are structural (element deleted, not `display:none`).
- Aria contract (SC-001 item (iv) / SC-007): each interactive control carries a non-empty `aria-label` + state attribute; progress = `role="progressbar"` + `aria-valuenow`/`aria-valuetext`; the rate ladder value is exposed only through the visually-hidden live region (`role="status"`, dot strip `aria-hidden`); the reason banner uses a single `role="status"` (the shipped alert+polite pair is removed as incompatible).
- Value-blindness invariant: NO numeric rate is painted anywhere on the surface (grep-verifiable: zero painted `×` numeral / `Taxa:` text). The bench leak dies with the S2 wiring.

## S5. Movement contract (SC-001 item (v) / SC-007 freeze leg)

Continuous motion set = {pill}; the pill carries a `@media (prefers-reduced-motion: reduce)` guard that freezes it at a visible on-track position (`animation-play-state: paused` + static offset — invisible/opacity-zero is a FAIL). Yellow pulse lives only inside the 3 s TTL window (`animation-iteration-count` bounded); failure color renders static (no keyframe); banner `slideIn` is one-shot ≤200 ms; the census counts declaration occurrences inside the audited file and fails on any loop outside the closed set.

## S6. MediaStyle constants block

`MediaMirror.kt` stays a declarative mirror: the app-painted value set is ONLY the accent `--sona-voice #F2F0E6` (constants block `:728-743`; no structural change, no new Kotlin logic — SC-035 purity untouched). Reason-line states keep their 004 behavior (3 s TTL expiry; no auto-resume).

## S7. Change discipline

Any element addition, token addition, or motion addition = coordinated edit to this contract + `data-model.md` + the spec (§VII discipline, semver treatment at release) — never an in-place convenience.
