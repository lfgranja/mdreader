# Research — VERIF-EMULATOR (008-ref-device-emulator)

**Status**: Complete — all 6 clarification gaps (G1–G6) resolved; Option A confirmed for all.
**Branch**: `005-adopt-design-sona` (`0c985ee`, 51 dirty); verification runs isolated.
**Date**: 2026-10-29

---

## Clarification Decisions (User-selected, Option A for all)

| ID | Question | User Decision | Evidence Source |
|---|---|---|---|
| G1 / Q1 | AVD image / API level? | `API 29` baseline; document delta; no exact-device claim | `spec.md` line 112; `quickstart.md` step 1 |
| G2 / Q2 | Sub-layer vs `gate-8-full` replacement? | Sub-layer only; physical device preserved | `spec.md` FR-003; contracts; `decision.md` |
| G3 / Q3 | `SC-044` interaction? | Independent; deferred; `offline_default` preserved | `spec.md` Assumptions line 115; `roadmap.md` |
| G4 / Q4 | Attestation format? | `VERIF-EMULATOR.md` with exclusions; separate document; `verify.sh` preserved | `spec.md` FR-005; `VERIF-EMULATOR.md` (template created); `quickstart.md` step 6 |
| G5 / Q5 | CI / isolation from dirty tree? | Isolated AVD / CI job / container; `git status` unchanged | `spec.md` FR-004 / SC-002; `problem.md` G5 |
| G6 / Q6 | Assessment identity? | Keep separate `ref-device-emulator` assessment; link, not subsume | `spec.md` Q6; `decision.md` |

---

## Design Artifacts Produced

- `VERIF-EMULATOR.md` (new) — attestation template (sections 1–8) per `FR-005`; links `spec.md`.
- `spec.md` updated — `FR-005` references template; new `Verification failure isolation and diagnostic boundary` clause (lines 77–97) linking failure isolation to `gate-8-full` exclusion and `AGENTS.md` diag rules.
- `checklists/requirements.md` — all 16 items `[x]`.
- `checklists/security.md` — all 6 items `[x]` (CHK001/CHK005 resolved by evidence and clause).
- `checklists/verification.md` — all 9 items `[x]` (CHK001 resolved by linked template).

## Evidence Sources (Research Agents Completed)

1. **`clarify-ver-chk001`** (completed 2m27s): SLSA Provenance v1.2 / Verification Summary (`slsa.dev`); in-toto Framework (`github.com/in-toto/attestation`); CISA Self-Attestation Form (`Self_Attestation_Common_Form_FINAL_508c.pdf`); NIST SP 800-161 Rev.1 / SSDF (`csrc.nist.gov`). **Recommendation applied**: separate `VERIF-EMULATOR.md` template (not embedded) with structured sections, reference to `verify.sh`, no false-equivalence claim.
2. **`clarify-sec-chk005`** (completed 3m40s): IETF draft `draft-lu-rats-federated-attestation-infrastructure-00` (§6.1–6.3); NIST SP 800-193 (`csrc.nist.gov`); NIST SP 800-155 (`csrc.nist.gov`); Physical AI Safety `Verification Gate` taxonomy (2026); `AGENTS.md` (§81–84); `docs/diagnostic-record.md`; contracts `VERIF-EMULATOR-structural.md`; internal checklist `audio-safety.md`. **Recommendation applied**: explicit clause in `spec.md` linking structural failure isolation to `gate-8-full` exclusion and `AGENTS.md` diag constraints (`local-only`, `≤200` chars identifier-pattern, `error` required on `failure`/`refusal`, no secrets).
3. **`clarify-sec-chk001`** (completed 6m31s): SLSA Provenance (`slsa.dev`); in-toto (`github.com/in-toto/attestation`); NIST SP 800-218 (`csrc.nist.gov`, PW.5); NIST.IR.8397 §2.4 (`csrc.nist.gov`); NIST EO 14028 Supply Chain Guidance; GitGuardian analysis (`blog.gitguardian.com`); EU EUDI Attestation Rulebook Template v1.5; `cosign`/`sigstore` docs; repo evidence (`AGENTS.md`, `security.md` CHK001, `VERIF-EMULATOR.md` §5). **Recommendation applied**: explicit secret-prohibition clause in `VERIF-EMULATOR.md` §5 and `spec.md` FR-005 reference.

---

## Design Decisions Recorded (Not Re-Litigated)

- **No new crate required** (`cleanroom-core` stays empty) — artifacts live in `specs/003-reference-device/verification/` and `specs/008-ref-device-emulator/`.
- **Closed-set preserved** — no new `EngineError` variants, rate ladder members (`[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]` unchanged), or `EventKind` variants added.
- **`Tauri` 2.12.1 preserved** — no upgrade; manual manifest edit rules unchanged.
- **Physical `gate-8-full` deferred** — `SC-009` (≤3 s play-to-audio), audio-output (`22050 Hz` mono → device format), Bluetooth/car routing, telephony/alarm interruption, 10/10 on-device trials, and `SC-044` (consent/cloud TTS) remain outside `VERIF-EMULATOR`.
- **No false-equivalence claim** — `SC-004` enforced by spec language, contracts, template (§6), and checklist (`SC-004` measurable: string search = 0 hits).

---

## Verification Gates Applicable

Per `AGENTS.md` and `spec.md` `SC-001`–`SC-004`:
- Gate 1 (format): `VERIF-EMULATOR.md` exists, sections complete.
- Gate 2 (tests): structural checklist passes (observable via AVD framework APIs / `adb logcat`).
- Gate 3 (docs): `spec.md` / contracts / quickstart reference `VERIF-EMULATOR.md`.
- Gate 4 (lint): `spec.md` / contracts / quickstart / `VERIF-EMULATOR.md` have no lint issues (documentation only; no Rust source edits).
- Gate 5 (supply chain): no new dependencies; no `Cargo.toml` / `Cargo.lock` changes.
- Gate 6 (deny-list): no denied crate added.
- Gate 7 (closed-set): no new engine/rate/event variants.
- Gate 8 (pipeline): byte-budget / SSML capability declared; sentence-level alignment exercised.
- Gate 9 (android build): `cargo tauri android build --target aarch64` produces artifact when implementation completes (`/speckit.implement`).

---

## References

- `.specify/assessments/ref-device-emulator/` (`intake.md`, `research.md`, `problem.md`, `concept.md`, `decision.md`): `go` — Option A confirmed.
- `specs/003-reference-device/` (parent contracts, data-model, plan).
- `specs/001-android-media-shell/` (`gate-8-full`, `VERIFICATION.md`).
- `AGENTS.md` (constitution v1.2.0 / 2.0.0): deny-secrets, diag rules, closed-set rules, verification gates.
- `ROADMAP.md`: line 27 (`VERIF-EMULATOR` + `gate-8-full` deferred), line 111/112.
