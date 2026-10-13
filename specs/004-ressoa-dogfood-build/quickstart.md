# Quickstart: Ressoa Dogfood Build validation

**Feature**: `004-ressoa-dogfood-build` | Proves the maintainer-only sideload build works end-to-end on the reference phone. No implementation code here — commands reference the contracts and data model above.

## Prerequisites

- Rust 1.85+ (Edition 2024), `cargo-nextest`, Android SDK (`aarch64` target), reference phone Redmi 2201117TY (Android 13 / API 33) per `specs/003-reference-device/`.
- Single fixed voice `pt_BR-faber-medium` bundled (`crates/adapter-synthesis/assets/voices/`); pure offline (Wi-Fi/data off for all trials).
- Fixed fixture committed to repo: Portuguese Markdown with headings + numbers/currency, ~2 h of material at 1.0x (validates SC-001 at every speed incl. 4.0x).
- Contracts: [ingest.md](contracts/ingest.md), [surface.md](contracts/surface.md), [earcon.md](contracts/earcon.md), [diagnostic-trace.md](contracts/diagnostic-trace.md) + `diagnostic-record.schema.json`.

## 1. Host gates (Linux, no device)

```sh
cargo fmt --all -- --check
cargo nextest run --workspace
cargo doc --workspace --no-deps
./scripts/verify.sh            # gates 1–8
./scripts/check-manifest.sh    # <service mediaPlayback> + FOREGROUND_SERVICE* + VIBRATE + POST_NOTIFICATIONS + minSdk 29 (must match plan.md + tasks T049)
./scripts/check-shell-purity.sh # zero playback logic in android-shell/
```

Expect: green; rewind/snap math, 7-rung ladder, BLAKE3 fingerprint, content-offset progress, trim-bound, closed-set (no `_ =>`) tests pass.

## 2. Build + sideload (maintainer only, nothing published)

```sh
cargo tauri android build --target aarch64
adb install -r <apk>
```

Expect: installs on the reference phone; no reviewer/listing/changelog involved.

## 3. On-device acceptance (reference phone only)

1. **Commute loop (SC-001)**: open fixture via system file picker → Play → lock → 30 min continuous audible reading with screen off, lock-screen controls only, zero unlocks. Repeat at each of the 7 speeds incl. 4.0x.
2. **First audio (SC-002)**: 10/10 cold-start trials (cold start = process-killed, fixture unopened, radios off), first audible speech ≤3 s after Play.
3. **Interruptions (SC-003)**: 10/10 per class — nav-transient → duck ~20% + restore; call/alarm → pause + 3 s reason, never auto-resume; headphone removal → pause ≤1 s, never on speaker; permanent loss → pause + 3 s, never auto-resume.
4. **Blind confirmations (SC-004)**: no-look, 10/10 per type (play, pause, skip, rate-step, warning, failure) via earcon + state change + haptic, sounds-on and sounds-off.
5. **Resume (SC-005)**: pause → move + rename file → reopen via picker → 10/10 offer continue-from-position; edit content → 10/10 no resume; in-app Stop → next open starts at beginning; deliberate-pause offer window is 3 s (silence = continue from stored). Kill/power-loss → resumes with ≤60 s replay (interruption replay ≤30 s).
6. **Lock-screen purity (SC-006)**: 10/10 inspections across playing/paused/section-change/rate-change/interruption — title + section + progress only, zero prose/time/rate/voice leaks; dismissal attempted while playing refused 10/10.
7. **Rate (SC-007)**: chosen rung persists across sessions/documents (10/10 per rung); voice quicker/slower never pitched (PITCH_TOLERANCE_CENTS=50) + blind listener check.
8. **Record (SC-008)**: `adb pull` app-private JSONL → full trace present (session/transport/per-section/per-rate/buffer/interruptions + measured latencies + failures), retained within 30-day/10 MB bounds, zero entries with prose.
9. **Lock-screen privacy (manual inspection)**: enable the phone's lock-screen privacy-hide-contents setting → app obeys, lock-screen hides contents, no workaround attempted.

## 4. Privacy + scope glance

- Lock-screen + in-app screen never show prose; diag never contains prose; diag never transmitted, no in-app surface.
- Out of scope (must be absent): second voice, cloud/network voice code, library view, distribution artifacts, head-unit/Auto, in-app diagnostics.
