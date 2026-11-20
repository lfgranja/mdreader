# Release (R7): 023 Directory UI Controls

- **Date**: 2026-11-18
- **Version**: v0.1.0 infrastructure/UI follow-up (no app SemVer bump; no Rust artifact change)
- **Validation**: R4 PASS (`.specify/validations/directory-ui-controls/validate.md`)

## Summary

Exposes the existing directory/chapter bridge actions in the canonical in-app surface: picker button, `directory-selected` refresh listener, and chapter navigation controls hidden outside directory mode.

## Changelog (IDs)

- Added: FR-DUI-001..007, SC-DUI-001..003; new test target `directory_ui_controls` (5 structural tests).
- Changed: five HTML bundles; `scripts/expected-tests.txt`; spec 010 T229/T230/T234 closed.
- Removed: dead optional `#btnPickDirectory` handler replaced by wired button.

## Rollback

Revert the commit; restore the five previous HTML bundles from git and remove the new test target from `scripts/expected-tests.txt`. No remote configuration changed.

## Residual risk / [NV]

- Device visual/interaction verification not performed; CI/merge pending at artifact creation time.
- T232/T233 remain open by explicit scope decision.
