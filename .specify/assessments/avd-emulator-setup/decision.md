# Decision: AVD Emulator Setup

- **Slug**: avd-emulator-setup
- **Decided**: 2026-11-20
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Device-dependent checks lack a reproducible local target; concretely tied to SC-009 and spec 008 verification. |
| Evidence strength | strong (prereqs) / medium (boot) | SDK/AVD/ADB prerequisites and config review are documented; full boot blocked in a container without `/dev/kvm`. |
| Value vs. inaction | strong | Unblocks local structural verification and gate-adjacent validation. |
| Feasibility / appetite | strong | Option A reuses the existing `mdreader_avd`; configuration-only, small appetite. |
| Strategic fit | strong | Aligns with prior `emulator-setup` go decision and spec 008 structural layer; no source drift. |
| Risk posture | adequate | ABI/latency caveats and environment blockers identified; mitigations are documentation and fail-closed reporting. |

## Verdict & Rationale

`go` for Option A. The setup is a prerequisite, not a product change; where the environment cannot boot the emulator, the correct outcome is a documented `blocked` state rather than a fabricated pass. This assessment complements (does not overwrite) the earlier `emulator-setup` assessment.

## If go — Handoff to `/speckit-specify`

- **Problem**: No reproducible local emulator path for device-dependent checks.
- **Chosen approach**: Option A — verify/use existing x86_64 `mdreader_avd` with spec 008 structural flows and SC-009 caveats.
- **In scope**: FR-001..FR-008; SDK/KVM/ADB verification, boot flags, connectivity, ABI-aware build/install/launch, structural verification, documentation.
- **Out of scope**: product source edits, new images/CI farm, physical-device parity claims.
- **Success metrics**: SC-001..006; T001–T009 evidence already recorded; T010–T023 to be executed where the environment permits.
- **Carried-forward questions**: host virtualization availability; emulator-vs-reference latency caveats.
