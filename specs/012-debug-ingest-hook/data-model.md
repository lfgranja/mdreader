# Data Model: 012-debug-ingest-hook

This feature is primarily validation/tooling infrastructure. Entities are **contract/evidence records**, not product database tables. Records cover debug-ingest automation runs (FR-001–FR-009, FR-013), release-exposure checks (FR-010), durable overlay materialization (FR-011), residual scope tracking (FR-012), and traceability/preconditions (`FR/SC/US/EC/ASM/DEP/EV/TASK/RES/T184/T226` IDs, CHK001/002/018/021/022/024/025/026/027/030/032).

## 1. AutomationRequest

Represents one debug-only automation invocation.

| Field | Type | Required | Validation / Notes |
|---|---|---|---|
| `request_id` / `run_key` | string | required for reported T184/T226/release evidence; recommended otherwise | Stable correlation key (UUID or deterministic `timestamp+device+operation+fixture_ref hash`). Prevents multi-run mixing. |
| `operation` | enum `document` \| `directory` \| `uri` | yes | Normalized per debug-ingest contract validity matrix. |
| `fixture_ref` | string | yes | App-accessible path or URI. Must be validated before ingest; inaccessible refs => explicit diagnosable/blocked result. Confirmed deterministic precedence (2026-11-05, observed in `DebugIngestActivity.handle()`): if `path` and `uri`/intent data are both supplied in document/directory modes, `path` takes precedence. `uri` mode uses intent data/`--es uri`; `file://` URIs may be routed via file path where supported; unsupported URI schemes produce explicit diagnosable errors. Any intentional change requires synchronized updates to `contracts/debug-ingest-automation.md`, `tasks.md`, tests, and implementation. |
| `component` | string | yes for ADB contract | `app.mdreader/.DebugIngestActivity` (debug build). |
| `action` | string | optional | `app.mdreader.action.DEBUG_INGEST` supported. |
| `build_variant` | enum `debug` \| `release` \| `non-debug` \| `unknown` | required in evidence | Release/non-debug must never produce hook success. |
| `device` | object `{serial, model?, sdk?, abi?}` | required in evidence | Binds result to device/emulator. |
| `artifact` | object `{path, sha256?, variant?, version?}` | required for release evidence; recommended for debug | Binds check to exact APK/AAB/install. |
| `timeout_ms` / run budget | integer | recommended | SC-002 budget 2 minutes/run including evidence capture. |

**State transitions**: `requested` → `invoked` → `result_emitted` (`ok:*`/`error:*`/`refused:*`) → `evidence_captured`. Authoritative/corroborated channel rules per debug-ingest contract.

## 2. FixtureReference

| Field | Type | Required | Validation |
|---|---|---|---|
| `kind` | enum `file_path` \| `directory_path` \| `file_uri` \| `content_uri` | yes | Derived from input. |
| `value` | string | yes | Must not be empty. |
| `accessible_to_app` | boolean/unknown | evidence | Existence outside app sandbox does not imply readability. SC-001 precondition failure => `blocked:fixture-inaccessible`. |
| `expected_filtering` | object (directory) | required when claiming directory pass by count | Expected accepted/rejected entry behavior by production filtering; used to interpret `chapters=N`. |

## 3. ValidationResult

Authoritative outcome of one automation run.

| Field | Type | Required | Validation |
|---|---|---|---|
| `correlation_id` / `run_key` | string | required in reported evidence | Binds result to `AutomationRequest.request_id`/run outputs; prevents multi-run mixing (FR-008, debug-ingest contract). |
| `status` | enum `pass` \| `fail` \| `blocked` \| `inconclusive` | yes | `pass` requires parseable `ok:*` for covered scope + crash/ANR not detected. `crash_anr_detected=unknown` => `inconclusive` unless separately disproven with evidence. `blocked` = precondition/tooling prevents explicit result (`blocked:fixture-inaccessible`, dependency/tooling blockers). `inconclusive` = ambiguous/conflicting/missing authoritative evidence. Fail-closed: never silent pass. |
| `raw_result` | string | yes when emitted | Stable prefixes: `ok:document:<path>`, `ok:uri:<uri>`, `ok:directory:<path>:chapters=<n>`, `error:<class/code>:<detail>`, `refused:not-debug`. |
| `scope_class` | enum `programmatic-non-ui` \| `release-negative` \| `static-artifact` \| `manual-ui-a11y` | yes | Debug hook runs are `programmatic-non-ui`; residual UI/a11y uses residual scope classes. |
| `crash_anr_detected` | boolean/unknown | yes | Must be reported; unknown is not pass. |
| `identity_binding` | object | required in reported evidence | Artifact identity (path/variant/version/SHA-256/signing/build metadata as available) and device/install identity (serial/model/SDK/ABI/package identity) bound to the result (FR-008, SC-001/SC-002, ASM/DEP-012-ABI-PACKAGE). |
| `channel` / `corroboration` | object | required in reported evidence | `authoritative_channel` (cache authoritative when readable/parseable per contract), `channels_used` (primary/secondary attempted), `corroboration_state` (`authoritative-only` \| `corroborated` \| `conflicting` \| `inconclusive`), `conflicts`. Corroborated fallback allowed only per debug-ingest contract; conflicts/missing authoritative evidence => `inconclusive` unless resolved. |
| `evidence_refs` | list[string] | required in reported evidence | Links to `EvidenceRecord` IDs/paths supporting this result. |
| `diagnostics` | object | recommended | Exception class/message, JNI/native load issues, unsupported extension, size refusal, no-chapters, fixture unreadable, busy/cold-start, dependency/precondition blockers. |
| `observed_counts` | object | optional | e.g. directory `chapters_accepted`; interpret against production filtering + fixture composition/expected filtering (`ASM-012-CHAPTERS-INTERP`). |
| `run_outcome_vs_script_outcome` | note/object | recommended when wrapper fails | Distinguish script/tooling failure from ingest-path outcome. |

**State transitions (fail-closed):** `requested` → `invoked` → `result_emitted` (`ok:*`/`error:*`/`refused:*`) → `evidence_captured` (FR-008 disposition) → terminal `pass|fail|blocked|inconclusive`. Missing required precondition/dependency/channel => `blocked`/`inconclusive` as applicable; never upgrade to `pass`. [INFERENCE] `result_emitted` may be absent for hard tooling blockers; terminal state still requires explicit `blocked` evidence.
## 4. EvidenceRecord

| Field | Type | Required | Validation |
|---|---|---|---|
| `evidence_id` | string | recommended; required in T184/T226/release/FR-011 reporting | Stable immutable reference for reports/tasks/traceability (`EV-012-*` where used; CHK032 no renumber/reuse). |
| `correlation_id` / `run_key` | string | required for run-scoped evidence | Binds evidence to `AutomationRequest`/`ValidationResult`/`MaterializationRun`/`ReleaseExposureCheck` as applicable. |
| `kind` | enum `hook_result` \| `logcat` \| `script_report` \| `manifest_scan` \| `apk_scan` \| `merger_report` \| `runtime_negative` \| `screenshot` \| `manual_note` \| `overlay_source` \| `materialization_check` \| `hash_record` | yes | Covers FR-008/FR-010/FR-011/FR-012 evidence. |
| `path_or_uri` | string | yes | Local path or artifact reference; for channel-only evidence record effective channel and `unavailable:<reason>`/write status explicitly. |
| `sha256` | string | required when artifact/output exists and integrity matters (release artifacts/check outputs; captured artifacts/hashes where available) | If unavailable, record `unavailable:<reason>`. Hashes bind provenance and prevent stale/mismatched evidence reuse (FR-010 identity binding, FR-011 hash records). |
| `preferred_location_or_channel` | string | required for reported run evidence | Intended output path/channel before execution. |
| `effective_location_or_channel` | string | required for reported run evidence | Actual path/channel used; may differ on fallback. |
| `write_status` | enum `written` \| `unavailable` \| `failed:<reason>` | required for reported run evidence | Preferred-output failure remains attributable; corroboration rules apply (FR-008, debug-ingest contract). |
| `reviewer_accessible_location` | string | required for retained evidence | Repo path or traceable link accessible to reviewers/owners (CHK001/CHK007 intent). |
| `retention_scope` | string | required for retained evidence | “Until covered `EV/RES/TASK/FR/SC` IDs are decided/closed or per repo policy; not required in ignored dirs” (FR-008/SC-006 boundary). |
| `channel_role` / `corroboration` | object/string | required when evidence is a run/result channel | `primary`/`secondary`/`authoritative`/`corroborating`; corroboration state/conflicts; aligns with `ValidationResult.channel/corroboration`. |
| `identity_binding` | object | required where artifact/device identity matters | Artifact identity (APK/AAB/manifest/report path, variant, SHA-256, build/signing/version metadata as available) and/or device/install identity (serial/model/SDK/ABI/package). Prevents stale/wrong-target evidence. |
| `supports` | list[string] | yes | FR/SC/US/EC/ASM/DEP/TASK/EV/RES IDs (`FR-008`, `FR-010`, `FR-011`, `SC-004`, `T184-PROG`, `T226-RES-A11Y`, `DEP-012-*`, etc.). |
| `captured_at_utc` | RFC3339 string | yes when output exists | If unavailable, record `unavailable:<reason>`. |
| `method` | enum `automated` \| `static-analysis` \| `manual` | yes | |
| `tool_name` / `tool_path` / `command_surface` | string | required for static/runtime/check evidence when tool varies | Provenance of producer (FR-010 tool/path, FR-011 apply/check). |

**Provenance/validation rules:** evidence MUST be attributable to a correlation/run and, when claimed, to exact artifact/device identities. Missing/unavailable hashes/timestamps/channels are recorded explicitly (`unavailable:<reason>`/`failed:<reason>`), never implied. Corroborated evidence is admissible only per contract; otherwise result is `inconclusive`/`blocked`, never pass.
## 5. ReleaseExposureCheck

| Field | Type | Required | Validation (FR-010) |
|---|---|---|---|
| `check_id` / `correlation_id` | string | required in reported evidence | Stable correlation/run identity for evidence bundle and traceability. |
| `mode` | enum `static` \| `runtime` \| `full` | yes | Only `full` may claim FR-010 pass. |
| `artifact_identity` | object | required for static/full | APK/AAB path, variant, hashes, merged manifest/merger report refs, tool/path; binds D2 to exact artifact. |
| `install_identity` | object | required for runtime/full | Package, version, signing/build metadata as available, device serial/model/SDK/ABI, verified release/non-debug target; binds D3. |
| `d1_scope` | check result | required for static/full | Debug-only placement/guards evidenced (source/guard checks necessary, not sufficient). |
| `d2_static_absence` | check result | required for static/full | No in-scope component/action/exported hook identifiers in release artifact/merged manifest (`DebugIngestActivity`, `DebugIngestHook`, `app.mdreader.action.DEBUG_INGEST`, debug manifest/intent-filter identifiers, materialized debug/androidTest overlay targets). |
| `d3_runtime_negative` | check result | required for runtime/full | Installed release cannot invoke hook; no `ok:*`/debug ingest success; no crash/ANR caused by negative check. |
| `d4_repeatable` | check result | yes | Fail-closed script/report produced with machine-reviewable evidence; no stale evidence reuse without identity match. |
| `overall` | enum `pass` \| `fail` \| `partial` \| `blocked-not-evaluated` | yes | `pass` only when required dimensions for mode pass (`full` => D1–D4). `static`/`runtime` never claim full FR-010 pass. `blocked-not-evaluated` records `satisfies_fr010_sc004=false`. |
| `class_presence_note` | string | required when class scan run | Must state class absence/presence is **not sufficient** proof of exposure (shared `android-shell` source set). |
| `blocked_not_evaluated` | object | required when overall blocked-not-evaluated | `{owner, decision_ref?, missing_inputs, attempted_modes, satisfies_fr010_sc004:false}`. Must not satisfy FR-010/SC-004. |
| `tool_provenance` / `evidence_refs` | object/list | required | Tool name/path/command surface used and linked `EvidenceRecord` IDs with hashes/identity binding. |

**State transitions (fail-closed):** `requested` → `inputs_validated` (artifact/install/tool identity for requested mode) → `dimensions_evaluated` (D1–D4 as applicable) → terminal `pass|fail|partial|blocked-not-evaluated`. Missing required input/tool/identity for requested mode => `fail` or `blocked-not-evaluated` as applicable; never silent pass or full FR-010 claim from non-full/blocked mode.

## 6. ResidualScopeItem

Tracks FR-012 work not covered by debug-hook automation.

| Field | Type | Required | Validation |
|---|---|---|---|
| `residual_id` | string | yes | Immutable ID, e.g. `T184-RES-SAF-CANCEL`, `T184-RES-A11Y`, `T226-RES-HARDWARE-CONTROLS` (CHK032). |
| `parent_task` | string | yes | `T184` or `T226`; parent fully complete only when required `*-PROG` and required residuals are `verified` or explicitly re-scoped by recorded decision (never implicit). |
| `criterion` | string | yes | Human-readable scenario/capability. |
| `scope_class` | enum `native-picker-ui` \| `accessibility` \| `system-controls` \| `device-ux` | yes | |
| `automation_status` | enum `not-covered-by-hook` \| `partially-covered` \| `covered` | yes | SAF cancel/TalkBack traversal = `not-covered-by-hook`; hook evidence cannot close these. |
| `status` | enum `open` \| `verified` \| `accepted-risk` \| `deferred` | yes | |
| `evidence_refs` | list[string] | required when verified/accepted-risk | Manual/a11y/device/system-control evidence as applicable (`ASM-012-MANUAL-A11Y`). |
| `owner` / `decision_ref` | string | required for accepted-risk/deferred | Dated decision ref, rationale, scope boundary; accepted-risk does not satisfy full verification or auto-complete parent while `blocks_full_completion=true`. |
| `blocks_full_completion` | boolean | yes | True for T184/T226 residuals unless explicitly re-scoped by recorded decision. |
| `full_parent_completion_claimed` | boolean | required in reporting context | Must be `false` for hook-only/programmatic coverage (FR-012/SC-003). [INFERENCE: tracked on report/parent summary; may be mirrored here for consistency.] |

**State transitions:** `open` → `verified` (appropriate evidence) | `accepted-risk` (dated owner/decision/rationale/scope/evidence where applicable) | `deferred` (owner/decision/rationale). No transition may implicitly mark parent T184/T226 fully complete. Blocking residuals with `blocks_full_completion=true` keep full parent completion false unless explicitly re-scoped.

## 7. Canonical vocabulary

- `ValidationResult.scope_class`: `programmatic-non-ui`, `release-negative`, `static-artifact`, `manual-ui-a11y`.
- `ResidualScopeItem.scope_class`: `native-picker-ui`, `accessibility`, `system-controls`, `device-ux`.
- Result prefixes: `ok:document:*`, `ok:uri:*`, `ok:directory:*:chapters=<n>`, `error:<class/code>:<detail>`, `refused:not-debug`.
- FR-008 minimum (authoritative, spec §FR-008): fixture/reference, operation, explicit parseable outcome/status, build+device identity, diagnostic capture, crash/ANR status/signal, residual scope note, and evidence disposition (preferred/effective path or channel, `write_status`, reviewer-accessible location, retention scope tied to covered IDs/policy). FR-008 source of truth: `spec.md` §FR-008. `data-model.md`/contracts/tests/tasks may add required fields but MUST NOT weaken; synchronize on FR-008 changes (NFR-012-MAINT-005). Contracts/data-model may add required fields but MUST NOT weaken FR-008.

## 8. OverlaySource (FR-011)

Canonical maintained source for debug-only automation overlay artifacts; never `src-tauri/gen/**`.

| Field | Type | Required | Validation |
|---|---|---|---|
| `source_id` | string | required | Stable ID for traceability (`TASK/EV` links) [INFERENCE]. |
| `canonical_root` | string | yes | Normative root: `specs/012-debug-ingest-hook/android-overlays/app/src/` (FR-011). |
| `files` | list[object] | yes | Per managed source: `path`, `kind` (`debug_manifest` \| `androidtest_smoke` \| other documented), `required` (debug manifest required; androidTest required only when canonical androidTest source exists per FR-013/SC-006), `exists`, `sha256`/hash record ref, `format_version_ref`. |
| `format_version_ref` | string/object ref | yes | Links to `OverlayFormatVersion`; missing/unknown handled fail-closed per CHK022/CHK024. |
| `ownership` | object | required | Spec 012 maintainer owns canonical overlay/contracts; Android/shell maintainer co-owns Kotlin/debug manifest/wiring surfaces (plan CHK022/CHK024, CHK001). |
| `durability_boundary` | note | yes | Canonical authority outside ignored generated output; `src-tauri/gen/**` is materialization target only (NFR-012-MAINT-002). |

## 9. MaterializationTarget (FR-011)

Generated Android project locations managed by FR-011 apply/check.

| Field | Type | Required | Validation |
|---|---|---|---|
| `target_id` | string | required | Stable ID [INFERENCE]. |
| `path` | string | yes | Normative targets: `src-tauri/gen/android/app/src/debug/AndroidManifest.xml` and, when canonical androidTest source exists, `src-tauri/gen/android/app/src/androidTest/.../DebugIngestSeamSmokeTest.kt` (or documented equivalent). |
| `source_ref` | string/object ref | yes | Maps to `OverlaySource.files[]`. |
| `managed` | boolean | yes | Managed targets are non-canonical and may be overwritten only by idempotent apply/check process. |
| `applicable` | enum `required` \| `not-applicable` | yes | androidTest target required only when canonical androidTest source exists; otherwise explicit `not-applicable` (FR-013/SC-006). |
| `forbidden_states` | list[string] | required | Missing/stale/forbidden content when required; manual ad-hoc patching in `src-tauri/gen/**` as authority; silent absence. Fail-closed. |

## 10. ManagedManifest / HashRecord (FR-011)

Per-file expected-vs-observed integrity record for managed overlay materialization.

| Field | Type | Required | Validation |
|---|---|---|---|
| `record_id` | string | recommended | Links to evidence/correlation. |
| `correlation_id` | string | required in check/apply evidence | Binds to `MaterializationRun`. |
| `source_path` | string | yes | Canonical source path. |
| `target_path` | string | yes | Materialization target path. |
| `expected_sha256` | string | required when source exists/readable | Hash of canonical content or canonical-derived expected materialized bytes [INFERENCE: exact normalization documented by implementation/check]. |
| `observed_sha256` | string | required when target evaluated | `unavailable:<reason>` if unreadable/missing. |
| `match` | boolean/unknown | yes | `true` only on verified match; missing/unreadable/unknown => fail-closed for required applicable target. |
| `status` | enum `pass` \| `fail` \| `blocked` \| `not-applicable` | yes | Aligns with `MaterializationRun` status semantics. |
| `drift_detail` | string/object | required on non-pass | Explicit expected-vs-actual/remediation-safe diagnostic. |

## 11. OverlayFormatVersion (FR-011, CHK022/CHK024)

| Field | Type | Required | Validation |
|---|---|---|---|
| `version` | string | required | Detected canonical overlay format version reported by apply/check. |
| `detection_method` | string | required | Metadata/file convention used by check contract (normative requirement: detectable). |
| `detection_status` | enum `detected` \| `v1-implicit` \| `missing` \| `unknown` | yes | `v1-implicit` allowed only if explicitly documented by check contract; otherwise `missing`/`unknown` => `fail`/`blocked`. |
| `compatibility` | enum `compatible` \| `backward-compatible` \| `migration-required` \| `incompatible` | required when evaluated | Incompatible evolution requires migration note and synchronized updates to canonical targets + checks + contracts/docs + future tasks (CHK024/NFR-012-MAINT-005). |
| `migration_note_ref` | string | required for migration-required/incompatible | Evidence/decision/task reference. |

## 12. MaterializationRun (FR-011, CHK018)

One apply/check execution for durable debug overlay materialization.

| Field | Type | Required | Validation |
|---|---|---|---|
| `run_id` / `correlation_id` | string | required | Binds run to hash records, evidence, and CHK018 trigger context. |
| `mode` | enum `apply` \| `check` | yes | `check` is non-mutating and fail-closed; `apply` is idempotent and writes only managed targets from canonical sources. |
| `trigger` | enum/string | required | CHK018 run point: local bootstrap/setup, `tauri android init`/Android gen regeneration, canonical overlay/hook/debug-manifest/androidTest source change, Tauri/Android toolchain/template/Gradle/AGP/NDK/SDK change affecting gen, CI bootstrap/Android debug job for Spec 012, before T184/T226 validation campaigns / SC-006 claim. |
| `detected_version` | object ref / string | required | `OverlayFormatVersion` detection result. |
| `status` | enum `pass` \| `fail` \| `blocked` \| `not-applicable` | yes | Explicit no-silent-missing semantics. `not-applicable` only for documented cases (e.g., androidTest target absent because canonical androidTest source absent). |
| `expected_vs_actual` | list[object] | required | Per target: path, expected source/state/hash, actual state/hash, result, remediation-safe detail. |
| `dep_layout_validation` | list[object refs / results] | required | Results for relevant `DependencyPreconditionRecord`s / layout assumptions (`DEP-012-TAURI-GEN`, `DEP-012-REPO-SCRIPTS`, `DEP-012-GIT-CHECKOUT`, `DEP-012-GEN-COMPAT`, `ASM-012-REPO-LAYOUT`, `ASM-012-SOURCESET-WIRING`) as invoked/applicable. Missing required dependency/layout drift => `fail`/`blocked`. |
| `tool_name` / `command_surface` | string | required | `apply-debug-ingest-overlay.sh` / `check-debug-ingest-overlay.sh` or equivalent documented command. |
| `evidence_refs` | list[string] | required | Hash records, check/apply output, diagnostics. |

**State transitions:** `requested` → `inputs_validated` (deps/layout/version detection; fail-closed on blockers) → `evaluated` (`check`) / `materialized` (`apply`, idempotent) → `verified` (post-apply/check comparison) → terminal `pass|fail|blocked|not-applicable`. **Fail-closed semantics:** missing required inputs/deps, missing/stale/forbidden required target, unknown/missing format version (unless documented `v1-implicit`), mutating behavior during `check`, non-idempotent apply detection [INFERENCE: detectable by repeated apply/check], or layout/wiring drift => terminal non-pass with explicit diagnostics; never silent pass or treating `src-tauri/gen/**` as canonical.

## 13. DependencyPreconditionRecord (ASM/DEP-012-*; FR-008/010/011/012)

Traceable dependency, precondition, or validated/bounded assumption used by automation, materialization, release-exposure, and residual evidence.

| Field | Type | Required | Validation |
|---|---|---|---|
| `id` | string | yes | Immutable `ASM-012-*` / `DEP-012-*` (CHK032). |
| `record_type` | enum `dependency` \| `precondition` \| `assumption` | yes | Matches spec traceability table. |
| `requirement_level` | enum `required` \| `optional` \| `validated-bounded` | yes | Drives fallback behavior. |
| `statement` | string | yes | Source-of-truth/dependency/assumption statement. |
| `verification_method` | string | yes | How observed/checked. |
| `observed` | object/string | required when evaluated | Actual tool/path/layout/version/accessibility state; `unavailable:<reason>` allowed. |
| `result` | enum `pass` \| `fail` \| `blocked` \| `not-applicable` | required when evaluated | Explicit; missing required dependency/precondition => `fail`/`blocked` for affected claim. |
| `failure_behavior` | string | yes | Fail-closed behavior (e.g., `blocked:not-evaluated`, `blocked:fixture-inaccessible`, explicit `error:*`, never silent pass). |
| `applies_to` | list[string] | recommended | `FR/SC/US/TASK/EV/RES` IDs, entity/run refs (`MaterializationRun`, `ReleaseExposureCheck`, `ValidationResult`). |
| `evidence_refs` | list[string] | required on non-pass or when claim depends on it | Traceable evidence/decision refs. |

**Rules:** records make hidden dependencies explicit (NFR-012-MAINT-004). A claim that depends on a failed/blocked required record is invalid for that scope and MUST be reported explicitly; optional dependencies may degrade scope only with explicit `not-applicable`/bounded evidence.


## 14. Host Contract Modes (additive, host-deterministic; 2026-11-06)

| Mode | Output/role | Integrity |
|---|---|---|
| `--print-activity-channel-contract` / `--classify-channel-attribution` | Cache/channel attribution; unwritten authoritative cache never implies success | FR-006/FR-008 |
| `--print-intent-parse-contract` / `--normalize-debug-ingest-intent` | Debug-only intent parse/normalize/validation (aliases, U1 precedence, missing-input errors); no side effects | I1/T057 fallback; **not** parallel production ingest; ASM-012-INGEST-SOT; CHK021 no gen mutation |
| `--print-directory-uri-contract` | Directory ref classification; unsupported/unresolvable => explicit `error:*`, no fake/ambiguous success | FR-002/FR-007 |
| `--build-device-evidence` | FR-008 minimum per-op builder; SC counters hooks; `unknown|detected` crash non-pass | `full_parent_completion_claimed=false` |
| `--print-device-run-contract` / `--classify-device-run` | Normal device-run evidence wiring/classification (result-file/status/identity/crash/corroboration) | T018/T024/T067/T028; executed runs require bound attributable FR-008/SC evidence |

FR-008 minimum field set remains authoritative in `spec.md` §FR-008; additive fields (including contract counters/classification helpers) must not weaken it (NFR-012-MAINT-005).
