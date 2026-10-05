# Quickstart: Android Media Shell validation

**Feature**: `001-android-media-shell` | Proves the shell works end-to-end. No
implementation code here — commands reference the contracts and data model above.

## Prerequisites

- Rust 1.85+, `cargo-nextest`, Android SDK (`aarch64` target), one API-29+ device.
- Two stand-in engines honouring [engine-handoff.md](contracts/engine-handoff.md)
  (SC-051 pair; real engines re-verify SC-044 later).

## 1. Host gates (Linux, no device)

```sh
cargo fmt --all -- --check
cargo nextest run --workspace
cargo doc --workspace --no-deps
# closed-set: no `_ =>` in crates/*/src ; deny-list: no denied crate in manifests/lock
./scripts/verify.sh   # runs gates 1–8 once it exists
```

Expect: green; rewind/snap, ladder, fingerprint, trim-bound unit tests pass.

## 2. Manifest + shell-purity checks (SC-006, SC-035)

```sh
cargo tauri android build --target aarch64
# automated check: <service mediaPlayback> + FOREGROUND_SERVICE* permissions present
# automated check: android-shell/ contains no transport/rate/position logic
```

Expect: both checks pass; removing the `<service>` declaration fails the build.

## 3. On-device trials (each 10/10)

1. **Locked playback**: play → lock 5 min → audio continuous, controls reachable (SC-001).
2. **Headphone pull**: unplug → pause ≤1 s, never on speaker (SC-002).
3. **Duck vs pause**: navigation prompt → ~20% then restore + sentence-snap resume ≤30 s
   total (SC-003); call/alarm → pause + `"Leitura pausada — outro app usando o áudio"` (SC-041).
4. **Transport**: ±5 s skips (boundary clamp), one-step ladder moves incl. 2.0x→4.0x with
   unchanged pitch, Stop-discards vs Pause-preserves, completion tears down residency (SC-004,
   SC-015/016/018/019/026/045/049).
5. **Latency**: play → audible ≤3 s, parsed doc, mid-range arm64 (SC-009).
6. **Recovery**: kill service → paused-at-position, no auto-audio (SC-011); deliberate
   reopen → 3 s offer auto-continues (SC-020); unexpected end → silent paused (SC-021);
   power-loss replay ≤30 s (SC-007).
7. **Failover (SC-051)**: kill stand-in A mid-document → named failure + offer → B takes
   over from last spoken position, unattended after 3 s; force B down → paused + reason.
8. **Record**: force device failure → local entry (position + name), 30-day/10 MB trim
   visible as trim, zero egress, unreadable by other apps, no in-app surface (SC-025/037).

## 4. Privacy glance (SC-010/030/046)

Locked notification shows title + section title + progress only — no body text, no
elapsed/remaining, no rate/voice; placeholder title when untitled; no section title
before the first heading.
