# Decision: 016-voz-lula — DEFER (do not implement)

- **Spec**: `specs/016-voz-lula/` (draft, 2026-11-08)
- **Decided**: 2026-11-10
- **Verdict**: `defer` (assessment gate; not `go`)
- **Basis**: `.specify/assessments/` has no artifacts for this slug; the spec
  is a bare draft; `findings/2026-11-10-next-cycle-research.md` §4 (voice
  cloning ethics & licensing) and §9.1(2).

## Why defer

1. **No model, no autonomous path.** The feature needs a trained Piper VITS
   `pt_BR-lula-medium` model. The repo ships only placeholder metadata
   (`PROVENANCE.md`, `MODEL_CARD`, `.onnx.json`) and **no `.onnx`/`tokens.txt`**.
   Obtaining the model requires either training from voice data or a donation;
   neither is executable autonomously, and the placeholder tree was removed
   from the working tree on 2026-11-10 (it broke the `exactly_one_bundled_voice_directory`
   constitution gate).
2. **Legal / ethical exposure (research §4, HIGH confidence).** A synthetic
   voice of a real, sitting public figure in a distributable app is high-risk:
   - **LGPD** treats voice as sensitive biometric data (art. 5º, II; art. 11
     requires specific consent).
   - **STJ** jurisprudence protects voice as a personality right (Súmula 403
     analogue).
   - **PL 1530/2026** would require express, documented consent for synthetic
     voice.
   - **TSE Res. 23.755/2026** restricts synthetic voice of candidates/public
     figures; the TSE–ElevenLabs "prohibited voices" list exists.
   - **Google Play / Apple** policies cover apps that create voice recordings
     of real people.
3. **Constitution conflict.** The constitution pins "exactly one bundled voice"
   (`pt_BR-faber-medium`, CC0) at v1; the `test_noscope` gate enforces it.
   Implementing 016 requires a constitution amendment + gate change, not just
   code.
4. **Low 80/20.** The feature is a novelty differentiator, not core reading
   value; the core UVP (offline, Markdown-native, durable resume, SONA) does
   not depend on it.

## If pursued later (conditions)

- Written, documented consent from the voice subject (or a clearly fictional
  CC0 voice instead).
- Licence + provenance + legal review before any model is bundled.
- Constitution amendment (voice set is no longer a single-member closed set)
  + `test_noscope` gate update + `deny`/license audit.
- Synthetic-content disclosure in the store listing.

## Disposition

- The draft spec/plan/tasks remain in `specs/016-voz-lula/` as a record.
- The placeholder asset tree is removed.
- No code, CLI, or bridge changes for 016.
