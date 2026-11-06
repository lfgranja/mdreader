# Contract: Debug Ingest Automation Seam

**Feature**: `012-debug-ingest-hook`  
**Covers**: FR-001–FR-009, FR-013, SC-001, SC-002, SC-005  
**Status**: Normative for planning/tasks; reflects existing debug hook behavior and required hardening.

## Purpose

Provide a debug/test-only automation entry that exercises the **same production** document/directory ingest path used by normal app selection flows, producing explicit, script-parseable evidence without manual SAF picker interaction.

## Authority / Pipeline Invariants

- Rust remains the sole playback/ingest authority. Kotlin hook code is transport/validation only.
- Document mode MUST route through `DocIngest` snapshot/fingerprint + `onDocumentIngested` JNI callback (same callback used by SAF document ingest).
- Directory mode MUST route through snapshot/fingerprint/chapter manifest construction and `onDirectorySelected` / manifest dispatch path used by production directory flow.
- The hook MUST NOT implement a parallel fake ingest/open path.
- Native SAF picker UI, cancel UX in system picker, and TalkBack/keyboard traversal of native picker/title UI are **out of contract scope** (FR-009, FR-012).

## Invocation Contract (ADB / debug intent)

**Component**: `app.mdreader/.DebugIngestActivity` (debug build only).  
**Action**: `app.mdreader.action.DEBUG_INGEST` (supported).  
**Normalization**: `mode` aliases are matched case-insensitively by the hook (normative contract: lowercase canonical names). Unknown/unsupported mode returns explicit `error:unsupported-mode:<mode>`.

**Validity matrix (deterministic)**:

| Canonical mode | Accepted aliases | Required input | Routed operation |
|---|---|---|---|
| `document` | `file`, `open_document` | `path` (app-accessible local file) or resolvable `file://`/intent data URI mapped to readable file | Production document ingest/open flow (FR-004) |
| `directory` | `dir`, `tree`, `open_directory` | `path` (app-accessible local directory) or resolvable directory reference supported by production/hook path handling | Production directory-open/manifest-dispatch flow (FR-005) |
| `uri` | — | `uri` extra or intent data URI | Resolve to document/file URI or supported content reference; unsupported/unresolvable => explicit `error:*` |

**Conflict/precedence** (deterministic, confirmed 2026-11-05 from code): `DebugIngestActivity.handle()`: for `document|file|open_document` and `directory|dir|tree|open_directory`, if `--es path`/`path` extra is present it is used first; otherwise intent `data` or `--es uri` is used (for `file://` URI the file path is used where supported). For `uri` mode, intent `data` or `--es uri` is required. `DebugIngestHook.ingestUri()`: `file://` is routed to file ingest; other URI schemes are rejected as unsupported by the debug hook (explicit diagnosable `error:*`). Therefore, when both `path` and `uri`/intent data are supplied in document/directory modes, `path` takes precedence (deterministic). Implementations/tests MUST preserve this unless explicitly changed with synchronized updates to this contract, `data-model.md`, `tasks.md`, tests, and implementation in the same change set.

**Extras**:

| Extra | Type | Required | Meaning |
|---|---|---|---|
| `mode` | string | recommended | Normalized as above. |
| `path` | string | conditional | App-accessible local path for document/directory modes. |
| `uri` | string or intent data URI | conditional | URI to ingest (`file://` may map to path; `content://` supported only where resolvable by app/production path). |

**Example**:
```bash
adb -s <serial> shell am start -n app.mdreader/.DebugIngestActivity \
  --es mode document --es path /data/user/0/app.mdreader/files/mdreader-test-fixtures/t184-teste.md
adb -s <serial> shell am start -n app.mdreader/.DebugIngestActivity \
  --es mode directory --es path /data/user/0/app.mdreader/files/mdreader-test-fixtures/md-dir
```
## Result Contract

Primary channels:

- **Cache file** (debug, app-internal): `cache/debug_ingest_result.txt` — authoritative machine-readable result when present and parseable.
- **Logcat**: `DebugIngestActivity` / `debug ingest result: ...` marker — corroborating; may be used for explicit result only when cache is missing/unreadable and logcat+script capture corroborate the same parseable result, marked `corroborated`.
- **Activity result extra**: `result` (where launcher captures it) — non-authoritative.
- **Script reports**: timestamped `REPORT.md`, `summary.txt`, logcat excerpts, screenshots, crash/ANR checks.

**Channel precedence**: cache authoritative when readable/parseable. If cache missing/unreadable, explicit result allowed only with corroborated logcat+script capture. Conflicts => `inconclusive` unless primary channel parseable. Missing authoritative/corroborated result => `inconclusive`, reported, and not counted as an explicit SC-002 pass.

**Parse grammar** (stable): result string starts with `ok:` | `error:` | `refused:` followed by operation/code and detail. `error:*` MUST include a class/code and short diagnostic detail sufficient for investigation.

**Result strings**:

| Prefix | Meaning |
|---|---|
| `ok:document:<absolute-path>` | Document fixture accepted and routed through production ingest callbacks. |
| `ok:uri:<uri>` | URI fixture accepted and routed through production ingest callbacks. |
| `ok:directory:<absolute-path>:chapters=<n>` | Directory accepted; `n` = chapters accepted by production directory filtering/manifest path for the exercised fixture. |
| `error:<class-or-code>:<detail>` | Diagnosable failure; process must not crash. |
| `refused:not-debug` | Runtime/build guard refused execution outside debug. |
| `error:missing-*` / `error:unsupported-*` / `error:no-chapters` / `error:native-not-loaded` / `error:busy:*` (when applicable) | Specific validation/runtime failures. |

**Interpretation rules**:

- `ok:*` proves only the exercised **programmatic non-UI** ingest path for the supplied fixture/reference (`scope_class=programmatic-non-ui`).
- `chapters=<n>` is defined as chapters accepted by production filtering/manifest behavior. Evidence/report MUST state fixture composition or expected filtering when claiming directory pass; do not assume all directory entries become chapters.
- `blocked` = precondition/tooling prevents explicit result (e.g. `blocked:fixture-inaccessible`); reported separately, excluded from SC-002 denominator. `inconclusive` = ambiguous/conflicting/missing authoritative evidence; not explicit until rerun resolves.
- `crash_anr_detected=unknown` is not evidence of absence and MUST NOT be treated as pass.
- Repeated invocations MUST produce attributable evidence (correlation key or isolated outputs). Concurrent/re-entrant triggers serialize if safe; otherwise reject with explicit `error:busy:<detail>`; never silently drop.
- `refused:not-debug` on a release/non-debug build is supporting runtime evidence, but FR-010 requires combined static artifact absence + runtime negative invocation + repeatable check.

## Guardrails

- `BuildConfig.DEBUG` guard in `DebugIngestActivity` and `DebugIngestHook` is mandatory defense-in-depth.
- Exported debug component must exist only in debug-variant manifest materialization.
- Native library load/cold-start readiness failures must fail gracefully (`error:native-not-loaded` or equivalent explicit diagnostic), not crash.
- Fixture accessibility precondition (SC-001): fixture must be app-accessible to the debug/test process. Existence outside the app sandbox does not imply readability; `/sdcard` paths may fail unless accessible to the app. Inaccessible fixtures produce explicit diagnosable/blocked results, never silent pass.

## Evidence Minimum (FR-008)

**Authoritative (spec.md §FR-008).** Every reported run includes: fixture/reference, operation, explicit parseable outcome/status, build+device identity, diagnostic capture, crash/ANR status/signal, residual scope note, and evidence disposition (preferred path/channel, effective path/channel, write status `written|unavailable|failed:<reason>`, reviewer-accessible location, retention scope tied to covered `EV/RES/TASK/FR/SC` IDs/policy; not required in ignored dirs). Hashes/timestamps required when artifact/output exists; if unavailable record `unavailable:<reason>`. Reported evidence also requires correlation key/run identity, `scope_class`, `channel/corroboration`, run vs script outcome, and supporting IDs. Contracts/data-model may add required fields but MUST NOT weaken FR-008.

**FR-008 source of truth**: `spec.md` §FR-008 (normative minimum). This contract and `data-model.md` MAY add required fields but MUST NOT weaken FR-008; when FR-008 changes, a synchronized diff/update is required (NFR-012-MAINT-005).

Preferred-output failure remains attributable: if corroborated by an authorized channel, report explicit `channel/corroboration`; if not corroborated, report `inconclusive`/`blocked` per contract — never silent pass. SC-006 hook-capability durability is separate from historical evidence retention.

Evidence-sufficiency dependencies (CHK030-evidence): `DEP-012-EVIDENCE-CHANNELS` (ADB/logcat/channels), `ASM-012-MANUAL-A11Y` (manual a11y/device/system controls outside hook), and `ASM-012-CHAPTERS-INTERP` (`chapters=<n>` vs production filtering + fixture composition/expected filtering) MUST be explicit; missing sufficiency blocks the affected claim.


## Host-deterministic contract modes (no adb/device side effects)

`scripts/automate-t184-t226-hook.sh` provides host-deterministic, machine-readable contract modes for TDD/CI/contract validation without adb/device execution. They MUST NOT weaken existing modes or FR-008/FR-010/FR-012 semantics.

| Mode | Purpose | Key fields/notes |
|---|---|---|
| `--print-contract`, `--classify-outcome`, `--print-report-schema`, `--print-residual-rules` | Core hook/report/residual contracts | Existing; preserved. |
| `--print-activity-channel-contract`, `--classify-channel-attribution` | FR-006/FR-008 cache/channel attribution | Unwritten/unavailable authoritative cache must not imply success; `write_status`/corroboration explicit; `crash_anr_status=unknown` never pass; `full_parent_completion_claimed=false`. |
| `--print-intent-parse-contract`, `--normalize-debug-ingest-intent` | T013/T020/I1 host-contract fallback (FR-002/FR-003/FR-007) | Validates debug-only parsing/normalization/validation semantics (aliases, U1 precedence, missing-input errors). **Not** a parallel production ingest path; Kotlin/Rust/JNI remain production authority (ASM-012-INGEST-SOT). No CHK021/`src-tauri/gen/**` mutation. |
| `--print-directory-uri-contract` | Directory URI/ref semantics (FR-002/FR-007) | Non-filesystem refs must route through production-equivalent directory-open/manifest-dispatch when supported or return explicit diagnosable `error:*`; never silent fallback/fake/ambiguous success. |
| `--build-device-evidence` | FR-008 per-op evidence object builder (host-deterministic) | Emits FR-008 minimum fields, `programmatic-non-ui`, `full_parent_completion_claimed=false`, SC-001/SC-002 counter hooks, `crash_anr_status unknown|detected` non-pass semantics. |
| `--print-device-run-contract`, `--classify-device-run [--result-file ...]` | T018/T024/T067 normal device-run evidence wiring contract | Classifies/wires per-run evidence shape (blocked:fixture-inaccessible/busy/cold-start, inconclusive, explicit non-pass cases) for normal adb/device execution support without weakening host contracts. Runtime adb execution remains opt-in/non-attesting unless complete bound evidence is captured. |

**Integrity notes (CHK021/T042/T028):** host-contract fallback modes (intent/dir-uri parse/normalize, device evidence builders/classifiers) do not authorize `src-tauri/gen/**` patches, generated Gradle/Tauri mutation, or parallel production ingest routes. T042 release mechanization requires fail-closed D1–D4/full identity binding; `static`/`runtime`/`blocked-not-evaluated` never satisfy FR-010/SC-004. T028 FR-008 report gaps are closed only by attributable machine-readable evidence in executed runs with SC denominators/formulae/correlation/build identity/retention as applicable.

## FR-011 Overlay Format & Ownership (CHK022/CHK024)

Normative durability/apply/check semantics are defined in [`durable-overlay-materialization.md`](./durable-overlay-materialization.md). Summary (must stay synchronized): canonical overlay root `specs/012-debug-ingest-hook/android-overlays/app/src/` declares/detects a format version (implementation-defined marker; normative requirement). `apply/check` report detected version. Backward-compatible materialization preferred; incompatible evolution requires migration note, synchronized updates to canonical targets + checks + contracts/docs + future tasks, and evidence. Missing/unknown version => `v1-implicit` only if explicitly documented by check contract, else `fail`/`blocked`. Hook-surface changes (`component`, action, canonical/target paths, result grammar/channels, run points, residual IDs) require synchronized updates and CHK001/CHK002 reviewer handoff.
