# Decision: PocketTTS Voice Cloning Spike (020)

- **Slug**: pockettts-voice-cloning-spike
- **Decided**: 2026-11-15 (artifact backfilled 2026-11-20)
- **Verdict**: go for the measurement spike (executed); **NO-GO for integration** per `measurements.md`
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md | `PLAN-80-20.md` | `measurements.md`

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | SD680 TTFA was the single unverified blocker for a Ressoa cloned voice |
| Evidence strength | strong | On-device matrix (9 configs × cold/warm) with device/binary/bundle/hash/commands recorded |
| Value vs. inaction | strong | Early negative result prevents a TTFA regression and wasted integration work |
| Feasibility / appetite | strong | Spike is external to production; small harness, no contract changes |
| Strategic fit | strong | Offline/zero-INTERNET and Rust authority preserved; Piper default kept |
| Risk posture | strong | Reversible; licensing/consent treated as hard precondition |

## Verdict & Rationale

`go` to run the device-first spike (Option A) — and, after execution, **NO-GO** to integrate Ressoa via PocketTTS on the measured device/runtime/model profile. All measured dimensions violate the gate: `first_chunk_ms` 9804–26113 ms cold / 6690–10599 ms warm vs ~1551 ms (3000 ms bound), `RTF` 2.035–6.565 (never < 1), `model_load_ms` 2994–3937 ms and `VmHWM` peak ~1.18–1.30 GB. The end-to-end metric was not executed and would be worse [INFERENCE]. Piper pt-BR stays default; cloning is deferred pending new evidence and explicit license/consent clearing.

## If go — Handoff to `/speckit-specify`

- **Problem**: unverified device feasibility of offline cloned pt-BR voice under TTFA/RTF/memory bar.
- **Chosen approach**: Option A — external C-API harness measurement + explicit gate, then integrate only on GO.
- **In scope (executed)**: reference/bundle selection, harness, measurement matrix, decision record, SDD artifacts.
- **Out of scope**: production integration, APK payload, third-party rights clearing, end-to-end app measurement.
- **Success metrics**: SC-PKT-001..006; recorded outcome NO-GO.
- **Carried-forward questions**: compatible bundle/runtime and license/consent — required before any revival; fallbacks T021/T022 remain available.
