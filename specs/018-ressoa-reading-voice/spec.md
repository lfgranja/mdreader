# Feature Specification: 018-sona-inspired-voice

- **Feature**: 018-sona-inspired-voice
- **Title**: Sona-Inspired Offline Reading Voice Catalog (Curated, Tuned, Multi-Voice Ready)
- **Status**: Draft / Ready for Plan
- **Created**: 2026-11-10
- **Assessment**: `.specify/assessments/sona-inspired-voice/decision.md` (Verdict: GO, conditioned MVP — requester Option B)
- **Target Release**: v0.5.0 (proposed)

## 1. Problem & Context

Ressoa/mdreader’s main design system (`DESIGN-SONA/`) defines a Sona-inspired, serene and musical "Engine Sona" identity (earcons, synesthetic visuals, calm accessibility posture), but reading TTS remains a single hardcoded bundled voice (`pt_BR-faber-medium`, Piper VITS via sherpa-onnx). There is no curated reading-voice profile aligned to that identity, no closed voice catalog, and no deterministic selection/cycle/persistence across the synthesis runtime, CLI, session store, and Tauri bridge.

This feature delivers **requester Option (B)**: a curated/offline pt-BR TTS reading voice selected and tuned to evoke **serenity/musicality ("Sona-inspired feel")** aligned to `DESIGN-SONA/`, implemented as a bundled alternative voice + tuning with evolution-ready multi-voice plumbing. It is explicitly **non-imitative**: no imitation/representation of League of Legends Sona, no Riot trademarks/character/performer likeness, and no LoL-branded voice identity in shipped ids/labels/assets/docs.

## 2. User Scenarios & Testing

### US1 — Select and persist a curated Sona-inspired reading voice (Priority: P1)

As a reader, I can list available bundled offline voices, select a curated Sona-inspired alternative (or keep the default), and have that choice persist and restore across app/session restarts so long-form reading uses a calm, clear voice aligned to DESIGN-SONA.

**Why this priority**: Core user-facing value of the feature (curated alternative + selection + persistence).

**Independent Test**: With bundled catalog present, list voices via CLI/bridge, set non-default voice, restart/restore session, and verify synthesis uses the selected voice deterministically and fails closed for unknown ids.

**Acceptance Scenarios**:
1. **Given** the bundled closed catalog, **When** voices are listed, **Then** each entry includes a stable deterministic id, display metadata, and provenance marker; default is `pt_BR-faber-medium`.
2. **Given** a valid selected voice id, **When** selection is persisted and the session/app restarts, **Then** the same voice is restored and used for synthesis (unless unavailable/corrupt, then fallback to default with named refusal/error as applicable).
3. **Given** no/unknown/malformed stored voice, **When** synthesis/session initializes, **Then** the system falls back to `pt_BR-faber-medium` without panic.

### US2 — Cycle voices safely during playback control flow (Priority: P1)

As a reader, I can cycle to next/previous bundled voice via existing shell controls and hear subsequent synthesized audio in that voice without corrupting playback state.

**Why this priority**: Multi-voice evolution and parity with existing `CycleVoiceNext/Prev` shell verbs.

**Independent Test**: Drive `CycleVoiceNext/Prev` across catalog boundaries and during stopped/playing states per shell contract; verify deterministic wrap/order and safe production restart at authoritative resume position.

**Acceptance Scenarios**:
1. **Given** N bundled voices in fixed declaration order, **When** `CycleVoiceNext/Prev` is invoked, **Then** selection advances exactly one step (with deterministic wrap) per press and published/inventory state reflects the new active voice.
2. **Given** active playback, **When** voice changes, **Then** current production/generation is invalidated and synthesis restarts from authoritative resume position in the new voice through the same Rust path; **never** splice or mix mixed-voice PCM into the active stream.
3. **Given** a stopped/completed or otherwise ineligible state per shell contract, **When** cycle is requested, **Then** the request is rejected/refused according to existing contract semantics (no state corruption).

### US3 — Offline synthesis contract preserved for every bundled voice (Priority: P1)

As a maintainer, every bundled curated voice synthesizes through the same offline adapter path with unchanged handoff/authority invariants.

**Why this priority**: Constitution and pipeline correctness (audio contract, fail-closed, Rust authority).

**Independent Test**: Synthesize representative segments per bundled voice and verify format/delivery/range metadata; simulate missing/corrupt assets and assert named failures.

**Acceptance Scenarios**:
1. **Given** any bundled voice, **When** a segment is synthesized, **Then** output is mono `f32` @ 22,050 Hz with echoed `resume_point`/`source_range` semantics preserved, using the same normalization/prosody pipeline and declared capabilities `(SsmlLevel::None, AlignmentGranularity::None)`.
2. **Given** missing/corrupt/format-incompatible voice assets or load failure, **When** synthesis/init is attempted, **Then** the system fails closed with a named `EngineError`/CLI error (no silent fallback to another voice, no silent resample/downmix).
3. **Given** offline host/device environment, **When** voices are used, **Then** no network access is required or introduced (`INTERNET` remains absent).

### US4 — Auditable provenance and non-imitative Sona-inspired identity (Priority: P2)

As a maintainer/reviewer, I can audit each bundled voice’s origin/license/config and confirm the feature expresses thematic "Sona-inspired feel" without LoL/Riot/performer claims.

**Why this priority**: Legal/provenance hygiene and explicit scope boundary.

**Independent Test**: Inspect bundled voice directories, manifests, and docs for required provenance artifacts and absence of prohibited branding/claims.

**Acceptance Scenarios**:
1. **Given** each bundled voice, **When** assets are inspected, **Then** the directory contains auditable provenance (`MODEL_CARD`, `PROVENANCE.md`, config/json as applicable) with upstream source, license, checksums/hashes, included files, and inference defaults.
2. **Given** shipped ids/labels/docs/CLI-bridge metadata, **When** reviewed, **Then** no Riot/LoL Sona character or performer likeness/branding is present; identity is thematic ("Sona-inspired feel"/calm-musical curation) aligned to DESIGN-SONA/Engine Sona internal naming only where already permitted.

## 3. Edge Cases

- **Unknown/unavailable/malformed persisted voice id:** fallback to `pt_BR-faber-medium` (or sole known default) without panic; record/log diagnosable refusal where appropriate.
- **Voice asset missing or corrupt at runtime:** fail closed for that voice/use with named error; never silently switch voices, resample, or downmix.
- **Catalog of size 1 (contingency/fallback build):** cycle becomes no-op-equivalent or single-step-stable per contract (deterministic, no panic).
- **Rapid repeated cycle/change:** apply deterministically at synthesis boundary; generation invalidation prevents stale mixed output.
- **Different upstream model defaults (e.g. `noise_scale`/`noise_w`/`length_scale`) per voice:** preserved per-voice as curated defaults unless evidence-backed tuning recorded.
- **Shared `espeak-ng-data` compatibility:** if a bundled voice requires incompatible phonemizer data, treat as asset validation failure (fail closed) and document contingency.
- **Android extraction migration:** upgrading from single-voice marker/layout to catalog layout must not leave partial/broken state; extraction is idempotent and version/marker-aware.

## 4. Requirements

### 4.1 Functional Requirements (FR)

- **FR-018-001 (Closed bundled catalog):** The system MUST define a closed, deterministic, bundled offline pt-BR voice catalog with stable ids matching bundled directory names, fixed declaration/cycle order, and no `Other`/`Unknown` variants. `pt_BR-faber-medium` MUST be included.
- **FR-018-002 (Curated alternative):** The catalog MUST include at least one curated 22050 Hz, CC0-provenanced alternative voice (target `pt_BR-cadu-medium`, fallback `pt_BR-jeff-medium` if validated) in addition to `pt_BR-faber-medium`, unless implementation records evidence that no alternative passes quality/runtime/packaging validation (contingency: tuned `faber`-only catalog with documented rationale and preserved multi-voice-ready API). `pt_BR-edresson-low` MUST NOT be bundled.
- **FR-018-003 (Per-voice provenance):** Each bundled voice directory MUST include auditable provenance artifacts (`MODEL_CARD`, `PROVENANCE.md`, model/config files, checksums/hashes, upstream source/license, inference defaults). `crates/adapter-synthesis/build.rs` manifest generation MUST cover the catalog layout.
- **FR-018-004 (Synthesis selection API):** `adapter-synthesis` MUST expose voice-aware synthesis/engine access (e.g. catalog listing, active/default voice, select/set, `synthesize` with explicit or active voice) while preserving a backward-compatible default path for existing callers where feasible. All synthesis MUST remain mono `f32` @ 22,050 Hz, fail-closed, same normalization/prosody pipeline, and declared capabilities `(SsmlLevel::None, AlignmentGranularity::None)` / `OfflineDefault`.
- **FR-018-005 (Engine lifecycle):** Runtime MUST support the catalog on pinned `sherpa-onnx = "=1.13.8"` via cached per-voice instances or safe recreate-on-switch, validated during implementation; selection/switch MUST NOT corrupt process state and MUST surface load failure as named error.
- **FR-018-006 (CLI parity):** `adapter-synthesis` CLI MUST provide voice catalog discovery and voice-scoped synthesis (e.g. `voices` listing and `synthesize --voice <id>` or equivalent additive interface), with deterministic machine-readable output consistent with existing CLI conventions.
- **FR-018-007 (Session persistence):** `crates/session` MUST persist and restore selected voice id (e.g. `last_voice`) alongside existing settings; unknown/unavailable/malformed stored values MUST resolve to `pt_BR-faber-medium` without panic.
- **FR-018-008 (Bridge/diagnostics/inventory):** `src-tauri` MUST seed/extract/apply catalog voices, expose catalog-aware diagnose/voice-inventory state, and provide get/set voice operations compatible with the same Rust synthesis path.
- **FR-018-009 (Cycle semantics):** `CycleVoiceNext/Prev` MUST traverse the closed catalog in fixed order (one step per press, deterministic wrap as defined by contract) and, when applied during/for playback, trigger safe production restart at authoritative resume position (generation invalidation + re-synthesis) with no mixed-voice PCM in the active stream.
- **FR-018-010 (Non-imitative identity):** Shipped code, assets, ids, labels, diagnostics metadata, and documentation MUST NOT claim or imply LoL Sona/Riot character voice or performer imitation/likeness; "Sona-inspired feel" is thematic curation/tuning only.
- **FR-018-011 (Offline/package integrity):** No `INTERNET` capability/permission introduced; Android packaging/extraction supports catalog layout and shared `espeak-ng-data` (or per-voice data if required by validation) idempotently.

### 4.2 Non-Functional / Invariants (NFR)

- **NFR-018-001 (Contracts preserved):** Handoff mono `f32` @ 22,050 Hz, no silent resample/downmix, Rust sole synthesis/playback authority, position/fingerprint/bookmark semantics unchanged except voice identity as explicit metadata where applicable.
- **NFR-018-002 (Fail-closed):** Missing/corrupt/incompatible assets or engine init failures produce named errors; never panic on user/input/state paths; never silently substitute a different voice.
- **NFR-018-003 (Quality gates):** Host gates 1–8 (`fmt`, `clippy -D warnings`, `nextest` + `scripts/expected-tests.txt` diff, docs/deny/audit/closed-set/shell checks per `scripts/verify.sh`) MUST pass. Gate 9/device validation reported separately.
- **NFR-018-004 (Evidence):** Auditory comparison/tuning rationale and chosen default are recorded in implementation evidence/docs.

## 5. Key Entities

- **`VoiceId` (closed):** Stable identifier matching bundled voice directory name (e.g. `pt_BR-faber-medium`, `pt_BR-cadu-medium`); serialized as plain string in CLI/bridge/session where applicable.
- **`VoiceSpec` / catalog entry:** Id, display metadata (non-branded), bundled asset paths, provenance references, per-voice inference defaults/tuning, and compatibility flags (22050 Hz).
- **`VoiceSelection` / persisted setting:** Active or last-selected `VoiceId`; restored with safe fallback to default.
- **Voice synthesis request:** Segment/text input + resolved `VoiceId` + existing normalization/prosody context, producing PCM and echoed metadata through unchanged handoff contract.

## 6. Success Criteria

- **SC-018-001:** Closed bundled catalog (≥2 voices including `pt_BR-faber-medium` unless documented contingency) is listable/selectable/cyclable deterministically via the same Rust path (CLI and bridge surfaces updated) with safe persistence/restore and default fallback.
- **SC-018-002:** Synthesis for every bundled voice conforms to mono `f32` @ 22,050 Hz handoff; missing/corrupt/incompatible assets fail closed with named errors; no silent resample/downmix or voice substitution.
- **SC-018-003:** Existing capabilities/tier/authority/position/resume/fingerprint invariants and normalization/prosody behavior remain intact (modulo intentional, evidenced, voice-scoped tuning); host gates 1–8 green with `scripts/expected-tests.txt` consistent with `cargo nextest list --workspace`.
- **SC-018-004:** Each bundled voice has auditable provenance artifacts and checksums; no Riot/LoL/performer branding appears in shipped ids/labels/assets/docs.
- **SC-018-005:** Implementation records auditory validation evidence for "Sona-inspired feel" and documents default-choice rationale (curated alternative vs faber default).

## 7. Assumptions & Dependencies

- Upstream `rhasspy/piper-voices` `pt_BR-cadu-medium` and/or `pt_BR-jeff-medium` remain available under CC0 with 22050 Hz configs compatible with pinned `sherpa-onnx = "=1.13.8"`; if not, contingency is tuned `faber` catalog/API groundwork with documented deferral of additional asset.
- Existing product/contract consumers of `CycleVoiceNext/Prev` and diagnose/inventory can be updated additively/compatibly in the same change set.
- Device/gate-9 execution may be unavailable in this environment; host-checkable evidence is mandatory and device evidence is recorded when run.

## 8. Out of Scope

- LoL Sona or actor voice imitation/cloning; Riot trademark/likeness use.
- User-imported (SAF), downloaded, cloud, marketplace, or paid voice packs; any `INTERNET` permission/capability.
- SSML/alignment capability changes, engine-tier/registry expansion beyond `OfflineDefault`.
- Major UI redesign beyond required voice controls/inventory/plumbing.
- Reworking unrelated earcon/visual design-system implementation except minimal consistency required by voice identity surfaces.