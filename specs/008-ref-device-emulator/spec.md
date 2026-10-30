# Feature Specification: VERIF-EMULATOR — Reference-Device Structural Verification Layer

**Feature Branch**: `008-ref-device-emulator` (derived from assessment `ref-device-emulator` / `specs/003-reference-device`; sequential; branch `005-adopt-design-sona` active; no source edit)

**Created**: 2026-10-29

**Status**: Draft — Option A (small structural layer) selected; `go` verdict from `.specify/assessments/ref-device-emulator/decision.md` (2026-10-29); 6 blocking gaps resolved by user (G1–G6); physical `gate-8-full` preserved; `SC-044` deferred; `cargo check` deferred to T002 per `specs/003-reference-device/tasks.md`; `verify.sh` unconfirmed on current working tree.

**Input**: Assessment `ref-device-emulator` (`intake.md`, `research.md`, `problem.md`, `concept.md`, `decision.md`); `specs/003-reference-device/` (spec/plan/tasks/checklists/contracts/data-model); `ROADMAP.md`; `AGENTS.md` (constitution v1.12.0); user request for local/free emulator verification of reference device without mandatory physical `adb` device.

**Related**: `specs/003-reference-device/` (parent; structural verification layer depends on 003 shell contract); `specs/001-android-media-shell/` (`VERIFICATION.md` / `gate-8-shell PASS`; `gate-8-full` deferred until `002` `SC-044` closes); `crates/adapter-synthesis/` (engine delivers mono f32 @ 22050 Hz; rate ladder `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]`); `crates/engine-core/` (closed-set `EngineError`: 7 variants, `RateMismatch`, `ChannelMismatch`, `InitFailed`, `Unreachable`, `DeviceRefused`, `DeviceLost`, `StartRefused`); `crates/diag/` (`EventKind` 9 variants, JSONL record, 30-day trim, never transmitted).

---
## Clarifications

- **Q1 — Which exact AVD image / API level for VERIF-EMULATOR?** (2026-10-29): User selected **Option A: Standardized `API 29` AVD baseline** (recommended). Justification: aligns with `specs/001-android-media-shell/` minimum (`mediaPlayback` FGS requires API 29+), avoids FGS behavior drift at newer APIs, and reflects `003-reference-device` reference device baseline. If reference device uses newer API (e.g., API 33/34), document version delta in `VERIF-EMULATOR.md`; do not claim exact-device match. (Source: `research.md` Gap 1 / `developer.android.com` SDK docs; see Option A recommendation.)
- **Q2 — Sub-layer vs replacement `gate-8-full`?** (2026-10-29): User selected **Option A — Sub-layer `VERIF-EMULATOR` only**, with explicit boundary that physical device remains required for `gate-8-full`. No false-equivalence claim permitted.
- **Q3 — Interaction with `SC-044` / consent?** (2026-10-29): User selected **Option A — Independent**; `003` structural verification does not require `SC-044` closure; `002` adapter remains `offline_default`; `SC-044` deferred preserved.
- **Q4 — Attestation format?** (2026-10-29): User selected **Option A — Document `VERIF-EMULATOR.md` with exclusions** (not altering `verify.sh`). Justification: keeps physical gate untouched; records structural evidence separately; aligns with `speckit.assess.converge` attestation pattern (statement + exclusions + reference to `verify.sh`).
- **Q5 — CI / hardware / environment?** (2026-10-29): User selected **Option A — Isolated AVD** (container/CI job, not in dirty `005-adopt-design-sona` working tree `0c985ee`). Justification: preserves `bg_14` / `51 dirty` state; avoids contaminating `005` feature branch.
- **Q6 — Assessment `ref-device-emulator` vs spec-note inside `003-spec`?** (2026-10-29): User selected **Option A — Keep assessment** (`ref-device-emulator` remains as separate assessment / audit trail; can be referenced from `003` via link, not subsumed). Justification: preserves `speckit.assess` process evidence; does not lose research history.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Verification maintainer confirms structural framework before physical device (P1)

Maintainer of `003-reference-device` opens a local Android SDK AVD (API 29+), installs the `mdreader` build, and confirms that `MediaSession` / `MediaStyle` notification, foreground service (`mediaPlayback`), focus split (`MAY_DUCK` vs permanent loss), and diagnostic JSONL write behave exactly per `003-reference-device/data-model.md`. This is the first verification step — not the final gate, but the structural gate that was previously missing.

**Why P1**: Without a structural-verification path, `003` has no verified progress between `plan.md` (140 lines, complete) and `tests/` (absent). The only verified green build is historical (`004` at `9501bba`). This story unblocks structural confidence at low cost.

**Independent Test**: Run AVD with `spec.md` structural checklist (manifest, service, notification, focus API, diag record, resume fingerprint, rate-step UI) — all observable via framework APIs / `adb logcat` / AVD extended controls; no audio-output verification claimed; no `SC-009` benchmark claimed.

**Acceptance Scenarios**:
1. **Given** AVD is running `API 29+` with `mediaPlayback` manifest declared, `Service` registered, and `MediaSession` initialized; **When** maintainer opens `mdreader` with a `003`-parsed Markdown file and presses Play; **Then** notification shows title + current section heading + progress indication (determinate if total known; indeterminate otherwise), with no body text, rate, voice, or elapsed/remaining time.
2. **Given** playback is active; **When** a transient interruption permits ducking (`MAY_DUCK`); **Then** document ducks to ~20% and continues reading; when interruption demands silence (call/alarm/permanent loss); **Then** playback pauses with 3-second reason shown on notification, never auto-resumes.
3. **Given** playback is paused; **When** maintainer resumes; **Then** resume offer (3-second countdown, `continue` default) appears; if no choice made, reading starts from stored `blake3` fingerprint position (document edited = different fingerprint = no resume). Stop playback or reaching document-end clears the stored `blake3` fingerprint, so resume never uses a stale position.
4. **Given** structural verification completes; **When** maintainer creates attestation; **Then** document clearly lists verified framework contracts and explicitly excludes audio-output, `SC-009`, Bluetooth/car, 10/10 on-device, and `SC-044` — with link to `verify.sh` (physical gate rules unchanged).

---

### User Story 2 — Build / CI uses structural layer to catch framework regressions early (P2)

A CI job (isolated from the dirty `005-adopt-design-sona` working tree) runs AVD structural verification on every `specs/003-reference-device/` change, catching manifest/service/notification/focus errors before physical-device verification is needed.

**Why P2**: Reduces the cost of structural regression; does not replace physical gate; supports `speckit.clarify` / `speckit.converge` pipeline.

**Independent Test**: CI job exits non-zero if any structural checklist item fails; exits zero only when structural items pass; never claims `gate-8-full` from emulator result.

**Acceptance Scenarios**:
1. **Given** a commit touches `specs/003-reference-device/plan.md` or `data-model.md` or contracts; **When** CI runs; **Then** AVD structural check runs; if manifest syntax invalid or `Service` declaration missing, job fails.
2. **Given** a commit touches `crates/player/src/backend.rs` (transport/position) or `crates/diag/src/writer.rs`; **When** CI runs; **Then** structural verification confirms framework contracts still satisfied; no false equivalence claimed.
3. **Given** `verify.sh` physical gate changes; **When** CI compares; **Then** `verify-emulator.sh` (if defined) does NOT claim to cover physical-only gates; exclusions documented.

---

### User Story 3 — Attestation / audit trail for structural verification (P3)

Verification owner produces a documented attestation (not `verify.sh` itself) that records which framework contracts were verified via AVD, which were excluded (audio, interruption, Bluetooth, benchmark), and references the physical-device gate procedure (`verify.sh`) for excluded items.

**Why P3**: Enables `speckit.assess.converge` to accept structured evidence without misrepresenting it as full gate-8; aligns with `AGENTS.md` constitution (deterministic gates, attestation, no unverified claims).

**Independent Test**: Attestation document passes internal review; contains explicit exclusions; links to `problem.md` and `decision.md`; no source changes claimed.

**Acceptance Scenarios**:
1. **Given** structural verification completed; **When** attestation is written; **Then** it includes: (a) AVD image / SDK version; (b) list of verified contracts (manifest, service, notification, focus API, diag); (c) explicit exclusions (audio-output, SC-009, Bluetooth, telephony/alarm interruption, 10/10); (d) link to physical `verify.sh` rules; (e) date / branch / commit.
2. **Given** future audit; **When** attestation is read; **Then** reader understands immediately that this is structural-only and does not substitute `gate-8-full`.

---

### Edge Cases

- **AVD image unavailable / incompatible with reference device (API 29)**: Use closest available API level (29+) with documented version delta; do not claim exact device match.
- **Manifest edit fails due to Tauri upgrade / `mobile::update_manifest` absence**: Confirm direct `AndroidManifest.xml` edit per `AGENTS.md`; document exception.
- **`cleanroom-core` remains empty**: Verification artifacts live in `specs/003-reference-device/verification/` or `scripts/`; do not invent new crate.
- `SC-044` closes before this feature: No change to `VERIF-EMULATOR`; physical device still required for audio/interruption; `SC-044` only affects adapter consent, not framework structure.
- `005-adopt-design-sona` branch (`0c985ee`) still dirty / interrupted `cargo check`: Structural verification must not alter working tree; use isolated CI/job.
## Verification failure isolation and diagnostic boundary

Failed structural verification (manifest mismatch, service declaration error,
or any `VERIF-EMULATOR` layer failure) is **NOT** a `gate-8-full` failure.
`gate-8-full` remains physical-only (device trial, real interruption,
≤3 s audio, Bluetooth/on-device); structural verification failure does
not block it and does not alter its criteria (`FR-003`, contracts
`Physical gate contracts unchanged`, assessment `ref-device-emulator`).

Diagnostic detail produced by any verification failure or refusal is
bounded by the `AGENTS.md` diagnostic-record rules (`spec.md` `SC-002`,
`SC-003`; `docs/diagnostic-record.md`):
- local-only storage (`diag` app-private), never transmitted, no in-app
  surface; retention 30 days / 10 MB oldest-first trim;
- `detail` ≤200 characters, identifier-pattern only (`^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+(...)`),
  never document text or prose (`prose_in_detail` violation);
- `error` field required on every `failure` and `refusal` event
  (`failure_without_error` / `refusal_without_error` violations);
- no secret exposure (`.env`, `.pem`, `.key`, tokens, cloud credentials)
  in any attestation or diagnostic record (`AGENTS.md` deny-secrets,
  `security.md` CHK001, `VERIF-EMULATOR.md` §5).

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a documented structural-verification path (`VERIF-EMULATOR`) for `003-reference-device` using an official Android SDK Emulator (API 29+), covering manifest declaration, `MediaSession` / `MediaStyle` notification, focus API response, diagnostic JSONL record, resume by `blake3` fingerprint, and rate-step UI — without claiming equivalence to `gate-8-full`.
- **FR-002**: System MUST explicitly document exclusions of `VERIF-EMULATOR` from physical-only gates: audio-output verification (`22050 Hz` mono → device format via `cpal`/`AAudio`), `SC-009` ≤3 s play-to-audio, real `MAY_DUCK` / telephony / alarm interruption, Bluetooth / car routing, 10/10 on-device trials.
- **FR-003**: System MUST preserve `verify.sh` and physical-device `gate-8-full` rules unchanged; `VERIF-EMULATOR` is a separate, clearly-labeled verification layer.
- **FR-004**: System MUST allow the verification layer to be executed independently of the dirty `005-adopt-design-sona` working tree (isolated AVD / CI job / container) and must not modify source files in `crates/*/src` or `src-tauri/src/`.
- **FR-005**: System MUST produce an attestation document (`VERIF-EMULATOR.md` or equivalent) listing verified contracts, exclusions, AVD version, and link to `verify.sh`; must reference `blake3` fingerprint for resume consistency. Template schema (sections 1–8): see `VERIF-EMULATOR.md` in this feature directory; aligns with SLSA/in-toto attestation format (standalone signed/template document) per `/speckit.clarify` evidence (SLSA VSA v1.2, CISA SSDF / Self-Attestation Form).
- **FR-006**: System MUST not invent new engine variants, rate ladder members, `EventKind` variants, or `EngineError` variants; must use existing `engine-core` closed-set (`crates/engine-core/src/closed.rs`).

### Key Entities *(include if feature involves data)*

- **VERIF-EMULATOR attestation**: Document (not code artifact) recording structural verification results, exclusions, AVD config, and reference to physical state (`verify.sh` / `9501bba`). No database entity; no new `redb` bookmark; no new `blake3` entry (fingerprint already exists in `session`).
- **003-reference-device structural-state**: Existing `specs/003-reference-device/data-model.md` sections (`§SurfaceInventory`, `§NotificationSurface`, `§ProgressIndication`, `§MoveSpec`, `§DiagnosticEvent`, `§DesignDirState`, `§RateLadder`) — no new entities; layer verifies existing contracts.
- **Device fingerprint (existing)**: `blake3` content hash (`session` crate) — used for resume; unchanged by this feature.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (Structural)**: `003-reference-device` structural-verification document (`VERIF-EMULATOR.md` or equivalent) exists, references `spec.md` / `plan.md` / contracts, lists all verified framework contracts (manifest / service / notification / focus / diag / resume / rate), and explicitly excludes `gate-8-full` physical-only items (audio-output, `SC-009`, Bluetooth, 10/10). — Measurable: document exists, exclusions listed, no false-equivalence claim.
- **SC-002 (Isolation)**: Verification layer runs without modifying `crates/*/src`, `src-tauri/src/`, or `verify.sh`; runs in isolated environment (AVD / container / CI job); does not disrupt current `005-adopt-design-sona` working tree (`0c985ee`, 51 dirty). — Measurable: `git status --short` unchanged after verification run; no new source edits.
- **SC-003 (Attestation auditability)**: Attestation includes AVD version, verified contract list, exclusion list, reference to `verify.sh` physical rules, date/branch, and sign-off path; readable by maintainer 6 months later. — Measurable: document passes internal review; no ambiguous exclusions.
- **SC-004 (No false equivalence)**: No claim in attestation, `README`, or `ROADMAP` that `VERIF-EMULATOR` substitutes `gate-8-full`; physical device remains required for `001-android-media-shell` full verification. — Measurable: string search of `VERIF-EMULATOR` docs for false equivalence language = 0 hits.

---

## Assumptions

- Target device reference is `API 29+` (minimum for `mediaPlayback` foreground-service type); AVD image selected matches closest available API; if exact reference device differs, structural verification holds (framework same across API 29–34) but device-format conversion requires physical verification.
- `verify.sh` and existing `tests/` (none for 003) remain physical-device-only; `VERIF-EMULATOR.md` is additive documentation, not a replacement.
- User / verification maintainer has access to Android SDK AVD environment (free, local) with sufficient memory/disk; if unavailable, feature is deferred (Option B / status quo).
- `SC-044` (consent / cloud TTS) remains deferred; `002` adapter remains `offline_default`; `VERIF-EMULATOR` does not require consent or cloud access.
- `cleanroom-core` remains empty; verification logic does not require new crate; artifacts live in `specs/003-reference-device/verification/`.
- `Tauri` `2.12.1` and manual manifest edit rules unchanged; `VERIF-EMULATOR` does not alter `AndroidManifest.xml` beyond existing `mediaPlayback` declaration.
