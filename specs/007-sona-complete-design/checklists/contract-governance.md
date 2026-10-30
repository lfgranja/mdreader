# Contract & Governance Requirements Quality Checklist: SONA Complete Design (007)

**Purpose**: Validate the quality, clarity, completeness, and measurability of contract/governance requirements for 007 (token sanctioned scopes, C-H1 amendment, inventory 9→10, §VII coordinated edit, chroma census scope-awareness, contract update workflow, version bump, traceability).
**Created**: 2026-10-28
**Feature**: [spec.md](../spec.md) | [status-surface.md](../contracts/status-surface.md) | [surface-audit.md](../contracts/surface-audit.md) | [data-model.md](../data-model.md)

**Note**: This custom checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied. It does not mean implementation work is complete.

## Token Sanctioned Scopes

- [x] CHK001 - Are the sanctioned paint scopes for jade (`#5AD6A8`) and celeste (`#B9A6F5`) enumerated with file + element context in the contract, rather than as bare hex names? [Completeness, Contract A1, C-H1, C-H11]
- [x] CHK002 - Is the rule that painting celeste/jade outside sanctioned scopes is a violation stated with an objective, scope-aware detection method? [Clarity, C-H1, C-H11, Spec §FR-007]
- [x] CHK003 - Are the permitted `rgba()`/`rgb()` alpha operations for glassmorphism defined as an explicit allowlist, with base colors required to be verbatim SonaToken members? [Clarity, Contract A1, Spec §FR-005/FR-009]
- [x] CHK004 - Is the rate-display paint pinned to `--sona-harp` (gold) with an explicit prohibition on celeste for the numeric rate, traceable across spec, contract, and data-model? [Consistency, Spec §FR-003, Contract A1, DataModel SonaToken]

## Closed Sets & Token Expansion

- [x] CHK005 - Is the SonaToken closed-set baseline count (10) and the additive-only expansion rule (no replacements) documented with a versioned baseline reference? [Completeness, Constitution §VII, DataModel SonaToken]
- [x] CHK006 - Are the gates for promoting a reserved token to painted status (justification, coordinated edit, version bump, census update) specified as an explicit checklist rather than prose? [Clarity, Constitution §VII, Contract A1/A6]
- [x] CHK007 - Is the "no catch-all arm" rule for closed sets (including the token set) reinforced with a fail-closed verification (no `_ =>` in `crates/*/src/`)? [Measurability, Constitution §VII, C-H1]
- [x] CHK008 - Is the banned erratum variant `#5FD0A0` specified as banned in all scopes, including sanctioned ones? [Completeness, C-H1, DataModel SonaToken]

## Surface Inventory (9 → 10)

- [x] CHK009 - Is the inventory count update from 9 to 10 reflected consistently across spec (FR-007/SC-005), contract (A2/C-H4), data-model, and existing tests? [Consistency, Spec §FR-007, Contract A2, DataModel SurfaceInventory]
- [x] CHK010 - Are the two new inventory elements (waveform canvas, repeat control) each given binding, aria, and role requirements in the inventory table? [Completeness, DataModel SurfaceInventory, Contract A2/A5]
- [x] CHK011 - Is `#progress` retention as an sr-only aria container (visually hidden, not `display:none`) defined with an explicit detection rule? [Clarity, Contract A2, C-H4, tests/surface_inventory.rs]
- [x] CHK012 - Are "no new `display:none` corpses" and "no dead CSS" requirements stated with an objective scan method? [Measurability, C-H4, tests/surface_inventory.rs]

## §VII Coordinated Edit & Change Discipline

- [x] CHK013 - Is the §VII coordinated-edit file set (spec + plan + tasks + code + docs + contract + data-model) explicitly enumerated per change type (token addition vs. element addition)? [Completeness, Constitution §VII, Contract A6]
- [x] CHK014 - Are the C-H1/C-H4 verification gates required as part of the coordinated edit, with defined pass/fail criteria? [Measurability, Contract A6, C-H1, C-H4]
- [x] CHK015 - Is the change-discipline rule ("all 007 additions follow the coordinated edit") specified with the exact file list and ordering? [Clarity, Contract A6, Constitution §VII]
- [x] CHK016 - Is the contract-amendment convention (amendment doc vs. base 005 contract, with base rules remaining in force) documented with a precedence rule? [Consistency, Contract header, A6]

## Chroma Census Scope-Awareness

- [x] CHK017 - Is the chroma census specified to be scope-aware (file + element context) rather than a flat hex list, with a defined scope grammar? [Clarity, C-H1, C-H11]
- [x] CHK018 - Are false-positive handling rules defined for the census (token declaration occurrences, case-insensitivity, comments, string literals)? [Edge Case, tests/contract_census.rs]
- [x] CHK019 - Is the canvas/ribbon draw code identified as the only sanctioned location for jade/celeste paint, with a checkable boundary? [Measurability, C-H1, C-H11, Contract A3/A7]

## Contract Update Workflow

- [x] CHK020 - Are contract amendments required to cite the amended base section (A1 S3, etc.) and to list unchanged sections explicitly? [Traceability, Contract A1–A8]
- [x] CHK021 - Is the served-bundle byte-identity gate (C-H8) defined with a comparison method and failure handling? [Measurability, C-H8, Contract A8]

## Version Bump & Rollback/Fail-Closed

- [x] CHK022 - Is the semver bump requirement tied to change type, with explicit version fields to update (constitution, DESIGN-SONA/VERSION, changelog)? [Clarity, Constitution §VII, Contract A6]
- [x] CHK023 - Are rollback/fail-closed rules specified for partial coordinated edits (e.g., census failure reverts token promotion)? [Coverage, Edge Case, Gap]
- [x] CHK024 - Is the fail-closed behavior for a missing or invalid census/inventory check defined (build failure vs. warning)? [Measurability, Gap]

## Traceability (Spec ↔ Contract ↔ Data-Model)

- [x] CHK025 - Is there a traceability matrix linking each functional requirement (FR-001…FR-011) to a contract amendment (A1–A8) and a data-model entity? [Completeness, Gap]

## Notes

- Mark items `[x]` only after review confirms the requirement-quality criterion is satisfied.
- Items are requirement-quality questions (unit tests for requirements), not implementation tests.
- Traceability markers reference spec sections, contract amendments (A1–A8), C-H rules, constitution §VII, and data-model entities.