# Idea Research: Local free emulator for reference-device visual verification

- **Slug**: ref-device-emulator
- **Created**: 2026-10-29
- **Evidence confidence (overall)**: medium (high on emulator capabilities/limitations from official docs; medium on whether this accelerates 003 converge, because 003 has no `tests/` own and `verify.sh` baseline unconfirmed; low on whether it replaces physical device — evidence is clear it does not)

## Users & Demand

- **Internal user / pipeline**: spec `003-reference-device` (Draft; plan/tasks/checklists/contracts/data-model present; `tests/` absent; `verification` absent; `cleanroom-core` 0 .rs) needs an evidence path to `gate-8-full`; the process demands `verify.sh` green before converge (constitution gate 8; `AGENTS.md` §verify; `specs/003-reference-device/plan.md`). — [cited: internal artifacts `specs/003-reference-device/plan.md`, `scripts/verify.sh`, `.specify/memory/constitution.md` v1.12.0; confidence: high]
- **No external user signal**: no support tickets, no sales asks, no usage data cited for an emulator layer; the request is process-driven (reduce physical-device dependency for `003` verification). — [ASSUMPTION: process-driven; no external source found; confidence: medium]
- **Trigger event**: session turn after `005-adopt-design-sona` 80/20 plan execution (branch `005-adopt-design-sona`, HEAD `0c985ee`, `cargo check` interrupted `bg_14`); user asked deep research on local/emulator verification for reference device. — [cited: session state; confidence: high]

## Prior Art

- **Internal — spec pipeline**: `001-android-media-shell` (`VERIFICATION.md` + `gate-8-shell PASS`, `gate-8-full DEFERRED`); `004-ressoa-dogfood-build` (`verify.sh` GREEN `9501bba`, Phase 17 complete, 2026-10-22); `002-fr-024-synthesis-adapter` (`V3_ATTESTATION` + `CLARIFICATION_BATCH_RESOLVED` + `adapter-synthesis/` 12 .rs); `005-adopt-design-sona` (clarify resolved, `tasks.md` ready, `DEBUG` dirty 51 files). — [cited: `specs/*/`; `ROADMAP.md`; `AGENTS.md`; confidence: high]
- **Internal — assessment**: `directory-reading` trapped at `intake.md` only; 4/5 assessments complete (`design-son-adoption`, `fr-024`, `ressoa-mobile-app`, `lente-sinestesica-completa`). New assessment `ref-device-emulator` registered (this turn, `.specify/assessments/ref-device-emulator/intake.md`). — [cited: `.specify/assessments/`; confidence: high]
- **Internal — design-system**: `DESIGN-SONA/` assets added; tokens/paleta defined; `005` adoption partial (`src-tauri/lib.rs`, `status_view.html`, `surface_audit/inventory.rs` dirty). — [cited: `DESIGN-SONA/assets/`; `ROADMAP.md`; confidence: high]
- **Internal — bug triage**: 5 dirs in `.specify/bugs/`; only `sona-waveform-missing-in-screenshot` has `assess.md` + `fix.md` + `test.md`; 2 empty; 2 assess-only. No regression demand for emulator layer. — [cited: `.specify/bugs/`; confidence: high]
- **External — spec-kit / speckit workflow**: `github.io/spec-kit` confirms `specify → clarify → plan → tasks → implement → converge`; assessment `intake → research → define → shape → decide`. The research stage explicitly demands evidence-gathering before design, matching this turn. — [sanitized: `https://github.io/spec-kit/quickstart.html`; host: `github.io`; policy: allowlisted; confidence: high]
- **External — Android manifest / Tauri**: `developer.android.com` confirms `<service android:foregroundServiceType="mediaPlayback">` + `FOREGROUND_SERVICE*` permissions required; `mobile::update_manifest` does not exist in Tauri 2.12.1; edit must be direct (as constitution and 001/004 specs state). Emulator supports the same manifest framework (API 29+ AVD). — [sanitized: `https://developer.android.com/develop/background-work/services/fgs/declare`; `https://developer.android.com/studio/run/emulator`; host: `developer.android.com`; policy: allowlisted; confidence: high]
- **External — Emulator limitations (critical)**: StackOverflow (`stackoverflow.com`) confirms emulator does NOT support Bluetooth (`https://stackoverflow.com/questions/22604305`); SDK docs (`developer.android.com/studio/run/emulator`) document general issues (no Bluetooth, limited audio routing, no real telephony/alarm interruption simulation). — [sanitized: `https://stackoverflow.com/questions/22604305`; `https://developer.android.com/studio/run/emulator`; host: `stackoverflow.com`/`developer.android.com`; policy: allowlisted; confidence: high]
- **Prior failure / precedent**: No prior attempt at `003` verification via emulator found in repo (only `spec.md`, `plan.md`, `tasks.md` exist; `tests/` missing; `cleanroom-core` 0 .rs; `verify.sh` not confirmed green on current tree). The only verified green build is historical `9501bba` (`004` dogfood). — [cited: `specs/003-reference-device/`; `MDREADER_VERIFIED_CLOSE_PLAN.md`; confidence: high]

## Market & Context

- **Alternative users rely on today**: physical Android device (API 29+) connected to `adb`; `verify.sh` gates 1–8; `reference-device` spec defines device-only verification (`SC-009` ≤3 s play-to-audio; `SC-035` Kotlin mirror only). No existing emulator layer in repo. — [cited: `specs/003-reference-device/`; `AGENTS.md`; confidence: high]
- **Cost of doing nothing**: `003` remains unimplemented (no `tests/`); `001` `gate-8-full` stays `DEFERRED` (owner=synthesis); `004` green is historical only; current working tree (`005-adopt-design-sona`) has 51 dirty files and interrupted `cargo check`; no verified build path exists. The emulator layer would not by itself fix these, but could accelerate structural verification of 003. — [cited: `git status`; `bg_14`; `ROADMAP.md`; confidence: high]
- **Trend / ecosystem**: Android Emulator is the official SDK tool; widely used for media-service testing; free; no licensing barrier; integrates with `adb` (emulator connects as `adb devices`). The limitation is not tooling — it is physical hardware fidelity (audio, focus, Bluetooth, on-device trial). — [cited: `developer.android.com`; confidence: high]

## Data & Constraints

- **Platform limits (verified)**: Android Emulator supports `mediaPlayback` foreground service type (same manifest), `MediaSessionService`, notifications, audio focus APIs (post-API 31 managed; pre-API 31 encouraged — not identical to real interruption), but **not** real Bluetooth, not real hardware audio conversion to device format, not real telephony/alarm interruption, not `cpal`/`AAudio` at declared 22050 Hz. — [cited: SDK docs + StackOverflow; confidence: high]
- **Rate ladder / engine contract**: `crates/engine-core/src/closed.rs` + `standin_a.rs` declare 7-rung ladder; adapter delivers mono f32 @ 22050 Hz; engine resamples; rate-change is engine-contract op; mismatch = named `EngineError`. Emulator cannot verify the device-format conversion step (only structural handshake). — [cited: `Cargo.toml`; `specs/002-fr-024-synthesis-adapter/`; `AGENTS.md`; confidence: high]
- **Diagnostic record**: `diag` 5 .rs files; `EventKind` 9 variants; `blake3` fingerprint; 30-day / 10 MB trim; never transmitted. Emulator can write JSONL locally; physical device required for replay after power-loss (≤30 s replay bound). — [cited: `crates/diag/src/`; `specs/001-android-media-shell/data-model.md`; confidence: high]
- **Closed-set / gate rules**: no `Other`/`Unknown`; no `_ =>`; `unwrap/expect/panic` denied (`expect` attribute allowed with reason); `clippy::all,pedantic` deny. Emulator does not relax these. — [cited: `AGENTS.md`; `Cargo.toml`; confidence: high]
- **Supply chain / build**: `vendor/sherpa-onnx-sys/` vendored (exception log entry 12); `deny.toml`; `cargo audit`; `Tauri` pinned `2.12.1`. Emulator layer introduces no supply-chain change (official SDK only). — [cited: `Cargo.toml`; `AGENTS.md`; confidence: high]

## Evidence Against the Idea

- **Emulator ≠ physical device**: the strongest counter-evidence is that `003-reference-device`'s purpose is to prove the shell works on a real Android device; an emulator verifies the same Android framework APIs but not the hardware-level behaviors (`SC-009` audio latency, `FOREGROUND_SERVICE_MEDIA_PLAYBACK` with real car Bluetooth, `ACTION_AUDIO_BECOMING_NOISY`, real focus loss with telephony). The idea risks giving false confidence if presented as replacing the physical gate. — [courced: SDK docs + StackOverflow + 001/003 spec clarifications on device behavior; confidence: high]
- **No external demand**: this is pure process optimization (verification pipeline), not a user-facing feature. Building a full emulator verification layer (scripts, docs, attestation) consumes effort that could go to `SC-044` (consent/cloud TTS) or `003` implementation directly. — [cited: intake source (user research ask); `ROADMAP.md`; confidence: medium — no market/usage data exists]
- **Maintenance burden**: adding `VERIF-EMULATOR` creates a second verification path that must stay synchronized with `verify.sh` (gates 1–8) and `constitution.md` changes; if `verify.sh` evolves, the emulator layer could diverge silently. — [ASSUMPTION: maintenance cost; no source quantifying divergence risk; confidence: low — must clarify magnitude]
- **Cleanroom-core empty (0 .rs)**: the workspace lacks a core abstraction layer; any emulator-specific verification logic might have no clean home, risking placement in `crates/player`, `crates/session`, or `src-tauri` ad hoc. — [cited: `ls crates/cleanroom-core/src/`; confidence: high]
- **Not needed to unblock 001**: `001` `gate-8-full` is deferred until `002` `SC-044` closes (second engine); emulator verification of 003 does not by itself unlock `SC-044`. — [cited: `specs/001-android-media-shell/spec.md`; `specs/002-fr-024-synthesis-adapter/spec.md`; `ROADMAP.md` line 27; confidence: high]

## Gaps & Open Questions

- [NEEDS CLARIFICATION: Qual emulador específico? Android SDK AVD (recomendado pelo docs) ou alternativa (Genymotion, BlueStack, Waydroid)? Cada um tem limitações distintas de áudio/foco/Bluetooth.)]
- [NEEDS CLARIFICATION: A camada `VERIF-EMULATOR` deve ser um documento/attestation adicionada a `003` (como `verification.md`), ou um script de CI (`verify-emulator.sh`)? Impacto na manutenção.] 
- [NEEDS CLARIFICATION: O usuário quer que o emulador substitua o físico para `gate-8-full`, ou apenas acelerifique a verificação estrutural antes do físico? A resposta muda completamente o esforço e o risco de falso-positivo.] 
- [NEEDS CLARIFICATION: Há restrição de hardware/CI para rodar AVD (memória, GPU, velocidade)? O workspace está em `005-adopt-design-sona` com `dirty` 51 arquivos e `cargo check` interrompido; adicionar AVD pode agravar o ambiente.] 
- [NEEDS CLARIFICATION: A ideia deve ser tratada como novo `assessment` (`ref-device-emulator`) ou apenas como nota/verificação dentro de `003-reference-device`? A intake já criou o assessment; pode ser revertido.] 
- [NEEDS CLARIFICATION: Qual critério de sucesso do `VERIF-EMULATOR`? `verify.sh` exit 0 no AVD? Attestation separada? Benchmark `SC-009` no AVD (que não será fiel)? Esses critérios definem se vale o esforço.] 

## Sources

- `https://github.io/spec-kit/quickstart.html` (host: `github.io`; policy: allowlisted; confidence: high) — speckit workflow confirmation
- `https://developer.android.com/develop/background-work/services/fgs/declare` (host: `developer.android.com`; policy: allowlisted; confidence: high) — manifest / FGS / `mediaPlayback`
- `https://developer.android.com/develop/background-work/services/fgs/service-types` (host: `developer.android.com`; policy: allowlisted; confidence: high) — service-type constants
- `https://developer.android.com/studio/run/emulator` (host: `developer.android.com`; policy: allowlisted; confidence: high) — emulator capabilities / limitations
- `https://stackoverflow.com/questions/22604305` (host: `stackoverflow.com`; policy: allowlisted; confidence: high) — Bluetooth not supported in emulator
- `https://developer.android.com/guide/topics/media-apps/audio-focus` (host: `developer.android.com`; policy: allowlisted; confidence: high) — audio focus management (post-API 31)
- Internal: `specs/003-reference-device/plan.md`, `specs/003-reference-device/spec.md`, `specs/005-adopt-design-sona/tasks.md`, `specs/001-android-media-shell/spec.md`, `.specify/assessments/ref-device-emulator/intake.md`, `AGENTS.md`, `ROADMAP.md` (updated 2026-10-29), `Cargo.toml`, `.specify/memory/constitution.md` v1.12.0 — all cited directly, confidence high

## Verification Note

No new source files created by this research. `research.md` written inside `.specify/assessments/ref-device-emulator/`. Previous `ROADMAP.md` edits (bugs, checklist, 80/20) preserved independently. `cargo check` not claimed PASS (`bg_14` interrupted); verification deferred to T002 per `005-adopt-design-sona/tasks.md` protocol if 003 implementation proceeds.
