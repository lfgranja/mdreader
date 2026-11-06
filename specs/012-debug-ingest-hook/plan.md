# Implementation Plan: Debug-Only Ingest Validation Hook

**Branch**: `012-debug-ingest-hook` (repo currently on `dev`; spec folder is the feature workspace) | **Date**: 2026-11-05 | **Spec**: [`specs/012-debug-ingest-hook/spec.md`](./spec.md)
**Input**: Feature specification from `specs/012-debug-ingest-hook/spec.md` (clarified Session 2026-11-05, FR-008/010/011/012/013 and SC-001–006 resolved)

## Summary

Harden the existing debug-only Android ingest validation seam so maintainers can deterministically validate Spec 009 T184 and Spec 010 T226 document/directory ingest behavior without manual SAF picker interaction, while preserving release security and evidence integrity. The seam accepts a fixture path/URI in debug builds and routes through the **same production** `DocIngest` snapshot/fingerprint + JNI callback path (`onDocumentIngested` / `onDirectorySelected` → Rust `open_document_bytes_named` / directory manifest handling), not a fake test path.

Architectural deltas required by the clarified spec: (1) **FR-011** make debug hook/scaffold durable across `src-tauri/gen/` regeneration via canonical maintained sources outside ignored generated output plus an idempotent, fail-closed apply/check process (overlay `dependency_layout_results`/DEP-ASM fail-closed); (2) **FR-010** prove release/non-debug absence/inertness with combined static release-artifact inspection, runtime negative invocation, and repeatable regression evidence (T042 mechanization: `check-debug-ingest-release-exposure.sh` D1–D4, bound identities, `static`/`runtime`/`blocked-not-evaluated` never satisfy FR-010/SC-004); (3) **FR-012** split T184/T226 reporting so programmatic non-UI coverage can be satisfied only for exercised scope while native SAF picker cancel and TalkBack/keyboard traversal remain separate residual tracked items. Host-deterministic hook contract modes (`--build-device-evidence`, `--print/classify-device-run-contract`, intent/dir-uri parse/normalize, channel attribution) support TDD/T018/T024/T067 without adb and without weakening FR-008/FR-010/FR-012 or creating parallel production ingest paths (CHK021/T028 integrity).

## Technical Context

**Language/Version**: Rust 2024 edition stable (MSRV ≥ 1.85) for Tauri/JNI backend; Kotlin (Android/Gradle, JVM target 1.8 in generated app) for Android shell/automation seam; POSIX shell/Python for repeatable checks and ADB evidence capture. Record observed tool versions and repo-declared pins/config when present; do not invent pins.
**Primary Dependencies**: Existing Android shell `DocIngest`/JNI bridge (`android-shell/src/main/kotlin/app/mdreader/*`, `src-tauri/src/lib.rs`/JNI handlers), Tauri 2.12.1 Android project under `src-tauri/gen/android/`, Android SDK/ADB, Gradle app build, `cargo-nextest` for host gates. No new runtime product dependencies required; avoid adding network or release-surface dependencies. Durability/build dependencies are normatively tracked in Spec 012 `DEP-012-*`/`ASM-012-*` with required/optional status and fail-closed fallback.
**Storage**: App-accessible files/fixtures on device (`/data/user/0/app.mdreader/files/...` preferred for debug `run-as` automation), `cache/debug_ingest_result.txt` (debug evidence channel), host-side evidence/report directories (timestamped). No database/schema changes. Use reviewer-accessible locations for retained evidence; ignored dirs are not required retention targets.
**Testing**: `cargo nextest --workspace` (gate 2) for host Rust; existing check scripts (`scripts/check-manifest.sh`, `check-apk-classes.sh`, `verify.sh` gates 1–8; gate 9 outside). Feature adds repeatable debug hook automation, durable overlay apply/check, release-exposure evidence check, and optional instrumented smoke coverage (FR-013).  
**Target Platform**: Android debug/test builds on device/emulator, minSdk 29; physical device evidence observed on `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp` (2201117TY, SDK 33, arm64-v8a). Release/non-debug artifacts inspected for absence evidence but hook never exposed.  
**Project Type**: Tauri mobile app (Rust workspace + Android generated Gradle project + Kotlin shell mirror) with debug-only validation tooling/contracts.  
**Performance Goals**: Automation run produces explicit success/failure evidence within 2 minutes per run in supported environments (SC-002); no playback/timing claims beyond exercised ingest path.  
**Constraints**: Pure offline; Rust sole playback/ingest authority; Kotlin shell transport/mirror only; debug-only hook never exposed in release; `src-tauri/gen/**` mostly gitignored/regenerable so canonical durable sources must live outside it; fail closed on missing/stale overlay or missing release-evidence inputs when a check is requested.  
**Scale/Scope**: Narrow validation seam for T184/T226. No user-facing product UX changes, no SAF picker automation, no accessibility certification, no gate 9 release claim from debug evidence.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate / Principle | Status | Notes |
|---|---|---|
| Rust owns core behavior; Kotlin/Android shell mirrors/transport only | PASS | Hook lives in Android shell but only snapshots/forwards fixtures and invokes existing `DocIngest`/JNI callbacks used by production SAF flows; no playback/position/rate logic added. |
| Pure offline / no INTERNET | PASS | Automation uses ADB/local fixtures/app cache; no new permissions/network paths. |
| Driving-first / UX contracts unchanged | PASS | No required user interaction added; debug-only seam not user-facing. |
| Closed sets / no silent fallbacks in product path | PASS (design) | Hook returns explicit `ok:*`/`error:*`/`refused:*` results; release exposure check must fail closed; unknown inputs rejected. |
| Host-checkable gates 1–8; gate 9 outside | PASS (design) | Overlay/apply/check and static release-manifest/report checks are host-scriptable; device runtime negative invocation and debug device evidence are device-gated; gate 9 remains authoritative Android artifact build outside `verify.sh`. |
| Errors explicit (no `unwrap`/`expect` in Rust) | PASS | Kotlin hook catches exceptions and returns diagnosable results; any Rust changes must follow workspace denies. |
| Exceptions log / supply chain | N/A for new deps | Plan adds scripts/docs/contracts, not new crates. Any future dependency changes must follow gate 5/6 and exceptions log. |
| Debug/test features not in production (security) | PASS (required) | FR-010 requires debug-only scope/guards, static release artifact absence, runtime negative invocation, repeatable regression evidence. Merged-manifest/component exposure is authoritative over class presence. |

**Post-Phase 1 re-check**: No constitution violations introduced by contracts/data model. Contract completeness confirmed: `contracts/debug-ingest-automation.md`, `contracts/release-exposure-evidence.md`, `contracts/t184-t226-residual-tracking.md`, and `contracts/durable-overlay-materialization.md` cover FR-008/009/010/011/012/013, SC-001–006 evidence shapes, CHK018/021/022/024/026/027/030-dur hooks, and fail-closed semantics. FR-011 canonical overlay path is fixed (`specs/012-debug-ingest-hook/android-overlays/app/src/`), overlay format/version is detectable/reportable by apply/check (CHK022; `v1-implicit` only when explicitly documented, else fail/blocked), apply/check is idempotent/fail-closed/non-mutating on check, and androidTest materialization is required only when canonical androidTest source exists (FR-013). FR-010 full claims require `mode=full` D1–D4 with manifest merger/merged+packaged manifest authoritative for D2 exposure; `blocked/not-evaluated`/partial never satisfies FR-010/SC-004. FR-012 requires split residual lifecycle, owning-task updates, and no hook-only full T184/T226 completion. Governance (CHK001/002/018/021), traceability/preconditions (CHK030/032), and maintainability NFRs (CHK025/030-dur: small blast radius, no duplicate authority, readable diagnostics, repo-toolchain portability, synchronized cross-artifact updates) are normatively specified.

## Governance (durability/maintainability)

### Review ownership & handoff (CHK001)
Primary reviewers: (1) Spec 012 author/maintainer and (2) Android/shell maintainer (changes touching `android-shell`, debug manifest/source-set wiring, hook Kotlin, release exposure). Include (3) Tauri/mobile maintainer when `src-tauri` materialization/build config or Android gen workflow is affected; include QA/evidence reviewer for FR-010/FR-012 evidence claims. Handoff MUST provide: affected `FR/SC/US/EC/ASM/DEP/TASK/EV/RES` IDs, `check-debug-ingest-overlay` and/or apply idempotence result for relevant scenarios, FR-010 mode/result or justified `blocked/not-evaluated` (`satisfies_fr010_sc004=false`), FR-012 split updates (`*-PROG`/residuals, `full_parent_completion_claimed=false` when hook-only), generated-tree impact (`src-tauri/gen/**` not canonical), and residual risks/owners.

### Finding classification (CHK002)
**Blocking (merge/readiness):** FR-010 exposure or missing/invalid negative evidence where required; FR-011 missing/stale/forbidden targets, silent failures, or non-idempotent/fail-closed behavior in defined scenarios; FR-012 full-completion overclaim, missing split/residual tracking, or orphan evidence; missing trace links for in-scope FR/SC/RES; mandatory preconditions failed without explicit `blocked`/diagnosable record; covered checks/scripts failing. **Advisory:** non-contract refactors, non-normative doc polish, non-required optimizations — logged with owner + follow-up ID/date/scope; never weaken requirements. Waivers/`accepted-risk` require dated decision, owner, rationale, scope, and evidence where applicable; they never satisfy FR-010/SC-004.

### FR-011 regeneration/apply-check run points (CHK018)
`check` is non-mutating and fail-closed; `apply` is idempotent. Run `check` (and `apply` when check fails or canonical/targets changed) at: local bootstrap/setup; after `tauri android init` or known Android project regeneration; after canonical overlay/hook/debug-manifest/canonical androidTest source changes; after Tauri CLI, Android templates/Gradle wrapper, AGP/Kotlin/NDK/SDK, or toolchain changes affecting gen; in CI bootstrap/Android debug jobs covering Spec 012; before T184/T226 validation campaigns and before claiming SC-006 evidence. Prefer incremental builds per Gradle guidance; require clean/regenerate+check when toolchain/generation inputs changed or stale/mismatched generated artifacts are detected.

### Durability dependencies and layout validation (CHK026/CHK027)
`DEP-012-TAURI-GEN`, `DEP-012-ANDROID-BUILD`, `DEP-012-REPO-SCRIPTS`, `DEP-012-GIT-CHECKOUT`, and `DEP-012-GEN-COMPAT` define required/optional status and fallback; missing required deps => `fail`/`blocked:not-evaluated` for the affected claim with explicit diagnostics. `ASM-012-REPO-LAYOUT` and `ASM-012-SOURCESET-WIRING` are validated/bounded assumptions: checks must verify required canonical/target paths and `android-shell/src/main/kotlin` wiring when dependent claims are made; drift/unverifiable wiring fails closed with expected-vs-actual evidence.

### Narrow generated-file patching (CHK021)
Default **MUST NOT**: do not manually patch `src-tauri/gen/**` or generated Gradle/Tauri files. Implement via canonical sources/overlays and FR-011 materialization. Exception only with recorded decision (`EV/RSK`/decision ref): exact file paths/patterns, minimal diff, justification that supported source/template/config cannot solve it, debug/release impact, named owner + expiry/removal task, idempotent context-checked automated script (safe re-run/no-op), reapplied after every CHK018 trigger and verified by `check`, fail-closed on missing target/context mismatch/partial application, and removal plan when upstream/template fixes land.

### Overlay format evolution & artifact ownership (CHK022/CHK024)
Canonical overlay format declares/detects a version (implementation-defined marker; requirement is normative). `apply/check` report detected version and fail closed on unknown/missing version unless the check contract explicitly documents `v1-implicit`. Prefer backward-compatible materialization; incompatible evolution requires migration note, synchronized updates to canonical targets + `apply/check` + contracts/docs + future tasks, and evidence. Ownership: Spec 012 maintainer owns canonical overlay/contracts; Android/shell maintainer co-owns `android-shell` hook Kotlin, debug manifest/source-set wiring, FR-010/FR-012 evidence surfaces; Tauri/mobile maintainer co-owns gen/materialization/build-config impacts. Hook-surface changes (`component`, action, canonical/target paths, result grammar/channels, CHK018 run points, residual IDs) require synchronized updates and reviewer handoff per CHK001/CHK002.

### Maintainability NFRs & cross-artifact consistency (CHK025/CHK030-dur)
Normative NFRs (traceable to Constitution/FR-011): **small blast radius** (canonical overlay, `android-shell` debug hook, scripts/checks/contracts/docs/tasks only unless explicitly justified/traced), **no duplicate authority** (`src-tauri/gen/**` never canonical; no parallel production ingest path), **readable diagnostics** (`pass|fail|blocked|not-applicable` + path/expected-vs-actual/remediation where safe), **repo-toolchain portability** (`DEP/ASM-012-*`, no hidden deps). **Consistency rule (blocking):** changes affecting FR-010/011/012/013, SC-001–006, IDs, contracts, paths, result grammar/channels, CHK018 run points, or residual IDs MUST synchronize `spec.md`, `plan.md`, `research.md` (if decision changes), `quickstart.md`, contracts, relevant checklists, and future `tasks.md`; CHK001/002 review treats divergence as blocking.

## Architecture Design

### Component Map (FR-011 / FR-010 / FR-012)

| Stage | Component | Canonical / authority | Role | Contract / IDs |
|---|---|---|---|---|
| Canonical sources | `specs/012-debug-ingest-hook/android-overlays/app/src/{debug,androidTest}/...` | FR-011 canonical overlay root (version-controlled; `src-tauri/gen/**` never canonical) | Durable debug manifest overlay and optional androidTest smoke scaffold source; format/version detectable by apply/check (CHK022). | `contracts/durable-overlay-materialization.md`, FR-011/013, SC-006 |
| Canonical sources | `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt`, `DebugIngestHook.kt`, minimal debug helpers in `DocIngest.kt` | Maintained Kotlin source compiled via generated `build.gradle.kts` source-set wiring (ASM-012-SOURCESET-WIRING) | Debug-only entry/hook/transport; guards + deterministic `ok:*`/`error:*`/`refused:*`; no playback/position/rate logic. | `contracts/debug-ingest-automation.md`, FR-001–009/013 |
| Materialization targets | `src-tauri/gen/android/app/src/debug/AndroidManifest.xml`, optional `src/androidTest/...` | Generated/regenerable target only | Receives idempotent FR-011 materialization; checked for presence/content/equivalence/forbidden state. androidTest target required only when canonical androidTest source exists. | `contracts/durable-overlay-materialization.md`, SC-006 |
| Apply/check control | `scripts/apply-debug-ingest-overlay.sh`, `scripts/check-debug-ingest-overlay.sh` (planned) | Repo script authority for FR-011 durability claims | `check` non-mutating/fail-closed; `apply` idempotent; CHK018 run points; validates `DEP/ASM-012-*` where claims depend. | `contracts/durable-overlay-materialization.md`, CHK018/021/022/024/026/027/030-dur |
| Debug hook invocation | ADB intent → `app.mdreader/.DebugIngestActivity` (`app.mdreader.action.DEBUG_INGEST`) | Debug build runtime surface only | Validates mode/path/URI, fixture accessibility, native readiness; serializes or rejects busy work; emits attributable result. | `contracts/debug-ingest-automation.md`, FR-001–009, SC-001/002 |
| Production ingest/JNI | `DocIngest` snapshot/fingerprint/chapter manifest + `onDocumentIngested` / `onDirectorySelected` → Rust JNI/open-document/directory-manifest handling | **Rust is sole ingest/playback authority**; JNI callbacks are the production path | Hook routes through the same production ingest path; no parallel fake ingest/open path. `chapters=<n>` means chapters accepted by production filtering/manifest behavior. | `contracts/debug-ingest-automation.md`, FR-004/005/008 |
| Evidence / FR-010 checks | `cache/debug_ingest_result.txt` (authoritative when readable/parseable), logcat/script corroboration, `scripts/check-debug-ingest-release-exposure.sh` (planned), merged/packaged manifest+merger report, runtime negative on verified release/non-debug install | FR-008 minimum + FR-010 D1–D4 combined evidence | Debug run evidence binds correlation/provenance; FR-010 full requires static artifact absence + runtime negative + repeatable check + identity binding. `blocked/not-evaluated`/partial never satisfy FR-010/SC-004. | `contracts/release-exposure-evidence.md`, `contracts/debug-ingest-automation.md`, FR-008/010 |
| FR-012 residual tracking | `T184-PROG`/`T226-PROG` + residual IDs in owning `specs/009.../tasks.md`, `specs/010.../tasks.md` and evidence refs | Owning task artifacts + Spec 012 residual contract | Programmatic hook evidence may satisfy only exercised non-UI scope; native SAF cancel, TalkBack/keyboard traversal, transition/resume/hardware UI remain separate residual lifecycle items. No hook-only full completion. | `contracts/t184-t226-residual-tracking.md`, FR-009/012, SC-003 |

### Data / Control Flow

**Debug validation run (T184/T226 exercised non-UI scope):**
1. Preconditions (fail-closed): required `DEP/ASM-012-*` satisfied for invoked path; fixture app-accessible to debug/test process; debug ABI/package installable; CHK018 `check`/`apply` current when regeneration/canonical changes occurred. Failure => `blocked:*` or diagnosable `error:*`, never silent pass.
2. Correlation/provenance assigned: run key/correlation ID, fixture ref, operation, build+device+artifact identity, preferred/effective evidence channels, timestamps/hashes when available (`unavailable:<reason>` otherwise).
3. ADB/debug intent invokes `DebugIngestActivity` (debug only). Hook validates/normalizes request (deterministic mode aliases/precedence), checks `BuildConfig.DEBUG`/readiness, and rejects/serializes conflicts (`error:busy:*` where applicable).
4. Hook reads fixture and routes through production `DocIngest` snapshot/fingerprint/chapter-manifest logic, then the same JNI callbacks used by SAF ingest (`onDocumentIngested` / `onDirectorySelected` → Rust authority). No parallel fake path.
5. Hook emits parseable `ok:*`/`error:*`/`refused:*`; wrapper captures authoritative/corroborated evidence per FR-008 (cache authoritative when readable/parseable; corroboration explicit; conflicts/missing authoritative+corroborated => `inconclusive`). Crash/ANR checked; `unknown` is not pass.
6. Result is classified as `programmatic-non-ui` only. Evidence links update `T184-PROG`/`T226-PROG` as exercised scope satisfied when pass; residual IDs remain `open|verified|accepted-risk|deferred` by their own evidence/decisions.

**Release-exposure check (FR-010):**
1. Precondition gate: `mode=full` requires releasable/non-debug artifact identity + required static inputs (merged/packaged manifest, merger report when available) + verified installed release/non-debug target + required tools. Missing required inputs => `blocked/not-evaluated` with owner/decision ref, missing inputs, attempted modes, `satisfies_fr010_sc004=false`.
2. D1 source/guard scope: debug component/overlay placement and runtime refusal/guards evidenced (necessary, not sufficient).
3. D2 static artifact inspection: inspect **actual release/non-debug merged/packaged manifest and merger report** (authoritative exposure signal) plus APK/AAB metadata for `DebugIngestActivity`, `DebugIngestHook`, `app.mdreader.action.DEBUG_INGEST`, exported hook intent filters/external invocation identifiers, and materialized debug/androidTest overlay leakage. Tool/path and artifact hashes recorded.
4. D3 runtime negative invocation: on verified release/non-debug install identity, attempt explicit component/action invocation; expect no resolve/start/hook success and no crash/ANR caused by the check.
5. D4 repeatable regression: script emits deterministic, machine-reviewable pass/fail/blocked, bound to artifact/install identity; never reuses stale evidence without identity match.
6. Full FR-010/SC-004 pass only when D1–D4 pass for `mode=full`; all other outcomes report scoped/non-satisfying status explicitly.

### Artifact Inventory

| Artifact / path | Class | Authority | Retention / evidence notes |
|---|---|---|---|
| `specs/012-debug-ingest-hook/contracts/*.md` (`debug-ingest-automation`, `durable-overlay-materialization`, `release-exposure-evidence`, `t184-t226-residual-tracking`) | Normative contract | Planning/task/implementation contract | Version-controlled; synchronized with spec/plan/research/quickstart/data-model/checklists when in-scope IDs/surfaces change (CHK025/030-dur). |
| `specs/012-debug-ingest-hook/android-overlays/app/src/**` | Canonical source | FR-011 authoritative | Durable, version-controlled; not generated output. |
| `android-shell/src/main/kotlin/app/mdreader/{DebugIngestActivity,DebugIngestHook,DocIngest}.kt` | Canonical source (Kotlin) | Hook + production ingest bridge | Small blast radius; no duplicate production ingest path. |
| `src-tauri/gen/android/app/src/{debug,androidTest}/...` materialized files | Generated target | Non-canonical | Regenerable; validated by FR-011 check/apply; never source of truth. |
| `cache/debug_ingest_result.txt` (device, debug) | Runtime evidence channel | Authoritative when readable/parseable | App-internal; capture to reviewer-accessible host evidence with path/channel/write status/corroboration. |
| Timestamped host evidence dirs / `REPORT.md` / `summary.txt` / logcat excerpts / scan outputs | Evidence records | FR-008/FR-010/FR-012 support | Reviewer-accessible locations recorded; retain until covered `EV/RES/TASK/FR/SC` IDs decided/closed or repo policy. Ignored dirs not required retention targets. |
| Release APK/AAB, merged/packaged manifest, merger report, static/tool outputs | Release exposure evidence | FR-010 D2/D4 inputs/outputs | Bind hashes, tool/path, variant, command surface; no stale identity reuse. |
| `specs/009-open-file-from-title/tasks.md`, `specs/010-directory-reading/tasks.md` | Owning residual/task records | FR-012 status authority for T184/T226 split | Required update locations; no orphan ad-hoc-only evidence may claim T184/T226 status. |

### Deep-Research Implications (normative design constraints)

- **Tauri gen [INFERENCE]**: exact `tauri android init`/Android generation overwrite behavior for every file under `src-tauri/gen/` is toolchain-dependent and not assumed complete. Durability therefore depends on canonical sources + idempotent, fail-closed `apply/check`, not generator promises.
- **Manifest merger D2 authoritative**: Android merged/packaged manifest and merger report are the authoritative exposure signals for release/non-debug claims. Source/guard checks are necessary defense-in-depth but insufficient alone; `apkanalyzer`/`aapt2`/manifest-merge outputs must be identity-bound and tool/path-recorded.
- **Defense-in-depth**: debug variant placement, runtime `BuildConfig.DEBUG`/hook refusal, static artifact inspection, runtime negative invocation, and repeatable regression checks are complementary. No single signal proves FR-010 absence; class absence is informative, not sufficient.
- **Correlation/provenance**: every run/check/evidence record carries correlation key, fixture/operation, build+device+artifact/install identity, tool versions observed/repo pins when present, canonical/target paths, hashes/timestamps when available (`unavailable:<reason>` otherwise), checkout/gen state when required, and effective reviewer-accessible location + write status. Conflicts/inconclusive states are explicit.
- **Idempotent, fail-closed apply/check**: `check` is non-mutating and fails closed on missing/stale/forbidden/unknown-format/unverifiable state; `apply` is safe no-op when current; rerun at CHK018 points. Narrow generated-file patching remains CHK021-exception only with recorded decision, owner+expiry, context-checked script, and removal plan.


## Project Structure

### Documentation (this feature)

```text
specs/012-debug-ingest-hook/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── debug-ingest-automation.md
│   ├── durable-overlay-materialization.md
│   ├── release-exposure-evidence.md
│   └── t184-t226-residual-tracking.md
├── checklists/
│   └── requirements.md
└── tasks.md             # created by /speckit.tasks, not by /speckit.plan
```

### Source Code / Artifacts (repository root)

```text
android-shell/src/main/kotlin/app/mdreader/
├── DebugIngestActivity.kt      # existing debug-only ADB entry; durable canonical Kotlin source
├── DebugIngestHook.kt          # existing debug-only helper; loads native lib, routes via DocIngest
└── DocIngest.kt                # production ingest; contains minimal debug helper entrypoints

specs/012-debug-ingest-hook/android-overlays/app/src/
├── debug/AndroidManifest.xml   # canonical FR-011 overlay source (materialize into gen)
└── androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt  # canonical optional smoke scaffold; required in apply/check only when this source exists

src-tauri/gen/android/app/      # generated (mostly gitignored); materialization target only
├── build.gradle.kts            # includes android-shell/src/main/kotlin (T028b path)
├── src/main/AndroidManifest.xml
├── src/debug/AndroidManifest.xml        # materialized by apply/check (must not be canonical)
└── src/androidTest/...                  # materialized by apply/check when canonical androidTest source exists; otherwise reported not-applicable

scripts/
├── automate-t184-t226-hook.sh           # existing debug automation (harden per contracts)
├── automate-t184-t226-adb.sh            # existing ADB evidence helper
├── check-manifest.sh / check-apk-classes.sh / verify.sh
├── apply-debug-ingest-overlay.sh        # planned FR-011 idempotent apply
├── check-debug-ingest-overlay.sh        # planned FR-011 fail-closed check
└── check-debug-ingest-release-exposure.sh  # planned FR-010 repeatable regression/evidence

specs/009-open-file-from-title/tasks.md  # T184 residual split tracking (FR-012)
specs/010-directory-reading/tasks.md      # T226 residual split tracking (FR-012)
```

**Structure Decision**: Keep product logic in existing Rust/Kotlin canonical locations. Add only narrow validation infrastructure: version-controlled overlay sources under the feature spec at the normative fixed path, generated-tree materialization via explicit scripts, and repeatable evidence/check scripts under `scripts/`. Do not make `src-tauri/gen/**` the source of truth. Instrumented test execution MAY use an Android emulator when a physical device is unavailable; emulator use does not satisfy SC-006 materialization requirements.

## Complexity Tracking

| Concern | Why needed | Simpler alternative rejected |
|---|---|---|
| Canonical overlay + apply/check scripts | `src-tauri/gen/**` is gitignored/regenerable; FR-011 requires no silent loss | Keeping debug manifest only in `gen` is fragile and fails FR-011; broad patching of generated Gradle/Tauri files is higher drift risk. |
| Combined FR-010 evidence (static + runtime + repeatable) | Manifest merge/build variants can expose components despite source intent; runtime-only or source-only proof is insufficient | Single-signal check gives false confidence; Android security guidance treats debug/test features in production as risk. |
| FR-012 split residual tracking | Programmatic hook cannot prove native SAF picker cancel/TalkBack/keyboard traversal | Marking T184/T226 fully complete from hook evidence would violate FR-012/SC-003 and overstate coverage. |
