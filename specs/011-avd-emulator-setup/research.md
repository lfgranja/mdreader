# Research: AVD Emulator Setup for mdreader/Ressoa

**Feature**: 011-avd-emulator-setup  
**Date**: 2025-11-04

## Summary

Research findings from codebase analysis and best practices for Android emulator setup with Rust/Tauri apps.

## Decisions

### Decision 1: Use existing AVD configuration
- **Decision**: Leverage pre-configured `mdreader_avd` (API 33 Google APIs x86_64, Pixel 5)
- **Rationale**: Already exists in environment, SDK/emulator/KVM present, matches host architecture
- **Alternatives considered**: Create new AVD (redundant), use physical device (not available)

### Decision 2: ABI selection strategy
- **Decision**: Use x86_64 for emulator testing/iteration, aarch64 for gate 9 builds
- **Rationale**: Native x86_64 on x86_64 host with KVM gives best performance; gate 9 requires aarch64 per project
- **Alternatives considered**: Always aarch64 (slower via translation), universal APK (larger)

### Decision 3: Graphics acceleration
- **Decision**: Start with software rendering (`-gpu swiftshader_indirect`) for stability
- **Rationale**: Current AVD config has hw.gpu.mode=auto; software rendering safer in containerized environments
- **Alternatives considered**: Hardware GPU (may not work in all environments)

### Decision 4: SC-009 measurement approach
- **Decision**: Document emulator caveats, treat as regression indicators
- **Rationale**: Project guidance explicitly states measurements on reference arm64 device; emulator has overhead
- **Alternatives considered**: Require physical device (not available)

## Key Findings

### From codebase
- Tauri 2.12.1, minSdk 29 enforced
- Gate 9: `cargo tauri android build --target aarch64`
- Gates 1-8: `./scripts/verify.sh` (no emulator needed)
- Structural verification per specs/008-ref-device-emulator/quickstart.md
- No INTERNET permission required
- Rust is playback authority, Kotlin shell is mirror only

### From best practices
- x86_64 + KVM is optimal for x86_64 hosts
- Cold boot for benchmarks, quickboot for iteration
- Disable animations for consistent results
- Use `android-emulator-runner` patterns in CI
- Document all measurement conditions

## Open Questions Resolved
All clarifications from spec resolved via research.
