# T086 — On-device trials (DEFERRED — hardware available, server cannot access ADB)
# Per tasks.md Phase 8 / plan.md §Testing / constitution Principle V.

## Required device
- Android 10 (API 29) minimum; reference device: mid-range arm64, 4x A78, 6 GB RAM (plan.md §Technical Context).
- Document already parsed; audio engine stand-in registered; T028a Tauri init complete.

## Trial procedures (10/10 per SC)
1. SC-001: Lock screen, screen off 5 min — audio continues.
2. SC-002: Headphone disconnect — pauses ≤1 s; Bluetooth A2DP loss — pauses ≤1 s.
3. SC-003: Call/alarm — duck 15-25%; resume at sentence start ≤30 s.
4. SC-041: Silenced pause — resume lands in gap; never mid-word; total ≤30 s.
5. SC-051: Failover — standin_a → standin_b; named EngineError on failure.
6. SC-009 / SC-054: Play action → audible ≤3 s.
7. SC-005: Record scan — zero `transport:"playing"` without advancing position.
8. SC-006 / SC-035: Manifest + shell-purity checks (automated via T081 / check-manifest / check-shell-purity).

## Gate 9
- `cargo tauri android build --target aarch64` must produce device artifact.
- Run on device or CI with Android SDK + API-29+ emulator.

## Verification attestation (T087)
Only T087 attests. T086 is device-bound; gate-9 is build-bound.
No agent self-attestation.
