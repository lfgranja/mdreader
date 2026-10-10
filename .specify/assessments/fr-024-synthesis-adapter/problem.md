# Problem Definition: FR-024 Síntese Mínima Viável — Adaptador sherpa-onnx + Piper pt-BR

- **Slug**: fr-024-synthesis-adapter
- **Created**: 2026-10-06
- **Inputs used**: intake.md | research.md

## Problem Statement

The `mdreader` Android media shell (`001-android-media-shell`) has fully specified and partially implemented playback authority (Rust `cpal`/AAudio, `MediaSession` mirror, focus duck/pause, resume by `blake3` fingerprint, diagnostic JSONL), but it relies solely on stand-in engines (`standin_a`/`standin_b`). Without a real synthesis adapter (`FR-024`), no Markdown document can be converted to audio, which means the driving-first P1 use cases (locked-screen continuous playback, navigation-downgrade, headphone-pull privacy) cannot be demonstrated to an actual user — the shell is a mechanism proof (`SC-051`) that never becomes a product.

## Affected Users & Stakeholders

- **Users (primary)**: drivers / commuters who need to listen to Markdown documents aloud with screen off (Principle VIII / `spec.md` US1-US3). They experience the gap as: "I can open the app, but there is no sound."
- **Stakeholders (decision / funding)**: project maintainer / spec author — needs `SC-044` (two real engines) and `gate-8-full` (byte budget + SSML + sentence-alignment) to declare the feature complete; the constitution records `gate-8-full: DEFERRED (owner=synthesis)` (`constitution.md` line 643).
- **Stakeholders (platform / compliance)**: Android OEMs (Samsung One UI 7, Xiaomi HyperOS) — notification `MediaStyle` reliability (R2, R3, R4) is independent of synthesis quality, but a broken synthesis experience makes the notification surface useless regardless.

## Goals

- Enable the `mdreader` shell to produce real offline mono-f32 audio at 22050 Hz from Markdown input, using the already-selected engine (`sherpa-onnx`) and voices (`Piper` pt-BR `faber`/`cadu`/`jeff`).
- Allow verification of the P1 driving scenarios (`US1-US3`) with actual audio rather than stand-in dummy output.
- Preserve every existing contract (`FR-013` focus, `FR-015` buffering, `FR-036` resume by fingerprint, `FR-039` diagnostic record, `FR-035` named failure) without weakening the closed-set rules (`constitution.md` §VII, gate 7).

## Non-Goals

- **Not a redesign of shell `001`**: the playback authority, focus response, notification mirror, and resume logic are already specified; this problem is only about the missing engine adapter.
- **Not cloud synthesis as primary**: offline `sherpa-onnx` + `Piper` is the declared default (`constitution.md` §Engine Registry); cloud escalation (`Google TTS`, `Azure`, `Edge`) is an enhancement layered on top (`constitution.md` line 343), not in scope for the minimum viable adapter.
- **Not perfect segmentation / SSML initially**: basic `pulldown-cmark` + `unicode-segmentation` with pt-BR abbreviation guard is acceptable for the first adapter (`intake.md`); `SentenceBoundary` / `WordBoundary` improves but is not required to prove the mechanism.
- **Not in-app diagnostic viewer or document picker**: those are explicitly deferred (`spec.md` line 51, `plan.md` line 18); they depend on synthesis working first.
- **Not a second engine registry**: no new list of voices or tiers; only the already-declared `offline_default` (`sherpa-onnx` + `Piper`) and `test_double` (`standin_a`/`standin_b`) are involved.

## Success Metrics

- **Measurable**: `SC-044` passes — two real working engines (`Piper` offline + stand-in or second real) demonstrated on `aarch64-linux-android` reference device (mid-range arm64, 6 GB RAM, `spec.md` line 50).
- **Measurable**: `SC-009` passes — ≤3 s from Play press to audible audio with real synthesis (vs. zero-cost stand-in today).
- **Measurable**: `SC-051` continues passing — failover mechanism still works when real adapter is integrated (no regression in `player` / `engine-core`).
- **Qualitative (labeled)**: A blind test (`SC-008`) with the lock-screen notification shows the document title + current section title + progress, and the audio responds correctly to transient focus loss (~20% duck) and permanent loss (pause + reason, no auto-resume).
- **Baseline (current)**: `SC-051` passes (mechanism verified with stand-ins); `SC-044` deferred; `SC-009` untested with real synthesis; `gate-8-full` deferred (`constitution.md` line 643); `SC-050` (byte-budget / alignment) unverified.

## Cost of Inaction

If this problem is never solved:
- The `mdreader` shell remains a verified mechanism (`SC-051`) but never a usable driving product (`US1-US3` unprovable with real users).
- `SC-044` and `gate-8-full` stay deferred permanently; the constitution's `Engine Registry` remains unimplemented for the offline default.
- The 30-day diagnostic record (`FR-039`) writes events (transport changes, focus transitions) but cannot diagnose failures of synthesis — the most likely failure mode for a new user — because there is no synthesis to fail.
- Investment in `player`, `session`, `diag`, and `engine-core` traits yields no external value until the adapter connects Markdown → audio.

## Open Questions

- [NEEDS CLARIFICATION: What is the measured latency of `sherpa-onnx` + `ort` (`NNAPI`) on the reference device for the first chunk at 22050 Hz mono? `SC-009` (≤3 s) may or may not be achievable without pre-loading.
- [NEEDS CLARIFICATION: Does `Piper` expose any sentence-boundary events, or is output fully continuous? If continuous, does basic `pulldown-cmark` segmentation satisfy the resume-lock contract (`FR-036`, `FR-023a`) firmly enough?
- [NEEDS CLARIFICATION: How does the 5,000-byte chunk budget (`constitution.md` line 353) interact with Markdown sections that exceed the limit — is splitting before synthesis required, and does it affect resume position alignment?
- [NEEDS CLARIFICATION: Should the adapter implement `FR-035` (named failure for format mismatch) in the first cycle, or is it acceptable to ship audio first and refine failure naming once the contract is exercised?
- [NEEDS CLARIFICATION: If `SC-051` (stand-in mechanism) must remain passing during integration, what is the exact integration order (adapter first, then real-engine verification, or parallel branches) to avoid `player` regression?
