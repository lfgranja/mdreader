# Research: 012-debug-ingest-hook

**Date**: 2026-11-05  
**Spec**: [`spec.md`](./spec.md)  
**Status**: Complete for planning (no `NEEDS CLARIFICATION` markers remain in spec). External official pages were consulted in prior clarification research; some live fetches in this execution environment were unavailable, so citations include verification notes where appropriate.

## Decision 1 — Debug-only boundary and release exposure evidence (FR-010)

- **Decision**: Treat release safety as a *combined evidence contract*, not a single check. Required: debug/test-only placement + runtime refusal/inertness outside debug; static inspection of the actual release/non-debug artifact (merged/packaged manifest/APK/AAB metadata) showing no debug component, external invocation identifiers, or unintended external exposure; runtime negative invocation on an installed release/non-debug build; repeatable project check capturing results as regression evidence.
- **Rationale**: Android manifest merge/build variants determine the final packaged manifest; source guards (`BuildConfig.DEBUG`) are defense-in-depth but cannot prove a component was not merged/exported. Dex/class presence is weaker than component/export/intent-filter exposure for external invocation. Existing repo pattern has `scripts/check-manifest.sh`/`check-apk-classes.sh` and gate 9 outside `verify.sh`.
- **Alternatives considered**:
  - *Source/guard-only*: rejected — does not prove merged release artifact.
  - *Runtime negative only*: rejected — ambiguous failures, device/install provenance issues, no static proof.
  - *Manifest inspection only*: rejected — can inspect wrong artifact/variant; misses runtime resolution edge cases.
  - *Class absence in dex*: rejected as sufficient — shared `android-shell` main sourceSet compiles hook code in all variants; component exposure is the primary risk.

**Authoritative references** (official/standards; verify current wording/date when implementing): Android “Test and debug features” security risk — `https://developer.android.com/privacy-and-security/risks/test-debug` (developer.android.com); manifest merge/build variants — `https://developer.android.com/build/manage-manifests`, `https://developer.android.com/build/build-variants`; artifact inspection — `https://developer.android.com/tools/apkanalyzer`, `https://developer.android.com/tools/aapt2`; runtime package inspection — Android `dumpsys`/ADB tooling (`https://developer.android.com/tools/dumpsys`, `https://developer.android.com/tools/adb`); OWASP MASVS/MASVS-PLATFORM-1 and MASTG platform/build-settings tests for IPC/export/debuggability context — `https://mas.owasp.org/`. Tauri Android project structure is relevant for where manifests live — `https://v2.tauri.app/develop/`, `https://v2.tauri.app/reference/config/`, `https://v2.tauri.app/reference/cli/` (regeneration overwrite semantics not fully specified in fetched pages; treat as [INFERENCE] requiring repo-level apply/check).

**Repo evidence**: `DebugIngestActivity.kt`/`DebugIngestHook.kt` guard with `BuildConfig.DEBUG`; debug-only overlay at `src-tauri/gen/android/app/src/debug/AndroidManifest.xml` exports `.DebugIngestActivity` with `app.mdreader.action.DEBUG_INGEST`; main manifest has no hook; `build.gradle.kts` includes `android-shell/src/main/kotlin` in main sourceSet and enables `buildConfig`; merger reports exist under `src-tauri/gen/android/app/build/outputs/logs/`.

## Decision 2 — Durable scaffolding across generated Android project refresh (FR-011)

- **Decision**: Hybrid durable strategy. Canonical automation implementation and required test/scaffold sources live in maintained, version-controlled locations outside ignored generated output. `src-tauri/gen/**` is a materialization/build target only for managed overlay files. An idempotent apply/check process copies/verifies canonical overlay files into standard Gradle locations (`src/debug`, `src/androidTest`) and fails closed when missing/stale/forbidden after regeneration. Existing Kotlin implementation remains canonical in `android-shell/src/main/kotlin/` because generated `build.gradle.kts` already compiles it.
- **Rationale**: `.gitignore` ignores `/src-tauri/gen/` except `MainActivity.kt` and `index.html`, so debug manifest/androidTest placed only there is not durable. Android/Gradle officially supports debug source-set manifests/tests and manifest merge; Tauri generated project files are an initialization/build integration surface whose overwrite guarantees are not fully documented, so repo-owned idempotent materialization is safer than manual edits in `gen`.
- **Alternatives considered**:
  - *Keep debug manifest/test only in `gen` + checklist*: rejected — silent loss, fails FR-011.
  - *Commit broad `gen` exceptions*: rejected — fights generated-project lifecycle and increases merge/regeneration drift.
  - *Patch generated `build.gradle.kts` heavily*: rejected as default — high drift risk; current sourceSet already includes `android-shell`. Use narrowly scoped, explicit apply/check only if needed.
  - *Tauri plugin for debug hook*: rejected — overkill and risks packaging/API surface; hook is debug-only validation seam.

**Authoritative references**: Android source sets/build variants/manifest merge — `https://developer.android.com/build/build-variants`, `https://developer.android.com/build/manage-manifests`; Tauri v2 CLI/config/develop — `https://v2.tauri.app/reference/cli/`, `https://v2.tauri.app/reference/config/`, `https://v2.tauri.app/develop/`. **[INFERENCE]**: exact `tauri android init` overwrite behavior for every generated Android file must be validated against the project’s installed CLI; the apply/check contract is designed to fail closed on drift.

**Repo evidence**: `.gitignore:14-16` ignores `src-tauri/gen/` except two files; `build.gradle.kts` adds `$rootDir/../../../android-shell/src/main/kotlin`; debug manifest and androidTest scaffold currently exist locally under generated paths and are not durable.

## Decision 3 — T184/T226 completion/evidence policy (FR-012)

- **Decision**: Split tracking with annotated residuals. Programmatic debug-hook evidence may mark only the exercised non-UI ingest/routing scope as satisfied, with explicit evidence links and build/device identity. Native SAF picker cancel and TalkBack/keyboard traversal of native picker/title UI remain separate residual tracked/open sub-items (or explicitly open items) until manually/accessibility-verified or formally accepted as out-of-scope risk by explicit decision. Never mark T184/T226 fully complete from debug-hook evidence alone.
- **Rationale**: Debug hook proves production ingest/JNI routing and deterministic fixture handling, but it bypasses native SAF picker UI and cannot certify TalkBack/focus/system picker behavior. Android accessibility guidance requires manual/assistive-technology evaluation for covered user interactions; QA/traceability practice (ISTQB residual-risk/exit-criteria concepts; W3C partial-evaluation discipline) requires explicit residuals rather than overclaiming.
- **Alternatives considered**:
  - *Mark T184/T226 complete with brief note*: rejected — easy to lose residual risk.
  - *Leave fully open despite strong programmatic evidence*: rejected — discards verified non-UI scope and obscures progress.
  - *Automate native SAF picker/UIAutomator as full proof*: rejected for this feature — OEM/locale/system UI fragility and out-of-scope per spec.

**Authoritative references** (verify current pages): Android accessibility testing/manual boundaries — `https://developer.android.com/guide/topics/ui/accessibility/testing`; testing strategy/manual+automated — `https://developer.android.com/training/testing/fundamentals`, `https://developer.android.com/training/testing/other-components/ui-automator`; traceability/residual risk/exit criteria concepts — ISTQB CTFL Syllabus v4.0.1 / glossary (`https://istqb.org/`, `https://glossary.istqb.org/`, verify exact version/citation used); partial-evidence reporting discipline — W3C WCAG/WCAG-EM (`https://www.w3.org/WAI/test-evaluate/`). ISO/IEC/IEEE 29119 is informative alignment unless project adopts a specific part.

**Repo evidence**: `specs/009-open-file-from-title/tasks.md` T184 is open manual/ADB validation for Scenarios 1–4 including SAF cancel and accessibility/keyboard; `specs/010-directory-reading/tasks.md` T226 open for Scenarios 1–5 including SAF tree selection UX, transitions, resume UI, BT/lock-screen controls, resilience in real selection, and device constraints. `scripts/automate-t184-t226-hook.sh` already warns that SAF cancel and TalkBack/keyboard traversal are residual UI-only cases.

## Decision 4 — Automation result contract shape

- **Decision**: Keep current debug hook’s explicit result strings/log marker/cache result file as the minimum automation oracle, and define a richer machine-parseable evidence schema for planning/contracts (`ValidationResult`/`EvidenceRecord`/`ReleaseExposureCheck`/residual records). Wrapper scripts must fail closed on missing/invalid evidence when the check is explicitly requested.
- **Rationale**: Physical device runs already proved `ok:document:*` and `ok:directory:*:chapters=*` via `debug_ingest_result.txt`/logcat. A structured schema supports FR-010/012 traceability and repeatable checks without forcing a large runtime redesign in the initial plan.
- **Alternatives considered**:
  - *Logcat-only parsing*: rejected — buffering/rotation/variance; use only as corroborating evidence.
  - *Activity `am start -W` timing only*: rejected — launch timing is not pass/fail for ingest.
  - *Instrumentation-only*: optional (FR-013) but not required for ADB seam acceptance.

## Open Implementation Notes (resolved/bounded by clarifications)

- Canonical overlay path is normative: `specs/012-debug-ingest-hook/android-overlays/app/src/{debug,androidTest}/...` (FR-011/Q7). Tasks may add documented equivalents only if contracts/spec are updated together.
- `chapters=<n>` is defined as chapters accepted by production directory filtering/manifest behavior. Evidence/report MUST state fixture composition or expected filtering when claiming directory pass (FR-008/SC-001/002, debug-ingest contract). The prior `chapters=4` note is not normative pass criteria.
- Release exposure check tooling may use `apkanalyzer`/`aapt2`/`aapt`/merged-manifest reports depending on environment; script MUST record which tool/path produced evidence and fail closed when required inputs/tools for a requested mode are unavailable (FR-010).

## Deep Research Integration (2026-11-05)

This section integrates focused deep research for Spec 012. It supplements Decisions 1–4 and the resolved implementation notes above; where it adds constraints, those constraints are normative for planning/implementation unless a later spec clarification supersedes them. Authoritative URLs were checked on **2026-11-05**. `[INFERENCE]` marks conclusions drawn from incomplete/unstable upstream documentation or environment-dependent behavior that the implementation MUST validate fail closed.

### Area 1 — Tauri v2 Android `android init/dev/build`, `src-tauri/gen/android`, regeneration/overwrite, safe customization

**Findings (authoritative, accessed 2026-11-05)**:
- Tauri CLI reference documents mobile/Android commands and project lifecycle: <https://v2.tauri.app/reference/cli/>.
- Tauri configuration merge behavior (including platform-specific config) is the supported customization surface: <https://v2.tauri.app/develop/configuration-files/>.
- Google Play distribution guide reflects the Tauri Android build flow context: <https://v2.tauri.app/distribute/google-play/>.
- Mobile plugin development shows the intended model for Android integration points: <https://v2.tauri.app/develop/plugins/develop-mobile/>.
- Community/issue signals indicate regeneration/manual-edit fragility and identifier-change workflows that can require delete+init: tauri-apps/tauri#13930 <https://github.com/tauri-apps/tauri/issues/13930> (opened 2025-07-31, later activity 2025-08-14); tauri-apps discussions#12681 <https://github.com/orgs/tauri-apps/discussions/12681> (2025-02-11). `[INFERENCE]`: issue/discussion reports are evidence of risk, not a complete Tauri overwrite contract.

**`[INFERENCE]` — overwrite semantics**: no explicit, versioned Tauri contract was found that guarantees per-file overwrite/preserve behavior for every file under `src-tauri/gen/android` across `android init`, `android dev`, `android build`, config changes, identifier changes, or CLI upgrades. Implementation MUST assume `gen/android` files may be regenerated, replaced, or become stale, and MUST NOT rely on undocumented preservation behavior.

**Risks**:
- Silent/stale overwrite of debug/test overlay files or Gradle integration.
- Package/path coupling after application identifier or template changes.
- Debug-only component or test seam leaking into release/non-debug artifacts.
- Non-reproducible local state from manual edits in ignored generated output.
- Toolchain drift (Tauri CLI/Gradle/AGP/template) causing behavior changes without spec-visible signal.

**Spec 012 architectural/contract implications**:
- Treat `src-tauri/gen/**` as volatile materialization output (reinforces Decision 2). Canonical Spec 012 overlays remain under `specs/012-debug-ingest-hook/android-overlays/...`; any apply step copies/verifies into generated standard Gradle locations only.
- Prefer Tauri-supported configuration (`tauri.conf.json`/platform config) over generated-file edits. Any generated-file mutation MUST be narrowly scoped, idempotent, checksummed/compared by the apply/check contract, and documented in evidence.
- Add regeneration resilience to acceptance: after a clean regeneration scenario or `gen` refresh, `check` MUST fail closed if required debug/test artifacts are absent/stale, and `apply` MUST restore them deterministically.
- Minimize footprint: no Tauri plugin, no broad `gen` git exceptions, no heavy `build.gradle.kts` patching unless a separately justified contract requires it.
- Evidence/provenance MUST record installed Tauri CLI version, relevant config hash/values, generation/apply/check timestamps, and whether `gen/android` was treated as refreshed materialization.

### Area 2 — Manifest merger, debug source set, `exported`, and release absence proof

**Findings (authoritative, accessed 2026-11-05)**:
- Manifest merge rules, priorities, and merge-tool controls: <https://developer.android.com/build/manage-manifests>.
- Android 12+ requires explicit `android:exported` when components declare intent filters: <https://developer.android.com/about/versions/12/behavior-changes-12>.
- Exported-component security risk guidance: <https://developer.android.com/privacy-and-security/risks/android-exported>.
- Intents/intent filters determine external resolution surface: <https://developer.android.com/guide/components/intents-filters>.
- Artifact/manifest inspection tools: `apkanalyzer` <https://developer.android.com/tools/apkanalyzer>, `aapt2` <https://developer.android.com/tools/aapt2>, `bundletool` <https://developer.android.com/tools/bundletool>.
- Build variants/debug source sets: <https://developer.android.com/build/build-variants>.

**`[INFERENCE]` — inspection details**:
- Manifest merger report file paths and exact report shapes are AGP/version/project dependent; release absence proof MUST NOT hardcode a single path without discovery/fallback and recorded provenance.
- Final authoritative proof of packaged exposure is the actual release/non-debug artifact (APK/AAB-derived installable artifact metadata), not only source manifests or an expected merge output.

**Risks**:
- Debug component merged into release or present with unintended `exported=true`/intent filter.
- Incorrect `exported` state causing install failure (Android 12+) or exposure.
- False confidence from inspecting debug artifact, wrong variant, stale build output, or source-only checks.
- Gradle/manifest-merge overrides (`tools:node`, library manifests, variant config) changing final packaged manifest.

**Spec 012 architectural/contract implications**:
- Debug hook placement MUST remain debug-only (`src/debug/AndroidManifest.xml` overlay / debug source set). Main manifest and shared non-debug source MUST NOT declare the debug Activity/action.
- Default-deny exposure: debug component uses narrowest viable manifest shape; any `exported`/intent-filter choice MUST be justified by the debug automation contract and recorded. No broad intent filters.
- Release absence proof is a chain, not a single check (extends Decision 1): (1) merge evidence where available, (2) authoritative inspection of the actual release/non-debug artifact, (3) artifact identity binding (path, variant, package, version/build identifiers, hashes/timestamps), (4) runtime negative invocation corroboration on an installed release/non-debug build when that mode is requested.
- Stable naming/provenance: record package name, component class, action/category, artifact path, and inspection command/tool output used.
- Forbidden: `tools:node="merge/replace"` or similar manifest-merge escape hatches that can preserve/expose debug hook outside debug, except a narrowly documented case that still passes release absence proof.

### Area 3 — Debug seam security defense-in-depth

**Findings (authoritative, accessed 2026-11-05)**:
- Android test/debug feature risk: <https://developer.android.com/topic/security/risks/test-debug>.
- Manifest `application` attributes (including debuggability context): <https://developer.android.com/guide/topics/manifest/application-element>.
- OWASP MASTG resilience/debuggability references: <https://mas.owasp.org/MASTG/knowledge/android/MASVS-RESILIENCE/MASTG-KNOW-0007/>, <https://mas.owasp.org/MASTG/best-practices/MASTG-BEST-0007/>, <https://mas.owasp.org/MASTG/tests/android/MASVS-RESILIENCE/MASTG-TEST-0226/>.

**`[INFERENCE]`**: `BuildConfig.DEBUG` is necessary defense-in-depth but not sufficient alone for Spec 012’s release-safety claim, because build/manifest merge, wrong artifact selection, or packaging mistakes can still create exposure independent of a runtime guard.

**Risks**:
- Debug Activity/action reachable in release/non-debug builds.
- Over-trusting a single guard (runtime, manifest, or source-set placement alone).
- Shared main source-set code accidentally exposing debug entry points or capabilities.

**Spec 012 architectural/contract implications**:
- Defense-in-depth layers (all required where applicable): build/manifest isolation (debug source set), compile-time/build-variant awareness, runtime refusal/inertness outside debug, and capability minimization (only deterministic fixture ingest/routing needed for T184/T226 evidence).
- Manifest non-exposure is primary for external invocation risk: release artifacts MUST contain no debug component, action, or unintended exported surface.
- Release absence evidence MUST be produced by the repeatable check and captured as regression evidence (Decision 1).
- Shared `android-shell/src/main/kotlin` rule: hook implementation may live in compiled main source for build wiring convenience, but it MUST remain unreachable/inert outside debug and MUST NOT add production UI, permissions, or exported manifest entries. Any externally invocable declaration stays debug-only.
- CI/local blocking check: requested release-safety verification fails closed on missing inputs/tools, wrong artifact identity, or positive exposure findings.

### Area 4 — ADB, fixtures, ABI/package identity

**Findings (authoritative, accessed 2026-11-05)**:
- ADB command behavior, including `am start` options (`-W`, `-S`, `-n`, `-a`, extras) and `pm path`: <https://developer.android.com/tools/adb>.
- Logcat is useful corroboration but buffered/rotated: <https://developer.android.com/tools/logcat>.
- `dumpsys` provides runtime package/component/device state: <https://developer.android.com/tools/dumpsys>.

**`[INFERENCE]`**:
- `run-as` and direct access to app private/scoped storage are device/app/build/image dependent and may be unavailable; automation MUST NOT require `run-as`.
- Logcat timing/retention is non-deterministic across devices and load; log markers are corroboration, not the sole pass/fail oracle unless paired with a deterministic app-controlled result.

**Risks**:
- Non-deterministic logcat leading to false negatives/positives.
- Fixture files inaccessible, wrong path, or wrong permissions.
- Exposing file URIs/capabilities beyond what the debug seam needs.
- Wrong package name, ABI, build variant, or device target causing misleading evidence.
- Treating Activity launch success (`am start -W`) as ingest/routing pass.

**Spec 012 architectural/contract implications**:
- Deterministic invocation contract: ADB/debug hook calls use explicit component/action, explicit mode, explicit fixture path/input, and a correlation ID propagated to result/log/evidence.
- Explicit semantic result is authoritative: app-controlled success/failure output (existing result file/log marker contract shape from Decision 4) determines pass/fail; `am start -W` is launch metadata only.
- Fixture strategy: prefer app-controlled deterministic fixtures. Acceptable patterns include pushing known fixtures to `/data/local/tmp` and passing an explicit path/mode through the debug-only flow, or using debug assets/resources when they remain debug-only. `run-as` is best-effort only and never required.
- ABI/package identity checks fail closed before claiming evidence: record and verify package, component, variant/build identity, device model/serial (redactable), ABI/`ro.product.cpu.abilist` or equivalent, and installed APK path(s) as needed.
- Logcat corroboration: capture filtered logs with correlation ID/build identity when available; absence of logs does not override an explicit negative result, and logs alone do not prove success without the semantic result contract.

### Area 5 — Idempotent apply/check

**Findings (authoritative, accessed 2026-11-05)**:
- Gradle incremental-build principles (inputs/outputs/up-to-date checks) inform idempotent automation design: <https://docs.gradle.org/current/userguide/incremental_build.html>.

**`[INFERENCE]`**: overlay apply/check design details below are best-practice contracts for Spec 012 because Tauri/AGP do not provide a project-specific durable-debug-overlay mechanism; they MUST be implemented as explicit repo automation, not assumed platform behavior.

**Risks**:
- Partial/corrupt materialization after interruption.
- Hidden drift between canonical overlays and generated files.
- Mutating checks that mask failures or create side effects.
- Editing shared generated files causing merge/regeneration conflicts.

**Spec 012 architectural/contract implications**:
- Strict separation: `check` is non-mutating and reports expected vs actual; `apply` mutates only managed Spec 012 overlay targets and is idempotent (reapplying identical canonical inputs produces no semantic drift).
- Canonical managed manifest: maintain a machine-readable/human-auditable manifest of managed files (source path, destination path, required variant/source-set, content hash/version, ownership). `check`/`apply` use it as the source of truth.
- Fail-closed behavior: missing canonical source, missing destination after apply, stale hash/content mismatch, unexpected managed target state (when contract requires exactness), missing required tools for a requested mode, or ambiguous `gen` state blocks the requested check.
- Expected-vs-actual reporting: emit JSON and human-readable output with file-level status (`ok`, `missing`, `stale`, `unexpected`, `error`), hashes, paths, and remediation.
- Managed regions/editing rule: prefer dedicated debug-only files (`src/debug/AndroidManifest.xml`, debug test sources, scripts/contracts) over patching shared generated files. If a shared file ever becomes unavoidable, use explicit managed markers/patch contract and fail closed on marker drift — `[INFERENCE]` no such shared-file patch is preferred for Spec 012.
- Clean/apply ordering: document when `apply` runs relative to Tauri/Gradle generation/build; release pipeline and release-safety checks verify absence from the correct non-debug artifact after the relevant build path.

### Area 6 — Evidence records, traceability, and correlation

**Findings (authoritative, accessed 2026-11-05)**:
- Provenance/inspection sources for evidence fields: `apkanalyzer` <https://developer.android.com/tools/apkanalyzer>, `aapt2` <https://developer.android.com/tools/aapt2>, `bundletool` <https://developer.android.com/tools/bundletool>, ADB/logcat/dumpsys references in Areas 2/4.

**`[INFERENCE]`**: the concrete JSON schema is a Spec 012 contract proposal; fields may evolve, but every completed/partial claim MUST remain machine-traceable, reproducible where possible, and explicit about residual scope.

**Risks**:
- Evidence cannot be tied to the exact artifact/device/run.
- Partial automation overclaims full T184/T226 completion.
- Sensitive data leakage in logs/screenshots/paths.
- Fallback evidence used without labeling its strength.

**Spec 012 architectural/contract implications**:
- Structured JSON evidence record (minimum fields): `spec_id`, `operation`, `timestamp_utc`, `tool_versions`, `correlation_id`, `git_ref`/workspace state as appropriate, artifact provenance (variant, path, package, hashes, build identifiers), device provenance (model/serial redaction policy, ABI, OS), inputs (fixture path/mode/intent args), outputs (semantic result, exit codes, key stdout/stderr summaries), file/log hashes or references, and pass/fail/partial status.
- Correlation propagation: the same `correlation_id` MUST appear in the ADB invocation metadata, app-controlled result, captured logs, and evidence JSON filename/content where feasible.
- Corroboration/fallback ladder: primary = explicit app-controlled semantic result/file; secondary = filtered logcat with correlation ID; tertiary = manifest/package/component inspection. For release absence, authoritative artifact inspection of the actual non-debug artifact is primary proof; runtime negative invocation is corroborating proof; merge reports are supporting evidence.
- Retention/index: store evidence under a documented spec-local or script-defined path with stable naming; maintain an index/summary that maps evidence to T184/T226 scope (exercised vs residual).
- Traceability matrix: every evidence record states which scenario/check IDs it satisfies and which residual UI/accessibility items remain open (reinforces Decision 3).
- Redaction/minimization: never record secrets/tokens; redact device serials or personal paths when sharing externally; capture only the minimum fixture/content needed for deterministic proof.


## Decision 5 — Host-deterministic contract modes and T018/T024/T067 wiring (2026-11-06, reconciliation)

- **Decision**: Add host-deterministic hook modes (`--print-activity-channel-contract`, `--classify-channel-attribution`, `--print-intent-parse-contract`, `--normalize-debug-ingest-intent`, `--print-directory-uri-contract`, `--build-device-evidence`, `--print-device-run-contract`, `--classify-device-run`) for RED/PBT/CI and normal device-run FR-008/SC classification support. Intent/dir-uri parse/normalize modes are debug-validation helpers only (I1 option B fallback), not parallel production ingest routes; ASM-012-INGEST-SOT/Kotlin/Rust/JNI remain authority. Device-run modes emit `full_parent_completion_claimed=false`, `crash_anr_status unknown|detected` non-pass semantics, and explicit blocked/inconclusive/busy/cold-start handling.
- **Rationale**: Enables host TDD without adb while preserving fail-closed FR-008/FR-010/FR-012 semantics; supports T028/T018/T024/T067 evidence wiring gaps without CHK021 gen mutation.
- **Integrity**: CHK021 (no `src-tauri/gen/**`/generated Gradle/Tauri patch without recorded decision), T042 (FR-010 full only D1–D4 + bound identities; non-full/blocked never satisfy SC-004), T028 (per-run attributable FR-008/SC evidence required for executed claims).
