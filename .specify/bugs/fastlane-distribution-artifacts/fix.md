# Bug Fix: remove fastlane/metadata distribution artifacts (Gate 2 noscope)

- **Slug**: fastlane-distribution-artifacts
- **Fixed**: 2026-11-15
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

CI run #55 (PR #211 merge commit `21a74141`) — sherpa-onnx fixes and router fix worked (Gate 1 passed, Gate 2 compiled, router tests green) — but Gate 2 failed at the `no_distribution_artifacts_in_tree` absence guard in `src-tauri/tests/test_noscope.rs:478` because `fastlane/metadata` existed in the tree (dev-only, commit `0e6311ed`, not on `main`). The guard enforces a maintainer-only sideload with nothing published to a store.

Fix: deleted the 8 tracked `fastlane/metadata/` files (android/en-US + android/pt-BR descriptions, titles, icons) and removed the now-empty directories.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `fastlane/metadata/android/en-US/full_description.txt` | removed | store listing description |
| `fastlane/metadata/android/en-US/images/icon.png` | removed | icon |
| `fastlane/metadata/android/en-US/short_description.txt` | removed | short description |
| `fastlane/metadata/android/en-US/title.txt` | removed | title |
| `fastlane/metadata/android/pt-BR/full_description.txt` | removed | pt-BR description |
| `fastlane/metadata/android/pt-BR/images/icon.png` | removed | pt-BR icon |
| `fastlane/metadata/android/pt-BR/short_description.txt` | removed | pt-BR short description |
| `fastlane/metadata/android/pt-BR/title.txt` | removed | pt-BR title |

Also restored `crates/adapter-synthesis/src/synth.rs::prewarm_default_voice()` (removed accidentally during earlier git checkout operations; `src-tauri/src/lib.rs` depends on it).

## Diff Highlights

8 deletions, ~42 lines removed (icons binary). No Rust code changed beyond the synth.rs restore.

## Tests Added or Updated

None — existing `no_distribution_artifacts_in_tree` guard pins correct behavior; deleting the artifacts makes it pass.

## Local Verification

- `cargo test -p src-tauri --test test_noscope no_distribution_artifacts_in_tree` → exit 0, 1 passed.
- `cargo fmt --all -- --check` → exit 0.
- `cargo test -p engine-core router` → exit 0, 4 passed (router fix still green).
- `cargo test -p adapter-synthesis --lib voice_catalog` → exit 0 (catalog tests green).

## Deviations from Assessment

None — applied exactly the proposed deletion of `fastlane/metadata`. The synth.rs `prewarm_default_voice()` restore was necessary because `src-tauri/src/lib.rs` calls it and the build was failing without it; unrelated to this bug but required for the noscope test to compile/run.

## Follow-ups

- Confirm CI run triggered by `244bd6bf` shows Gate 2 green on GitHub (PR #211).
- PR #211 bundles: sherpa-onnx vendored archive + `SHERPA_ONNX_ARCHIVE_DIR`, sherpa-onnx-sys `build.rs` content-checked cache, router `required_capabilities` fix, and fastlane/metadata deletion. All needed for Gate 2 to pass end-to-end.

Slug: fastlane-distribution-artifacts
## 2026-11-20 — Reconciliação do guard (ciclo de coerência documental)

O bug está RESOLVIDO no estado atual: `src-tauri/tests/test_noscope.rs:466-476` (`no_distribution_artifacts_in_tree`) agora verifica apenas `play-store-listing`, `CHANGELOG-STORE` e `listing.md`, com comentário explícito de que `fastlane/metadata` é intencionalmente mantido para F-Droid rc2. O `test.md` anterior ("Reopen", 2026-11-16) registrou o estado antes dessa reconciliação; não foi reescrito (append-only). Evidência: `cargo test -p src-tauri --test test_noscope` verde no gate 2 de `scripts/verify.sh` (VERIFY_EXIT=0, 2026-11-20).
