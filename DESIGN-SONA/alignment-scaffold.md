# DESIGN-SONA Alignment Scaffold (template-derived; NOT production restyle)
# Source contracts: DESIGN-SONA/tokens.md §1-4 + contracts/status-surface.md S3 + contracts/earcon-mapping.md
# Feature: 005-adopt-design-sona | Date: 2026-10-26 | Status: SCAFFOLD only
# Coordinated edit required (constitution §VII) before any production change: tokens.md + data-model.md §SonaToken + status-surface.md + spec.md + VERSION bump.

## Rate / Ladder Row Fixes (scaffold annotations — deferred to T012/T027)

| Row / Token | Contract Source | Status in scaffold | Required fix (NOT applied) |
|---|---|---|---|
| `--sona-void` / `--sona-stage` (backgrounds) | tokens.md §1, S3 (slates `#0f172a`/`#1e293b` REPLACED) | scaffold annotated; old values still in `status_view.html` (backup `.bak`) | Replace with `--sona-void`/`--sona-stage`; byte-identity check C-H8 |
| `--sona-etwahl` (`#8FD8E8`) | tokens.md §1 (primary accent; ONLY progress/flow/focus; NEVER failure/warning) | scaffold marked; `status_view.html` still uses `#2563eb` accent | Replace accent token; restrict to `:focus-visible` per S3 |
| `--sona-celeste` (`#B9A6F5`) — rate / Song of Celerity | tokens.md §2 (vacancy: token stays in lexicon but paint is empty / violation) | scaffold marked PROHIBITED on surface | Zero paint of `#B9A6F5`; verify via C-H1/hex-census |
| `--sona-aria` (`#5AD6A8`) — brand only | tokens.md §2 (exclusive brand Onda 2; PROHIBITED in operational player UI) | scaffold annotated; must NOT appear on StatusView | Confirm absence in C-H4 inventory / C-H3 accent-only |
| Rate ladder (`RateStepper`) — closed set `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]` | tokens.md §1 / components.md §3.2 / contracts/status-surface.md S4 (no painted numeric rate) | scaffold marked; bench `#F2F0E6` progress still present | Remove painted numeric rate; expose through `role="status"` live region only |
| Yellow pulse (`#FFD400`) — caution 3 s TTL only | tokens.md §2 / S3 | scaffold marked; must respect `animation-iteration-count` bound | Bound ≤3 s; never on failure color; one-shot banner `slideIn` ≤200ms (S5) |
| Red failure (`#FF4D63`) — permanent failure only | tokens.md §2 / S3 | scaffold marked | Static render (no keyframe); separated from caution ≥2.25:1 (C-H2) |
| Contrast verification (all pairs ≥4.5:1 text / ≥3:1 non-text over `#0B0F1A`) | tokens.md §1 table (11.20–16.75:1 verified) | scaffold noted; must recompute per C-H7 | Computed both modes; same table both states |

## Movement / Freeze (S5 contract)
- Continuous-motion set = `{pill}` only; freeze guard `@media (prefers-reduced-motion: reduce)` with `animation-play-state: paused` + static offset (never invisible).
- Yellow pulse bounded ≤3 s; banner `slideIn` one-shot ≤200 ms; failure static.
- Scaffold inserted in `status_view.html` (line 231, `sona-scaffold-tokens`). Production freeze logic deferred.

## Source / Template Confirmation
- Contract source: `DESIGN-SONA/tokens.md` (v1.0.0, 2026-10-06) lines 1–97; `contracts/status-surface.md` S3-S5; `contracts/earcon-mapping.md` E3/E4.
- Template file used: existing `DESIGN-SONA/tokens.md` + `status_view.html` (source of record per S2).
- NOT production: no token substitution applied to served surface; `src-tauri/src/status_view.html.bak` preserves original byte-id.
- Atomic commit covered: `scaffold(d)` (this file only; coordinated edits to tokens.md + data-model.md deferred to T012).
