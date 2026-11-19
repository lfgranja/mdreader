# Tasks: 023 — Directory UI Controls

**Input**: `specs/023-directory-ui-controls/{spec,plan,analyze}.md`
**Assess decision**: `go` (Option A). **Tests**: TDD RED before implementation.

## Phase 1 — RED (write first)

- [x] T001 Write `src-tauri/tests/directory_ui_controls.rs` asserting: `#btnPickDirectory` markup + aria-label + click/keydown wiring; `directory-selected` listener calling `refreshStatusView`; `#btnPrevChapter`/`#btnNextChapter` markup + actions + `hidden` toggle. (recorded failing output in converge.md)
- [ ] T002 Confirm RED: `cargo nextest run -p src-tauri --test directory_ui_controls` fails on the new assertions.

## Phase 2 — Implementation

- [ ] T003 Edit `src-tauri/src/status_view.html`: add directory button, chapter-nav group, CSS (existing tokens only), `els` entries, click/keydown handlers, `directory-selected` listener, `renderChapterNav()` called from `renderAll()`.
- [ ] T004 Copy canonical view to `src-tauri/src/assets/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html`, `dist/index.html`, `src-tauri/dist/index.html`; verify all five sha256 equal.
- [ ] T005 Mark spec 010 follow-ups T229/T230/T234 done; keep T232/T233 open.

## Phase 3 — Verification

- [ ] T006 Targeted nextest GREEN + fmt + clippy.
- [ ] T007 Analyze (read-only), converge (append-only), validate R4, release R7.
- [ ] T008 PR → CI green → deliberate merge to `dev` → cleanup.

## Dependencies

T001–T002 before T003; T003 before T004; T004 before T006; T006 before T007; T007 before T008.
