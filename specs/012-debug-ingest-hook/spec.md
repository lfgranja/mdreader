# Feature Specification: Debug-Only Ingest Validation Hook

**Feature Branch**: `[012-debug-ingest-hook]`

**Created**: 2026-11-05

**Status**: Draft

**Input**: User description: "Add a debug-only/test hook (intent/exported test activity or command protected by debug/build flag) that accepts a URI/path and routes through the same document-ingest/directory-opening production flow, plus instrumented/Espresso tests if desired. Physical device paired for validation: `adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp`." Assessment handoff: `.specify/assessments/debug-ingest-hook-automation/decision.md` (verdict: go, Option A).

## Clarifications

### Session 2026-11-05

- Q: What evidence must prove release/non-debug artifacts do not expose the automation entry? (FR-010) → A: A combination of evidence is required: (1) the automation entry is scoped to debug/test only by build-variant placement and refuses/is inert outside debug; (2) the actual release/non-debug artifact is statically inspected for absence of the automation component, its external invocation identifiers, and unintended external exposure; (3) a runtime negative invocation check on an installed release/non-debug build confirms the entry cannot be invoked; (4) a repeatable project check captures these results as regression evidence for future changes.
- Q: Where must debug-only automation artifacts live durably given generated Android project files under `src-tauri/gen/` may be refreshed? (FR-011) → A: Use a hybrid durable strategy: canonical automation implementation and test/scaffold sources live in maintained, version-controlled project locations outside ignored generated output; any generated-project wiring or debug artifact placement required by the Android build is materialized by an idempotent apply/check process that fails closed when missing or stale after regeneration.
- Q: When programmatic ingest passes but native SAF picker cancel and TalkBack/keyboard traversal remain unverified, how should T184/T226 be tracked? (FR-012) → A: Split tracking with annotated residuals. Record programmatic ingest-path coverage as satisfied only for the exercised non-UI scope with explicit evidence links; keep native SAF picker cancel and TalkBack/keyboard traversal as separate residual tracked items (or explicitly open sub-items) until verified or formally accepted as out-of-scope risk. Do not mark T184/T226 fully complete based on debug-hook evidence alone.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Deterministic document ingest validation (Priority: P1)

As a maintainer or QA engineer, I can trigger document ingestion from an automation entry available only in debug/test builds so that I can verify the real document-open ingest path on a device or emulator without manually using the system document picker.

**Why this priority**: Document ingest is the core path required for Spec 009 T184 programmatic validation and is the smallest independently valuable slice.

**Independent Test**: Can be fully tested by launching the debug-only automation entry with an accessible document fixture reference and observing an explicit success/failure result plus evidence that the production ingest path completed without crash/ANR indicators.

**Acceptance Scenarios**:

1. **Given** a debug/test build with a valid, accessible document fixture and no human picker interaction, **When** an authorized automation command invokes document ingest with that fixture reference, **Then** the system routes the request through the same production document-ingest/open flow and reports an explicit success result.
2. **Given** a debug/test build and an invalid, missing, unreadable, or unsupported document reference, **When** the automation command invokes document ingest, **Then** the system reports an explicit, diagnosable failure result and does not crash.
3. **Given** a release/non-debug build, **When** the same automation entry is invoked or discovered, **Then** the automation capability is unavailable or refuses execution and is not exposed as a production feature.

---

### User Story 2 - Deterministic directory ingest validation (Priority: P1)

As a maintainer or QA engineer, I can trigger directory ingestion from the same debug-only automation boundary so that I can verify directory-reading/open behavior on a device or emulator without manually selecting a directory in the system picker.

**Why this priority**: Directory ingest is required for Spec 010 T226 and must exercise the same production directory-selection/manifest dispatch behavior.

**Independent Test**: Can be fully tested by launching the debug-only automation entry with an accessible directory fixture reference and observing an explicit result that includes chapter/entry handling evidence and no crash/ANR indicators.

**Acceptance Scenarios**:

1. **Given** a debug/test build with an accessible directory fixture containing supported and unsupported/invalid entries, **When** an authorized automation command invokes directory ingest, **Then** the system routes the request through the same production directory-open flow and reports an explicit result including accepted chapter/entry handling evidence.
2. **Given** a debug/test build and an empty, missing, or unreadable directory reference, **When** the automation command invokes directory ingest, **Then** the system reports an explicit, diagnosable failure result and does not crash.

---

### User Story 3 - Trustworthy evidence and residual-scope reporting (Priority: P2)

As a project maintainer, I can review automation output and validation notes that clearly separate programmatic ingest evidence from residual system-picker UI/a11y cases so that task completion and release readiness decisions are not overstated.

**Why this priority**: The hook is valuable only if its evidence is trusted and its limits are explicit; otherwise it can create false confidence.

**Independent Test**: Can be fully tested by running the document and directory automation flows and confirming the produced evidence states pass/fail, target fixture, captured diagnostic signals, and any residual UI-only scenarios that were not covered.

**Acceptance Scenarios**:

1. **Given** completed document and directory automation runs, **When** a maintainer reviews the evidence, **Then** each run reports the fixture/reference exercised, explicit outcome, and relevant diagnostic signals (including crash/ANR absence or presence).
2. **Given** automation evidence for programmatic ingest paths, **When** validation status is communicated for T184/T226, **Then** the communication explicitly identifies residual native system-picker UI/a11y cases not covered by the automation seam, links evidence for the covered programmatic scope, and tracks uncovered residual items separately rather than marking full task completion.

---

### User Story 4 - Optional instrumented test coverage (Priority: P3)

As a maintainer, I can add instrumented test coverage around the debug-only automation seam so that regressions in the validation path are detected in Android test workflows where practical.

**Why this priority**: Useful for long-term regression safety, but lower priority than establishing a trustworthy debug-only seam and evidence boundary.

**Independent Test**: Can be fully tested by running the relevant instrumented test target in a debug/test environment and confirming it exercises the automation seam without requiring manual system-picker interaction.

**Acceptance Scenarios**:

1. **Given** a debug/test environment and instrumented test support, **When** the relevant automation-seam tests run, **Then** they verify the debug-only entry behavior and report pass/fail without manual picker interaction.

### Edge Cases

- What happens when the fixture reference is a local path that is not readable by the app process, even if the file exists elsewhere on the device?
- What happens when a directory contains a mix of supported, empty, oversized, binary, or invalid-name entries?
- How does the system handle repeated invocations, concurrent/re-entrant triggers, or invocation while the app process is not already initialized?
- What happens when a caller supplies a URI/reference form that is accepted by the automation entry but cannot be resolved by the production ingest path?
- What happens when diagnostic evidence cannot be written to the preferred output location, but the ingest result is known?
- How is a pass result interpreted when residual system-picker UI behavior (cancel, focus, TalkBack traversal) remains untested?
- What happens when generated Android project files are refreshed and required debug-only automation wiring/scaffolding has not been reapplied from canonical sources?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an automation entry for document-ingest validation that is available only in debug/test contexts and is not available as a production capability.
- **FR-002**: The system MUST provide an automation entry for directory-ingest validation that is available only in debug/test contexts and is not available as a production capability.
- **FR-003**: Each automation entry MUST accept a caller-supplied fixture reference (path or URI form, as supported by the chosen debug/test boundary) and validate required input before attempting ingestion.
- **FR-004**: Document-ingest automation MUST route through the same production document-ingest/open flow used by normal app document selection, rather than a separate fake or test-only ingest path.
- **FR-005**: Directory-ingest automation MUST route through the same production directory-open/manifest-dispatch flow used by normal app directory selection, rather than a separate fake or test-only directory path.
- **FR-006**: The system MUST return an explicit, machine-readable or script-parseable outcome for each automation run, including success/failure status and enough diagnostic detail to investigate failures.
- **FR-007**: The system MUST handle invalid, missing, unreadable, unsupported, or unresolvable fixture references without crashing the app process.
- **FR-008**: Automation runs MUST produce evidence suitable for device/emulator validation, including at minimum the exercised fixture reference, outcome, and crash/ANR signal status.
- **FR-009**: The project MUST document the boundary between programmatic ingest evidence and residual native system-picker UI/a11y cases (including picker cancel and keyboard/TalkBack traversal of native picker/title UI) so these cases are not reported as covered by the debug-only seam.
- **FR-010**: Release/non-debug artifacts MUST NOT expose the automation entry as an externally invocable or user-facing production feature. The project MUST record combined evidence that: debug/test-only placement and runtime refusal make the entry unavailable outside debug; the actual release/non-debug artifact shows no automation component, external invocation identifier, or unintended external exposure; a runtime negative invocation check on an installed release/non-debug build cannot invoke the entry; and a repeatable project check captures these results as regression evidence.
- **FR-011**: Debug-only automation artifacts required for maintenance MUST have a durable project location/process so routine project generation or build refresh does not silently remove required validation capability. Canonical automation implementation and test/scaffold sources MUST live in maintained, version-controlled project locations outside ignored generated output; any generated-project wiring or debug artifact placement required by the build MUST be materialized by an idempotent apply/check process that fails closed when missing or stale after regeneration.
- **FR-012**: Validation reporting for Spec 009 T184 and Spec 010 T226 MUST distinguish programmatic ingest-path coverage from residual UI-only coverage. When programmatic ingest passes but native SAF picker cancel and TalkBack/keyboard traversal remain unverified, the project MUST record the programmatic scope as satisfied only for the exercised non-UI path with explicit evidence links, keep residual picker UI/a11y cases as separate tracked items or explicitly open sub-items, and MUST NOT mark T184/T226 fully complete based on debug-hook evidence alone.
- **FR-013**: The project SHOULD provide instrumented test coverage for the automation seam where practical, without requiring manual system-picker interaction.

### Key Entities *(include if feature involves data)*

- **Automation Request**: A debug/test-only invocation that identifies the operation (document or directory), fixture reference, and caller context needed to produce a result.
- **Fixture Reference**: The path or URI-like identifier supplied for validation; may resolve successfully, be inaccessible to the app process, or be invalid/unsupported.
- **Validation Result**: The explicit outcome of an automation run, including status, diagnostic detail, and any counts/evidence relevant to the exercised ingest path.
- **Evidence Record**: Human-reviewable output (report/log/result capture) that records what was exercised, what passed/failed, and which residual scenarios were not covered.
- **Residual Scope Note**: A documented statement of UI/a11y or system behaviors intentionally not proven by the automation seam.
- **Durable Automation Source**: Canonical, version-controlled source for automation logic and required test/scaffold artifacts that remains valid when generated project files are refreshed.
- **Release Exposure Check**: The combined static, runtime, and repeatable regression evidence used to prove a release/non-debug artifact does not expose the automation entry.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Maintainers can execute document-ingest and directory-ingest validation runs without human system-picker interaction in 100% of attempted runs where fixtures are accessible to the debug/test environment.
- **SC-002**: At least 95% of automation runs on a supported debug/test device or emulator produce an explicit success or diagnosable failure result within 2 minutes per run, including evidence capture.
- **SC-003**: For the programmatic scenarios covered by this feature, validation evidence identifies residual native system-picker UI/a11y cases in 100% of reported T184/T226 validation summaries.
- **SC-004**: Release/non-debug validation records zero externally invocable automation-entry exposures, supported by static artifact inspection, a runtime negative invocation check, and a repeatable regression check for the tested release/non-debug artifact.
- **SC-005**: Existing host/workspace quality gates relevant to the changed behavior remain passing after the feature is hardened, with no new crash/ANR indicators observed in the covered debug device/emulator validation runs.
- **SC-006**: After a generated-project refresh, maintainers can restore or verify required debug-only automation artifacts through a documented apply/check process with 100% pass/fail visibility (no silent missing scaffolding).

## Assumptions

- The primary audience is maintainers, QA engineers, and future automation operators validating mdreader on Android debug/test builds.
- The automation seam is for validation and evidence collection; it is not a user-facing product capability.
- Reusing the same production ingest/open behavior is required for trustworthy T184/T226 programmatic evidence; UI-level parity with native picker interaction is not required.
- Physical-device and emulator validation are both valuable; debug-package ABI differences may require selecting the correct debug artifact for the target device.
- Fixture placement may need to use app-accessible storage on some devices; path existence outside the app sandbox does not guarantee app readability.
- Directory fixture pass criteria will be documented from the actual production filtering behavior (for example, counting accepted supported entries rather than all files present), unless project owners define a stricter rule.
- Instrumented test coverage is desirable but optional for initial acceptance if the debug-only seam and evidence reporting are complete and trustworthy.
- Generated Android project output is not a durable source of truth in this repository; durable automation sources and an idempotent materialization/check process are required for maintainability.

## Dependencies

- Existing production document and directory ingest/open flows remain the source of truth for behavior exercised by the automation seam.
- Debug/test build configuration and project build tooling must support a debug-only automation entry and evidence capture workflow.
- Device/emulator access and fixture deployment remain available for validation runs.
- The project’s generated-Android refresh workflow remains compatible with an idempotent apply/check process for required debug-only automation artifacts.

## Out of Scope

- Automating native Android system document picker UI across OEMs/locales, including picker cancel and TalkBack/keyboard traversal of system picker/title UI.
- Redesigning playback, TTS, Rust playback authority, or end-user SAF selection UX.
- Treating debug-only automation evidence as release artifact, gate 9, or full accessibility certification.
- Providing a general-purpose debug backdoor beyond the narrow document/directory validation needs of T184/T226.
- Automatically claiming T184/T226 fully complete when only programmatic non-UI ingest paths have been verified.
