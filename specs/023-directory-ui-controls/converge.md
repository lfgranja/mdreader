# Converge: 023 Directory UI Controls

**Date**: 2026-11-18
**Status**: implemented; PR/merge [NV] until observed

## Delivered

- Assess `directory-ui-controls` (intake/research/problem/concept/decision = go, Option A).
- Spec/plan/checklist/tasks/analyze/converge in `specs/023-directory-ui-controls/`.
- `src-tauri/src/status_view.html`: `#btnPickDirectory` (click + keydown), `#directory-selected` listener calling `refreshStatusView()`, `#chapterNav` group with `#btnPrevChapter`/`#btnNextChapter` using the `hidden` attribute toggled by `renderChapterNav()` from `chapterIndex`/`chapterCount`.
- Mirrored to `src-tauri/src/assets/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html`, `dist/index.html`, `src-tauri/dist/index.html` — all sha256 `dcefad7e…`.
- `src-tauri/tests/directory_ui_controls.rs`: 5 structural tests (new nextest target).
- `scripts/expected-tests.txt`: +1 target (`src-tauri::directory_ui_controls`).
- `specs/010-directory-reading/tasks.md`: T229/T230/T234 closed; T232/T233 unchanged.

## TDD evidence

- RED: `cargo nextest run -p src-tauri --test directory_ui_controls` → 2 failed (`chapter_nav_buttons_dispatch_existing_actions`, `chapter_nav_uses_hidden_attribute_not_display_none`), 1 passed (bundle parity), 2 not run.
- GREEN: same command after implementation → 5/5 passed.

## Host evidence

- `PROPTEST_CASES=32 bash scripts/verify.sh` → `VERIFY_EXIT=0` (gate 1–8 subset + supply chain) measured on the target checkout this session.
- Nextest census: live 137 targets, expected 137, diff empty.
- Bundle hashes: all five identical.

## Honest status

- Device visual/interaction verification [NV] — host structural checks are authoritative for this wiring change.
- CI/PR/merge: [NV] until opened and observed green.
- Rollback: revert commit; no remote config changed.
