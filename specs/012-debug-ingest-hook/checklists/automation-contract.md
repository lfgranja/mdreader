# Automation Contract Requirements Quality Checklist: Debug-Only Ingest Validation Hook

**Purpose**: Validate that the debug-only ingest automation seam requirements are complete, clear, consistent, measurable, and bounded for trustworthy programmatic document/directory validation without manual SAF picker interaction.
**Created**: 2026-11-06
**Feature**: [../spec.md](../spec.md)

**Note**: This checklist evaluates the *quality of requirements* for the automation contract, not implementation behavior. It is generated from Spec 012 artifacts and authoritative Android/OWASP/W3C/ISTQB guidance.

**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.

**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

## Debug-Only Invocation Contract

- [x] CHK001 Are debug/test-only availability requirements explicitly defined for both document and directory automation entries, including what “unavailable or refuses execution” means outside debug contexts? [Completeness, Spec §FR-001, Spec §FR-002, Spec §US1-AC3, Contract debug-ingest §Guardrails]
- [x] CHK002 Is the invocation contract fully specified for required identifiers and routing fields (`component`, supported `action`, `mode`, and `path`/`uri`/intent data alternatives), including which combinations are valid for each operation? [Clarity, Spec §FR-003, Spec §US1-AC1, Spec §US2-AC1, Contract debug-ingest §Invocation Contract, Data Model §1]
- [x] CHK003 Are input-validation requirements defined to occur before any ingest attempt for missing, malformed, unsupported, or unresolvable fixture references? [Completeness, Spec §FR-003, Spec §FR-007, Spec §Edge Cases, Contract debug-ingest §Guardrails]
- [x] CHK004 Are mode/operation normalization requirements unambiguous (e.g., document/file/open_document, directory/dir/tree/open_directory, `uri`) so equivalent caller inputs cannot produce divergent documented outcomes? [Consistency, Spec §FR-003, Contract debug-ingest §Invocation Contract, Data Model §1]
- [x] CHK005 Are requirements clear about the boundary between this narrow document/directory validation seam and prohibited general-purpose debug backdoor behavior? [Boundary, Spec §Out of Scope, Contract debug-ingest §Purpose, Gap]

## Production-Path Fidelity

- [x] CHK006 Are “same production document-ingest/open flow” requirements specific enough to identify the authoritative callbacks/dispatch path that automation must exercise, excluding parallel fake or test-only ingest paths? [Clarity, Spec §FR-004, Spec §US1-AC1, Contract debug-ingest §Authority / Pipeline Invariants]
- [x] CHK007 Are “same production directory-open/manifest-dispatch flow” requirements specific enough to identify accepted chapter/manifest handling evidence that must originate from production behavior rather than hook-specific synthesis? [Clarity, Spec §FR-005, Spec §US2-AC1, Contract debug-ingest §Authority / Pipeline Invariants]
- [x] CHK008 Are Rust-as-sole-authority and Kotlin-as-transport/validation-only responsibilities consistently reflected in automation-contract requirements so hook behavior cannot be interpreted as an alternate ingest authority? [Consistency, Spec §FR-004, Spec §FR-005, Plan §Constitution Check, Contract debug-ingest §Authority / Pipeline Invariants]
- [x] CHK009 Are requirements explicit that passing automation evidence does not imply parity with native SAF picker UI/system behavior, while still preserving production ingest-path fidelity for the exercised non-UI scope? [Consistency, Spec §FR-009, Spec §FR-012, Spec §Assumptions, Contract debug-ingest §Authority / Pipeline Invariants]

## Fixture Reference Forms & App Accessibility

- [x] CHK010 Are supported fixture reference forms (local path, `file://`-style URI where applicable, `content://`/intent data URI, and directory references) completely enumerated with resolution/acceptance rules per operation? [Completeness, Spec §FR-003, Spec §Edge Cases, Contract debug-ingest §Invocation Contract, Data Model §2]
- [x] CHK011 Are app-accessibility and sandbox-readability requirements defined so “path exists on device” is distinguished from “readable/resolvable by the app process,” including non-`run-as`/external-storage cases? [Clarity, Spec §FR-007, Spec §Edge Cases, Spec §Assumptions, Contract debug-ingest §Guardrails, Data Model §2]
- [x] CHK012 Is the required outcome specified for references accepted by the automation entry but unresolvable/unreadable/unsupported by the production ingest path (including `content://` or caller-supplied URI forms)? [Completeness, Spec §FR-007, Spec §US1-AC2, Spec §US2-AC2, Spec §Edge Cases, Contract debug-ingest §Result Contract]

## Result Contract & Channel Authority

- [x] CHK013 Are success, diagnosable failure, and refusal result forms completely specified with stable, machine-parseable prefixes/codes and required detail fields? [Completeness, Spec §FR-006, Spec §US1-AC1, Spec §US1-AC2, Spec §US2-AC2, Contract debug-ingest §Result Contract]
- [x] CHK014 Is the authoritative/preferred result channel clearly defined when `cache/debug_ingest_result.txt`, logcat markers, activity-result extras, and script reports disagree, are missing, or are partially available? [Clarity, Ambiguity, Spec §FR-006, Spec §FR-008, Contract debug-ingest §Result Contract, Contract debug-ingest §Evidence Minimum (FR-008)]
- [x] CHK015 Are requirements defined for what constitutes an “explicit, script-parseable outcome” (format, parse rules, required fields) so SC-001/SC-002 can be evaluated objectively? [Measurability, Spec §FR-006, Spec §SC-001, Spec §SC-002, Contract debug-ingest §Result Contract]
- [x] CHK016 Is the `chapters=<n>` directory result semantically defined against production filtering/acceptance behavior (supported/unsupported, empty, oversized, binary, invalid-name entries) so count interpretation is unambiguous? [Clarity, Measurability, Spec §US2-AC1, Spec §FR-005, Spec §Edge Cases, Spec §Assumptions, Contract debug-ingest §Interpretation rules]

## Failure, Diagnostic & Runtime Resilience Requirements

- [x] CHK017 Are requirements complete for diagnosable failure coverage across invalid, missing, unreadable, unsupported, unresolvable, native-library-unavailable, and no-chapters cases, including minimum diagnostic detail? [Completeness, Spec §FR-007, Spec §US1-AC2, Spec §US2-AC2, Contract debug-ingest §Result Contract, Contract debug-ingest §Guardrails, Data Model §3]
- [x] CHK018 Are crash/ANR and process-stability requirements defined as part of pass/fail interpretation for automation runs, not only as incidental reporting? [Clarity, Spec §FR-007, Spec §FR-008, Spec §SC-005, Contract debug-ingest §Evidence Minimum (FR-008), Data Model §3]
- [x] CHK019 Are requirements specified for partial or inconclusive evidence states (e.g., known ingest result but preferred evidence output unavailable, missing log/cache correlation) including whether such runs count as explicit outcomes? [Coverage, Edge Case, Spec §FR-008, Spec §Edge Cases, Spec §SC-002, Gap]

## Concurrency, Idempotency & App Initialization

- [x] CHK020 Are repeated-invocation requirements defined (idempotency expectations, result isolation, fixture reuse, evidence overwrite/correlation behavior) so successive runs remain attributable and diagnosable? [Coverage, Gap, Spec §Edge Cases, Data Model §1]
- [x] CHK021 Are concurrent or re-entrant trigger requirements defined, including whether parallel runs are supported, serialized, rejected, or marked inconclusive with explicit diagnostics? [Coverage, Gap, Spec §Edge Cases]
- [x] CHK022 Are requirements defined for invocation when the app process is not already initialized, including startup sequencing, native-library readiness, and timeout/diagnostic expectations? [Completeness, Spec §Edge Cases, Contract debug-ingest §Guardrails, Gap]

## Evidence Minimum & Traceability

- [x] CHK023 Are the FR-008 evidence minimum fields (fixture/reference, operation, build/device identity, result/status, diagnostic capture, crash/ANR status, residual scope note) specified with sufficient granularity and required presence rules for every reported run? [Completeness, Spec §FR-008, Spec §US3-AC1, Contract debug-ingest §Evidence Minimum (FR-008), Data Model §4]
- [x] CHK024 Are provenance/identity-binding requirements defined for evidence artifacts (build/debug artifact identity, device serial/model/SDK/ABI, report path/timestamp/hash where practical) so results are traceable to the exercised environment? [Measurability, Traceability, Spec §FR-008, Contract residual-tracking §Evidence Record Minimum, Data Model §4, Gap]
- [x] CHK025 Are requirements explicit that programmatic `ok:*` outcomes are scoped as `programmatic-non-ui` evidence and cannot be used to claim native SAF picker cancel or TalkBack/keyboard traversal coverage? [Consistency, Spec §FR-009, Spec §FR-012, Spec §US3-AC2, Contract debug-ingest §Interpretation rules, Data Model §3]

## Acceptance Criteria Quality & Contract Consistency

- [x] CHK026 Is SC-001 (“100% of attempted runs where fixtures are accessible to the debug/test environment”) objectively measurable, with “accessible to the debug/test environment” defined and separated from app-process readability? [Measurability, Ambiguity, Spec §SC-001, Spec §Assumptions, Contract debug-ingest §Guardrails]
- [x] CHK027 Is SC-002’s “at least 95%…” and “within 2 minutes per run, including evidence capture” bounded by scope (supported environments, run lifecycle start/end, what counts as an explicit result, and handling of blocked/inconclusive runs)? [Measurability, Clarity, Spec §SC-002, Data Model §1, Gap]
- [x] CHK028 Is SC-005 aligned with the automation contract by defining which host/workspace quality gates and which debug device/emulator validation runs are relevant to changed behavior, and what constitutes “no new crash/ANR indicators”? [Consistency, Measurability, Spec §SC-005, Plan §Technical Context, Gap]
- [x] CHK029 Are spec, contract, and data-model terms consistent for operations (`document`/`directory`/`uri`), result prefixes, statuses (`pass`/`fail`/`blocked`/`inconclusive`), and evidence kinds/channels? [Consistency, Spec §FR-003, Spec §FR-006, Contract debug-ingest §Invocation Contract, Contract debug-ingest §Result Contract, Data Model §1, Data Model §3, Data Model §4]
- [x] CHK030 Are dependencies and assumptions specific enough to be validated (production ingest flows remain source of truth, debug build/test tooling available, fixture deployment available, correct debug ABI/package installable, app-accessible storage required when needed)? [Dependencies, Assumptions, Spec §Preconditions & Traceability, Spec §Assumptions, Spec §Dependencies, Gap]

- [x] CHK032 Is a stable requirement and acceptance-criteria ID/traceability scheme established across Spec 012 and T184/T226 evidence/reporting so `FR-*`, `SC-*`, `US*`, edge cases, and residual items can be referenced unambiguously? [Traceability, Spec §Preconditions & Traceability, Spec §FR-001, Spec §SC-001, Contract residual-tracking §T184/T226, Gap]
## Ambiguities, Gaps & Traceability

- [x] CHK031 Are open implementation notes that affect requirements interpretation (directory filtering vs `chapters=N`, channel/tool variability, release-evidence tooling) resolved or explicitly bounded as non-normative so they cannot silently change pass criteria? [Ambiguity, Traceability, Spec §Assumptions, Research Decision 4 Open Implementation Notes, Contract debug-ingest §Interpretation rules, Gap]

## References

- Android Developers. *Intents and intent filters* / *Test from the command line* (`adb`, `am start`, activity results, intent extras). https://developer.android.com/guide/components/intents-filters ; https://developer.android.com/studio/test/command-line
- Android Developers. *App data and files* / *Storage use cases and best practices* / *Access documents and other files from shared storage* (app-accessible paths, sandbox, SAF/content URIs, `run-as` debug context implications). https://developer.android.com/training/data-storage ; https://developer.android.com/training/data-storage/use-cases ; https://developer.android.com/training/data-storage/shared/documents-files
- Android Developers. *Build and test your app* / *App manifest overview* / *Configure build variants* (debug/test-only entries, manifest/build-variant boundaries). https://developer.android.com/studio/build ; https://developer.android.com/guide/topics/manifest/manifest-intro ; https://developer.android.com/build/build-variants
- Android Developers (locale examples consulted for cross-language coverage): 日本語 *アプリ マニフェストの概要* / *インテントとインテント フィルタ*; 한국어 *앱 매니페스트 개요* / *인텐트 및 인텐트 필터*; Deutsch *App-Manifest* / *Intents und Intent-Filter*; Español *Información general del manifiesto de la app* / *Intents y filtros de intents*; Português *Visão geral do manifesto do app* / *Intents e filtros de intents*. https://developer.android.com/guide/topics/manifest/manifest-intro?hl=ja ; https://developer.android.com/guide/components/intents-filters?hl=ja ; https://developer.android.com/guide/topics/manifest/manifest-intro?hl=ko ; https://developer.android.com/guide/components/intents-filters?hl=ko ; https://developer.android.com/guide/topics/manifest/manifest-intro?hl=de ; https://developer.android.com/guide/components/intents-filters?hl=de ; https://developer.android.com/guide/topics/manifest/manifest-intro?hl=es-419 ; https://developer.android.com/guide/components/intents-filters?hl=es-419 ; https://developer.android.com/guide/topics/manifest/manifest-intro?hl=pt-br ; https://developer.android.com/guide/components/intents-filters?hl=pt-br
- OWASP MASVS / MASTG (debug/test code must not be exposed in production; evidence should be traceable and security-relevant behavior explicitly bounded). https://mas.owasp.org/MASVS/ ; https://mas.owasp.org/MASTG/
- W3C WCAG 2.2 (accessibility claims require evidence appropriate to scope; automated/non-UI checks cannot substitute for manual assistive-technology evaluation where applicable). https://www.w3.org/TR/WCAG22/
- ISTQB CTFL / Test Management guidance concepts (test conditions, traceability, pass/fail criteria, and residual risk should be explicit). https://istqb.org/certifications/certified-tester-foundation-level-ctfl-v4-0/ ; https://glossary.istqb.org/

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied.
- Leave items unchecked when they still require clarification, correction, or reviewer evaluation.
- `/speckit.implement` reads checklist checkbox state as a gate and must not modify markers.
- `checklists/requirements.md` has a separate built-in lifecycle maintained by `/speckit.specify` and `/speckit.clarify`.
- Add findings inline and link to updated spec/contract/data-model sections when resolving gaps.
