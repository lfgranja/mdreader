# Idea Research: Ressoa Reading Voice (018)

- **Slug**: ressoa-reading-voice
- **Created**: 2026-11-10 (artifact backfilled 2026-11-20)
- **Evidence confidence (overall)**: high (repo-declared constraints), medium (auditory quality — not independently re-measured here)

## Users & Demand

- Readers of long-form pt-BR Markdown want a calm, melodic reading voice consistent with the Ressoa identity; today the only option is a single hardcoded voice — [source: `specs/018-ressoa-reading-voice/spec.md:9-12`] (confidence: high)
- Multi-voice selection/cycle/persistence is a stated product evolution need (shell verbs `CycleVoiceNext/Prev` already exist) — [source: `specs/018-ressoa-reading-voice/spec.md:16-33`] (confidence: high)

## Prior Art

- Current synthesis is Piper VITS via pinned `sherpa-onnx = "=1.13.8"`, voice `pt_BR-faber-medium`, handoff mono f32 @ 22,050 Hz, declared capabilities `(SsmlLevel::None, AlignmentGranularity::None)` — [source: `specs/018-ressoa-reading-voice/spec.md:9,40-44`; `specs/018-ressoa-reading-voice/plan.md:15-24`] (confidence: high)
- Upstream `rhasspy/piper-voices` supplies CC0 pt-BR voices at 22050 Hz; target `pt_BR-cadu-medium`, fallback `pt_BR-jeff-medium`; `pt_BR-edresson-low` explicitly excluded — [source: `specs/018-ressoa-reading-voice/spec.md` FR-018-002, §7] (confidence: medium — availability/config compatibility is an implementation validation)
- Existing surfaces requiring additive updates: `adapter-synthesis` CLI (`voices`, `synthesize --voice`), `crates/session` (`last_voice`), `src-tauri` diagnose/inventory/get/set/cycle — [source: `specs/018-ressoa-reading-voice/spec.md` FR-018-006..009] (confidence: high)

## Market & Context

- Cost of doing nothing: the DESIGN-RESSOA identity remains unexpressed in the voice actually heard by users; no evolution path for multiple curated voices.
- The change is deliberately offline: no `INTERNET` permission/capability is introduced (air-gapped posture preserved).

## Data & Constraints

- Closed deterministic catalog with stable ids equal to bundled directory names; no `Other`/`Unknown` variants. [source: FR-018-001]
- Provenance artifacts required per voice (`MODEL_CARD`, `PROVENANCE.md`, config, checksums, upstream license). [source: FR-018-003]
- Voice switch during playback must invalidate generation and restart at authoritative resume position; no mixed-voice PCM. [source: FR-018-009, US2]
- Missing/corrupt/incompatible assets or engine failure fail closed with named error — no silent substitution, resample or downmix. [source: NFR-018-002, US3]
- Host gates 1–8 (`scripts/verify.sh`) plus `scripts/expected-tests.txt` consistency are mandatory; device/gate-9 reported separately. [source: NFR-018-003]
- Non-imitative identity: no third-party character/performer branding in ids, labels, assets, docs. [source: FR-018-010]

## Evidence Against the Idea

- Bundling an extra ONNX voice increases APK size, extraction time and load/RAM cost; a single tuned voice may be the pragmatic 80/20 outcome. Mitigation: documented contingency (tuned `faber`-only catalog) with preserved multi-voice-ready API. — [inferred from `spec.md` FR-018-002/§7] (confidence: medium)
- Auditory quality ("harmonious, melodic, calm, soft") is subjective and not deterministically gateable; mitigation: recorded comparative evidence and default-choice rationale (SC-018-005). — [inferred] (confidence: medium)

## Gaps & Open Questions

- Which alternative voice (if any) passes quality/runtime/packaging validation — resolved during implementation per FR-018-002 contingency.
- Engine caching vs recreate-on-switch under the pinned runtime — resolved during implementation per FR-018-005.

## Sources

- `specs/018-ressoa-reading-voice/spec.md` (host: local repo, policy: allowlisted path)
- `specs/018-ressoa-reading-voice/plan.md` (host: local repo)
- `DESIGN-RESSOA/` design-system documents (host: local repo)
