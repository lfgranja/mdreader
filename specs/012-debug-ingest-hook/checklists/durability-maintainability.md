# Durability & Maintainability Requirements Quality Checklist: Debug-Only Ingest Validation Hook

**Purpose**: Validate that requirements for the debug-only Android ingest validation hook are complete, clear, consistent, measurable, and durable across regeneration, gitignore/source-set boundaries, and ownership changes—focusing on requirement quality, not implementation behavior.
**Created**: 2026-11-06
**Feature**: [spec.md](../spec.md) · [plan.md](../plan.md) · [research.md](../research.md) · [quickstart.md](../quickstart.md)
**Depth**: Standard (author self-review / peer PR)
**Audience**: Spec author and Android/shell maintainers

**Note**: This checklist evaluates the *quality of written requirements* for Spec 012. It does not verify runtime behavior, execute commands, inspect UI, or assert that the hook “works correctly/displays”. Mark items `[x]` when the requirement text is complete, unambiguous, measurable, and traceable; add comments/links for gaps.

## Review Ownership

- [x] CHK001 — Primary reviewer(s) for durability/maintainability requirements are explicitly identified (e.g., Spec 012 author + Android/shell maintainer) with review handoff criteria stated. [Spec §FR-011, Plan §Governance §Review ownership & handoff, Gap]
- [x] CHK002 — Requirement text distinguishes which checklist findings block merge/readiness (durability, fail-closed, release-exposure) versus advisory maintainability improvements. [Spec §FR-011, SC-006, Plan §Governance §Finding classification, Gap]

## Marker Semantics

Use the same marker vocabulary as the checklist template: `[ ]` unchecked, `[x]` satisfied by requirement text, `[Gap]` missing requirement, `[Ambiguity]` unclear requirement, `[Conflict]` contradictory requirement, `[Assumption]` unstated dependency/behavior, `[INFERENCE]` claim not fully guaranteed by authoritative documentation.

## Canonical vs Materialized Locations

- [x] CHK003 — Are requirements explicit that canonical hook sources live outside `src-tauri/gen/**` and that `gen` is a materialized/regenerable output, not the source of truth? [Spec §FR-011, Clarifications Q2, Plan §Project Structure, Data Model §canonical source location]
- [x] CHK004 — Is the exact canonical overlay path (or path-selection rule) specified unambiguously, including which repository root it is relative to and which files/directories it contains? [Spec §FR-011, Plan §Project Structure, Ambiguity]
- [x] CHK005 — Are requirements clear that the same logical overlay targets the Android project’s managed `src/debug` and optional `src/androidTest` locations, with each target’s purpose and required contents defined? [Spec §FR-011, FR-013, Plan §Project Structure, Contract residual-tracking §managed targets]
- [x] CHK006 — Are gitignore and “mostly gitignored/regenerable `gen`” constraints translated into requirement language that forbids relying on ignored generated files as the only durable copy? [Spec §FR-011, Assumption, Gap]
- [x] CHK007 — Are requirements for materialized/generated copies bounded to Tauri/Android project integration needs, preventing unnecessary duplication of hook logic across canonical and generated trees? [Spec §FR-011, Plan §Constitution Check III, Gap]

## Managed Overlays, Source Sets, and Existing Kotlin Path

- [x] CHK008 — Do requirements consistently align debug-hook placement with Android Gradle variant/source-set semantics (`src/debug`, `src/androidTest`) rather than ad hoc locations? [Spec §FR-011, FR-013, Plan §Technical Context, Contract debug-ingest §build variants]
- [x] CHK009 — Are requirements explicit that hook Kotlin sources remain consistent with the existing canonical `android-shell/src/main/kotlin` policy where shared logic belongs, avoiding a second production-like convention? [Spec §FR-005, Plan §Constitution Check II, Gap]
- [x] CHK010 — Is the boundary between debug-only overlay code and shared shell/transport code specified so maintainers can determine where a future change must be made? [Spec §FR-002, FR-011, Data Model §Hook Contract, Gap]
- [x] CHK011 — Are manifest/resource/component registration requirements for the debug validation activity specified at the requirement level (name, intent/action exposure scope, debug source set) without prescribing broad generated Gradle/Tauri patching? [Spec §FR-001, FR-011, Plan §Project Structure, Contract debug-ingest §activity/manifest]
- [x] CHK012 — If `src/androidTest` scaffolding is optional (FR-013), are requirements clear about when it must exist, what minimum scaffold is required, and how absence is treated relative to durability/readiness? [Spec §FR-013, SC-006, Ambiguity]

## Apply/Check Semantics and Fail-Closed Conditions

- [x] CHK013 — Are idempotent `apply` semantics defined with measurable outcomes (no-op on aligned state, deterministic changes on drift, stable result across repeated runs)? [Spec §FR-011, SC-006, Research Decision 4, Contract residual-tracking §apply/check]
- [x] CHK014 — Are `check` semantics defined as non-mutating/read-only in effect, with explicit pass/fail/stale/missing outputs that can be consumed by humans and automation? [Spec §SC-006, Research Decision 4, Gap]
- [x] CHK015 — Are fail-closed conditions enumerated: missing canonical source, missing materialized target, stale content, forbidden placement in main/release, unexpected debug identifiers outside allowed locations, and unreadable/partial materialization? [Spec §FR-011, FR-010, SC-006, Plan §Risk, Contract residual-tracking §fail-closed]
- [x] CHK016 — For each fail-closed condition, are requirements clear about what evidence/message is emitted (path, expected vs actual, remediation hint) without exposing sensitive runtime data? [Spec §SC-006, Gap]
- [x] CHK017 — Are success criteria defined so “no silent missing scaffolding” is objectively assessable (explicit status, exit/status contract, or report artifact) rather than implied? [Spec §SC-006, Measurability, Gap]

## Regeneration Triggers, Tauri Lifecycle, and [INFERENCE] Gaps

- [x] CHK018 — Are regeneration/refresh triggers specified (Tauri Android `init`/regeneration, clean build artifacts, CI bootstrap, local dev setup) with required `apply/check` run points for each? [Spec §FR-011, Spec §Dependencies, Quickstart §FR-011, Plan §Governance §FR-011 regeneration/apply-check run points, Gap]
- [x] CHK019 — Is uncertainty about Tauri v2 Android `gen` overwrite behavior explicitly recorded as `[INFERENCE]`/assumption in requirements, with fail-closed mitigation rather than undocumented reliance? [Research Decision 4, Plan §Technical Context, Spec §Dependencies, INFERENCE]
- [x] CHK020 — Are requirements explicit that apply/check must be safe to run before build and after regeneration, and that generated Gradle/Tauri files are not broadly patched except narrowly justified, documented exceptions? [Spec §FR-011, Plan §Project Structure, Ambiguity]
- [x] CHK021 — If narrow patching of generated build/config files is ever required, are requirements defined for scope, review/ownership, idempotence, and re-application after regeneration? [Spec §FR-011, Plan §Governance §Narrow generated-file patching, Gap]

## Ownership, Change Propagation, and Maintainability

- [x] CHK022 — Are ownership/responsibility requirements stated for maintaining canonical overlays, apply/check scripts, contract docs, and T184/T226 residual reporting when the hook surface changes? [Spec §FR-011, Spec §FR-012, Plan §Governance §Review ownership & handoff, Plan §Governance §Overlay format evolution & artifact ownership, Contract debug-ingest §FR-011 Overlay Format & Ownership, Gap]
- [x] CHK023 — Are change-propagation requirements defined so updates to component name, intent action, manifest placement, or canonical paths require synchronized updates to contracts, checks, and evidence documentation? [Spec §FR-001, FR-011, FR-012, Contract debug-ingest §change propagation, Consistency, Gap]
- [x] CHK024 — Are requirements for versioning/evolution of the durable overlay format specified (backward-compatible materialization, migration notes, or explicit incompatibility handling)? [Spec §FR-011, Plan §Governance §Overlay format evolution & artifact ownership, Contract debug-ingest §FR-011 Overlay Format & Ownership, Gap]
- [x] CHK025 — Are non-functional maintainability requirements stated (small blast radius, avoid duplicate authority, readable diagnostics, script portability in repo toolchain) and traceable to constitution/plan constraints? [Plan §Constitution Check, Spec §FR-011, Spec §Maintainability & Consistency, Plan §Governance §Maintainability NFRs & cross-artifact consistency, Gap]

## Dependencies, Assumptions, and SC-006 Measurability

- [x] CHK026 — Are external dependencies required for durability (Tauri CLI/Android project generation, Gradle, repo scripts, git checkout state) documented with required/optional status and fallback behavior? [Spec §Dependencies, Plan §Technical Context, Spec §Preconditions & Traceability, Plan §Governance §Durability dependencies and layout validation, Gap]
- [x] CHK027 — Are assumptions about repository layout, gitignore exceptions, and existing `android-shell`/`build.gradle.kts` source-set wiring validated or explicitly bounded to prevent silent requirement drift? [Spec §Assumptions, Research Decision 2, Spec §Preconditions & Traceability, Plan §Governance §Durability dependencies and layout validation, Assumption]
- [x] CHK028 — Is SC-006 measurable end-to-end (“after a generated-project refresh, maintainers can restore or verify required debug-only automation artifacts…” with 100% pass/fail visibility) with defined refresh scope, required artifacts list, and evidence of visibility? [Spec §SC-006, FR-011, Measurability]
- [x] CHK029 — Do requirements distinguish durability of *hook capability* from durability of *historical evidence artifacts* (reports under ignored/output dirs) to avoid conflating SC-006 with evidence retention? [Spec §SC-006, FR-008, Gap]
- [x] CHK030 — Are consistency checks required across `spec.md`, `plan.md`, `research.md`, `quickstart.md`, contracts, and future tasks so FR-011/SC-006 semantics cannot diverge during implementation? [Spec §FR-011, FR-012, Spec §Maintainability & Consistency, Plan §Governance §Maintainability NFRs & cross-artifact consistency, Consistency, Traceability]

## References

Authoritative sources consulted (verify dates/wording at implementation time; do not fabricate local policy from these links):

- Android Developers — *Configure build variants*: https://developer.android.com/build/build-variants (source sets including `debug`/`androidTest`, variant-specific manifests/resources). [en]
- Android Developers — *Merge multiple manifest files*: https://developer.android.com/build/manage-manifests (manifest merge behavior and variant/source-set precedence). [en]
- Android Developers — *Configure your build* / Gradle Android Plugin behavior for source sets and tasks: https://developer.android.com/build and https://developer.android.com/build/dependencies (project/build configuration boundaries). [en]
- Android Developers — *Test in Android Studio* / instrumented tests (`androidTest`): https://developer.android.com/training/testing/instrumented-tests (canonical `androidTest` placement context). [en]
- Tauri v2 — *Develop* / project structure & mobile workflow: https://v2.tauri.app/develop/ and https://v2.tauri.app/start/prerequisites/ (generated mobile integration context). [en]
- Tauri v2 — *CLI reference* (`android init`, `android dev/build` behavior context): https://v2.tauri.app/reference/cli/ . [en]
- Tauri v2 — *Configuration reference*: https://v2.tauri.app/reference/config/ . [en]
- Tauri v2 localized docs (availability varies; use as secondary confirmation of concepts, not overwrite guarantees): https://v2.tauri.app/ja/ , https://v2.tauri.app/zh-cn/ , https://v2.tauri.app/es/ , https://v2.tauri.app/pt/ . [ja/zh/es/pt, as available]
- Android Developers localized build-variant docs (concept confirmation): https://developer.android.com/build/build-variants?hl=ja , https://developer.android.com/build/build-variants?hl=de , https://developer.android.com/build/build-variants?hl=es-419 , https://developer.android.com/build/build-variants?hl=pt-br . [ja/de/es/pt]

**[INFERENCE]**: Tauri v2 public docs describe Android/mobile workflows and CLI commands but do not provide a complete, file-by-file guarantee about which `src-tauri/gen/android` outputs are preserved vs overwritten across every `android init`/regeneration scenario. Requirements therefore treat overwrite behavior as uncertain and rely on idempotent, fail-closed apply/check materialization of canonical sources.
