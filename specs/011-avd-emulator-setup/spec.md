# Feature Specification: AVD Emulator Setup for mdreader/Ressoa

**Feature Branch**: `011-avd-emulator-setup`

**Created**: 2025-11-04

**Status**: Draft

**Input**: User description: Set up AVD emulator with appropriate system architecture to run benchmarks, tests, validations and gates necessary to mdreader/Ressoa.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify and Start Existing AVD (Priority: P1)

A developer needs to use the pre-configured Android emulator to run device-dependent tests for mdreader/Ressoa. They need to verify the existing AVD configuration is appropriate and start it with proper hardware acceleration.

**Why this priority**: The environment already has an AVD configured (`mdreader_avd`) with SDK/emulator/KVM available. This is the fastest path to unblock device-dependent validations.

**Independent Test**: Can be tested by checking AVD configuration, starting the emulator, and verifying ADB connectivity without building/installing the app.

**Acceptance Scenarios**:

1. **Given** Android SDK and emulator are installed with KVM available, **When** developer checks AVD configuration, **Then** `mdreader_avd` is visible with x86_64 ABI and API 33 (or compatible) configuration.
2. **Given** AVD is configured, **When** developer starts the emulator with hardware acceleration, **Then** emulator boots successfully and is detectable by ADB.
3. **Given** emulator is running, **When** developer runs `adb devices`, **Then** the emulator appears as a connected device.

---

### User Story 2 - Build and Install App on Emulator (Priority: P1)

A developer needs to build the mdreader Android artifact and install it on the emulator to run app-level tests and validations.

**Why this priority**: Required for gate 9 testing and app-specific validations per specs/008.

**Independent Test**: Can be tested by building for appropriate ABI and installing via ADB, independent of running full test suites.

**Acceptance Scenarios**:

1. **Given** emulator is running and connected via ADB, **When** developer builds the Android artifact, **Then** build completes successfully (gate 9 compatible).
2. **Given** APK/AAB is built, **When** developer installs it on the x86_64 emulator, **Then** installation succeeds and app is launchable.
3. **Given** app is installed, **When** developer launches it, **Then** app starts without immediate crashes.

---

### User Story 3 - Execute Structural Validation on Emulator (Priority: P2)

A developer needs to follow the structural verification procedures from specs/008-ref-device-emulator to validate contracts (notification, focus/ducking/interruption, resume by fingerprint, diag JSONL) without requiring a physical device.

**Why this priority**: Enables structural verification when physical device is unavailable, as documented in the project.

**Independent Test**: Can be tested by following the quickstart procedures and producing a VERIF-EMULATOR attestation.

**Acceptance Scenarios**:

1. **Given** app is installed on emulator, **When** developer opens a parsed document and presses Play, **Then** playback behavior matches structural contracts per specs/008.
2. **Given** playback is running, **When** developer tests focus/ducking/interruption scenarios, **Then** behavior matches documented contracts.
3. **Given** validation is complete, **When** developer documents results, **Then** a VERIF-EMULATOR attestation can be written following the template.

---

### Edge Cases

- **ABI mismatch**: What happens if the app is built for aarch64 but emulator only runs x86_64? Need to build for compatible ABI or use multi-ABI build.
- **Graphics issues**: In containerized/headless environments, emulator graphics may need software rendering (`-gpu swiftshader_indirect` or similar).
- **KVM unavailable**: If hardware acceleration is not available, emulator performance may be poor; may need software acceleration.
- **Emulator boot timeout**: Emulator may take time to boot in certain environments; need appropriate wait/retry logic.
- **SC-009 measurement caveats**: Latency measurements on emulator may differ from reference arm64 device; results must be documented with appropriate caveats.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST verify the existing AVD configuration (`mdreader_avd`) has appropriate architecture (x86_64) for the x86_64 host with KVM support.
- **FR-002**: System MUST be able to start the emulator with hardware acceleration when KVM is available.
- **FR-003**: System MUST establish ADB connectivity to the running emulator.
- **FR-004**: System MUST support building the Android artifact compatible with the emulator's ABI (gate 9 compatible).
- **FR-005**: System MUST support installing the built artifact on the emulator via ADB.
- **FR-006**: System MUST allow launching and basic interaction with the installed app.
- **FR-007**: System MUST support following structural verification procedures per specs/008-ref-device-emulator/quickstart.md.
- **FR-008**: When running SC-009 latency measurements on emulator, results MUST be documented with caveats noting difference from reference arm64 device class.

### Key Entities *(include if feature involves data)*

- **AVD Configuration**: Android Virtual Device settings including name, device profile, API level, ABI, system image path, hardware acceleration settings.
- **Emulator Instance**: Running emulator process with ADB connection state.
- **Android Artifact**: Built APK/AAB for installation on emulator/device.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: AVD boots successfully with hardware acceleration on x86_64 host within reasonable time (emulator becomes detectable by ADB).
- **SC-002**: ADB can detect and connect to the emulator (device shows as online).
- **SC-003**: Android artifact can be built and installed successfully on the emulator.
- **SC-004**: App launches successfully on emulator without immediate crashes.
- **SC-005**: Structural verification flows from specs/008 can be executed (notification, focus/ducking/interruption, resume by fingerprint behaviors observable).
- **SC-006**: When device-dependent validations are run, results are documented with appropriate emulator caveats where measurements may differ from physical reference devices.

## Assumptions

- Android SDK is installed at standard location (`~/Android/Sdk`) with emulator, platform-tools, cmdline-tools available.
- KVM virtualization is available on x86_64 host (kvm_amd/kvm_intel modules loaded).
- A pre-configured AVD (`mdreader_avd`) exists and is suitable for basic testing.
- Network access is available if additional components needed (though research indicates components are present).
- For SC-009 measurements, emulator results will be used with documented caveats per project guidance.

## Clarifications

### Session 2025-11-04

- Q: Is the existing `mdreader_avd` properly configured (RAM/CPU/GPU) for running the mdreader app smoothly? → A: Use as-is with software rendering flags (`-gpu swiftshader_indirect`) for stability in containerized environments; hardware acceleration via KVM is available on x86_64 host.
- Q: Can we successfully build and install the app on this AVD in this environment? → A: Build for `x86_64-linux-android` target for native emulator performance, or use aarch64 with translation; Tauri/Android SDK/NDK are configured (gate 9 uses aarch64).
- Q: For SC-009 latency measurements, how should emulator results be documented/interpreted? → A: Document emulator caveats (x86_64 vs reference arm64, translation overhead, graphics mode, cold boot conditions) and treat as regression indicators; prefer physical device for final gate compliance per project guidance.
- Q: Is the emulator currently running or does it need to be started? → A: Check with `adb devices` first; start with appropriate flags (`-gpu swiftshader_indirect`) if not running.
- Q: Are there any environment-specific issues (graphics, permissions, nested virtualization) that would prevent emulator from running well? → A: Verify KVM access (/dev/kvm), use software rendering in constrained/container environments, consider headless mode if needed.
- Q: Should we build for x86_64 ABI for emulator testing or use the aarch64 build target? → A: Use x86_64 for emulator testing/iteration, aarch64 for gate 9/final device builds as specified by the project.
