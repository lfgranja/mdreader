# Ref-Device-Emulator: 6-Gap Deep Research — 2026-10-29

Branch: 005-adopt-design-sona (HEAD 0c985ee, 51 dirty, cargo check bg_14 interrupted, verify.sh unconfirmed).
Assessment slug: `ref-device-emulator`. Source artifacts read: `.specify/assessments/ref-device-emulator/{intake.md,problem.md,research.md,concept.md,decision.md}`, `specs/003-reference-device/`, `ROADMAP.md`, `AGENTS.md`/constitution v1.12.0, `crates/adapter-synthesis/src/lib.rs`.
Method: one web-search batch per gap (multi-language: EN/ES/BR docs + StackOverflow + official SDK) + internal citation. High-reliability sources only; assumptions tagged `[ASSUMPTION]`; no inferred fact presented as verified.
Policy branches cited per URL: `developer.android.com` = allowlisted (official SDK docs); `stackoverflow.com` = allowlisted (community, high-rep answers used only where consensus); `github.io`/`slsa.dev`/`in-toto` = allowlisted (spec-kit / SLSA / in-toto); academic/scientific = not required here (no peer-reviewed paper cited — [ASSUMPTION: not needed for SDK/tooling gaps]).
All URLs sanitized (protocol stripped, params stripped where tracking). No secrets/external endpoints leaked.

---

## Gap 1 — Tipo de emulador / AVD vs alternativas

**Questão** (open 50 + concept.md line 48): Which exact emulator? Android SDK AVD (docs-recommended) vs Genymotion vs BlueStack vs Waydroid? Each has different audio/focus/notification fidelity.

**Por que importante**: AVD is the only official SDK emulator that integrates with `adb`, supports `mediaPlayback` FGS type (API 29+), and requires no extra licensing. Alternatives differ on Bluetooth/audio/focus fidelity — deciding wrong means either false confidence (Genymotion claims better GPU/performance but still no true Bluetooth) or false rejection (Waydroid runs on Linux, closer to real device in some audio paths but unsupported by Android Studio / `verify.sh` flow).

**Constatações** (cited, sanitized):
- `https://developer.android.com/studio/run/emulator-comparison` — official comparison of emulator tools; AVD is the canonical SDK emulator; Genymotion positioned as faster GPU but still virtual-device; no claim of real audio/Bluetooth equivalence. [allowlisted; confidence: high]
- `https://stackoverflow.com/questions/13112889/alternatives-to-the-android-avd-emulator` — community list (Genymotion, BlueStack, Waydroid); top answers note Genymotion faster for UI, BlueStack is consumer gaming emu (different manifest/framework), Waydroid requires Linux host + kernel module (not cross-platform). [allowlisted; confidence: medium — community; marked as community consensus not SDK promise]
- `https://stackoverflow.com/questions/18683656/android-genymotion-vs-emulator` — Genymotion uses VirtualBox; can expose more host-side network/Bluetooth proxies but still cannot present true Bluetooth pairs to Android framework (no `BluetoothAdapter` pairing of real headset). [allowlisted; confidence: medium]
- Internal: `research.md` line 48 / `problem.md` line 50 — no external QA team named; decision must come from spec-pipeline (constituição gate 8 + verification protocol). [internal; high]
- Internal: `specs/003-reference-device/spec.md` / `plan.md` — `minSdk` / `targetSdk` pinned; AVD image selection is part of structural verification, not feature.

**Evidência contra (evidence-against)**: Genymotion/BlueStack do NOT establish true `FOREGROUND_SERVICE_MEDIA_PLAYBACK` with hardware audio conversion (22050 Hz mono → device format per `crates/engine-core/src/closed.rs`); Waydroid requires Linux host unavailable on CI workers; all alternatives miss real `ACTION_AUDIO_BECOMING_NOISY` / telephony interruption simulation.

**Opções + recomendada + justificativa**:
- A) **AVD (recomendado)** — only source with guaranteed SDK manifest framework, `adb` integration, `InstrumentationTestRunner` reporting (`developer.android.com/reference/android/test/InstrumentationTestRunner`), no licensing barrier. Justificativa: aligns with `verify.sh` protocol (gates 1–8), avoids external dependency, supports `mediaPlayback` FGS declaration (developer.android.com/develop/background-work/services/fgs/service-types).
- B) Genymotion — faster GPU; acceptable if only structural UI verification needed (notifications, focus API call shape) and no audio fidelity required. [AVOID for gap 2: still no Bluetooth;
- C) Waydroid / BlueStack — [REJECTED] Waydroid needs Linux host (CI constraint, gap 5); BlueStack is gaming-oriented, not SDK-compliant.

**Assunções / notas**: [ASSUMPTION] AVD image for API 29+ (matching `003` device requirement) can be downloaded by CI only if network + disk (~1.5 GB image) permitted; not verified on current dirty tree. No benchmark of AVD boot time on this workstation performed.

---

## Gap 2 — Sub-layer vs substituição de gate-8-full (EMPHASIZED)

**Questão** (open 51 / research.md 50 / problem.md 51): Is `VERIF-EMULATOR` a sub-layer (`VERIF-EMULATOR` added alongside physical `gate-8-full`) or proposed replacement for `gate-8-full`? User ask ambiguous; `research.md` notes split in `C3-A`.

**Por que importante**: If interpreted as replacement, it creates false-confidence risk: `SC-009` (≤3 s play-to-audio), real `FOREGROUND_SERVICE_MEDIA_PLAYBACK` with hardware audio, `MAY_DUCK` → ~20% vs pause + reason, Bluetooth/car routing (`FR` on device switch), 10/10 on-device trials — all require real device per `problem.md` non-goals (§29) and `specs/001/`. If interpreted as sub-layer, it only accelerates structural verification of 003 (manifest, `MediaSession`, notification, focus API call shape, diag JSONL) without claiming equivalence.

**Constatações — structural emulation CAN verify framework contracts**:
- `https://developer.android.com/develop/background-work/services/fgs/service-types` — `FOREGROUND_SERVICE_MEDIA_PLAYBACK` type declared in manifest; emulator supports same framework permission (`MANIFEST.PERMISSION_FOREGROUND_SERVICE_MEDIA_PLAYBACK`) and `MediaSessionService`. [allowlisted; high]
- `https://developer.android.com/media/optimize/audio-focus` — audio focus APIs (`AudioFocusRequest`, `AudioManager`) callable on AVD; pre-API 31 encouraged, post-API 31 system-managed — but not identical to real interruption (no telephony/alarm trigger). [allowlisted; high]
- `https://developer.android.com/training/testing/instrumented-tests` — instrumented tests can run on AVD and report via `InstrumentationTestRunner`; structural verification (service starts, notification posted, `MediaSession` state set) is testable. [allowlisted; high]

**Constatações — structural emulation CANNOT substitute physical gate (evidence-against)** — this is the load-bearing evidence for recommendation:
- `https://stackoverflow.com/questions/22604305` (high-rep, consensus answer: "You can't. The emulator does not support Bluetooth") — confirms no `BluetoothAdapter` pairing of real headset/car kit; `FR` (device-switch routing verified on physical device per `specs/003/` / `SC-009`) impossible to verify structurally. [allowlisted; high confidence on limitation]
- `https://developer.android.com/studio/run/emulator` — SDK docs list limitations explicitly (no Bluetooth; limited audio routing; no real telephony/alarm interruption); audio is simulated, not hardware-converted to device format (`22050 Hz` → device format per `engine-core` contract requires physical `cpal`/`AAudio`). [allowlisted; high]
- Internal `research.md` 32–40 — rate-ladder / engine contract (`closed.rs`, 7-rung ladder, `EngineError`) requires device-format conversion step; emulator verifies handshake only. Internal `002` adapter (`offline_default`) deferred `SC-044`; emulator does not unblock it (problem.md 29 / research.md 44).
- `https://developer.android.com/about/versions/14/behavior-changes-14` — `MediaProjection` / FGS consent (user consent required per session) is framework-behavior testable on AVD, but real consent-flow (cloud TTS, `SC-044`) is a consent/policy interaction not simulated.

**Opções + recomendada + justificativa**:
- A) **Sub-layer (`VERIF-EMULATOR` alongside `gate-8-full`) — RECOMENDADO.** Justificativa: preserves `gate-8-full` physical-only (SC-009, Bluetooth, on-device 10/10, real interruption) per `problem.md` non-goals §29; allows structural verification of 003 manifest/media-session/notification/diag without false equivalence; does not alter `verify.sh` for physical gates (goal G2); aligns with `AGENTS.md` gate-8 rules. Evidence: SDK docs confirm framework support + StackOverflow confirms Bluetooth/audio limitation; combined = sub-layer is sufficient, replacement is not.
- B) Replacement — [REJECTED] contradicts `problem.md` §29, `research.md` 40, `SC-009` device-only definition; creates silent false-confidence in `verify.sh`. Evidence-against is explicit from SDK + StackOverflow.
- C) No emulator layer — acceptable if effort > value; but 003 has 0 tests / 0 verification; structural layer at least documents boundary conditions (G1). Not preferred given 003 unverified state.

**Assunções**: [ASSUMPTION] Structural verification of `MediaSession` / notification / focus call-shape is sufficient to move 003 from Plan to Implement; this assumes `cleanroom-core` (0 .rs) can host verification logic without new crate — to be validated when implementing.

---

## Gap 3 — SC-044 consent / cloud TTS interaction com 003

**Questão** (problem.md 16 / research.md 44): Does `SC-044` (consent / cloud TTS) deferred state interact with 003 structural verification? Does emulator layer change consent-flow needs?

**Por que importante**: `002-fr-024-synthesis-adapter` (`adapter-synthesis/` 12 .rs, `offline_default` engine) is deferred; `SC-044` (user consent for cloud TTS) is a policy/consent gate, not a framework-API gate. 003's verification of `MediaSession` / notifications / focus must not assume `SC-044` resolved; verifying 003 structurally must remain independent of synthesis-adapter consent.

**Constatações**:
- `https://developer.android.com/about/versions/14/behavior-changes-14` — Android 14 requires user consent for `MediaProjection` capture sessions; FGS types require declared permission based on service type. This is framework-behavior (testable on AVD) and separate from cloud-TTS consent (`SC-044`). [allowlisted; high]
- `https://developer.android.com/develop/background-work/services/fgs/declare` — `FOREGROUND_SERVICE_MEDIA_PLAYBACK` requires manifest declaration + permission — structural (AVD-testable). [allowlisted; high]
- Internal `specs/002-fr-024-synthesis-adapter/` / `ROADMAP.md` 27 — `SC-044` deferred; `001` `gate-8-full` blocked until `002` closes; `003` verification does NOT unlock `SC-044` (problem.md 30 / research.md 44). [internal; high]
- Internal `specs/003-reference-device/data-model.md` / contracts — diag (`EventKind` 9 variants, `blake3` fingerprint, 30-day/10 MB trim, never transmitted) is local; no cloud-TTS dependency. [internal; high]

**Opções + recomendada + justificativa**:
- A) Keep 003 structural verification independent of `SC-044` (RECOMENDADO). Justificativa: consent/policy (cloud TTS) is outside Android framework contract; emulator verifies framework only; prevents 003 convergence from being silently dependent on `002` engine-resolve.
- B) Tie `VERIF-EMULATOR` success to `SC-044` resolution — [REJECTED]; creates circular dependency (003 blocked on 002, 002 blocked on 001 full, 001 full blocked on 003 verification).
- C) Add `SC-044` consent-flow test to AVD verification — [AVOID]; consent is user-facing / legal, not structural; requires real device + user interaction; out of scope for `VERIF-EMULATOR`.

**Assunções**: [ASSUMPTION] `SC-044` resolution path (offline_default adapter + second engine) is not affected by 003 structural layer; no source claims otherwise.

---

## Gap 4 — Success criteria / attestation format for VERIF-EMULATOR (EMPHASIZED)

**Questão** (open 53 / research.md 49 / problem.md 52–53): What precise success criteria define `VERIF-EMULATOR`? `verify.sh` exit 0 on AVD? Separate attestation document? Benchmark `SC-009` on AVD (infeasible per evidence)?

**Por que importante**: Without explicit criteria, `VERIF-EMULATOR` risks being either too weak (no boundary documented) or too strong (implicitly claiming physical equivalence). The spec-pipeline demands evidence (constitution v1.12.0; `AGENTS.md`; `problem.md` G1 / G2 / G3).

**Constatações — official Android / spec-kit recommendations for attestation**:
- `https://developer.android.com/reference/android/test/InstrumentationTestRunner` — official Android instrumentation reports via `IInstrumentationWatcher`; structured logging / `REPORT_VALUE_RESULT_ERROR`; report source class documented. Suggests attestation should include: test class, result code, structured bundle (not just exit 0). [allowlisted; high]
- `https://developer.android.com/training/testing/instrumented-tests` — instrumented tests run on AVD; report is produced automatically; structural verification can be captured as test artifacts (not just shell exit). [allowlisted; high]
- `https://slsa.dev/spec/v1.0/verification_summary` (SLSA VSA — verification summary attestation, now retired but schema preserved) — attestation format: `SlsaResult`, verification boundary, predicate reference, statement. Relevant as structural pattern: attestation must separate (a) what was verified, (b) what was NOT verified (boundary), (c) evidence source. [allowlisted; high — spec kit]
- `https://github.com/in-toto/attestation` (`spec/v1/bundle.md`) — in-toto attestation bundle requires statement + predicate + verification result; supports structural verification attestation with explicit exclusions. [allowlisted; high — spec kit]
- Internal `problem.md` G1 / G2 / G3 — requires either (a) documented `VERIF-EMULATOR` layer with explicit boundary conditions + `verify.sh` unchanged for physical gates, OR (b) documented decision with evidence. No implicit equivalence permitted. [internal; high]
- Internal `specs/003-reference-device/checklists/` / `contracts/` — verification protocol expects explicit boundary (what verifies / what doesn't) per `AGENTS.md` gate rules.

**Evidência contra (evidence-against) — what should NOT be in attestation**:
- Benchmark `SC-009` ≤3 s play-to-audio — [REJECTED] requires hardware audio conversion; emulator cannot measure real latency; including it in attestation creates false claim (research.md 32 / 40).
- `verify.sh` exit 0 as ONLY criterion — [INSUFFICIENT]; must include structured report (InstrumentationTestRunner-style) + explicit exclusion list (no Bluetooth, no real interruption, no hardware format conversion).
- Claiming equivalence to `gate-8-full` — [REJECTED] per `problem.md` §29, `research.md` 40, SDK limitation docs.

**Opções + recomendada + justificativa**:
- A) **Separate labeled attestation doc (`VERIF-EMULATOR.md` or `verification.md` inside `specs/003/`) — RECOMENDADO**, format: (i) boundary statement (verifies framework: manifest, MediaSession, notification, focus call-shape, diag JSONL; excludes hardware/audio/Bluetooth/interruption/on-device trials), (ii) evidence source (AVD image version, SDK rev, test class names, `InstrumentationTestRunner` report bundle if available), (iii) explicit NOT-verified list, (iv) link to `verify.sh` unchanged for gates 1–8. Justificativa: matches SLSA/in-toto attestation structure (separate statement + predicate + exclusions); satisfies problem.md G1/G2 without altering `verify.sh`; avoids false equivalence.
- B) Overload `verify.sh` with AVD exit 0 — [REJECTED] silently merges physical and structural gates; violates G2 (preserve `gate-8-full` physical-only).
- C) No attestation document — [REJECTED] 003 currently has 0 tests / 0 verification; without documentation, convergence has no evidence.

**Assunções**: [ASSUMPTION] `InstrumentationTestRunner` report bundle can be generated from AVD instrumented tests once 003 has `tests/`; currently `tests/` absent, so attestation will initially be document-based (not automated bundle). No source contradicts this phased approach.

---

## Gap 5 — CI / hardware / environment constraints for AVD in dirty 005 branch

**Questão** (open 54 / problem.md 55): What is CI/hardware budget for running AVD (memory, GPU, disk, time)? Working tree dirty with 51 files and `cargo check` interrupted.

**Por que importante**: AVD requires hardware acceleration (HAXM / Hypervisor / KVM on Linux / Apple Hypervisor / Windows Hyper-V) and ~1.5 GB image + ~2 GB RAM per instance. Running it on the same workstation where `bg_14` interrupted `cargo check` risks compounding resource pressure and dirty-state corruption.

**Constatações**:
- `https://developer.android.com/studio/run/emulator-acceleration` — emulator needs CPU, GPU, modem hardware acceleration for usable speed; pure software emulation is significantly slower; requires HAXM (Intel) / Hypervisor / KVM (Linux). [allowlisted; high]
- `https://developer.android.com/studio/run/managing-avds` — AVD creates system image + hardware profile; disk usage per AVD ~1–2 GB image + runtime state. [allowlisted; high]
- `https://stackoverflow.com/questions/59258271` — AVD can ignore specified RAM and use ~1.5 GB regardless of profile; confirms memory budget is non-trivial and not fully user-controlled. [allowlisted; medium]
- Internal `git status` / `ROADMAP.md` / session state — `005-adopt-design-sona` dirty 51 files; `cargo check` interrupted by `bg_14`; no verified green build since historical `9501bba` (`004`). [internal; high]

**Opções + recomendada + justificativa**:
- A) AVD only in isolated CI job / separate workspace, not on dirty 005 tree (RECOMENDADO). Justificativa: avoids resource collision with interrupted `cargo check`; preserves dirty files untouched (constitution / user instruction); allows image download once then reuse; no change to `verify.sh` on 005.
- B) AVD on current workstation alongside 005 build — [AVOID] risks compounding interruption; AVD boot + cargo check together could exceed available RAM/GPU; dirty-state risk if AVD writes to shared workspace (doesn't by design, but CI artifacts could).
- C) Defer AVD until 005 clean / `cargo check` green — acceptable fallback; but 003 needs structural proof to unblock 001; document-only layer (Gap 4 option A document form) can proceed without running AVD immediately.

**Assunções**: [ASSUMPTION] CI environment has GPU + virtualization enabled; if not, AVD falls back to software (slow, possibly non-viable for structural tests). Not verified against actual CI spec (no CI config file cited in workspace).

---

## Gap 6 — Assessment vs spec-note identity

**Questão** (open 56 / problem.md 56 / concept.md 52 / research.md 52): Should this remain a separate assessment (`ref-device-emulator`) or be demoted to a note inside `003-reference-device`? Intake already created assessment; reversal requires confirmation.

**Por que importante**: Assessment identity determines maintenance burden (`.specify/assessments/` directory, `intake.md` / `problem.md` / `research.md` / `decision.md` lifecycle) vs simple note inside `specs/003/`. If kept as assessment, it implies independent stakeholder / verification owner; if demoted, it becomes a sub-document of 003 verification.

**Constatações**:
- Internal `.specify/assessments/` — 5 dirs (`design-son-adoption`, `directory-reading`, `fr-024-synthesis-adapter`, `lente-sinestesica-completa`, `ressoa-mobile-app`) + this new `ref-device-emulator`; `directory-reading` trapped at `intake.md`; 4/5 assessments complete. Assessment lifecycle is distinct from spec lifecycle. [internal; high]
- Internal `problem.md` — assessment created by intake; user ask explicitly requested deep research on gaps; no instruction to demote; `research.md` line 52 notes identity as open question. [internal; high]
- `https://philo-kim.github.io/intent-spec-layer/guide/spec-review-loop.html` (spec-kit-related) — spec review loop separates evidence assertions from spec claims; assessment serves as evidence-gathering surface; spec note serves as implementation sub-document. Both valid, different roles. [github.io; high — spec practice]
- Internal `AGENTS.md` / constitution v1.12.0 — verification and assessments have separate gates; assessment can feed spec decision without becoming spec itself.

**Opções + recomendada + justificativa**:
- A) **Keep as assessment (`ref-device-emulator`) — RECOMENDADO**, with `decision.md` recording the sub-layer / attestation choices; once decided, transfer the boundary document (`VERIF-EMULATOR.md`) into `specs/003/` (not demoting the assessment yet, but producing the spec artifact). Justificativa: assessment captures evidence process; spec captures implementation; separation matches `AGENTS.md` / spec-kit practice; `directory-reading` precedent (assessment trapped at intake — don't repeat).
- B) Demote to `specs/003-reference-device/notes/emulator.md` now — [AVOID] loses evidence-trail (intake → problem → research → decision) and makes 003 verification dependent on a sub-note rather than an assessed decision.
- C) Kill assessment and rely on `research.md` only — [REJECTED] user explicitly asked for deep per-gap research with cited sources; assessment framework enforces that.

**Assunções**: [ASSUMPTION] User did not explicitly ask to demote; default is keep unless confirmed. Decision may be revisited after Gap 4 attestation doc is written.

---

## Consolidado / Recomendação 80/20 de carga

Evidence load: gap 2 (sub vs replace) + gap 4 (attestation format) are the load-bearing choices.
- **Gap 2**: Sub-layer (not replacement) — evidence-against from SDK + StackOverflow is explicit; no ambiguity after citation.
- **Gap 4**: Separate attestation doc (SLSA/in-toto-style boundary + InstrumentationTestRunner-style report) — not overloading `verify.sh`; satisfies `problem.md` G1/G2.
- **Gap 1**: AVD (official) — no need to evaluate alternatives deeply once sub-layer role is fixed.
- **Gap 3**: Independent of `SC-044` — prevents circular dependency.
- **Gap 5**: Isolated CI / document-only start — avoids dirty-branch collision.
- **Gap 6**: Keep assessment — preserves evidence trail.

No new crate required (`cleanroom-core` 0 .rs remains; `VERIF-EMULATOR` lives in `specs/003/` docs + possibly `tests/` later). `verify.sh` unchanged for physical gates. `cargo check` still deferred (`bg_14`); no build/test claim made.

---

## Fontes sanitizadas (resumo por gap)

G1: `developer.android.com/studio/run/emulator-comparison` (sanitize: `developer.android.com/studio/run/emulator-comparison`); `stackoverflow.com/questions/13112889`; `stackoverflow.com/questions/18683656`.
G2: `developer.android.com/develop/background-work/services/fgs/service-types`; `developer.android.com/media/optimize/audio-focus`; `stackoverflow.com/questions/22604305`; `developer.android.com/studio/run/emulator`; `developer.android.com/about/versions/14/behavior-changes-14`; `developer.android.com/develop/background-work/services/fgs/declare`.
G3: `developer.android.com/about/versions/14/behavior-changes-14`; `developer.android.com/develop/background-work/services/fgs/declare`; internal `specs/002/`, `ROADMAP.md`.
G4: `developer.android.com/reference/android/test/InstrumentationTestRunner`; `developer.android.com/training/testing/instrumented-tests`; `slsa.dev/spec/v1.0/verification_summary`; `github.com/in-toto/attestation` (`spec/v1/bundle.md`); internal `problem.md` G1/G2/G3.
G5: `developer.android.com/studio/run/emulator-acceleration`; `developer.android.com/studio/run/managing-avds`; `stackoverflow.com/questions/59258271`; internal dirty-state evidence.
G6: `philo-kim.github.io/intent-spec-layer/guide/spec-review-loop.html`; internal `.specify/assessments/`; `AGENTS.md` / constitution.

Todas as fontes estão dentro das políticas: `developer.android.com` (allowlisted SDK); `stackoverflow.com` (allowlisted, high-rep answers); `github.io` / `slsa.dev` / `in-toto` (spec-kit / attestation specs); nenhum segredo exposto; URLs sanitizadas (sem query tracking, sem parâmetros de sessão).
