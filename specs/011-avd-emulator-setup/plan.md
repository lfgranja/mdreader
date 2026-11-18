# Implementation Plan: AVD Emulator Setup for mdreader/Ressoa

**Branch**: `011-avd-emulator-setup` | **Date**: 2025-11-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/011-avd-emulator-setup/spec.md`

## Summary

Set up and automate use of Android Virtual Device (AVD) with appropriate system architecture to run benchmarks, tests, validations and gates necessary for mdreader/Ressoa. The environment already has Android SDK, emulator, KVM, and a pre-configured `mdreader_avd` (API 33 Google APIs x86_64, Pixel 5). Focus is on verifying configuration, starting emulator with proper acceleration, establishing ADB connectivity, building/installing the app with appropriate ABI, and enabling structural verification per specs/008-ref-device-emulator. For latency measurements (SC-009), emulator results must be documented with appropriate caveats.

## Technical Context

**Language/Version**: Shell scripts (bash), Rust 2024 (for builds), Kotlin (Android shell - existing)  
**Primary Dependencies**: Android SDK/Emulator, ADB, KVM (Linux), Tauri CLI 2.x, cargo, NDK  
**Storage**: N/A  
**Testing**: Manual verification + existing test suites via `./scripts/verify.sh` (gates 1-8), structural verification per specs/008  
**Target Platform**: Linux x86_64 host with Android emulator (API 29+), target Android minSdk 29  
**Project Type**: Infrastructure/DevOps setup (tooling/configuration)  
**Performance Goals**: Emulator boots with KVM acceleration, responsive for structural testing; document latency measurement caveats  
**Constraints**: minSdk 29, no INTERNET permission, Rust as playback authority, Kotlin shell mirror only, gate 9 builds aarch64, emulator ABI considerations  
**Scale/Scope**: Single developer environment setup, support for benchmarks/tests/validations/gates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Based on `.specify/memory/constitution.md` and project rules:
- **Pure-offline**: Must not add INTERNET permission; check-manifest enforces absence ✓
- **minSdk 29**: Must maintain ✓
- **Gates**: Gates 1-8 host-checkable via verify.sh; gate 9 is device build ✓
- **No source modifications**: This is setup/configuration only, no changes to crates/src-tauri/android-shell ✓
- **Emulator verification**: Follows specs/008-ref-device-emulator procedures ✓
- **Documentation**: Will produce VERIF-EMULATOR attestation if needed per specs/008 ✓

No violations expected.

## Project Structure

### Documentation (this feature)

```text
specs/011-avd-emulator-setup/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

No source code changes required. This is infrastructure setup using existing:
```text
~/Android/Sdk/                    # Android SDK/emulator
~/.android/avd/mdreader_avd.avd/  # Existing AVD
scripts/verify.sh                 # Gates 1-8
src-tauri/                        # Tauri Android build
specs/008-ref-device-emulator/    # Verification procedures
```

**Structure Decision**: Infrastructure/setup only - no code changes, documentation and usage procedures only.

## Complexity Tracking

No violations to track.
