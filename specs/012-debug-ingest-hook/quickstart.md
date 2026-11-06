# Quickstart: 012-debug-ingest-hook

Validation/run guide for the debug-only ingest automation seam (T184/T226 programmatic scope), FR-011 durable overlay workflow, and FR-010 release-exposure evidence. This is **not** full implementation code.

## Prerequisites

- Repo root: `/home/luis/development/mdreader` (or current checkout).
- Android SDK/ADB available (example: `~/Android/Sdk/platform-tools/adb`).
- Debug-capable Android target: physical device or emulator with the **correct debug ABI artifact** installed. Current known physical device: `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` (2201117TY, SDK 33, arm64-v8a).
- App package: `app.mdreader`.
- Feature docs: [`spec.md`](./spec.md), [`contracts/debug-ingest-automation.md`](./contracts/debug-ingest-automation.md), [`contracts/release-exposure-evidence.md`](./contracts/release-exposure-evidence.md), [`contracts/t184-t226-residual-tracking.md`](./contracts/t184-t226-residual-tracking.md), [`contracts/durable-overlay-materialization.md`](./contracts/durable-overlay-materialization.md).
 +  - Before coding precedence-dependent behavior/tests, confirm actual `path` + `uri`/intent-data precedence in `DebugIngestActivity.kt` / `DebugIngestHook.kt` (current behavior: `--es path` takes precedence over intent `data`/`--es uri`; `file://` data/URI is routed via its path where supported) and synchronize contract, data-model, tasks, and tests.
## 1. FR-011 — Apply/verify durable debug overlay (CHK018 run points)

Run `check-debug-ingest-overlay.sh` (non-mutating, fail-closed) and `apply-debug-ingest-overlay.sh` when check fails or canonical/targets changed at: local bootstrap/setup; after `tauri android init` or known Android gen regeneration; after canonical overlay/hook/debug-manifest changes; after Tauri/Android toolchain/template/Gradle/AGP/NDK/SDK changes affecting gen; in CI bootstrap/Android debug jobs for Spec 012; before T184/T226 validation campaigns or SC-006 evidence.

```bash
# planned/contracted commands (implement in tasks)
./scripts/apply-debug-ingest-overlay.sh
./scripts/check-debug-ingest-overlay.sh   # fail closed if missing/stale/forbidden
```

Expected behavior:
- Canonical sources remain outside ignored generated output (normative root `specs/012-debug-ingest-hook/android-overlays/app/src/` + `android-shell/src/main/kotlin/app/mdreader/`); see [`contracts/durable-overlay-materialization.md`](./contracts/durable-overlay-materialization.md).
- Managed debug manifest/androidTest targets in `src-tauri/gen/android/app/src/{debug,androidTest}/` are created/updated only as idempotent materialization.
- androidTest is required in apply/check only when its canonical overlay source exists; otherwise check reports `not-applicable`.
- Check fails closed if required overlay is missing, stale, forbidden, or debug hook identifiers appear in forbidden main/release locations.
- Narrow patching of generated files is **not** allowed by default (CHK021); any unavoidable exception requires recorded decision, idempotent reapplied check, and fail-closed behavior.
## 2. Build/install debug artifact

```bash
# host gates as needed
./scripts/verify.sh

# Android debug build (gate 9 is authoritative outside verify.sh)
cargo tauri android build --target aarch64 --debug
# or project-equivalent debug Gradle target for the selected ABI
```

Install the ABI-appropriate debug APK and confirm the debug hook component resolves:

```bash
adb -s <serial> install -r -d -g <path-to-debug-apk>
adb -s <serial> shell dumpsys package app.mdreader | grep -n 'DebugIngestActivity\|DEBUG_INGEST'
```

Expected: debug build shows debug ingest component/action. **Do not** use this as release evidence.

## 3. Prepare accessible fixtures (SC-001 precondition)

Fixtures MUST be app-accessible to the debug/test process. Prefer app-internal storage on debug builds (`run-as` available). `/sdcard` paths may exist but be unreadable by the app process; inaccessible fixtures produce `blocked:fixture-inaccessible` or explicit diagnosable results and are excluded from SC-002 denominator.
```bash
# example pattern
adb -s <serial> push /tmp/mdreader-test-fixtures /data/local/tmp/mdreader-test-fixtures
adb -s <serial> shell "run-as app.mdreader mkdir -p /data/user/0/app.mdreader/files/mdreader-test-fixtures/md-dir"
adb -s <serial> shell "run-as app.mdreader cp /data/local/tmp/mdreader-test-fixtures/t184-teste.md /data/user/0/app.mdreader/files/mdreader-test-fixtures/"
adb -s <serial> shell "run-as app.mdreader cp /data/local/tmp/mdreader-test-fixtures/md-dir/* /data/user/0/app.mdreader/files/mdreader-test-fixtures/md-dir/"
```

## 4. Run programmatic T184/T226 validation (debug seam)

```bash
./scripts/automate-t184-t226-hook.sh <serial> app.mdreader
```

Direct ADB smoke:

```bash
adb -s <serial> shell am force-stop app.mdreader
adb -s <serial> logcat -c
adb -s <serial> shell am start -n app.mdreader/.DebugIngestActivity \
  --es mode document --es path /data/user/0/app.mdreader/files/mdreader-test-fixtures/t184-teste.md
sleep 5
adb -s <serial> shell "run-as app.mdreader cat cache/debug_ingest_result.txt"

adb -s <serial> shell am start -n app.mdreader/.DebugIngestActivity \
  --es mode directory --es path /data/user/0/app.mdreader/files/mdreader-test-fixtures/md-dir
sleep 8
adb -s <serial> shell "run-as app.mdreader cat cache/debug_ingest_result.txt"
```

Expected outcomes (programmatic non-UI scope):
- document: `ok:document:.../t184-teste.md`
- directory: `ok:directory:.../md-dir:chapters=<n>` where `<n>` matches production filtering of the fixture set (state fixture composition/expected filtering in evidence)
- no crash/ANR indicators in captured evidence (`unknown` is not absence/pass)
- report explicitly states residual native SAF picker cancel and TalkBack/keyboard traversal remain unverified
 +  - evidence includes FR-008 minimum fields and correlation key/run identity (source of truth: `spec.md` §FR-008 normative minimum; contracts/data-model/tasks are additive only)
## 5. FR-012 — Record split residual status

Update/report T184/T226 using [`contracts/t184-t226-residual-tracking.md`](./contracts/t184-t226-residual-tracking.md):
- mark `T184-PROG`/`T226-PROG` satisfied **only for exercised non-UI scope** with evidence links and `full_parent_completion_claimed=false`;
- update owning task files `specs/009-open-file-from-title/tasks.md` and `specs/010-directory-reading/tasks.md` (no orphan ad-hoc-only evidence);
- keep residual items open/verified/accepted-risk/deferred per lifecycle rules; accepted-risk does not complete parent while blocking;
- never mark T184/T226 fully complete from debug-hook evidence alone.
## 6. FR-010 — Release/non-debug exposure evidence (release candidate)

Build/install the release/non-debug artifact under test, then run the combined check:

```bash
# planned/contracted command (implement in tasks)
./scripts/check-debug-ingest-release-exposure.sh --mode full --variant universalRelease \
  --apk src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk \
  --serial <serial> --evidence-dir ./release-exposure-evidence-$(date -u +%Y%m%dT%H%M%SZ)
```

Expected evidence:
- static merged/packaged manifest and artifact scan show no `DebugIngestActivity` / `DEBUG_INGEST` / exported hook surface;
- runtime negative invocation on installed release cannot resolve/start hook and has verified install identity;
- report binds results to exact artifact hashes, tool/path, and device/install identity;
- class presence, if reported, includes `class_presence_note` (insufficient proof);
- missing required inputs or impossible full evidence => `blocked/not-evaluated` (or fail closed per mode), which MUST NOT satisfy FR-010/SC-004.
## Validation checklist

- [ ] FR-011 apply/check passes after any `gen` refresh.
- [ ] Debug hook produces explicit `ok:*`/`error:*` for document+directory fixtures on accessible paths.
 +  - [ ] Evidence includes crash/ANR status, residual UI-only scope, and FR-008 minimum fields (normative: `spec.md` §FR-008; contracts/data-model/tasks additive only).
- [ ] T184/T226 split tracking applied; no full completion claimed from hook-only evidence.
- [ ] FR-010 release exposure bundle recorded before claiming release absence.
