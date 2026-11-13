# Ressoa (mdreader) Release Publication Guide

## Overview
This document outlines the publication process for mdreader (Ressoa), an Android media-shell TTS audiobook reader built with Rust as the sole playback authority and Kotlin as a mirror-only shell.

## Prerequisites

### 1. Build Environment
- Rust toolchain (stable channel)
- Android SDK with minSdk 29
- Tauri 2.12.1 bridge
- cpal 0.18.2 backend with AAudio

### 2. Signing Keys
- **Keystore location**: `~/upload-keystore.jks`
- **Keystore alias**: `upload`
- **Note**: The keystore file is NOT committed to git for security. Store it securely.

## Release Build Process

### Step 1: Build the Release APK
```bash
cd /home/luis/development/mdreader
cargo tauri android build --target aarch64 --release
```

This produces:
- `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk` (~188MB)

### 2. Verify the APK Signature
```bash
# Verify the APK signature
/home/luis/Android/Sdk/build-tools/36.0.0/apksigner verify --print-certs \
    src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk

# Expected output:
# Signer #1 certificate DN: CN=Ressoa, O=mdreader, C=BR
# Signer #1 certificate SHA-256 digest: afa1ad07fea7876139bb8c1ed0502283817effc6818ef968b055b41a40f646e5
```

### 3. Verify the Android Emulator (if available)
- AVD: `mdreader_avd` (API 33)
- KVM: Available for hardware acceleration
- Android SDK: `/home/luis/Android/Sdk`

### 4. Google Play Store Publication
- **Product Name**: Ressoa
- **Identifier**: `app.mdreader`
- **Target**: Android minSdk 29
- **Developer Account**: Required ($25 one-time fee)
- **Play App Signing**: Required for Android App Bundles
- **Data Safety**: Document no internet permission (android.permission.INTERNET absent)
- **Privacy Policy**: Required (state 100% offline, no telemetry)

### 5. F-Droid Publication
- **Source Code**: Must be fully open source
- **Reproducible Builds**: Required for F-Droid inclusion
- **Anti-Features**: None (no ads, no tracking, no internet)
- **Repository**: Add to F-Droid repository

### 6. Monetization Strategy
- **Model**: Freemium + Perpetual License ("Pay Once, Own Forever")
- **Free Tier**: Unlimited Markdown reading with default Piper pt-BR voice, transport controls, speed adjustment (0.75x-4.0x), durable bookmarks in redb
- **Pro Tier**: One-time payment (R$29,90-R$49,90 or $9.99-$14.99 USD)
- **Pro Features**: Multiple local voice catalogs, directory reading, additional visual themes, exportable diagnostic reports
- **Cost Structure**: Zero server cost (all computation local)

## Key Architecture Decisions

1. **Rust = Sole Playback Authority**: Position/transport/rate/focus/interruption/recovery live in Rust crates
2. **Kotlin/Android Shell = Mirror Only**: Renders published JSON verbatim; no position/progress/haptics computation
3. **Tauri 2.12.1 Bridge**: One consumer among many of the Rust crates
4. **Closed Sets Everywhere**: Exhaustive `match`, no `_ =>` arms, no `Other`/`Unknown`
5. **Library-First**: Every capability reachable as Rust API; CLI thin (`main.rs` → `cli::run`)
6. **One Code Path**: Identical CLI and IPC (`tauri::ipc::Channel`, no `InvokeBody::Streaming`)
7. **Zero Android API in Rust**: Kotlin pushes answers across boundary

## Distribution Channels
- Google Play Store (primary)
- F-Droid (open source alternative)
- Direct APK download (alternative)
