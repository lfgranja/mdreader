# Session state — 001-android-media-shell (US4/US5 implementation)

Updated: 2026-10-12 (wave 2 fully landed; workspace GREEN).

## Position
- Wave 1 LANDED: T057, T061c, T065, T073, T058–T060 (`57bb525`),
  T075 (`247ecbb`).
- Wave 2 LANDED: T064 Residency.kt (`0ad77cc`) + shell-ipc.md
  residency intent set; T074 diag scanners + CLI (`1f3d72f`);
  T065b status view (`99e71c4`); T063 session CLI (`aa3e10f`);
  T076 recovery machine (`4b9f78b`, 216/216 player + 517/517
  workspace); T061 surface resolver (`97030af`).
- Orchestrator Phase-11 test-defect repairs landed with their
  tasks: prop_resume `#[test]` (`195437d`), prop_fallback E0382
  clones (`6e08db8`), US5 gated-suite repairs + US4 ladder
  off-by-one ×2 + L9 duration oracle + fixture token (in
  `4b9f78b` / `97030af`).
- Workspace: `cargo nextest run --workspace` — 517 passed,
  1 skipped. Baseline before US4/US5 impl: 299+1.

## Next
1. Wave 3 (launch now, parallel, disjoint files):
   T062 MediaMirror.kt (deps: T061 ✓) · T078 failure.rs
   (deps: T065 ✓ + T076 ✓).
2. Gates: T066 (US4 GREEN) and T079 (US5 GREEN — regenerate
   scripts/expected-tests.txt from `cargo nextest list
   --workspace`; 27 targets pre-wave-2, re-count now).
3. Phase 8: T080–T083 parallel hardening → T084 host gates →
   T085 quickstart → T086 on-device (hardware-blocked, document
   carried-forward) → T087 VERIFICATION.md attestation.

## Watch-items
- Kotlin compile verification SDK-gated (ANDROID_HOME unset,
  kotlinc absent) — T062/T064 written to established mirror
  patterns (FocusMirror.kt / Residency.kt idiom).
- `cargo clippy` workspace-wide fails on pre-existing
  root-Cargo.toml lint-group priority errors (clippy 1.98
  `nursery`/`cargo`, Cargo.toml:26/27) + engine-core
  cli.rs:220/327/389, trait_.rs:212, doc-structure — T080
  scope; nextest gates unaffected.
- src-tauri/Cargo.toml lacks `[lints] workspace = true` —
  T080 lint-drift gate; test whether adding it breaks the
  build before committing (pre-existing `.expect` at
  src-tauri/src/lib.rs:97 would violate `expect_used`).
- nextest 0.9.133: `--workspace-type all` unsupported — use
  `--workspace`; stop-on-first-failure per binary — verify
  per-target for complete results.
- proptest 1.11: `prop_assert_eq!` moves operands — reused
  operands need `.clone()`/`.as_str()` at the assert site.
- Commit discipline: stage ONLY task-owned files explicitly;
  never `git add -A` while parallel agents are mid-edit.
  Agents never commit and never edit tests.
