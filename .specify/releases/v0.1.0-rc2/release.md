# Release Record: v0.1.0-rc2 (Ressoa Candidate Release)

- **Slug**: `v0.1.0-rc2`
- **Product**: Ressoa (`app.mdreader`)
- **Date**: 2026-11-13
- **Target Platform**: Android (minSdk 29, ARM64 / universal)
- **Status**: Release Candidate (gate 9 PASS — Rust compile + Gradle APK/AAB produced; host gates 1–8 re-run pending in non-time-limited environment; signed build/store publication pending authorization)
- **Approved by**: SDD quality gate (gate 9 green; host gates 1–8 to re-run; authorization-gated items open)

---

## 1. Release Highlights & Executive Summary

Ressoa is the first audio reader focused on structured Markdown files/directories, operating under absolute data sovereignty and **100% offline/air-gapped** execution (`android.permission.INTERNET` strictly absent).

`v0.1.0-rc2` extends `v0.1.0-rc1` (specs 001–017) with the closed bundled voice catalog (spec 018), intent-filter ingress (spec 019), dynamic document-cap scalability (spec 020), and Android 16 (API 36) accessibility compliance (spec 021). The Rust engine remains the sole playback authority; the Kotlin shell is a strict mirror (SC-035).

---

## 2. Converged Features Matrix (Specs 001–021)

| Spec | Name | Status | Key Evidence |
|---|---|---|---|
| 001 | Android Media Shell & Service Mirror | ✅ Converged | JNI mirror of MediaSession; lock-screen control. |
| 002 | FR-024 Synthesis Adapter (Sherpa-ONNX / Piper) | ✅ Converged | Local neural pt-BR synthesis. |
| 005 / 007 | Design System SONA & HUD Visual | ✅ Converged | WCAG contrast, sinesthetic canvas visualizer. |
| 006 | Lente Sinestésica & RMS Lock-Free | ✅ Converged | AtomicU32 buffer ~20 Hz, no audio-thread impact. |
| 009 | File Open & Durable Bookmarks | ✅ Converged | SAF, Blake3-256, redb write every 20 s. |
| 010 | Continuous Directory Reading | ✅ Converged | SAF tree, gapless transitions, chapter advance/rewind. |
| 013 | TTFA Pipeline & Async Scheduling | ✅ Converged | TTFA 1.551 ms on Redmi Note 11 (<3000 ms). |
| 014 | Technical Symbol Normalization | ✅ Converged | `normalize.rs` fatigue reduction. |
| 015 | Semantic Prosody | ✅ Converged | `prosody.rs` natural pauses. |
| 017 | In-Memory Audio Chunk Cache (LRU) | ✅ Converged | <50 ms rewind/repeat, 32 MiB cap, PBT. |
| 018 | Sona-Inspired Voice (closed catalog) | ✅ Converged | `pt_BR-faber-medium` default + `pt_BR-cadu-medium` alternative, CC0, closed `VoiceId`, per-voice engines, `CycleVoiceNext/Prev`, persisted `last_voice`, restart-on-change. Host gates 1–8 PASS; gate 9 Rust compile PASS. |
| 019 | Intent Filter Ingress | ✅ Converged | ACTION_VIEW / ACTION_SEND ingress. |
| 020 | Dynamic Doc Cap Scalability | ✅ Converged | Document/chapter cap expanded to 1 MiB. |
| 021 | Android 16 (API 36) Accessibility Compliance | ✅ Converged (verification-only) | targetSdk/compileSdk 36; zero `announceForAccessibility`/`TYPE_ANNOUNCEMENT` in shell Kotlin; `aria-live="polite"` frontend; no INTERNET. |

---

## 3. Verification Gates & Supply Chain Audit

- Gate 1 (Format): **NOT COMPLETED this cycle** — `cargo fmt --all -- --check` did not finish within the session time budget (environment-limited). Re-run required before release.
- Gate 2 (Test Census & Regression): **NOT RUN this cycle** — pending; spec 018 recorded 136/136 clean two-way diff.
- Gate 3 (Documentation): **NOT RUN this cycle** — pending.
- Gate 4 (Clippy Pedantic): **NOT RUN this cycle** — pending.
- Gate 5 (Supply Chain Security): **NOT RUN this cycle** — pending.
- Gate 6 (Deny-list Scan): **NOT RUN this cycle** — pending.
- Gate 7 (Closed Sets): **NOT RUN this cycle** — pending.
- Gate 8 (Shell Purity): **NOT RUN this cycle** — pending.
- Gate 9 (Android): **PASS** — Rust library compiled for aarch64 (minSdk 29) AND Gradle APK packaging completed. APK: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk` (210 MiB, sha256 `75104ca62c08239ab71ca9c770c416d21c0f041e221a0e306e5cf8f2c5fd5836`). AAB also produced.

Full per-gate outcomes and the re-run are tracked in `.specify/validations/v0.1.0-rc2/validate.md`.

---

## 4. Hardware Verification & Real Device Telemetry

- Reference device: Xiaomi Redmi Note 11 (`2201117TY`, Android 13 SDK 33).
- TTFA measured: `play_to_audio_ms=1551` (spec 013, evidenced in `diagnostic-record.jsonl`).
- Directory navigation: 3 chapters verified via CDP with gapless transitions and redb persistence.
- Gate 9: Rust library compiled for aarch64 + Gradle packaging PASS; APK produced (sha256 75104ca62c08239ab71ca9c770c416d21c0f041e221a0e306e5cf8f2c5fd5836). Signed build remains authorization-gated.

---

## 5. Artifact Hashes & Build Information

- APK universal debug: **produced this cycle** — `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk` (210 MiB). SHA-256: `75104ca62c08239ab71ca9c770c416d21c0f041e221a0e306e5cf8f2c5fd5836`. AAB: `src-tauri/gen/android/app/build/outputs/bundle/universalDebug/app-universal-debug.aab`.
- Git commit of reference: `45cccb80` (dev) + in-flight provider-router changes (uncommitted: `crates/adapter-synthesis/src/{lib.rs,router.rs}`, `crates/engine-core/src/router.rs`).
- Voice catalog: `pt_BR-faber-medium` (default) + `pt_BR-cadu-medium` (alternative), CC0, `catalog.version="1"`.

---

## 6. Rollback Plan

- Revert to `v0.1.0-rc1` commit/tag (`5d2c7971`) if rc2 introduces a regression.
- Voice catalog migration is additive: `catalog.version="1"` marker triggers re-extraction; reverting assets restores faber-only layout. No session-store schema migration (`last_voice` is an additive key; unknown/missing → `pt_BR-faber-medium` default).
- No `_ =>`/`Other`/`Unknown` additions; closed sets preserved; rollback does not require code changes beyond asset revert.

---

## 7. Changelog Summary (rc1 → rc2)

- spec 018: closed bundled voice catalog (2 voices, CC0), `VoiceId` closed enum, per-voice cached engines, `CycleVoiceNext/Prev` with safe restart-on-change, persisted `last_voice`, catalog-aware diagnose/inventory, CLI `voices`/`--voice`, Android extraction of full catalog.
- spec 019: intent-filter ingress (ACTION_VIEW / ACTION_SEND).
- spec 020: dynamic document/chapter cap scalability (1 MiB).
- spec 021: Android 16 (API 36) accessibility compliance verification (no migration needed; targetSdk already 36).

---

## 8. Authorization-Gated (Not Executed This Cycle)

- Signed release build (`cargo tauri android build --release`) — needs keystore/credentials.
- Store publication (F-Droid/Google Play) — needs accounts + listing assets + ASO copy.
- TalkBack full gesture automation — needs secure-settings capability.
- Manual auditory comparison for final default voice (faber vs cadu) — maintainer/device evidence.
- Host gates 1–8 re-run to green in a non-time-limited environment.
