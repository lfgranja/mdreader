# Implementation Plan: Adotar o DESIGN-SONA (Option A subset)

**Branch**: `005-adopt-design-sona` | **Date**: 2026-10-24 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/005-adopt-design-sona/spec.md` (assessment handoff: go for the Option A scope — tokens, dual mark, Sona earcons on the existing surfaces; notification-first, dark-only, no full Svelte UI). All clarify runs resolved (36 recorded Q/A bullets, 15 research notes; zero NEEDS CLARIFICATION per `checklists/requirements.md` 16/16).

## Summary

Adopt the DESIGN-SONA design system v2.2.0 (pinned tag `design-sona-v2.2.0`, Option-A subset) over exactly **two surfaces**: (1) the **MediaStyle notification** — accent-only (the platform's single accent channel = `--sona-voice` `#F2F0E6`, already shipped by 004's T044), title + current section + progress, with the warning/failure states carried by the static reason line + Ostinato/Sforzando earcons, never by extra color; (2) the **single in-app StatusView** (`src-tauri/src/status_view.html` — the surface of record, wired into the actually-served bundle as a prerequisite) — closed token palette with **role exclusivity** (10 audited hexes: 8 `--sona-*` + `--signal-caution` `#FFD400` + `--signal-failure` `#FF4D63`; jade `#5AD6A8`/erratum `#5FD0A0` banned in any audited file; `--sona-celeste` `#B9A6F5` reserved-unpainted; `--sona-etwahl` `#8FD8E8` = focus-ring `:focus-visible` only), marfim progress fill over the `--sona-boundary` track (`#FFFFFF` refuses — contrast 1.14:1), indeterminate = solid sliding pill replacing 004's gradient shimmer, **reduced Lente** stamps (Concha Harmônica "R" header asset, `--sona-stage` `#141B2D` metadata card, gold positional rate dot on the closed 7-rung ladder — value-blind everywhere painted), closed element inventory (004-contract elements kept + dead remainders structurally removed), aria semantics inventory (item (iv)) + movement rule (item (v): pill freezes under `prefers-reduced-motion: reduce`; yellow pulse inside the 3 s TTL only; failure = static paint; banner one-shot ≤200 ms). (3) The **earcon layer**: 6 names → 7 confirmations (play→Adagio, pause→Andante, rate→Unison 880 Hz/60 ms, skip-fwd/skip-back→Allegro directional chirps, warning→Ostinato, failure→Sforzando E4 ≈330 Hz square + minor-2nd) over the **shipped 004 Rust grammar** (`crates/player/src/earcon.rs`, 22050 Hz mono f32, band 300–2500 Hz — pitches are NEVER re-specified), additive mix earcon↔narração only with full gain on the duck-exempt short channel, burst policy = truncate/substitute «latest wins» (`enqueue_earcon` tail drop) + typed truncation counter, system-initiated Ostinato/Sforzando always interrupt and fire at state change, pause drop-and-refire untouched (user-initiated only), ≤100 ms action→first-consumed-sample measured, emission failure = typed diagnostic event (never silent, max one re-emission), **haptics inert during the SC-002 gate (sound-only attribution; E3)**. (4) The **design-dir alignment task** (Q1/Q2): single declared version v2.2.0 + `DESIGN-SONA/CHANGELOG.md` + `VERSION` + git tag + stale earcon/etwahl rows + `#5AD6A8`/errata reconciliation — one coordinated task after the token/earcon work. Zero new dependencies; zero new crates; no Kotlin playback logic (shell purity SC-035 untouched).

## Technical Context

**Language/Version**: Rust, Edition 2024, MSRV 1.85 workspace / rustc ≥1.88 build-floor (unchanged); tracked `stable` channel via `rust-toolchain.toml`. Kotlin only in the existing thin shell, and only for constants — no new logic.

**Primary Dependencies**: **NONE NEW**. Existing: `tauri` 2.12.1; `crates/player` (`earcon.rs` grammar, `backend.rs` short-mix channel + `enqueue_earcon`); `src-tauri` (`status_view.rs`, `status_view.html` restyle target, `lib.rs` mirror publisher — `haptic=None` stays, and `tauri.conf.json` `frontendDist` wiring); `android-shell/…/MediaMirror.kt` (MediaStyle constants — accent already `#F2F0E6`; `:730-743` stays declarative); `DESIGN-SONA/` dir (v2.2.0 normalization); `scripts/verify.sh` (new host checks). Authorities referenced directly from contracts: `specs/004-ressoa-dogfood-build/contracts/earcon-mapping.md` E1–E7, `contracts/status-surface.md` S1–S7 / A1–A4, `contracts/surface-audit.md` A1–A4 — not copied; earlier `earcon.md`/`surface.md` references corrected to current filenames.

**Storage**: N/A — no new storage. `redb` bookmarks/settings untouched; the diagnostic JSONL record gains typed event kinds only through a **coordinated §VII extension** (truncation counter, emission-failure events).

**Testing**: `cargo nextest run --workspace` (new: closed-hex census, contrast calculator, element inventory, aria inventory, movement census, accent-only constant check, burst/truncate/substitute behavior, first-consumed-sample ≤100 ms oracle — reusing 004's technique; `scripts/expected-tests.txt` pinned fail-closed, T157 precedent). Deterministic host checks wired into `scripts/verify.sh`. Device legs (10/10 trials) per `quickstart.md` §3 on the reference phone.

**Target Platform**: Android aarch64 device of record Redmi 2201117TY (Android 13/API 33); headless Linux host for all deterministic checks; gate 9 (`cargo tauri android build --target aarch64`) unchanged.

**Project Type**: Amendment to the existing Cargo-workspace modular monolith + Tauri app — no new crate, no new toolchain (Option-A explicitly excludes a frontend framework).

**Performance Goals**: Earcon audible ≤ **100 ms** from action/state-change (first consumed sample, deterministic host oracle); banner entry one-shot ≤ **200 ms**; yellow pulse TTL **3 s** (then stop); pill freeze under reduced-motion (host + device legs); 004's notification/reason-line behavior unchanged.

**Constraints**: Closed hex set auditable by census (10 members; out-of-role paint = violation class); no numeric rate painted anywhere on the driving surface (value-blind, FR-012/004 purity); jade never paints the player; single-column geometry preserved (no 62%/38% restructure this feature); excluded screens unreachable; launcher icon explicitly out of scope (D4); `status_view.html` must be the actually-rendered surface or the device legs of SC-001/SC-006 cannot pass; pt-BR only; zero egress.

**Scale/Scope**: 2 surfaces redone in tokens; 10-member closed hex (8 `--sona-*` + `--signal-caution` `#FFD400` + `--signal-failure` `#FF4D63`) audited by census; ~11–15 new host tests (C-H1..H9 + latency oracle + burst) + 4–5 verify.sh checks; 1 design-dir alignment task; 1 packaging wiring; no screen additions; device trials are the only non-host acceptance legs (mirroring 004's T051 posture).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I (Library-First, thin shell)**: PASS — no new crates; the restyle lives in the app shell's own asset (`status_view.html`) exactly as 004's in-app screen does; behavioral changes land in existing crates (`player`) and `src-tauri`; every capability stays CLI-drivable (`earcon`/`rate`/`get`/`put`/`diag scan` verbs unchanged).
- **II (Text I/O)**: PASS — no new IPC-only behavior; new diagnostics surface through the existing typed record + `diag scan --format json`; host checks are standalone-runnable scripts, not TUI-only.
- **III (Test-First)**: PASS — every host-verifiable criterion (census, contrast math, inventories, movement census, accent check, burst policy, latency oracle) is listed in `quickstart.md` §1 and enforced red-to-green in the tasks phase; device-only legs defer to §3 trials 10/10; no implementation before failing tests.
- **IV (Stable + supply-chain)**: PASS — **zero** new dependencies; `Cargo.lock` untouched; no pre-release, vendor, or source deviation introduced.
- **V (Deterministic gates)**: PASS — census/contrast/inventory/aria/movement/accent checks are exact commands exiting 0/non-zero, fail-closed (missing surface file fails the check, never skips), wired into `verify.sh` ordering.
- **VI (Engine-neutral)**: N/A→PASS — no TTS engine/adapter change; earcons are player-side confirmations outside the engine trait.
- **VII (Closed sets, no catch-alls)**: PASS with one sanctioned coordinated extension — the burst policy adds typed diagnostic event kinds (truncation counter, emission-failure) to the diag record's kind set: coordinated edit, docs, and treatment as §VII change at release (no catch-all, no silent branch; the "one re-emission then typed event" rule has no implicit arm). The audited hex set, earcon confirmation set (7), rate ladder (7 rungs), and haptic-disposition set are closed tables in `data-model.md`.
- **VIII (Driving-first)**: PASS — the feature IS this principle at the surface level: eyes-free earcon confirmation with measured ≤100 ms; notification structure untouched (accent-only, no value leakage — value-blind preserved); every new in-app stimulus is redundant (color+shape+text+position; mutism dignity FR-006), and no added interaction demands looking.
- **IX (Offline-first)**: PASS — earcons synthesize locally at 22050 Hz; no network path added.
- **X / XI**: PASS — no paid tier, no egress, no new credential surface; the SC-002 trial is device-local (audio played by the media speaker; matrix artifact is an adb-pull/repo file).
- **Tauri Platform Contract**: PASS — Kotlin stays mirror-only (SC-035); accent constant value already ships; **no manifest/permission change, no icon change** (D4 keeps `gen/android` regeneration hazard out of scope); audio handoff untouched; Android Auto still deferred.
- **Audio Pipeline Contracts**: PASS — synthesis/parsing/alignment untouched; progress stays content-offset (FR-013/004-based); `haptic=None` disposition documented rather than behaviorally changed.
- **Driving-Safety / Licence**: PASS — no new licence, no model weight change.

No violations. No Complexity Tracking entry required. (Phase-1 re-check: see «Post-Design Re-check (Phase 1 complete)» below.)

## Project Structure

### Documentation (this feature)

```text
specs/005-adopt-design-sona/
├── plan.md              # This file
├── research.md          # Phase 0 output (this run)
├── data-model.md        # Phase 1 output (this run)
├── quickstart.md        # Phase 1 output (this run)
├── contracts/           # Phase 1 output (this run): status-surface.md, earcon-mapping.md, surface-audit.md
├── checklists/          # Existing (16/16)
├── research/            # Existing: clarify-{b1..b4,c1..c3,d1..d5,e1..e3}-*.md (decision evidence)
├── spec.md              # Feature spec (36 clarifications integrated)
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src-tauri/src/status_view.html      # The single in-app surface of record — token/role/aria/movement restyle target
src-tauri/src/status_view.rs        # Host logic for the view (element inventory + wiring hooks)
src-tauri/tauri.conf.json           # Packaging: served bundle must BE the restyled surface (frontendDist wiring)
crates/player/src/backend.rs        # enqueue_earcon: truncate/substitute «latest wins» + typed burst/failure counters
crates/player/src/earcon.rs         # Grammar asserts for the new event kinds (band/duration unchanged)
src-tauri/src/lib.rs                # Mirror publisher — `haptic=None` disposition stays (lib.rs:6047); no behavior change
android-shell/src/main/kotlin/…/MediaMirror.kt   # MediaStyle constants only; accent #F2F0E6 already ships; :730-743 declarative path untouched
DESIGN-SONA/                        # Alignment task: tokens.md, components.md, DESIGN-SONA.md, preview.html rows
                                    #   + NEW: CHANGELOG.md, VERSION; tag design-sona-v2.2.0
                                    #   assets/logo/logo-primary-concha-r.svg (in-app header mark)
specs/004-ressoa-dogfood-build/contracts/{earcon.md, surface.md}  # Authorities referenced by contracts/*
scripts/verify.sh + scripts/expected-tests.txt   # New fail-closed host checks + test-inventory sync
```

**Structure Decision**: Amendment-carrying (Assumptions: hybrid — new feature dir + point amendments to `001`/`004` specs where the Q1/Q2 or SC-002 protocol changes their text). The **source-of-record audit list** is closed in `contracts/status-surface.md`: exactly `status_view.html`, `status_view.rs`, `MediaMirror.kt` (constants block), the `DESIGN-SONA/` audited files (post-normalization), and the in-app header asset — the census refuses extra audited files as a change detector, fail-closed.

## Post-Design Re-check (Phase 1 complete)

- VII re-check: the closed-set extension is confined to typed diagnostic kinds; the closed tables above carry no catch-alls; design-dir normalization preserves single-version vocabulary (no fork of `design-sona-v2.2.0`).
- V re-check: every new acceptance leg has a deterministic host form (SC-001 items (i)–(v), SC-005 latency oracle, SC-006 host half); device-only legs (SC-002/SC-003/SC-004/SC-006 visual/SC-007) are human trials per `quickstart.md` §3 with 10/10 sheets, mirroring 004's T051/T052 posture and explicitly NOT claimed by host checks.
- Gates 5/6 re-check: no graph change; gate 7 re-check: no `_ =>` introduced; gate 9 re-check: no manifest/Gradle edit, so the Android build is carried by 004's artifact — re-verified at task close anyway.
