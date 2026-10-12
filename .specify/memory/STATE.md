# Session state — 001-android-media-shell (US4/US5 implementation)

Updated: 2026-10-11 (post-checkpoint continuation).

## Position
- Wave 1 LANDED: T057, T061c, T065, T073, T058–T060 (`57bb525`), T075 (`247ecbb`).
- Phase-11 test fixes landed: T070 (`91f2f4e`), prop_resume `#[test]` (`195437d`),
  prop_fallback E0382 clones (`6e08db8`).
- Wave 2 RUNNING (6 background agents, disjoint files):
  T061 surface.rs · T063 session cli/main · T065b status_view.rs ·
  T074 diag scan/cli/main · T064 Residency.kt · T076 recovery.rs.
- After wave 2: verify each agent's suite, orchestrator fixes any test-vs-impl
  mismatch, commit atomically (stage ONLY task-owned files), check off tasks.md.

## Next
1. Wave 3: T062 MediaMirror.kt (after T061), T078 failure.rs (after T065+T076).
2. Phase 8: T079 (regenerate scripts/expected-tests.txt from
   `cargo nextest list --workspace`), T080–T083 hardening, T084→T087 gates.
3. On-device trials (T086) hardware-blocked — document carried-forward in VERIFICATION.md.

## Watch-items
- Kotlin compile verification SDK-gated (ANDROID_HOME unset, kotlinc absent).
- `cargo clippy` workspace-wide fails on pre-existing engine-core lib lints
  (clippy 1.98 lint_groups_priority) — T080 scope.
- src-tauri/Cargo.toml lacks `[lints] workspace = true` — T080 lint-drift gate;
  test whether adding it breaks the build before committing.
- nextest 0.9.133: `--workspace-type all` unsupported — use `--workspace`.
- Commit discipline: stage ONLY task-owned files explicitly; never `git add -A`
  while parallel agents are mid-edit. Agents never commit and never edit tests.
