# Quickstart: AVD Emulator Setup for mdreader/Ressoa

## Prerequisites Check
```bash
# Check KVM
lsmod | grep kvm
# Check Android SDK
ls ~/Android/Sdk/cmdline-tools/bin/avdmanager 2>&1 | head -1
# Check AVD
~/Android/Sdk/cmdline-tools/bin/avdmanager list avd 2>&1 | grep mdreader_avd
```

## Start Emulator
```bash
# Start with software rendering (safe for containers)
~/Android/Sdk/emulator/emulator -avd mdreader_avd -no-boot-anim -no-audio -gpu swiftshader_indirect -no-window &
# Wait for boot
adb wait-for-device
while [ "$(adb shell getprop sys.boot_completed 2>&1 | tr -d '\r')" != "1" ]; do sleep 2; done
adb devices
```

## Build and Install (for testing)
```bash
cd /home/luis/development/mdreader
# For emulator (x86_64)
cargo tauri android build --target x86_64-linux-android
# Install
adb install -r src-tauri/gen/android/app/build/outputs/apk/*/debug/*.apk
```

## Build for Gate 9
```bash
cargo tauri android build --target aarch64-linux-android
```

## Structural Verification
Follow `specs/008-ref-device-emulator/quickstart.md`:
1. Install build on AVD
2. Open parsed Markdown, press Play
3. Verify notification, focus/ducking/interruption, resume by fingerprint, diag JSONL
4. Write VERIF-EMULATOR.md with AVD version, branch, date

## Notes
- SC-009 measurements on emulator must document caveats (x86_64 vs arm64 reference device)
- Use x86_64 target for emulator testing, aarch64 for gate 9
- Gates 1-8 don't require emulator: `./scripts/verify.sh`
