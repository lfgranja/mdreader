# Release (R7): 016-voz-lula — TTS voz Lula da Silva

- **Date**: 2026-11-20
- **Version**: no release (design artifact set only; no app SemVer bump)
- **Validation**: R4 DEFERRED (`.specify/validations/voz-lula/validate.md`)
- **Implementation decision**: `defer` (`specs/016-voz-lula/decision.md`, 2026-11-10)

## Summary

Completes the SDD artifact set (assessment `go` for specification, checklist, validation, release) for the proposed selectable pt-BR "Lula" TTS voice. **No code, CLI, bridge, model asset or configuration is shipped.** The feature remains deferred pending consent/licensing, legal review and a constitution/gate amendment.

## Changelog (IDs)

- **Added (docs only)**: `.specify/assessments/voz-lula/{intake,research,problem,concept,decision}.md` (decision `go` for specification), `specs/016-voz-lula/checklists/requirements.md`, `.specify/validations/voz-lula/validate.md`, this release record.
- **Specified but not delivered**: FR-01..05 (`Voice` enum, canonical 22 050 Hz mono f32 handoff, fail-closed `InitFailed`, CLI `--voice` + Tauri `set_voice`, `pt_BR-lula-medium` asset convention); SC-001..005 not demonstrated.
- **Removed**: nothing — placeholder voice asset tree was already removed on 2026-11-10 (it broke the single-voice constitution gate).
- **Unchanged**: default `faber` voice, `exactly_one_bundled_voice_directory` / `test_noscope` gates, offline-first synthesis.

## Rollback

No runtime change to roll back. Withdraw the design artifacts if the feature is formally cancelled; no persisted data, remote configuration or voice assets are affected.

## Residual risk / [NV]

- High legal/ethical exposure (LGPD, STJ, PL 1530/2026, TSE Res. 23.755/2026, store policies) for a synthetic voice of a sitting public figure — see research.md.
- No model or autonomous acquisition path; no host gates, device validation or CI runs are claimed for this deferred feature.
