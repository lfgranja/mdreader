# Bug Assessment: playback stalls at end of document (no session_end)

- **Slug**: playback-stall-at-end-of-document
- **Created**: 2026-11-21
- **Source**: validação visual/device do ciclo 026 (APK `bf7971b6`+primário)
- **Verdict**: likely valid, needs reproduction in clean session
- **Severity**: medium-high

## Report
After ~199.2 s of continuous playback of "HAMLET trecho.md" (snapshot 3897 bytes, ends "Sai."), `action=progress` events froze at `position_secs=199.20331065760243` for >100 s. No `session_end`, no `action=complete`, no FATAL/ANR/underrun in logcat. MediaSession stayed `state=3` (PLAYING) with pause button active; waveform kept animating.

## Hypothesis
The 160-byte primer split (ciclo 026) may leave a dangling handover at end-of-document: the last segment finishes but `note_first_consumed_sample`/`complete` is never emitted. Could also be a pre-existing issue unrelated to the primer.

## Evidence
- `/tmp/device_captures/stall_199s.png` (UI still "playing")
- diagnostic-record.jsonl lines 780-787 (position frozen)
- `dumpsys media_session`: state=3, position=9190 (stale)
