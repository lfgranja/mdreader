# Feature Specification: 020 PocketTTS Voice Cloning Spike (Ressoa)

- **Feature**: 020-pockettts-voice-cloning-spike
- **Title**: Device-first go/no-go spike for offline pt-BR cloned voice (Ressoa) with competitive TTFA
- **Status**: Spike executed — decision recorded (NO-GO for integration on measured profile)
- **Created**: 2026-11-15 (artifact backfilled 2026-11-20)
- **Source of truth**: `PLAN-80-20.md` (80/20 plan, 2026-11-15) and `measurements.md` (on-device results)
- **Companion docs (kept intact)**: `measurements.md`, `PLAN-80-20.md`

## 1. Context & Problem

Ressoa/mdreader currently synthesizes offline with Piper VITS through pinned `sherpa-onnx = "=1.13.8"`, with a mono f32 @ 22,050 Hz handoff and closed Piper voices. The design system calls for a "Ressoa" reading voice; one candidate is zero-shot voice cloning from a clean pt-BR reference using PocketTTS. The open question is **not** whether cloning is possible in principle, but whether it is viable on the reference low-end device (`2201117TY`, Redmi Note 11, SD680, Android 13) under the project's TTFA budget.

Attested baseline: `play_to_audio_ms = 1551 ms` (T051/T052, `verification/T051_T052_attestation.md`); SC-002 bound is 3000 ms. A later direct debug-APK measurement recorded `14059 ms` for the current app build (see `measurements.md`) — the attested 1551 ms remains the comparison reference.

This spec formalizes the spike's requirements, the measurement gate and the recorded decision. It is a **decision/evidence artifact**: it does not authorize shipping a cloned voice.

## 2. User Scenarios & Testing *(spike context)*

### US1 — Decide viability on the reference device (Priority: P1) 🎯 core spike

As a maintainer, I need a measured, reproducible go/no-go on `2201117TY` before any integration work, so that Ressoa does not regress its offline TTFA posture for a cosmetic voice feature.

**Independent Test**: Run the on-device harness with the Ressoa 20–30 s reference and short/long pt-BR text across `num_steps ∈ {1,2,4}` and threads `∈ {1,2,4}`, cold and warm, recording `first_chunk_ms`, `rtf`, `rss_peak`; compare against 1551 ms / 3000 ms.

**Acceptance Scenarios**:
1. **Given** the reference device and clean Ressoa reference, **When** the measurement matrix runs, **Then** every configuration's raw metrics are recorded in `measurements.md` with device identity, bundle hash and command line.
2. **Given** measured `first_chunk_ms`, `RTF` and RSS, **When** the 80/20 gate is applied, **Then** GO requires TTFA ≤ ~1551 ms (cold and warm), sustained `RTF < 1` and acceptable RSS/stability; otherwise NO-GO.
3. **Given** a NO-GO result, **When** the decision is recorded, **Then** Piper remains the default and cloning is deferred behind explicit license/consent clearing and a future re-measurement.

### US2 — Keep the production pipeline intact while probing (Priority: P1)

As a maintainer, the spike must not change the production synthesis contract: no cloned-voice payload in the APK, no new permission, no changed handoff.

**Acceptance Scenarios**:
1. **Given** the spike assets (~140 MB model + reference), **When** packaging is considered, **Then** they are treated as on-demand optional download, never embedded as a multi-GB payload.
2. **Given** the production adapter (VITS-only, 22,050 Hz), **When** the spike runs outside the app via a standalone harness, **Then** no production Rust path, CLI, bridge or contract is modified by the spike.

### US3 — Preserve fallbacks and legal posture (Priority: P2)

As a maintainer/reviewer, a negative result must leave a concrete, cheaper 80/20 path and must not distribute third-party voice/weights without rights.

**Acceptance Scenarios**:
1. **Given** NO-GO, **When** the result is published, **Then** fallbacks (Piper prewarm/handover profiling; bake/custom Piper 22.05 kHz only with rights) are recorded.
2. **Given** Kyutai/HF gated weights, third-party voice audio and video-extracted reference, **When** distribution is contemplated, **Then** license and consent clearing is an explicit precondition, not assumed.

## 3. Edge Cases

- `first_chunk_ms` measures synthesis callback time, **not** end-to-end `play_to_audio_ms` (excludes app pipeline, enqueue/pre-roll, player, audible output). End-to-end TTFA can only be worse; a synthesis-side miss is therefore already disqualifying.
- Bundle↔runtime compatibility (`sherpa-onnx` 1.13.8 vs ≥2.0.1/BOS fix) is not fully verified; a mismatch can invalidate a configuration.
- CLI `offline-tts` binaries (termux-static 1.13.8/2.0.1) failed on device due to a missing libc++ symbol; the C-API harness is the working substitute.
- `VmHWM` remains high (~1.2–1.3 GB) after free even though RSS returns to ~34–55 MB; memory stability must be read from both metrics.
- Device connectivity (`adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`, `192.168.100.102:39761`) and debug-APK instrumentability are prerequisites for measurement.

## 4. Requirements

### 4.1 Functional Requirements (FR)

- **FR-PKT-001 (Reference asset)**: The spike MUST use the clean canonical Ressoa reference `tts-work/voices/ressoa/reference_30s_24k.wav` (PCM16 mono 24,000 Hz, 29.88 s, SHA256 `ca471e2fe598aa146a38f80a5976ca51cc84a80434ae92368bfe1fd1477e52a0`, manifest `tts-work/voices/ressoa/voice.json`) and preserve its hash record.
- **FR-PKT-002 (Model bundle, on-demand)**: The spike MUST use a PocketTTS pt bundle in sherpa layout (`haex-space/pocket-tts-sherpa-onnx/pt`, ~140 MB) kept out of the APK and treated as on-demand download with manifest/hash/license.
- **FR-PKT-003 (Device harness)**: The spike MUST run a standalone arm64 harness linked against the on-device sherpa C API (v1.13.8 prebuilt), measuring `read_ref_ms`, `model_load_ms`, `first_chunk_ms`, `synth_total_ms`, `audio_secs`, `rtf`, `VmRSS`/`VmHWM`.
- **FR-PKT-004 (Measurement matrix)**: The spike MUST measure `num_steps ∈ {1,2,4}` × threads `∈ {1,2,4}`, cold and warm (same-process reuse), with `speed = 1.0` except explicitly flagged smoke runs, and record all raw results in `measurements.md`.
- **FR-PKT-005 (Go/No-go gate)**: GO for integration requires `first_chunk_ms` (cold and warm) ≤ ~1551 ms attested baseline, sustained `RTF < 1` and acceptable RSS/stability. Any miss is NO-GO; the decision MUST be recorded in `measurements.md` with the supporting numbers.
- **FR-PKT-006 (End-to-end caveat)**: Because the harness measures synthesis only, the spike MUST label `first_chunk_ms` explicitly as not equivalent to `play_to_audio_ms`; end-to-end integration MUST NOT be claimed without an instrumented app measurement.
- **FR-PKT-007 (Production untouched)**: The spike MUST NOT change the production adapter, handoff format (mono f32 @ 22,050 Hz), closed `VoiceId` vocabulary, CLI/bridge contracts, or add any permission/capability.
- **FR-PKT-008 (Integration only on GO)**: A PocketTTS backend in `adapter-synthesis` (reference audio + embedding cache + `generate_with_callback` streaming, resample 24→22,050 Hz in adapter, controlled `VoiceId` extension, packaging/license) is permitted **only after** a GO gate result.
- **FR-PKT-009 (Fallback on NO-GO)**: On NO-GO, the default remains Piper pt-BR and the recorded fallbacks are Piper TTFA prewarm/segmentation/handover profiling and (only with rights and data) bake/custom Piper 22.05 kHz.
- **FR-PKT-010 (License & consent)**: Distribution of cloned-voice weights/reference requires explicit clearing of Kyutai/HF gated terms and consent for the reference voice; no distribution is implied by this spec.
- **FR-PKT-011 (Reproducibility)**: `measurements.md` MUST retain device identity, harness source/binary paths, bundle/reference paths, per-configuration command lines and raw matrix for reproducibility.

### 4.2 Non-Functional / Constraints (NFR)

- **NFR-PKT-001**: No `INTERNET` permission or network dependency is introduced in the app; on-demand download is a planning direction, not an implemented capability.
- **NFR-PKT-002**: Memory ceiling and stability on SD680 must be reported from `VmRSS`/`VmHWM`; ~1.2–1.3 GB peak is a risk signal, not acceptable for a low-end profile.
- **NFR-PKT-003**: Host gates 1–8 and device gate 9 remain the authority for production changes; the spike produces evidence, not a release.

## 5. Key Entities

- **Reference (Ressoa)**: canonical clean pt-BR WAV used as clone input.
- **PocketTTS pt bundle**: sherpa-layout model files under `/data/local/tmp/ressoa-pocket-pt/pt/`.
- **Harness metrics**: `first_chunk_ms`, `rtf`, `model_load_ms`, `read_ref_ms`, `VmHWM` — synthesis-side measurements.
- **Gate decision**: GO/NO-GO recorded against the 1551 ms / 3000 ms bounds and `RTF < 1`.

## 6. Success Criteria

- **SC-PKT-001**: The measurement matrix is complete and reproducible in `measurements.md` (9 configs × cold/warm, device + commands recorded).
- **SC-PKT-002**: The 80/20 gate is applied explicitly against `first_chunk_ms ≤ ~1551 ms`, `RTF < 1`, RSS/stability; the outcome is unambiguous.
- **SC-PKT-003**: The recorded decision matches the evidence; on NO-GO no integration work is authorized and Piper remains default.
- **SC-PKT-004**: The synthesis-vs-playback distinction is explicit; no end-to-end TTFA claim is made from synthesis-only data.
- **SC-PKT-005**: Production contracts and permissions are unchanged by the spike.
- **SC-PKT-006**: Fallback plan and license/consent preconditions are recorded for any future revival.

## 7. Assumptions & Dependencies

- `PLAN-80-20.md` defines phases 0–3 and the gate; `measurements.md` records the executed evidence.
- Reference device availability and debuggable APK (`DebugIngestActivity`) for any future end-to-end measurement.
- sherpa-onnx prebuilt arm64 C API (v1.13.8) is the working runtime; newer bundles may require ≥2.0.1 and a BOS fix.

## 8. Out of Scope

- Shipping or bundling a cloned voice in the APK; user-facing voice cloning; cloud inference.
- Modifying the production synthesis adapter, handoff format or closed voice vocabulary as part of the spike.
- Distributing gated weights or third-party voice audio without explicit license/consent clearing.
