# Data Model: AVD Emulator Setup

**Feature**: 011-avd-emulator-setup  
**Date**: 2025-11-04

## Entities

### AVD Configuration
Represents the Android Virtual Device setup.

- **name** (string): AVD name (e.g., "mdreader_avd")
- **device_profile** (string): Device profile (e.g., "pixel_5")
- **api_level** (int): Android API level (>= 29, e.g., 33)
- **abi** (string): System image ABI (x86_64 recommended for host)
- **system_image** (string): System image path/reference (e.g., "system-images;android-33;google_apis;x86_64")
- **kvm_enabled** (bool): Whether KVM hardware acceleration available
- **gpu_mode** (string): Graphics mode (auto/swiftshader_indirect/host)
- **config_path** (string): Path to config.ini

### Emulator Instance
Represents a running emulator instance.

- **avd_name** (string): Reference to AVD Configuration
- **process_id** (int): Emulator process ID
- **adb_serial** (string): ADB serial (e.g., "emulator-5554")
- **boot_completed** (bool): Whether boot completed
- **state** (enum): starting | running | stopped | error

### Android Artifact
Represents built Android application.

- **build_target** (string): Rust target (aarch64-linux-android or x86_64-linux-android)
- **artifact_type** (enum): apk | aab
- **abi** (string): Native library ABI
- **path** (string): Path to artifact
- **installed** (bool): Whether installed on emulator

### Verification Session
Represents a validation session.

- **type** (enum): structural | benchmark | gate-validation
- **emulator_info** (reference): Emulator Instance
- **caveats_documented** (bool): Whether emulator caveats documented
- **results** (string): Path to results/attestation (e.g., VERIF-EMULATOR.md)

## Relationships

- AVD Configuration -> Emulator Instance (1:many)
- Emulator Instance -> Android Artifact (installation)
- Verification Session -> Emulator Instance (uses)

## Validation Rules

- AVD must target API >= 29
- ABI should match host architecture for optimal performance
- KVM required for good performance on x86_64 Linux
- SC-009 measurements must document emulator caveats
- No INTERNET permission allowed in manifest
