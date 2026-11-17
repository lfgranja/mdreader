# Contract: Durable Overlay Materialization (FR-011, SC-006)

**Feature**: `012-debug-ingest-hook`  
**Covers**: FR-011, FR-013 (androidTest scaffold materialization condition), SC-006, CHK001/002/018/021/022/024/025/026/027/030-dur, NFR-012-MAINT-001–005  
**Status**: Normative for planning/tasks/scripts. `apply`/`check` behavior is authoritative for Spec 012 durable overlay claims.

## 1. Purpose & Scope

This contract defines how debug-only automation artifacts required for maintenance remain durable across Android project regeneration/build refresh. It prevents silent loss of debug hook wiring/scaffolds when `src-tauri/gen/**` is refreshed.

**In scope**: canonical overlay sources, generated materialization targets, overlay format/versioning, `apply`/`check` command contracts, managed manifest/hash strategy, narrow generated-file patching exception, dependency/layout validation, security boundaries, evidence/provenance, ownership/sync, and traceability.

**Out of scope**: product playback/ingest behavior (Rust authority, `ASM-012-INGEST-SOT`), release exposure claims except as they depend on materialized debug targets (`release-exposure-evidence.md`, FR-010), native SAF picker UI/a11y residuals (FR-009/FR-012), and historical evidence retention in ignored/output directories (SC-006 covers hook-capability durability, not retention).

## 2. Authority & Canonical Locations (NFR-012-MAINT-002, CHK024)

| Role | Normative path(s) | Authority |
|---|---|---|
| Canonical overlay root | `specs/012-debug-ingest-hook/android-overlays/app/src/` | **Canonical, version-controlled source of truth** for managed overlay files. `apply` copies/verifies from here only. |
| Canonical Kotlin hook sources | `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt`, `DebugIngestHook.kt`, and related debug-only hook code; `DocIngest.kt` contains only minimal debug-helper entrypoints as already scoped by product architecture | **Canonical implementation sources**. Compiled via existing `android-shell/src/main/kotlin` source-set wiring (`ASM-012-SOURCESET-WIRING`). This contract does not move Kotlin authority into `gen`. |
| Generated materialization targets | `src-tauri/gen/android/app/src/debug/AndroidManifest.xml`, `src-tauri/gen/android/app/src/androidTest/**` (managed androidTest target only when required) | **Materialization targets only**. `apply`/`check` MAY read/write these managed targets; they are never canonical inputs except as explicit targets verified by this contract. |
| Generated Android project | `src-tauri/gen/**` | **Never canonical**. Mostly gitignored/regenerable. Manual edits are prohibited except the narrow CHK021 exception. `[INFERENCE]` Exact `tauri android init` overwrite behavior for every generated Android file is not fully documented for the installed CLI; `check`/`apply` MUST fail closed on detected drift regardless of overwrite cause. |

`NFR-012-MAINT-002`: `src-tauri/gen/**` and generated/output artifacts are never canonical; this seam MUST NOT create a parallel production ingest path. Rust remains sole playback/ingest authority (`ASM-012-INGEST-SOT`).

## 3. Managed Targets Table

| Managed target | Canonical source | Generated target | Required rule | Apply/check behavior |
|---|---|---|---|---|
| Debug manifest overlay | `specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml` | `src-tauri/gen/android/app/src/debug/AndroidManifest.xml` | **Required** for FR-011/SC-006 whenever debug automation durability is claimed. MUST declare debug-only exported hook component/intent-filter identifiers only as permitted by debug variant (`.DebugIngestActivity`, `app.mdreader.action.DEBUG_INGEST`) and MUST NOT weaken release boundaries. | `apply`: create parent dirs and materialize canonical content idempotently. `check`: verify target exists, is not forbidden, matches expected managed state/hash/format rules, and reports path/expected-vs-actual/remediation on drift. Missing/stale/forbidden => `fail` (fail closed). |
| androidTest smoke scaffold | `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt` (or documented equivalent under `androidTest/`; canonical path/pattern MUST be documented if equivalent used) | `src-tauri/gen/android/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt` (or equivalent documented target path matching canonical source) | **Conditional** (FR-013/SC-006): required for `apply`/`check` whenever its canonical androidTest source exists. If canonical androidTest source is absent, target is **optional** and reported `not-applicable` (not missing). Instrumented test execution MAY use an emulator when a physical device is unavailable; emulator use does not change durability/materialization requirements. | `apply`: if canonical source exists, materialize target idempotently; if absent, report `not-applicable` and do not create a fake scaffold unless explicitly requested by a separate contract/change. `check`: if canonical source exists, target must match managed state; if source absent, report `not-applicable`. |
| Future managed marker/overlay file | Under canonical overlay root, explicitly registered by this contract and `apply/check` implementation | Corresponding path under `src-tauri/gen/android/app/src/{debug,androidTest,...}` as explicitly registered | **Not implied**. Each future target requires a contract update, traceability update, and synchronized script/test/doc changes (NFR-012-MAINT-005, CHK024/030-dur). Default state until registered: out of scope / unmanaged. | `apply/check` MUST ignore unknown unmanaged files unless explicitly registered; MUST NOT silently delete or rewrite unmanaged generated files. Forbidden-state checks apply only to registered managed targets and explicitly forbidden hook-surface exposures. |

**Managed target identity**: implementations SHOULD use a stable managed-target ID in diagnostics/evidence (e.g., `MAT-012-DEBUG-MANIFEST`, `MAT-012-ANDROIDTEST-SMOKE`) mapped to the rows above.

## 4. Overlay Format Versioning (CHK024)

The canonical overlay format MUST declare a format version detectable by `apply`/`check`.

- **Detection mechanism**: implementation-defined marker (e.g., repository-tracked metadata file under the canonical overlay root, front-matter marker in a dedicated non-source metadata file, or equivalent machine-detectable convention). The mechanism MUST be documented in script help/output and this contract when implemented. Managed Android source/XML files MUST remain valid for their target toolchains.
- **Reporting**: `apply` and `check` MUST report detected canonical overlay format version, expected/handled version range (if any), and detected-vs-handled status.
- **v1-implicit rule**: missing/unknown version is treated as `v1-implicit` **only if** the `check` contract output/script documentation explicitly declares v1-implicit compatibility for the current implementation. Otherwise missing/unknown version => `fail` or `blocked` (fail closed), never silent pass.
- **Backward compatibility**: materialization SHOULD remain backward-compatible when practical. `check` MAY accept older compatible versions only when explicitly supported and reported.
- **Incompatible evolution**: requires (1) migration note, (2) synchronized updates to canonical targets + `apply`/`check` + contracts/docs/checklists/future tasks, (3) evidence demonstrating migration/apply/check behavior, and (4) CHK001/CHK002 review. Incompatible changes without synchronization are blocking.

## 5. `apply` Command Contract (FR-011, SC-006, CHK018)

**Purpose**: idempotently materialize managed canonical overlay targets into generated Android project locations.

**Design target command surface**:
```bash
scripts/apply-debug-ingest-overlay.sh [--root <repo-root>] [--gen-dir <src-tauri/gen/android/app>] [--dry-run] [--json] [--evidence-dir <dir>]
```

**Normative behavior**:
- **Idempotent**: running `apply` repeatedly with unchanged canonical inputs and unchanged compatible generated context produces the same managed target state and no unnecessary semantic drift. Repeated runs report `pass`/`no-op` as appropriate.
- **Source of truth**: reads only canonical overlay sources and required layout/dependency context; NEVER treats `src-tauri/gen/**` as canonical input.
- **Scope**: touches only registered managed targets (and required parent directories) by default. MUST NOT rewrite unrelated generated files.
- **Format/version**: detects and reports overlay format version before mutating managed targets; incompatible/unknown version handling follows §4.
- **AndroidTest conditional**: applies androidTest target only when canonical androidTest source exists; otherwise reports `not-applicable` for that target.
- **Fail closed**: returns non-pass status on missing required canonical source (debug manifest), missing/unverifiable required layout/dependency context for the requested operation, unsupported/undocumented unknown format version, path safety violation, write failure, partial application that cannot be safely completed, or detected forbidden target state that `apply` is not explicitly authorized to remediate.
- **Partial-state safety**: if partial application is possible, implementation MUST use an atomic/checked strategy where practical or leave explicit diagnostics; it MUST NOT report success after partial/ambiguous writes.
- **Diagnostics**: report per managed target: `pass|fail|blocked|not-applicable`, path, action (`applied|noop|skipped|blocked`), expected-vs-actual summary where relevant, remediation, detected format version, and evidence/provenance fields.
- **No sensitive data**: diagnostics/evidence MUST NOT expose sensitive runtime/user data (NFR-012-MAINT-003).

## 6. `check` Command Contract (FR-011, SC-006, CHK018)

**Purpose**: verify durable overlay materialization without mutating managed targets.

**Design target command surface**:
```bash
scripts/check-debug-ingest-overlay.sh [--root <repo-root>] [--gen-dir <src-tauri/gen/android/app>] [--json] [--evidence-dir <dir>]
```

**Normative behavior**:
- **Non-mutating**: MUST NOT create, modify, delete, or normalize canonical sources or generated targets. Read-only inspection only.
- **Fail closed**: missing required target, stale/mismatched target, forbidden state, missing required canonical source (debug manifest), unknown/missing format version when not explicitly `v1-implicit`, or missing required layout/dependency context => `fail` or `blocked` with explicit diagnostics. Never silent pass.
- **Idempotence visibility**: reports whether `apply` is needed (`noop`/aligned vs `drift`/missing/stale) but does not perform it.
- **AndroidTest conditional**: reports `not-applicable` when canonical androidTest source is absent; reports `pass|fail|blocked` when source exists.
- **Exit/status/output schema** (normative fields; implementation MAY add fields):
  - overall: `pass|fail|blocked`
  - per target: `status` (`pass|fail|blocked|not-applicable`), `target_path`, `canonical_source` (or `n/a`), `expected`, `actual`, `remediation`, `hash_or_state` summary when safe/useful
  - context: `detected_overlay_format_version`, `handled_version_range`/compatibility note, `repo_root`, `gen_dir`, dependency/layout validation results, `captured_at_utc` or `unavailable:<reason>`, `tool_name`/`tool_path`
  - fail-closed rule: overall is `pass` only when every required managed target is `pass` or explicitly `not-applicable` by rule and no blocking dependency/layout/format error applies.
- **Human/machine output**: human-readable diagnostics required; JSON output SHOULD be supported for automation/evidence.

## 7. CHK018 Required Run Points

Run `check` (non-mutating, fail-closed) at every point below. Run `apply` when `check` fails or canonical/target materialization inputs changed:

1. Local bootstrap/setup for Spec 012 Android debug validation.
2. After `tauri android init` or any known Android project regeneration affecting `src-tauri/gen/android/`.
3. After changes to canonical overlay files, hook debug manifest/wiring sources, or canonical androidTest source (when present).
4. After Tauri CLI, Android templates/Gradle wrapper, AGP/Kotlin/NDK/SDK, or toolchain changes affecting generated Android project behavior.
5. In CI bootstrap/Android debug jobs that cover Spec 012 durability or debug-hook validation.
6. Before T184/T226 validation campaigns and before claiming SC-006 evidence.

`check` MUST be safe to run repeatedly. Prefer incremental builds per Gradle guidance; require clean/regenerate + `check`/`apply` when toolchain/generation inputs changed or stale/mismatched generated artifacts are detected.

## 8. Managed Manifest/Hash Strategy

- **Canonical comparison basis**: managed target state is compared against canonical source content and contract rules. Implementations SHOULD compute/report a content hash (e.g., SHA-256) for canonical source and managed target when useful for diagnostics/evidence; hashes are additive evidence, not a substitute for semantic safety checks.
- **Manifest safety checks**: debug manifest materialization MUST be validated as a managed debug-overlay target (path/variant location, expected identifiers where applicable). `check`/`apply` MUST NOT treat arbitrary manifest content in `gen` as authoritative and MUST NOT materialize hook identifiers into main/release/non-debug targets.
- **Allowed drift**: no unmanaged drift inside registered managed targets is silently accepted. If a target must contain generated/local variation, that variation MUST be explicitly specified by a future contract revision; absent that revision, drift => `fail`.
- **Forbidden states** (examples, non-exhaustive): missing required debug manifest target; androidTest target required but missing/stale; managed target differs from canonical state; hook component/action exported outside intended debug materialization target; target path escapes expected gen dir; unknown format without documented `v1-implicit`; partial/corrupt managed file detected.
- **Evidence hashes**: when `apply/check` produces evidence artifacts, include hashes/timestamps when available; otherwise record `unavailable:<reason>` (FR-008-aligned evidence discipline).

## 9. Narrow Generated-File Patching Exception (CHK021)

**Default `MUST NOT`**: do not manually patch `src-tauri/gen/**` or generated Gradle/Tauri files. Use canonical sources/overlays and FR-011 materialization.

**Exception allowed only with a recorded decision** (`EV/RSK`/decision ref) containing all of the following:

| CHK021 requirement | Required record |
|---|---|
| Exact scope | File paths/patterns, minimal diff summary, and affected FR/SC/TASK/RES IDs. |
| Justification | Why supported source/template/config/canonical overlay cannot solve it; debug/release impact analysis. |
| Ownership/expiry | Named owner and expiry/removal task/date trigger. |
| Automation | Idempotent, context-checked automated script (safe re-run/no-op when already applied). |
| Reapplication | Reapplied after every CHK018 trigger that can affect the patched generated target. |
| Verification | Verified by `check` (and FR-010 checks where exposure/wiring is relevant). |
| Fail closed | Missing target, context mismatch, or partial application => `fail`/`blocked` with remediation; never manual ad-hoc fixup. |
| Removal plan | Action when upstream/template/toolchain fix lands; remove exception and script/patch path when obsolete. |

**Never allowed**: manual ad-hoc edits to generated files, undocumented patch scripts, silent best-effort patching, or patches that weaken FR-010/FR-011/FR-012 guarantees.

## 10. Dependencies & Layout Validation (CHK026/CHK027, `DEP/ASM-012-*`)

`apply`/`check` MUST validate required context for claims they make and fail closed on unverifiable required assumptions.

| ID | Required/validated input | Validation | Fail-closed behavior |
|---|---|---|---|
| `DEP-012-TAURI-GEN` | Tauri CLI/Android gen workflow availability when materialization is invoked in a context that depends on it | Record tool availability/version observed and repo-declared pin/config when present | `blocked`/`fail` for FR-011 claim that depends on unavailable gen workflow; never silent pass |
| `DEP-012-ANDROID-BUILD` | Gradle/Android SDK/tooling for Android build validation claims | Only validate when command scope claims build validation; record observed tools/outcomes | `blocked` for claimed Android build scope; overlay `check` alone may remain explicit host/file-scope only |
| `DEP-012-REPO-SCRIPTS` | `scripts/apply-debug-ingest-overlay.sh`, `scripts/check-debug-ingest-overlay.sh`, POSIX shell/Python as invoked | Check script presence/interpreter/tool availability for invoked mode | `fail`/`blocked` with diagnostic; do not infer pass |
| `DEP-012-GIT-CHECKOUT` | Known checkout state for canonical sources/targets | Record commit/worktree state when practical; validate canonical/target paths and drift explicitly | `fail`/`blocked` when determinism/drift cannot be assessed for the claim |
| `DEP-012-GEN-COMPAT` | Generated Android refresh workflow compatible with FR-011 materialization | Run `check`/`apply` after CHK018 triggers | SC-006/FR-011 fail with remediation |
| `ASM-012-REPO-LAYOUT` | Canonical overlay root `specs/012-debug-ingest-hook/android-overlays/app/src/`; canonical Kotlin `android-shell/src/main/kotlin/app/mdreader/`; generated targets under `src-tauri/gen/android/app/src/{debug,androidTest}/`; `gen` regenerable/non-canonical | Validate required paths when claim depends on them; bound to recorded paths | Drift => `fail`/`blocked` with expected-vs-actual paths |
| `ASM-012-SOURCESET-WIRING` | `src-tauri/gen/android/app/build.gradle.kts` (or equivalent maintained generated build config) includes `android-shell/src/main/kotlin` as required by existing integration | Validate wiring when materialization/build/hook-compilation claims depend on it; record checked file/version context | Missing/unverifiable wiring => `blocked`/`fail` for dependent claim |

`CHK026/CHK027`: every durability dependency is treated as explicit `required|optional` for the invoked scope; optional dependencies MAY be omitted only with explicit scoped status, never silent pass.

## 11. Security (FR-010 not weakened)

- Materialization MUST preserve debug-only placement of hook surfaces. Debug manifest/component/action identifiers are permitted only in intended debug materialization targets.
- `apply`/`check` MUST NOT add, preserve, or authorize hook exposure in main/release/non-debug manifests, merged release artifacts, or unintended exported surfaces.
- D1 source/guard checks are necessary but not sufficient; this contract supports durability, while FR-010/release-exposure-evidence remains authoritative for release absence/inertness claims.
- Diagnostics MUST avoid sensitive runtime data (fixture contents, user data, secrets). Paths and identifiers are allowed only as needed for safe remediation/evidence.
- Managed target writes MUST be path-safe (no traversal outside configured canonical root / gen dir targets).

## 12. Evidence/Provenance Fields

`apply`/`check` outputs and retained evidence SHOULD include, when applicable:

| Field | Meaning |
|---|---|
| `tool_name` / `tool_path` | Script/command identity used. |
| `mode` / `operation` | `apply` or `check`, plus options (`--dry-run`, `--json`, etc.). |
| `repo_root`, `canonical_overlay_root`, `gen_dir` | Bounded paths evaluated. |
| `detected_overlay_format_version` | CHK024 result. |
| `dependency_layout_results` | `DEP/ASM-012-*` validations relevant to scope (implemented by `check-debug-ingest-overlay.sh`; per-check `pass`/`fail`/`blocked`/`not-applicable`, fail-closed for required claims). |
| `managed_targets[]` | Per-target status/path/source/expected/actual/remediation/hash summary. |
| `overall_status` | `pass|fail|blocked` (`apply` may also report no-op/pass detail). |
| `captured_at_utc` | RFC3339 timestamp or `unavailable:<reason>`. |
| `evidence_dir` / output artifact paths | Reviewer-accessible location when evidence is written. |
| `write_status` | `written|unavailable|failed:<reason>` when evidence output requested. |
| `supports_ids` | FR/SC/CHK/NFR/DEP/ASM/TASK/EV/RES IDs supported by the run. |
| `git_state` | Commit/worktree summary when practical, or `unavailable:<reason>`. |

Evidence storage/retention follows FR-008 discipline where reported: preferred/effective path, write status, reviewer-accessible location, retention until covered IDs are decided/closed or repo policy; ignored dirs are not required retention targets.

## 13. Ownership & Sync (CHK001/002/022/024)

| Area | Owner | Sync rule |
|---|---|---|
| Canonical overlay root and this contract | Spec 012 maintainer | Owns canonical structure/versioning/target registry and contract text. |
| Kotlin debug hook sources, debug manifest semantics, source-set wiring, FR-010/FR-012 evidence surfaces | Android/shell maintainer (co-owner) | Changes to hook component/action, manifest identifiers, Kotlin source paths, or wiring require synchronized overlay/materialization/release checks and contracts. |
| Generated Android project / Tauri gen/materialization/build-config impacts | Tauri/mobile maintainer (co-owner when affected) | Gen workflow/toolchain changes require CHK018 rerun and layout/dependency validation updates. |
| QA/evidence claims | QA/evidence reviewer (FR-010/FR-012 scope) | Durability evidence may support SC-006; it MUST NOT be overstated as FR-010/FR-012 closure. |

**Review handoff (CHK001)**: provide affected IDs, `check`/`apply` result (idempotence/drift), format version, gen impact (`src-tauri/gen/**` not canonical), and residual risks. **Blocking findings (CHK002)**: missing/stale/forbidden targets, non-idempotent/non-fail-closed behavior in defined scenarios, silent failures, missing trace links for in-scope FR/SC/RES, or manual generated-file patching without CHK021 decision.

## 14. Traceability IDs

| ID | Contract coverage |
|---|---|
| `FR-011` | Durable canonical overlay + idempotent/fail-closed `apply`/`check`, no `gen` canonical authority. |
| `FR-013` / `SC-006` | androidTest scaffold required only when canonical androidTest source exists; required durable targets are debug hook Kotlin sources in `android-shell/src/main/kotlin/` plus debug manifest overlay target. |
| `CHK018` | Required `check`/`apply` run points after gen/toolchain/canonical changes and before T184/T226/SC-006 claims. |
| `CHK021` | Default no generated-file patching; narrow exception decision requirements. |
| `CHK022` / `CHK024` | Ownership and canonical overlay format versioning/evolution. |
| `CHK025` / `CHK030-dur` | Small blast radius, no duplicate authority, readable diagnostics, repo-toolchain portability, cross-artifact consistency. |
| `CHK026` / `CHK027` | `DEP/ASM-012-*` required/optional validation and layout/wiring fail-closed behavior. |
| `NFR-012-MAINT-001–005` | Maintainability constraints and synchronized artifact updates. |
| `DEP-012-TAURI-GEN`, `DEP-012-ANDROID-BUILD`, `DEP-012-REPO-SCRIPTS`, `DEP-012-GIT-CHECKOUT`, `DEP-012-GEN-COMPAT` | Dependency preconditions/fallbacks. |
| `ASM-012-REPO-LAYOUT`, `ASM-012-SOURCESET-WIRING`, `ASM-012-INGEST-SOT` | Bounded layout/wiring assumptions and production ingest authority boundary. |

Future implementation tasks/scripts MUST reference this contract and update affected trace links per CHK032. Changes affecting this contract's paths, grammar, run points, or target rules MUST synchronize `spec.md`, `plan.md`, `research.md` (if decision changes), `quickstart.md`, contracts, relevant checklists, and future `tasks.md` in the same change set/review (NFR-012-MAINT-005).