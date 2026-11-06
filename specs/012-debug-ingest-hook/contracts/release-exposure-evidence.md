# Contract: Release Exposure Evidence (FR-010)

**Feature**: `012-debug-ingest-hook`  
**Covers**: FR-010, SC-004  
**Status**: Normative for planning/tasks.

## Purpose

Prove that release/non-debug artifacts do **not** expose the debug ingest automation entry as an externally invocable or user-facing production feature.

## Required Evidence Dimensions

A claim of FR-010 pass requires **all** dimensions below for `mode=full`:

| Dimension | Required checks | Authoritative signal |
|---|---|---|
| **D1 Debug-only scope** | Debug component/manifest materialization is debug-only; `DebugIngestActivity`/`DebugIngestHook` refuse or are inert outside debug. Source/guard checks are necessary but not sufficient. | Source/config placement + guard evidence. |
| **D2 Static release artifact absence** | Inspect actual release/non-debug packaged/merged manifest, merger report when available, and APK/AAB metadata. Record tool/path used. | No `DebugIngestActivity` component, no `app.mdreader.action.DEBUG_INGEST`, no exported hook intent filter/external invocation identifier, no debug manifest injection in release merge report. |
| **D3 Runtime negative invocation** | On installed release/non-debug build with verified install identity, attempt explicit component/action invocation used by debug automation. | Invocation cannot resolve/start hook; no `ok:document`/`ok:directory`/`debug ingest result: ok:*` success; no crash/ANR caused by the negative check. |
| **D4 Repeatable regression check** | Scriptable check emits deterministic pass/fail and machine-reviewable evidence bound to artifact/install identity. | Fail-closed on missing required inputs when check requested; never reuses stale evidence without identity match. |

**In-scope artifacts/identifiers (normative):** `DebugIngestActivity`, `DebugIngestHook`, `app.mdreader.action.DEBUG_INGEST`, debug manifest/intent-filter identifiers, and materialized debug/androidTest overlay targets.

## Artifact Identity Binding

Every evidence bundle records:
- variant/build type (`release`/`non-debug`), command used to build if known;
- APK/AAB path and SHA-256;
- merged manifest / packaged manifest / merger report path and hashes when inspected;
- tool name/path and command surface used for static/runtime checks;
- package id `app.mdreader`, version/signing/build metadata as available;
- device serial/model/SDK/ABI and verified install identity for runtime checks (must not be a debug build when claiming release-negative).

## Command Surfaces (design target)

```bash
# Static-only (host/artifact), no device
scripts/check-debug-ingest-release-exposure.sh --mode static --variant universalRelease \
  --apk <release.apk> [--merger-report <report>] [--merged-manifest <manifest>] \
  --evidence-dir <dir>

# Runtime-only negative check on installed release build
scripts/check-debug-ingest-release-exposure.sh --mode runtime --serial <adb-serial> \
  --package app.mdreader --component app.mdreader/.DebugIngestActivity \
  --action app.mdreader.action.DEBUG_INGEST --evidence-dir <dir>

# Full FR-010 claim
scripts/check-debug-ingest-release-exposure.sh --mode full --variant universalRelease \
  --apk <release.apk> --serial <adb-serial> --evidence-dir <dir>
```

## Pass/Fail Rules

- Missing required input for requested mode => **FAIL** (fail closed). Required inputs include: artifact/manifest/merger inputs required for static/full, device/install identity for runtime/full, and required tools available for requested mode.
- `full` => overall `pass` only if D1–D4 pass.
- `static` => may report static regression status; MUST NOT claim full FR-010 pass without D3.
- `runtime` => may report negative invocation evidence; MUST NOT claim full FR-010 pass without D2 artifact identity.
- Debug component/action/exported hook identifier found in release artifact/merge => **FAIL** regardless of runtime guard.
- Runtime invocation reaches hook success path on release/non-debug install => **FAIL**.
- If full evidence is impossible, record `blocked/not-evaluated` with owner/decision ref, missing inputs, and attempted modes. `blocked/not-evaluated` MUST NOT satisfy FR-010/SC-004.

**Required negative scenarios (normative):** debug component merged/exported in release artifact; `DEBUG_INGEST` action exposed in release manifest; runtime invocation succeeds (`ok:*`/debug ingest success) on release/non-debug build. Each MUST FAIL the check.

## Class Presence Caveat

Dex/class scanning may be recorded as supplemental evidence. Because debug hook Kotlin sources are compiled via the shared `android-shell` main source set, class presence/absence is **not sufficient** to prove or disprove external exposure. When class/dex scan runs, evidence MUST include `class_presence_note` stating this limitation. Authoritative external-exposure signals are manifest/component/export/action resolution plus runtime invocation behavior.

## Integration

- Gate 9 is the authoritative Android build gate and runs outside `scripts/verify.sh` (constitution/repo workflow). FR-010 checks are release/device-aware; integrate as explicit opt-in/feature validation or release-candidate checks, not as silent dependencies of host-only gates 1–8.
- Any change to debug hook component name, intent action, manifest placement, or canonical source paths must update this check and related contracts. Release-exposure checks MUST NOT rely on non-durable `src-tauri/gen/**` files as canonical inputs except as explicit materialization targets verified by FR-011 apply/check.


## Mechanization fields (T042/FR-010, fail-closed)

`check-debug-ingest-release-exposure.sh` modes: `static|runtime|full`. `--adb-runner` / `DEBUG_INGEST_ADB_RUNNER` may be injected for host-testable D3 contract execution; direct adb is not auto-run by fail-closed contract checks. Output MUST include tool provenance, bound `artifact_identity`/`install_identity` as applicable, per-dimension D1–D4 status/details, and `satisfies_fr010_sc004`/overall classification. `overall=pass` for `mode=full` only with D1–D4 pass and bound identities (`install_identity.verified_release_non_debug=true` for runtime/full claims). `class_presence_note` required when class scan runs. Missing required inputs/tools/identity => `blocked-not-evaluated`/non-full as applicable with `satisfies_fr010_sc004=false`; never full FR-010/SC-004 pass. (CHK021: no gen patch; no weakening of FR-010.)
