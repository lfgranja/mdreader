# Constitution Compliance Checklist: Ressoa Dogfood Build

**Purpose**: Verify that the spec and plan conform to every principle and constraint in the mdreader Constitution (v1.5.0) before implementation begins
**Created**: 2026-10-13
**Constitution Version**: 1.5.0 (ratified 2026-10-01, last amended 2026-10-12)
**Spec**: [spec.md](../spec.md)
**Plan**: [plan.md](../plan.md)

---

## Core Principles

### I. Library-First, Thin Tauri Shell

- [x] Every capability lives in a standalone Rust crate with one clear purpose — plan confirms zero new crates; `earcon.rs` + ingest/persistence wiring live in existing `player`/`session`/`src-tauri` (plan.md L35, L76–108)
- [x] The Tauri application is one consumer of the crates, never the only one — plan states "Tauri app one consumer" (plan.md L23) and "Tauri stays a delegating consumer" (plan.md L128)
- [x] The engine is drivable headless on Linux; every behavior reachable from the phone is reachable from a command line — plan confirms "every behavior CLI-drivable" (plan.md L35) with `player earcon …`, `session ingest …`, `diag scan` CLIs (plan.md L36)

### II. Text I/O Protocol (CLI-First)

- [x] Every crate exposes its capability over text: arguments and stdin in, stdout out, errors to stderr — plan confirms CLIs for `player` (earcon, progress), `session` (ingest, get/put/clear/rate), `diag` (scan) (plan.md L36, L85–89)
- [x] Machine-readable output is available via an explicit `--format json` flag — plan confirms `--format json` on all CLIs (plan.md L36)
- [x] The same protocol drives the Tauri frontend over IPC, so IPC and CLI exercise identical code — plan states "IPC calls the same functions" (plan.md L36)

### III. Test-First (NON-NEGOTIABLE)

- [x] Tests are written, shown to fail, then implemented — plan confirms "host-testable (progress fraction, snap math, ladder, fingerprint, earcon band/duration, trim, exhaustiveness) red-green" (plan.md L37)
- [x] Red-Green-Refactor is enforced by review — plan states "no implementation before failing tests in tasks phase" (plan.md L37)
- [x] The test suite is the only evidence for hard-to-verify behaviors — plan confirms on-device 10/10 trials for SC-001…SC-010 (plan.md L19, L118)

### IV. Latest Stable Versions, Supply-Chain Gated

- [x] Every dependency is the most recent stable release — plan confirms "no new runtime dependencies" (procedural earcons = no assets; `Vibrator` = platform API) (plan.md L38)
- [x] Exceptions are recorded with exact version, technical reason, and accepted risk — plan confirms `ort` 2.0.0-rc.13 covered by existing registry/exception (plan.md L38); `proptest` re-verified on crates.io per tasks.md T002c (plan.md L38)
- [x] The supply-chain gate fails closed — plan confirms "crates.io-only, `Cargo.lock` committed, `cargo deny` + `cargo audit` gates" (plan.md L38)

### V. Deterministic Gates Over Judgment

- [x] Every decision that can be made deterministically is made deterministically — plan confirms "manifest + shell-purity checks retained" (plan.md L39)
- [x] Gates are fail-closed: a missing input fails its gate rather than being skipped — plan states "missing input fails, never skips" (plan.md L39)
- [x] `verify.sh` ordering is kept — plan confirms "verify.sh ordering kept" (plan.md L39)

### VI. Engine-Neutral Core, Adapters at the Edges

- [x] TTS engines are reached through a Rust trait — plan confirms "no new engine/tier; single `OfflineDefault` selectable" (plan.md L40)
- [x] No vendor SDK type, HTTP response shape, SDK error code, or provider-specific identifier crosses into the core — plan confirms `engine-core` untouched (plan.md L90)
- [x] Every adapter declares its SSML capability level; degradation of prosody is explicit and observable — plan confirms rate stretch stays engine-contract op (plan.md L40)

### VII. Closed Sets, No Catch-Alls

- [x] Error types, provider identifiers, stage verbs, capability levels, and policy enums are closed sets — plan confirms "8 product reasons map N:1 onto untouched closed taxonomies" (plan.md L41)
- [x] No `Other`, `Internal`, `Unknown`, or catch-all arm appears on a closed-set enum — plan confirms "no `Other`/`Unknown`, no `_ =>`, gate 7" (plan.md L27, L41)
- [x] Adding a variant is treated as a breaking change requiring coordinated edits, semver bump, and documentation update — plan confirms "wire `EngineError` stays 7 members, no semver bump" (plan.md L41)

### VIII. Driving-First (NON-NEGOTIABLE)

- [x] No required interaction demands looking at the screen — FR-001 confirms playback continues with screen off and locked (spec.md L134)
- [x] Every operation reachable while driving is reachable from the lock screen or notification without unlocking — FR-002 confirms play, pause, skip on lock-screen (spec.md L135); FR-012 confirms rate-step action on lock-screen without value display (spec.md L145)
- [x] Playback continues with the screen off, survives the device being locked, and survives an interrupted network connection — FR-001 (spec.md L134); FR-015 confirms pure offline (spec.md L148)

### IX. Offline-First

- [x] The engine produces audible speech with no network connectivity — FR-015 confirms "pure offline with zero network-dependent voice code" (spec.md L148)
- [x] The offline path is `sherpa-onnx` running Piper pt-BR voice models — plan confirms `sherpa-onnx` 1.13.8 + Piper `pt_BR-faber-medium` (plan.md L15)
- [x] Cloud synthesis is an enhancement layered on top, never the sole path — plan confirms "any future network voice is a later opt-in, never the sole path" (spec.md L148)

### X. Free-Tier Budgeted, Not Free-Hopeful

- [x] No build and no run requires a paid tier — plan confirms "no paid tier; idle pause bills nothing" (plan.md L44)
- [x] Requests are chunked below the smallest documented per-request limit, measured in bytes — plan confirms "byte-budget chunking unchanged (adapter-side ≤5000 B)" (plan.md L44)
- [x] A test enforces the byte budget — plan confirms byte-budget test under 5,000 bytes/request (constitution gate 8, plan.md L44)

### XI. Credential and Content Sovereignty

- [x] No API key, token, or secret appears in the repository, committed file, or application binary — plan confirms "no secrets" (plan.md L45)
- [x] Document content does not reach a third-party synthesis provider without explicit, per-provider, revocable user consent — plan confirms "no second voice/cloud/credentials/consent surface" (plan.md L45)
- [x] No telemetry, usage analytics, or crash reporting ships enabled by default — plan confirms "diag device-local private, never transmitted, adb-pull only" (plan.md L45)

---

## Additional Constraints

### Technology Stack

- [x] Language is Rust, Edition 2024, MSRV 1.85 — plan confirms "Rust, Edition 2024, MSRV 1.85 (constitution-fixed)" (plan.md L13)
- [x] App shell is Tauri 2.12.1 (not 3.0.0-alpha) — plan confirms "`tauri` 2.12.1 (stable 2.x)" (plan.md L15)
- [x] Frontend is Svelte 5 (Runes mode) — plan confirms "Svelte 5 Runes frontend per Technical Context" (plan.md L101)
- [x] All specified crates used (`pulldown-cmark`, `unicode-segmentation`, `ort`, `sherpa-onnx`, `thiserror`, `tracing`, `clap`, `serde`, `redb`) are in the Technology Stack list — plan confirms all (plan.md L15)

### Dependency Deny-Lists

- [x] No denied crate appears in any manifest or `Cargo.lock` — plan confirms no new runtime dependencies (plan.md L38)
- [x] The deny-list is closed; no new entries added — plan confirms no changes to deny-list (plan.md L38)
- [x] `sherpa-onnx`, `ort`, `espeak-rs`, and `msedge-tts` are not banned — plan confirms `sherpa-onnx` and `ort` in use (plan.md L15)

### Licence Policy

- [x] Copyleft is permitted; `deny.toml` allows the required licences — plan confirms "no new licences (CC0 `faber` already admitted)" (plan.md L48)
- [x] Crate licence and model-weight licence are audited separately — plan confirms CC0 `faber` voice (plan.md L48)
- [x] Weight licences are recorded per voice — plan confirms `pt_BR-faber-medium` (CC0) (plan.md L15)

### Rust Code Rigor

- [x] `unsafe_code = "deny"`, `missing_docs = "deny"`, `missing_debug_implementations = "deny"`, `unreachable_pub = "deny"`, `unused_results = "deny"` — plan confirms "Rust rigor lints (`unsafe` deny, pedantic clippy deny, no unwrap/expect/println)" (plan.md L27)
- [x] `clippy::all = deny`, `clippy::pedantic = deny` — plan confirms "pedantic clippy deny" (plan.md L27)
- [x] The only sanctioned suppression form is `#[expect(lint, reason = "...")]` — plan confirms no weakening of `Cargo.toml`/`clippy.toml` (plan.md L27)

### Tauri Platform Contract

- [x] Kotlin is confined to the platform shell: `Service`, `MediaSession`, notification, audio-focus request — plan confirms "Kotlin confined to picker intent + Service/MediaSession/notification/focus-request + haptics" (plan.md L46)
- [x] Single playback authority: Rust owns playback state; Kotlin `MediaSession` is a mirror — plan confirms "forward-only mirror driven by the Rust reason" (plan.md L46)
- [x] Foreground service is declared by editing `AndroidManifest.xml`/Gradle directly (no `mobile::update_manifest` API) — plan confirms "manual `<service mediaPlayback>` manifest edit re-verified every Tauri upgrade" (plan.md L46)
- [x] The deliberate deviation from Android ducking best practice (ADR-001) is respected — plan confirms "transient duck ~20% per ADR-001 deviation stands" (plan.md L48)

### TTS Engine Registry

- [x] The admissible engine set is closed; no engine added or removed — plan confirms "no new engine/tier" (plan.md L40)
- [x] The offline default is `sherpa-onnx` + Piper pt-BR (`faber`, `cadu`, `jeff`) — plan confirms `sherpa-onnx` + `pt_BR-faber-medium` (plan.md L15)
- [x] Capability differences are declared rather than assumed — plan confirms "capability declaration untouched" (plan.md L40)

### Audio Pipeline Contracts

- [x] Markdown is parsed to an AST carrying source ranges; filtered by a spoken-content policy; segmented into sentences with a pt-BR abbreviation guard — plan confirms `pulldown-cmark` + `unicode-segmentation` (plan.md L15)
- [x] Content is assembled into engine-neutral SSML; synthesised per chunk under the byte budget — plan confirms `adapter-synthesis` reuse + "byte-budget chunking unchanged" (plan.md L44, L93)
- [x] Alignment is at sentence or node granularity as designed precision; the pipeline degrades observably — plan confirms "sentence/node granularity as designed precision" + "degradation observable" (plan.md L47)

### Driving-Safety Requirements

- [x] Playback continues with the screen off and while the device is locked — FR-001 (spec.md L134)
- [x] All driving-time controls are reachable from the lock screen or notification — FR-002 confirms play, pause, skip on lock-screen (spec.md L135); FR-012 confirms rate-step on lock-screen (spec.md L145)
- [x] The app does not require a network connection to begin or continue playback — FR-015 (spec.md L148)
- [x] Audio starts within a bounded time from the control action — FR-003 confirms ≤3 seconds (spec.md L136); SC-002 confirms 10/10 trials (spec.md L168)

---

## Notes

**Pass/Fail Counts**:
- Core Principles (I–XI): 33 items — 33 pass, 0 fail
- Additional Constraints (8 sections): 27 items — 27 pass, 0 fail
- **Total: 60 items — 60 pass, 0 fail**

**Summary**: The spec and plan conform to all 11 Core Principles and all 8 Additional Constraint sections of the mdreader Constitution (v1.5.0). No violations were found. The plan explicitly addresses each principle in its Constitution Check section (plan.md L31–50), and this checklist verifies those claims against the spec content.

**Key compliance highlights**:
- Zero new crates; all capabilities live in existing focused crates (Principle I)
- Zero new runtime dependencies; procedural earcons need no assets (Principle IV)
- Pure offline with zero network voice code (Principle IX)
- Closed sets everywhere: 8 product reasons, 7 rate rungs, 3 transports, 4 end-kinds (Principle VII)
- Single playback authority: Rust owns playback; Kotlin is a forward-only mirror (Tauri Platform Contract)
- CC0 voice (`faber`) already admitted; no new licences (Licence Policy)
- Content-offset progress uses parsed units; audio-duration estimates never feed it (Audio Pipeline Contracts)
- Lock-screen rate-step action without value display (Driving-Safety Requirements)

**Re-check trigger**: This checklist must be re-verified after Phase 1 design and again before Phase 2 implementation, per the constitution's Compliance Review requirement (constitution.md L825–830).
