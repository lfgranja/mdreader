# Release Exposure Requirements Quality Checklist: Debug-Only Ingest Validation Hook

**Purpose**: Requirements quality review focused on FR-010, SC-004, and Clarification Q1 (release/non-debug exposure evidence). Evaluates completeness, clarity, consistency, measurability, coverage, edge cases, NFRs, dependencies/assumptions, and ambiguity/conflict for release-exposure prevention.
**Created**: 2026-11-06
**Feature**: [../spec.md](../spec.md)
**Note**: This is a requirements-quality checklist (not implementation verification). No items ask to "verify/test/confirm behavior". Every item must be reviewed against the requirements/docs, not execution results.
**Review Ownership**: Reviewer-owned requirements-quality artifact. Mark `[x]` only when the criterion is satisfied for requirements quality.
**Marker Semantics**: `[x]` means reviewed and satisfied for requirements quality; does **not** mean implementation complete.

## D1 - Debug-Only Scope & Guards

- [x] CHK001 Is FR-010's requirement for "debug/test-only placement and runtime refusal/inertness outside debug" stated with sufficient specificity to distinguish placement from guards (defense-in-depth)? [Spec §FR-010] [Contract release-exposure §D1] [Data Model §5]
- [x] CHK002 Are the specific debug-only artifacts (e.g., `DebugIngestActivity`, `DebugIngestHook`, action `app.mdreader.action.DEBUG_INGEST`) explicitly enumerated as the items subject to D1 scope (not implied generally)? [Spec §FR-001,FR-002,FR-010] [Research Decision 1]
- [x] CHK003 Is it unambiguously specified that source/guard checks alone are *necessary but not sufficient* for release exposure claims? [Spec §FR-010] [Research Decision 1] [Contract release-exposure §D1,§Class Presence Caveat]
- [x] CHK004 Are refusal/inertness behaviors outside debug specified (cannot resolve/start, no success path) rather than only "unavailable"? [Spec §US1-AC3, §FR-010] [Contract release-exposure §D3]

## D2 - Static Release Artifact Absence (Manifest/Merger/APK/AAB)

- [x] CHK005 Does FR-010 require inspection of the *actual* release/non-debug packaged/merged artifact (not just source) with explicit artifact identity binding? [Spec §FR-010] [Data Model §5] [Contract release-exposure §Artifact Identity Binding]
- [x] CHK006 Are all external-exposure surfaces enumerated: component, action/intent-filter, external invocation identifier(s), and unintended external exposure? [Spec §FR-010] [Contract release-exposure §D2]
- [x] CHK007 Is inspection of merged manifest and merger report (when available) required, and is the precedence/role of each stated? [Research Decision 1] [Contract release-exposure §D2, §Command Surfaces]
- [x] CHK008 Is the risk of "wrong artifact/variant" explicitly required to be controlled (variant/build type recorded; evidence bound to specific APK/AAB)? [Spec §FR-010] [Data Model §5] [Contract release-exposure §Artifact Identity Binding]
- [x] CHK009 Does the requirement clearly address that hook code may be compiled into all variants via `android-shell` main sourceSet (shared sourceSet) and therefore class/dex presence must *not* be treated as proof of exposure? [Research Decision 1,2] [Contract release-exposure §Class Presence Caveat] [Data Model §5 d4/class_presence_note]
- [x] CHK010 Are failure conditions stated (debug component/action found in release artifact/merge => FAIL) as requirements, not just guidance? [Contract release-exposure §Pass/Fail Rules]

## D3 - Runtime Negative Invocation on Installed Release/Non-Debug

- [x] CHK011 Is a runtime negative invocation check on an *installed* release/non-debug build required as a distinct dimension (D3)? [Spec §FR-010] [Contract release-exposure §D3]
- [x] CHK012 Are the required negative outcomes specified with precision: cannot resolve/start hook; no `ok:document`/`ok:directory`/`ok:*` success; no crash/ANR caused by the negative check? [Spec §FR-010,FR-007] [Contract release-exposure §D3,§Pass/Fail Rules]
- [x] CHK013 Is install identity (package, version/signing/build metadata as available, device serial) required to bind runtime evidence? [Data Model §5] [Contract release-exposure §Artifact Identity Binding]
- [x] CHK014 Are device/install provenance issues (wrong install, debug build installed instead of release) addressed as a requirement for unambiguous interpretation of negative results? [Spec §Assumptions] [Research Decision 1] [Gap]

## D4 - Repeatable Fail-Closed Regression Evidence

- [x] CHK015 Is D4 required to produce a repeatable, scriptable regression check capturing results bound to artifact/install identity? [Spec §FR-010] [Data Model §5] [Contract release-exposure §D4]
- [x] CHK016 Are fail-closed rules specified: missing required inputs for requested mode => FAIL; never reuse stale evidence without identity match? [Contract release-exposure §Pass/Fail Rules]
- [x] CHK017 Is there a requirement to record which tool/path produced evidence (e.g., `apkanalyzer`/`aapt2`/`aapt`/merged-manifest reports) when tools vary by environment? [Research Decision 1] [Gap]
- [x] CHK018 Are evidence records required to be persisted with hashes/provenance (APK/AAB/manifest/merger report hashes) for auditability? [Data Model §4,§5] [Contract release-exposure §Artifact Identity Binding]

## Mode Boundaries (`static` / `runtime` / `full`) & Claim Limits

- [x] CHK019 Is the boundary explicit: `mode=static` reports static status only and MUST NOT claim full FR-010 pass without D3? [Contract release-exposure §Pass/Fail Rules]
- [x] CHK020 Is the boundary explicit: `mode=runtime` reports negative invocation only and MUST NOT claim full FR-010 pass without D2 artifact identity? [Contract release-exposure §Pass/Fail Rules]
- [x] CHK021 Is the boundary explicit: `mode=full` is the only mode that may claim FR-010 pass, and requires D1–D4 all passing? [Spec §FR-010] [Data Model §5] [Contract release-exposure §Pass/Fail Rules]
- [x] CHK022 Is "zero externally invocable automation-entry exposures" stated as a measurable claim with its supporting dimensions (SC-004) rather than an unqualified assertion? [Spec §SC-004] [Contract release-exposure §Purpose,§Pass/Fail Rules]

## Shared SourceSet & Architectural Caveats

- [x] CHK023 Is the shared `android-shell` main sourceSet inclusion (hook code compiled in all variants) stated as a known risk requiring component/export-based proof? [Research Decision 1,2] [Contract release-exposure §Class Presence Caveat]
- [x] CHK024 Is the requirement to treat class-presence-not-sufficient as normative (to be recorded in evidence, e.g., `class_presence_note`) rather than optional? [Data Model §5] [Contract release-exposure §Class Presence Caveat]

## Missing-Input Fail-Closed Rules & Integration Boundaries

- [x] CHK025 Are missing-input fail-closed rules complete across dimensions (artifact missing, merged manifest/merger report missing when required, device not installed with target build, required tools unavailable)? [Contract release-exposure §Pass/Fail Rules] [Research Open Notes]
- [x] CHK026 Is the integration boundary with gate 9 specified (release/device-aware checks outside `scripts/verify.sh`, opt-in/RC checks, not silent deps of host gates 1–8)? [Contract release-exposure §Integration]
- [x] CHK027 Are durable scaffolding implications for release checks considered (if canonical overlays regenerated/missing, release-evidence process behavior must not rely on non-durable `src-tauri/gen/**` files)? [Spec §FR-011] [Research Decision 2] [Gap]

## Measurability of "Zero Externally Invocable Exposures" & Risk Acceptance

- [x] CHK028 Is SC-004's "zero externally invocable automation-entry exposures" measurable with explicit pass/fail criteria tied to D1–D4 (no ambiguous pass states)? [Spec §SC-004] [Contract release-exposure §Pass/Fail Rules]
- [x] CHK029 Are negative test cases specified (e.g., debug component merged into release, action exported in release manifest, runtime invocation succeeds on release build) with required outcomes (all must FAIL the release-exposure check)? [Contract release-exposure §Pass/Fail Rules] [Spec §Edge Cases]
- [x] CHK030 If evidence is impossible in an environment (missing tools/artifact/device), is the requirement to fail closed or to require explicit risk acceptance/exception documented with traceable approval (not silent pass)? [Contract release-exposure §Pass/Fail Rules] [Data Model §6 (residual/approval)] [Gap]
- [x] CHK031 Are residual/exceptions (if any) required to be tracked with owner/decision reference and not used to satisfy FR-010/SC-004 automatically? [Data Model §6] [Spec §FR-012] [Gap]

## Traceability & References

- [x] CHK032 Are all D1–D4 dimensions traceable to FR-010 and SC-004 with explicit references? [Spec §FR-010,§SC-004] [Contract release-exposure §Required Evidence Dimensions]
- [x] CHK033 Do requirements distinguish between "component/action/exported intent-filter/external invocation identifiers" and generic "class presence" consistently across spec/research/contract/data model? [Research Decision 1] [Contract release-exposure §Class Presence Caveat] [Data Model §5]
- [x] CHK034 Are authoritative references sufficient and verifiable (Android Test/debug risks, manifest merge/build variants, apkanalyzer/aapt2, dumpsys/adb, OWASP MASVS/MASTG for debug/export/IPC) without relying on un-cited assumptions? [Research Decision 1] [Contract release-exposure] [References below]

## References

- Android — Test and debug features security risk: [developer.android.com/privacy-and-security/risks/test-debug](https://developer.android.com/privacy-and-security/risks/test-debug)
- Android — Manage manifests / Manifest merger: [developer.android.com/build/manage-manifests](https://developer.android.com/build/manage-manifests)
- Android — Configure build variants: [developer.android.com/build/build-variants](https://developer.android.com/build/build-variants)
- Android — `apkanalyzer`: [developer.android.com/tools/apkanalyzer](https://developer.android.com/tools/apkanalyzer)
- Android — `aapt2`: [developer.android.com/tools/aapt2](https://developer.android.com/tools/aapt2)
- Android — `adb`: [developer.android.com/tools/adb](https://developer.android.com/tools/adb)
- Android — `dumpsys`: [developer.android.com/tools/dumpsys](https://developer.android.com/tools/dumpsys)
- OWASP MASVS / MASTG (Platform, IPC/Export, Build/Debug): [mas.owasp.org/MASVS](https://mas.owasp.org/MASVS/) / [mas.owasp.org/MASTG](https://mas.owasp.org/MASTG/)
- Tauri v2 CLI/Config/Develop (generated project lifecycle): [v2.tauri.app/reference/cli](https://v2.tauri.app/reference/cli/), [v2.tauri.app/reference/config](https://v2.tauri.app/reference/config/), [v2.tauri.app/develop](https://v2.tauri.app/develop/)
- ISTQB (residual risk/exit criteria glossary): [istqb.org](https://istqb.org/) / [glossary.istqb.org](https://glossary.istqb.org/)
- W3C WCAG/WCAG-EM (partial evaluation discipline): [w3.org/WAI/test-evaluate](https://www.w3.org/WAI/test-evaluate/)

**Notes on markers:** `[Gap]` indicates a requirement-quality gap (needs clarification/correction in requirements, not implementation TODO). `[Ambiguity]`, `[Conflict]`, `[Assumption]`, `[INFERENCE]` used per instructions where authoritative guarantees are not fully specified in repo docs.
