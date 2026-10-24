# Quickstart: Adotar o DESIGN-SONA (Option-A subset) — validation & trial guide

**Feature**: `005-adopt-design-sona` | **Date**: 2026-10-24 | Contracts: [status-surface.md](contracts/status-surface.md) · [earcon-mapping.md](contracts/earcon-mapping.md) · [surface-audit.md](contracts/surface-audit.md) | Data model: [data-model.md](data-model.md)

Zero new dependencies; the restyle is audited by deterministic host checks; the human legs below mirror 004's T051/T052 posture (10/10 sheets, reference phone only, never claimed by host checks).

## Prerequisites

- Rust stable toolchain (`rust-toolchain.toml` tracked channel; workspace mins: MSRV 1.85 build floor rustc ≥1.88 unchanged), `cargo-nextest`.
- Android SDK + `cargo tauri android build --target aarch64` working (gate 9 posture from 004). Reference phone: Redmi 2201117TY (API 33).
- For §3: a parked car (SC-002 channel = media speaker), TalkBack enabled, and the trial sheet template (below). Person: the maintainer-driver is both operator and listener (dogfood posture, per Assumption #4); a second person is ONLY required on the pilot buzz-check if no rigid mount is improvised (the admin role can self-perform the check).

## 1. Host gates (Linux, no device)

```sh
scripts/verify.sh                 # full gate run — MUST exit 0; new gate step: `surface-audit`
cargo nextest run --workspace     # census / contrast / inventory / aria / movement / accent /
                                  # byte-identity-when-built / short-mix burst + ≤100 ms oracle twins
                                  # names follow contracts/surface-audit.md A2 (C-H1…C-H9)
```

The twins parse `data-model.md` §SonaToken + the auditee list of `contracts/surface-audit.md` A1; a listed auditee gone missing is a FAILED check, not a skip (`fail-closed`). Spot-checks:

```sh
# hex census twin (both progress states), role-exclusivity twin and movement census:
cargo nextest run -E 'test(chroma)' --workspace        # C-H1/H2
cargo nextest run -E 'test(inventory)' --workspace     # C-H4/H5/H6
cargo nextest run -E 'test(earcon)' --workspace        # C-H9 latency + burst policy
```

Expected results: zero off-set hexes; zero celeste/jade paint; `#8FD8E8` present ONLY in `:focus-visible` rules; MediaStyle block paints exactly `#F2F0E6`; the reduced-motion freeze guard present; zero infinite loops outside the closed movement set; every enumerated control labeled; `role="progressbar"`+valuetext present; visually-hidden `role="status"` present; safety counters on bursts are typed tally (no catch-all arm); first-consumed-sample oracle ≤100 ms for every trigger; design v2.2.0 headers + `DESIGN-SONA/VERSION` present; stale rows aligned (the R1 task is audited by the same census — the dir is an auditee).

## 2. Build + sideload

```sh
cargo tauri android build --target aarch64   # gate 9
adb install -r <apk>
```

Then on-device glance: the app's in-app screen IS the restyled StatusView (the served surface identity — no bench `dist/index.html` tokens-less page), the header carries the Concha "R" mark, and `diag` shows no new egress-related entries.

## 3. On-device trials (10/10 sheets; order as priced below)

**3.1 SC-003 — notification purity (10/10 inspections).** Start playback, lock, inspect: only título + seção atual + progresso; accent = marfim only; no body text / rate / voice / mark; warning state renders the static reason line + Ostinato with 3 s TTL (transient-loss exercise: place a call or navigate prompt — the reason line expires, playback behavior per 004).

**3.2 SC-001 visual purity trial.** Drive the surface through determinate AND indeterminate progress; confirm colors match `data-model.md` §SonaToken (text marfim, section periwinkle, track/track fill, gold dot on the ladder, cautions at 3 s pulses, failure static red); no third color anywhere; single column; no excluded screen reachable (try back/links — nothing opens).

**3.3 SC-002 — blind-test protocol (the human gate; do NOT skip).**
1. Device on a rigid mount; the media speaker is the gate channel (parked car); system assistant/UI sounds silenced — the MEDIA channel stays ON; hands off the device body after start (haptic-disposition line of `contracts/earcon-mapping.md` E6).
2. Pilot pass (admin): fire each of the 7 confirmations and confirm ZERO buzz (a buzz fails the disposition — sounds-only attribution), then the volunteer (maintainer) proceeds eyes-closed/screen-off.
3. Familiarization 5–10 min WITH feedback; then randomized constant stimulus: ≥15 trials × 7 types, no two equal consecutively, no feedback, 5–10% trap attempts (count them), rapid-sequence drill (2–3 heterogeneous inside 100–150 ms, per-attempt answer).
4. Fill the confusion-matrix artifact (CSV; direction-of-skip sub-score); score per-attempt.
5. Pass = zero systematic confusions per type; ANY systematic confusion on Allegro-trás×Andante or Unison×Allegro-frente (or elsewhere) is MEASURED evidence that authorizes a 004-contract amendment — recorded, never silently patched. Replicate ≥24 h later on the same device/channel and re-score.

**3.4 SC-004 — mute trial.** Device/system sound OFF (mutism dignity): every driving action still discoverable by color+shape+text+position; the warning/failure states still legible statically. Verify no interaction depends on hearing the earcon.

**3.5 SC-005 — device latency probe.** On the media speaker, trigger play/pause/skip-fwd/skip-back/rate-step and measure action→first audible sample; expect each ≤100 ms (the host oracle proved the bound; this leg re-checks it acoustically). Record all measurements in the trial sheet.

**3.6 SC-006 — Lente reduzida + exclusivity.** Visual pass (C4 header mark on the card, mark NOT on notification/lock-screen); confirm in-app stamp set (§SurfaceInventory), Ressonador Canônico absence on the device, and that no attempt reaches the excluded screens.

**3.7 SC-007 — TalkBack + freeze.** TalkBack ON: labels announce (10/10), stepping the rate announces «velocidade <rung>» via the live region (value NEVER painted), progress announces role/state, the 3 s reason line and failure states announce. Then enable system «Remove animations» (reduced motion): the pill freezes VISIBLE on the track (not gone), and all §3.7 announcements remain unchanged (10/10); record the freeze-leg result in the same sheet.

## 4. Privacy + scope glance

- Dependency diff: none (`Cargo.lock` untouched); no egress entries expected in `diag` beyond 004's known set; the diagnostic record's privacy/trim behavior unchanged (typed kinds extended only).
- Haptics inert during §3.3 is verified by the disposition line (E3), not by settings: the system «feedback háptico» toggle does NOT gate direct `VibratorManager` calls — the pilot check + `haptic=None` in the mirror payload is the only authoritative disposition.
- Launcher icon: unchanged (D4 exclusion). Ressonador Canônico = host faces only (host CLI banners + repo artifacts). Remote = none; sideload install only.

## Trial sheets (minimal)

| metric | where recorded |
|---|---|
| SC-002 matrix | 7 types × answered/total; traps count; per-attempt list; replication count |
| SC-005 release latencies | trigger → ms (5 shown) |
| SC-007 responses | 10 announcements + freeze-leg re-check |
