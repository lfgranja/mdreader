# 005-adopt-design-sona — Remediation Checklist (verified only)

Scope: verified remediated items from analysis A1–A10 (plan 005-adopt-design-sona-plan). Unremediated / deferred items are NOT listed here (see requirements.md / plan.md for deferred device / T029–T031). Truthful attestation only; references are RED file names + commit SHAs from repo history.

Verification command (reference, RED compile check):
  rustc --edition 2021 --test

---

## 1. SC duplication removed
- Status: REMEDIATED (PASS)
- Reference: test_sc_duplication_red PASS
- Commit: c39b2ef (005 P1: SC-001 consolidated, duplication removed, deferred owner named; device-build/gate-9)
- Evidence: spec.md SC block consolidated to single SC-001..SC-007; duplicate SC block removed; count = 1 (grep -c SC-001 = 1).
- Deferred / not claimed: none.

## 2. SC-001 split resolved (host + device + owner)
- Status: REMEDIATED (PASS)
- Reference: test_sc001_split_red PASS
- Commit: c39b2ef
- Evidence: spec.md SC-001 line 156: host leg = deterministic 10-hex census verified by host tool over touched files; device leg = deferred (owner=device-build / gate-9 / SC-003 mirror — visual trial deferred, espelho da postura SC-003, nunca passagem silenciosa); no new token outside closed set (D2).
- Contract / owner named truthfully: device-build / gate-9.

## 3. FR-006 RED + contract ref
- Status: REMEDIATED (PASS)
- Reference: test_fr006_fr007_red PASS
- Commits: 81a9987 (FR-006 / FR-007 RED + contract refs added; RED pass) + 6bdf16f (FR-006/FR-007 clauses actually appended to contracts/surface-audit.md; gap from 81a9987; spec references verified 7x surface-audit.md; RED unchanged)
- Evidence: contracts/surface-audit.md §FR-006 (line 43–45): Pre = audio path available / mute state indeterminate; Post = sound plays regardless of mute; Invariant = earcon never blocks access (redundancy, not barrier). spec.md lines 137 references contracts/surface-audit.md §FR-006, contracts/earcon-mapping.md haptic+silent path.

## 4. FR-007 RED + contract ref
- Status: REMEDIATED (PASS)
- Reference: test_fr006_fr007_red PASS (same test, paired clause)
- Commits: 81a9987 + 6bdf16f
- Evidence: contracts/surface-audit.md §FR-007 (line 43, 45): Pre = MediaStyle notification / lock-screen render; Post = NO brand logo / sound-wave glyph shown; Invariant = accent-only (`#F2F0E6`) + title + progress; logo waves vetoed (FR-028/FR-029). spec.md line 138 references contracts/surface-audit.md §FR-007 and FR-028/FR-029.

## 5. Contract drift corrected
- Status: REMEDIATED (PASS)
- Reference: test_contract_drift_red PASS
- Commit: 05990b5 (005 P3: contract drift corrected + extensions.yml documented (deferred device); RED pass)
- Evidence: contracts/surface-audit.md authored 2026-10-26 with FR-006 / FR-007 clauses; contract name refers consistently to surface-audit.md (not a drifted alias); spec.md references use same file (7x verified per 6bdf16f message).

## 6. extensions.yml documented truthful deferred device
- Status: REMEDIATED (PASS) — deferred device documented truthfully, NOT claimed verified
- Reference: test_extensions_empty_red PASS
- Commit: 05990b5
- Evidence: .specify/extensions/extensions.yml (verified 2026-10-26, P3): device.value = Android API 29+; verified = false; gate = T029-T031 (deferred — owner=device-build); justification = "No Android device present (no /system/build.prop, no /dev/android); device leg of SC-001 / SC-006 deferred per finding A7 / SC-001 deferred device. Will verify when device-build delivers." No false claim of device verification.

## 7. Property census closed (10 hex)
- Status: REMEDIATED (PASS)
- Reference: test_property_census_closed_red PASS
- Evidence: spec.md SC-001 line 156: "census determinístico de 10 hex tokens (8 sona + 2 pinos de sinal `--signal-caution` `#FFD400`, `--signal-failure` `#FF4D63`)"; closed set; host tool verifies over files touched; no new token outside set (D2); SC-006 references same closed hex set + WCAG 2.1 AA + inventory of elements.
- Note: census is closed and verified on host; device leg remains deferred (see item 6 / SC-001 device leg); this is truthful, not overstated.

---

## Deferred / not remediated (truthful — excluded from PASS list above)
- Device verification T029–T031: deferred (no Android device present); owner = device-build / gate-9; extensions.yml records false + justification.
- Visual trial / SC-003 device leg: deferred mirror of SC-003 posture.
- Property census device-leg verification: follows device build.

## Verification command reference
  rustc --edition 2021 --test
(Used to compile RED test references; RED files named per items 1–7 above; actual RED pass recorded in commits cited.)
