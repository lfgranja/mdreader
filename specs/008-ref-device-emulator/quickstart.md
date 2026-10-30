# Quickstart: VERIF-EMULATOR — Structural Verification Layer (Option A)

**Feature**: `specs/008-ref-device-emulator/`  
**Branch**: `005-adopt-design-sona` (or feature branch `008-ref-device-emulator` if branched)  
**Status**: Draft — structural verification layer (not physical `gate-8-full`)

## Prerequisites

- Android SDK AVD (API 29+) installed locally or available in isolated CI container.
- `mdreader` build artifact (from `005-adopt-design-sona` or feature branch) installed on AVD.
- `adb` available to interact with AVD.
- No source modifications to `crates/`, `src-tauri/`, `android-shell/` required.
- `verify.sh` (physical gate) not required for structural verification; do not alter `verify.sh`.

## Validation Steps

1. **Launch AVD** (`API 29` baseline; document any version delta if reference device differs).
2. **Install build** on AVD; confirm `mediaPlayback` service declared in `AndroidManifest.xml` (manual edit; no `mobile::update_manifest`).
3. **Open `003-reference-device` parsed Markdown**; press Play.
4. **Confirm structural contracts** (observable via framework / `adb logcat` / AVD controls):
   - Notification displays title + current section title + progress (determinate/indeterminate per total known) — no body/rate/voice.
   - `MAY_DUCK` behavior: temporary interruption ducks to ~20% and continues.
   - Permanent interruption (simulated via focus loss / audio change): playback pauses with 3-second reason; never auto-resumes.
   - Resume after pause: 3-second offer (`continue` default); resume by `blake3` fingerprint (document edit = different fingerprint = no resume).
5. **Confirm diag JSONL**: `diag` writes to app-private storage (30-day trim); no transmission; `EventKind` 9 variants present.
6. **Write attestation** (`VERIF-EMULATOR.md` or equivalent):
   - List verified contracts (manifest / service / notification / focus / diag / resume / rate ladder UI).
   - List exclusions (audio-output `22050 Hz`, `SC-009` ≤3 s, Bluetooth/car, telephony/alarm, 10/10).
   - Reference `verify.sh` rules for excluded physical-only gates.
   - `Record AVD version, branch (`005-adopt-design-sona` / feature), date, sign-off.`
   - `Reference `VERIF-EMULATOR.md` §5 (security/consent boundary — no secrets: .env/.pem/.key/token prohibition; identifier-pattern only) and §6 (no false-equivalence claim); diagnostic references use identifier-pattern only (`≤200` chars, `error` required on failure/refusal).`
7. **Confirm no false-equivalence claim**: Search attestation for language implying `gate-8-full`; must find 0 hits.

## Expected Outcomes

- `VERIF-EMULATOR.md` produced and readable.
- No new source edits; `git status` unchanged except attestation file.
- `checklist` updated if any new structural evidence changes item state (likely no change; all pass from design).
- Next: `/speckit.clarify` if any ambiguity found during structural run; else `/speckit.converge` for `003-reference-device` with `VERIF-EMULATOR` evidence.

## References

- `spec.md`: `specs/008-ref-device-emulator/spec.md` (FR-001..FR-006; SC-001..SC-004)
- `problem.md`: `specs/008-ref-device-emulator/problem.md`
- `contracts/VERIF-EMULATOR-structural.md`: structural verification contract references
- `data-model.md`: `specs/008-ref-device-emulator/data-model.md` (attestation document model)
- `decision.md`: `specs/assessments/ref-device-emulator/decision.md` (go — Option A)
- `ROADMAP.md`: line 27 (`VERIF-EMULATOR` + `gate-8-full` DEFERRED), line 111 (80/20 updated)
- `VERIF-EMULATOR.md`: `specs/008-ref-device-emulator/VERIF-EMULATOR.md` (attestation template)
