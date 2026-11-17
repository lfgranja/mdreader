# AVD Configuration Review: mdreader_avd

**Reviewed**: 2025-11-04  
**Location**: `~/.android/avd/mdreader_avd.avd/config.ini`  
**AVD Name**: mdreader_avd (display/build names `<build>` in config)

## Configuration

| Setting | Value | Notes |
|---------|-------|-------|
| API Level | 33 (Android 13) | Target: android-33 |
| System Image | Google APIs x86_64 | `system-images;android-33;google_apis;x86_64` |
| System Image Path | `/home/luis/Android/Sdk/system-images/android-33/google_apis/x86_64/` | Absolute path configured |
| ABI/Architecture | x86_64 | `hw.cpu.arch=x86_64`, `abi.type=x86_64` - KVM compatible |
| Device Profile | Pixel 5 | Google Pixel 5, 1080x2340 @440dpi, portrait |
| CPU Cores | 4 | `hw.cpu.ncore=4` |
| RAM | 2048 MB | `hw.ramSize=2048` |
| VM Heap | 256 MB | `hw.heapSize=256` |
| Internal Storage | 2048 MB | `disk.dataPartition.size=2G` |
| SD Card | 512 MB | `sdcard.size=512M` |
| GPU | Auto (disabled) | `hw.gpu.enabled=no`, `hw.gpu.mode=auto` - use software rendering flags when launching |
| Play Store | Disabled | `PlayStore.enabled=no` |

## Requirements Compliance

- ✓ API ≥ 29 (API 33)
- ✓ x86_64 ABI for x86_64 host with KVM support
- ✓ Google APIs system image available
- ✓ Pixel 5 device profile
- ✓ Appropriate resources configured

## Recommendations

- Launch with software rendering for stability in containerized environments: `-gpu swiftshader_indirect`
- Use `-no-boot-anim -no-audio` for faster startup
- KVM not available in current environment per verification, so software emulation will be used
- For gate 9 builds use aarch64 target; for emulator testing use x86_64 target or aarch64 with translation
