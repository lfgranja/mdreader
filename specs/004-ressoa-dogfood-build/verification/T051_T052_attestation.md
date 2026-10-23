# T051 / T052 — On-device acceptance & diagnostic-record audit

**Feature**: `004-ressoa-dogfood-build`
**Reference phone**: Redmi 2201117TY (`spesn`), Android 13 / API 33, serial `53cf9aa7`
**Date**: 2026-10-22
**Operator**: AI agent (OpenCode) — automated device-side steps only

## Device-side steps performed

1. Sideloaded release APK `app-universal-release.apk` (sha256 `2e3555f377dceab462acbda7187d355efc1eed3c1cf8389ddddab9529ece08f5`) via `adb install -r` — success.
2. Launched app (`app.mdreader/.MainActivity`) — app data directory created at `/data/data/app.mdreader`.
3. Pulled the diagnostic record via `adb shell run-as app.mdreader cat ...`:
   - Source: `/data/data/app.mdreader/diagnostic-record.jsonl`
   - Local: `/tmp/opencode/diagnostic-record.jsonl`
   - Size: **0 bytes (0 lines)** — no sessions were run on the device, so the record contains no events.
4. Ran `diag scan` (T052) on the pulled file:
   - Command: `diag scan /tmp/opencode/diagnostic-record.jsonl`
   - Output (JSON): `{"violations":[]}`
   - Output (human): `capability: scan` / `violation-count: 0`
   - Exit status: `0`
5. Restored release APK on the device (debug build removed).

## T052 result

- **diag scan**: PASS on the pulled record (0 violations, exit 0).
- Caveat: the pulled record is empty (no on-device sessions were exercised), so this scan attests only that the CLI and empty-file path work and that no prose is present vacuously. It does NOT exercise schema conformance on real session entries, `Trim`/`Gap` markers, or `error`-on-failure/refusal (no such entries exist in the empty record).

## T051 result — NOT ATTESTED

The SC-001…SC-010 criteria (and the §3.11 lock-screen privacy inspection) are
perceptual / manual trials defined in `tests/device/trials-004.md` (30-minute
locked commute sessions, blind confirmations, interruption classes, replay
caps, lock-screen purity inspections, rate persistence, offline sessions,
in-app purity). They require a human operator with the device in hand (audio
perception, blind tester, physical interruption injection, screenshots).

This agent cannot:
- perceive audio or perform blind confirmations,
- inject physical interruptions (calls, headphone removal, permanent loss),
- hold a 30-minute locked session and judge continuity,
- capture/inspect lock-screen screenshots without vision.

Therefore **T051 is not attested by this run**. No per-SC 10/10 verdict is
claimed. The tasks.md checkboxes for T051 remain unchecked; T052 is marked
complete for the empty-record scan only, with the caveat above.

## diag scan output (attached)

```
$ diag scan /tmp/opencode/diagnostic-record.jsonl
{"violations":[]}
exit=0

$ diag scan --format human /tmp/opencode/diagnostic-record.jsonl
capability: scan
violation-count: 0
exit=0
```

## Stop-the-line

None observed in the automated steps. Perceptual trials unrun — no bound
renegotiation attempted.
