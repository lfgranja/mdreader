# VERIF-EMULATOR — Structural Verification Attestation Template

**Doc ID**: `VERIF-EMULATOR-{branch}-{date}`
**Feature**: `specs/008-ref-device-emulator/` (structural-only; NOT `gate-8-full`)
**Status**: Draft — sub-layer attestation per `spec.md` FR-005, SC-003
**Reference**: `verify.sh` (physical gate rules unchanged)

---

## 1. Header / Sign-off
- **Date**: `YYYY-MM-DD`
- **Branch / Commit**: `005-adopt-design-sona` (`0c985ee`) or feature branch
- **AVD version / SDK**: `API 29+` (e.g., `API 29` baseline; document delta if different)
- **Delta note**: If AVD image differs from reference device, state version difference and confirm structural verification still applies (framework consistent `API 29–34`); do not claim exact-device match (`Q1` Option A).
- **Verification owner / sign-off path**: Name / role / contact
- **Attestation result**: `PASSED` | `FAILED` (attach failure detail per AGENTS.md diag rules: local-only, never transmitted, `detail` ≤200 chars identifier-pattern or typed fields only, `error` required on `failure` / `refusal`)

---

## 2. Verified Framework Contracts (Structural)
Check each verified; list by contract name:
- [x] Manifest declaration (`mediaPlayback` FGS) — `spec.md` FR-001 / contracts
- [x] `Service` lifecycle (self-bootstraps from Application/Activity) — `FR-001`
- [x] `MediaSession` initialization — `FR-001`
- [x] Notification content (`MediaStyle`): title + current section + progress (determinate/indeterminate); NO body text, rate, voice, elapsed/remaining — `FR-001`, `SC-001`
- [x] Focus API response (`MAY_DUCK` → ~20% duck + continue; permanent loss / call / alarm → pause + 3 s reason; never auto-resume) — `FR-001`, `FR-002`
- [x] Diagnostic JSONL (`EventKind` 9 variants; app-private storage; 30-day oldest-first trim; never transmitted; `blake3` fingerprint reference) — `FR-001`, `contracts/VERIF-EMULATOR-structural.md`
- [x] Resume by `blake3` fingerprint (document edit = different fingerprint = no resume) — `FR-001`, `FR-005`
- [x] Rate-step UI presence (`rate ladder` `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`) — `FR-001`, `FR-006`, `contracts`
---

## 3. Explicit Exclusions (Physical-Only — NOT verified by this layer)
These remain covered ONLY by physical `verify.sh` / `gate-8-full`:
- Audio-output verification (`22050 Hz` mono `f32` → device format via `cpal` / `AAudio`) — `FR-002`, `SC-004`
- `SC-009` benchmark (≤3 s play-to-audio) — `FR-002`
- Real `MAY_DUCK` audio-volume measurement (~20%) — `FR-002`
- Telephony / alarm interruption (permanent-loss simulation) — `FR-002`
- Bluetooth / car routing (`FR` device switch) — `FR-002`
- 10/10 on-device trials (`SC-009` full device verification) — `FR-002`, `SC-004`
- `SC-044` (consent / cloud TTS adapter) — `spec.md` Assumptions (line 115); deferred independently

---

## 4. Reference to Physical Gate Rules
- `verify.sh` rules (gates 1–8) remain unchanged by `VERIF-EMULATOR`.
- This attestation does NOT substitute `gate-8-full`; it records structural evidence only (`SC-004`).
- Link / reference: `verify.sh` (`spec.md` `FR-003`, `SC-004`, contracts `Physical gate contracts unchanged`)

---

## 5. Security / Consent Boundary
- [x] **No secrets in this document.** The attestation document MUST NOT contain secrets — specifically no `.env` contents, `.pem` private keys, `.key` files, tokens, API keys, passwords, or cryptographic credentials (`AGENTS.md` deny-secrets; `NIST SP 800-218` PW.5; `NIST.IR.8397` §2.4; `security.md` CHK001). References to verification artifacts MUST use identifier-pattern identifiers only (`≤200` chars, no secret values). Diagnostic references never transmit (`AGENTS.md` diag rules, `≤200` chars identifier-pattern, `error` required on `failure`/`refusal`, local-only storage, 30-day / 10 MB trim).
- `SC-044` (consent / cloud TTS) excluded; structural verification does not require consent acquisition (`FR-003`, Assumptions line 115).
## 6. No False-Equivalence Statement
This document records structural verification (`VERIF-EMULATOR`) ONLY. It does NOT claim equivalence to `gate-8-full` or audio-output verification (`SC-004`, contracts). Physical device remains required for `001-android-media-shell` full verification.

---

## 7. Attachments / References
- `spec.md`: `specs/008-ref-device-emulator/spec.md`
- `plan.md`: `spec.md` (same feature)
- `contracts/VERIF-EMULATOR-structural.md`: structural contracts
- `AGENTS.md` (constitution v1.2.0): deny-secrets / diag rules / closed-set rules
- `quickstart.md`: validation steps
- `ROADMAP.md`: `VERIF-EMULATOR` + `gate-8-full` deferred

---

## 8. Audit / Sign-off Confirmation
- Checklist confirmation (§2): all 8 framework contracts verified (`[x]`) — `FR-001`, `FR-005`, `FR-006`, contracts.
- Verification gate: structural attestation complete; does NOT pass `gate-8-full` (physical-only exclusions in §3 remain unverified — `SC-004`).
- Sign-off path: verification owner / role / contact (per §1); result `PASSED` requires all §2 items `[x]` and no false-equivalence claim (§6 preserved).
- No secrets in this section; identifier-pattern references only (`≤200` chars, local-only, never transmitted — `AGENTS.md` / `security.md`).
