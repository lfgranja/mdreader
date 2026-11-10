# Contract: AVD Emulator Setup

## Purpose
Define the interface/contract for setting up and using AVD emulator for mdreader/Ressoa testing.

## Prerequisites
- Android SDK installed at `~/Android/Sdk` or `ANDROID_HOME`
- Emulator, platform-tools, system images installed
- KVM available on Linux x86_64 host
- User has permissions for `/dev/kvm`

## AVD Requirements
- Name: `mdreader_avd` (or configurable)
- API Level: >= 29
- ABI: x86_64 for x86_64 hosts (recommended)
- Device Profile: Pixel 5 or compatible
- System Image: Google APIs or AOSP

## Operations

### Start Emulator
```bash
emulator -avd <name> -no-boot-anim -no-audio -gpu swiftshader_indirect [flags]
```
- Must wait for boot completion
- Must be detectable via ADB

### Check Connectivity
```bash
adb devices
adb shell 'getprop sys.boot_completed'
```

### Build and Install
```bash
# For emulator testing (x86_64)
cargo tauri android build --target x86_64-linux-android

# For gate 9
cargo tauri android build --target aarch64-linux-android

# Install
adb install -r <apk>
```

### Verification
Follow specs/008-ref-device-emulator/quickstart.md for structural verification.

## Constraints
- No INTERNET permission
- minSdk 29
- SC-009 measurements must document emulator caveats
- Gate 9 requires aarch64 build
