# Idea Research: 016-voz-lula

- **Slug**: voz-lula
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: medium (feature idea high; asset/legal path low)

## Users & Demand

- A recognizable pt-BR voice honouring a sitting public figure is framed as a cultural/accessibility differentiator for Brazilian readers — [source: `specs/016-voz-lula/spec.md` US1] (confidence: medium)

## Prior Art

- `faber` (CC0) is the current default voice; `pt_BR-faber-medium` assets define the handoff convention (mono f32 @ 22 050 Hz) and directory layout reused by the proposed `lula` voice — [source: spec 016 FR-02/FR-05] (confidence: high)
- Voice selection seam: `--voice` CLI flag and Tauri `set_voice` bridge are the planned configuration points (FR-04). [source: spec 016]
- Related assessments `custom-voices` and `sona-inspired-voice` cover adjacent voice-customization ideas. [source: `.specify/assessments/`]

## Market & Context

- Cost of doing nothing: no cultural-differentiator voice; core UVP (offline Markdown reading, durable resume, SONA) does not depend on it.

## Data & Constraints

- No `.onnx`/`tokens.txt` in repo; only placeholder metadata existed and was removed 2026-11-10 because it broke the `exactly_one_bundled_voice_directory` constitution gate. [source: `specs/016-voz-lula/decision.md`]
- Constitution pins exactly one bundled voice (`pt_BR-faber-medium`, CC0) at v1; `test_noscope` gate enforces it — implementation needs a constitution amendment + gate update. [source: `specs/016-voz-lula/decision.md`]
- Offline-first (Principle IX): no network fetch at synthesis time.

## Evidence Against the Idea

- **Legal/ethical exposure (HIGH)**: LGPD treats voice as sensitive biometric data (art. 5º II, art. 11 specific consent); STJ protects voice as personality right (Súmula 403 analogue); PL 1530/2026 would require express documented consent; TSE Res. 23.755/2026 restricts synthetic voice of candidates/public figures; Google Play/Apple policies cover voice recordings of real people. [source: `findings/2026-11-10-next-cycle-research.md` §4 via decision.md] (confidence: high)
- No autonomous acquisition path for the model (training from public-domain audio or donation required); not executable without external consent/data. [source: decision.md]

## Gaps & Open Questions

- **Q1**: model via training (Fish Speech / GPT-SoVITS + Piper export) or pre-existing model? Pending.
- **Q2**: reuse faber `espeak-ng-data` tree or dedicated tree? Recommended: reuse (same pt-BR language).

## Sources

- `specs/016-voz-lula/{spec.md,plan.md,tasks.md,decision.md}` (host: local repo, policy: allowlisted path)
- `.specify/assessments/{custom-voices,sona-inspired-voice}/` (host: local repo)
- `findings/2026-11-10-next-cycle-research.md` §4, §9.1(2) (host: local repo)
