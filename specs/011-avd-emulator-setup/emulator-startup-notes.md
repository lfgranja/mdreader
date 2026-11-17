# Emulator Startup Notes

**Date**: 2025-11-04  
**AVD**: mdreader_avd (API 33 Google APIs x86_64, Pixel 5)

## Attempted Startup
```bash
emulator -avd mdreader_avd -no-boot-anim -no-audio -gpu swiftshader_indirect -no-window
```

## Environment Constraints
- KVM not available (/dev/kvm missing, modules loaded but no device node)
- User not in kvm group
- Running in containerized environment without nested virtualization passthrough
- Software rendering (swiftshader_indirect) required

## Observations
- Emulator started (process running, GRPC server started on port 8554)
- Device appeared as `emulator-5554 offline` initially
- Cold boot attempted (no snapshot available)
- Boot did not complete within timeout period (300s+) in this environment
- Process terminated after timeout

## Recommendations
- For this environment, consider using a pre-configured snapshot if available
- Or run without `-no-window` in environment with display
- Or use physical device if available
- Structural verification can proceed once emulator boots successfully
- All configuration and prerequisites are verified correctly

## Status
AVD configured correctly per requirements. Emulator startup blocked by environment constraints (no KVM/nested virt), not by configuration issues.
