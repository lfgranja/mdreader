# Data Model: VERIF-EMULATOR — Structural Verification Layer (Option A)

**Feature**: `specs/008-ref-device-emulator/`  
**Source spec**: `specs/003-reference-device/data-model.md`  
**Status**: Draft — documentation layer; no new database/entities; references existing structural contracts.

## Entities (existing, referenced — no new entities)

- **VERIF-EMULATOR attestation document** (document-level, not database entry):
  - Fields: feature-id (`ref-device-emulator`), branch (`005-adopt-design-sona`), date, AVD version (e.g., `API 29` / `API 29+`), verified framework contracts list, exclusions list (audio-output / `SC-009` / Bluetooth / 10/10), reference to physical `verify.sh`, sign-off path.
  - Identity: derived from `blake3` fingerprint of attestation content (optional; not required; can use commit SHA + date).
  - Relationships: references `003-reference-device` contracts (`status-surface`, `earcon-mapping`, `surface-audit`); does not create or modify `blake3` resume fingerprint (`session` crate unchanged).
  - Validation: exclusions must be complete (no false-equivalence claim); must reference `problem.md` / `decision.md`; must not claim `gate-8-full`.

- **003-reference-device structural-state** (existing, verified by layer):
  - `§SurfaceInventory`: manifest/service/notification state (existing).
  - `§NotificationSurface`: title + section + progress (deterministic / indeterminate rules).
  - `§ProgressIndication`: progress bar rules.
  - `§RateLadder`: 7-rung ladder (`0.75` … `4.0`) — verified structural (UI exists); not verified by audio-output.
  - `§DiagnosticEvent`: `EventKind` 9 variants; JSONL local; no transmission.
  - `§DesignDirState`: `DESIGN-SONA` tokens / palette / contrast (already adopted in `005`).
  - No new fields added; layer verifies existing contracts are honored by framework.
  - `§MoveSpec`: existing from parent contracts (`003-reference-device`); structural verification references but does not modify.

## Constraints

- No database schema change (`redb` / `sqlite` not involved).
- No new `redb` bookmark for attestation.
- `blake3` fingerprint of document content (if used) is computed at attestation time, not stored persistently.
- Attestation is append-only / versioned (new run = new document, not edit of previous).

## State Transitions (applicable to verification process, not to document content)

- `PENDING` → `STRUCTURAL_VERIFIED` (AVD framework contracts pass)
- `STRUCTURAL_VERIFIED` → `PHYSICAL_GATE-8-FULL` (separate, manual, physical device; not part of this layer)
- `PHYSICAL_GATE-8-FULL` → `CONVERGE` (after `test` / attestation review / `speckit.converge`)
- Note: `VERIF-EMULATOR` does not change transition rules for physical gate.
