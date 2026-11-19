# Idea Research: Directory UI Controls

- **Slug**: directory-ui-controls
- **Created**: 2026-11-18
- **Evidence confidence (overall)**: high

## Users & Demand

- Readers using directory reading need to open a folder and move between chapters from the in-app surface; the bridge actions exist but are unreachable — [source: `origin/dev:src-tauri/src/lib.rs:4495-4497`] (confidence: high)

## Prior Art

- The view already has an optional JS handler for `#btnPickDirectory` (`status_view.html:1244-1247`) but no button markup — [source: local file] (confidence: high)
- `#titleCard` also opens the document picker with click + keydown (`status_view.html:1342-1348`) — pattern to reuse (confidence: high)
- `shell_jni.rs:1456` emits `tauri://channel/directory-selected`; the view listens for `mirror-publish` and `rms` via `window.__TAURI__.event.listen` (`status_view.html:1363,1449`) — pattern to reuse (confidence: high)

## Market & Context

- Cost of doing nothing: the P1 directory-reading feature is only controllable via external intents/ADB; normal users cannot switch chapters in-app.

## Data & Constraints

- All five tracked HTML copies are byte-identical (sha256 `e3322b82…`): `dist/index.html`, `src-tauri/dist/index.html`, `src-tauri/src/status_view.html`, `src-tauri/src/assets/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html`. [source: `sha256sum`, measured]
- Surface audit forbids literal `display:none` corpses (`src-tauri/src/surface_audit/inventory.rs:31-51`); `data-elements="10"` and `role="status"` count == 2 are pinned by tests (`tests/contract_census.rs:31`, `tests/005-red/test_t202_aria_hidden_status.rs:11`). Use `hidden` attribute + JS toggle, no new tokens/status roles. [source: tests, measured]
- Surface tests pin existing `aria-label` strings for transport buttons (`tests/us2_rate_blind.rs:24-31`); new labels must be distinct. [source: tests, measured]
- Rust authority unchanged: UI only dispatches existing actions; no playback logic in frontend. [source: AGENTS.md]

## Evidence Against the Idea

- Adding controls increases surface complexity and risks accessibility-count regressions. Mitigation: structural tests asserting exact invariants and bundle parity. — [inferred] (confidence: medium)

## Gaps & Open Questions

- None blocking.

## Sources

- `specs/010-directory-reading/tasks.md` (host: local repo, policy: allowlisted path)
- `src-tauri/src/lib.rs`, `src-tauri/src/shell_jni.rs`, `src-tauri/src/status_view.html` (host: local repo, policy: allowlisted path)
- `src-tauri/src/surface_audit/inventory.rs`, `tests/contract_census.rs`, `tests/005-red/*`, `tests/us2_rate_blind.rs` (host: local repo)
