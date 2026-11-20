# Concept: AVD Emulator Setup

- **Slug**: avd-emulator-setup
- **Created**: 2026-11-20
- **Recommended option**: Option A

## Options

### Option A — Verify and use the existing `mdreader_avd` (configuration/boot/ADB/build/install + spec 008 flows)
- **Sketch**: Confirm SDK/KVM/ADB and x86_64 API 33 config; boot with stable flags (`-gpu swiftshader_indirect -no-boot-anim -no-audio`); verify `adb devices`; build the emulator-ABI artifact, install/launch; run spec 008 structural checks and record SC-009 caveats; document everything in quickstart/notes.
- **Appetite**: small (environment-gated for full boot)
- **Trade-offs**: Reuses existing assets with no product-code risk; full boot depends on host virtualization support. Residual measurements are explicitly caveated rather than treated as reference-grade.
- **Rabbit holes**: ABI mismatch (x86_64 emulator vs aarch64 gate 9), software GPU slowness, misreading emulator latency as device latency, silent source edits.

### Option B — Create a new system image / dedicated CI emulator farm
- **Sketch**: provision new AVD images and CI runners.
- **Appetite**: large
- **Trade-offs**: Unnecessary for the current need; rejected.

### Option C — Physical device only
- **Sketch**: skip emulator, rely on hardware.
- **Appetite**: small but blocking
- **Trade-offs**: Does not unblock local/headless verification; rejected.

## Recommendation

Option A. It matches the prior `emulator-setup` go decision and the spec's FR-001..FR-008 with minimal, configuration-only work.

## Out of Scope (recommended option)

- Product source changes, new images/CI farms, parity claims for SC-009, gate 9 release build certification.

## Assumptions to Validate

- KVM availability is host-dependent; `emulator-startup-notes.md` documents a container without `/dev/kvm`.
- x86_64 emulator artifact is buildable with the repo toolchain (T010–T012, still open).
