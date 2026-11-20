# Release (R7): 010 Leitura Contínua de Diretórios de Markdown

- **Date**: 2026-11-20
- **Version**: v0.1.0 feature release (directory playback / ChapterQueue)
- **Validation**: R4 PASS (`.specify/validations/directory-reading/validate.md`)

## Summary

Adds hands-free continuous reading of Markdown/text directories via SAF `ACTION_OPEN_DOCUMENT_TREE`: background indexing/natural sort, Rust-owned `ChapterQueue` with gapless transitions, composite `DIRECTORY_BOOKMARKS` persistence, precise resume offer, contextual transport skip and resilient filtering of invalid entries.

## Changelog (IDs)

- Added: FR-001..FR-013, SC-001..004; `ChapterStatus`/directory actions (T185–T189), composite fingerprint + `DIRECTORY_BOOKMARKS` (T190–T191, T193–T194), `ChapterQueue` + JNI directory selection (T192, T194), US1/US2/US3 implementations (T195–T218), bridge chapter navigation and UI controls (T227–T234).
- Changed: `status_view.html` chapter UI, Kotlin SAF scanning, Rust queue/bookmark store; ROADMAP CONVERGED (49/49), hardware attestation.
- Removed: per-file manual open requirement for continuous listening.
- Task IDs: suffixed T226-* subtasks de-checkboxed in `tasks.md` to remove duplicate T226 IDs; wording preserved.

## Rollback

Revert the feature commit range (e.g. `1aa568d4` governance attestation and associated spec-010 code commits); restore previous `status_view.html`, Kotlin/JNI and `crates/{player,session}` sources from git; re-run gates 1–8. Existing `bookmarks.redb` retains a separate `DIRECTORY_BOOKMARKS` table — no destructive data migration on rollback.

## Residual risk / [NV]

- Hardware measurements are tied to the recorded device/conditions (TTFA 1,551 ms); no fresh gate run or CI/PR merge in this artifact.
