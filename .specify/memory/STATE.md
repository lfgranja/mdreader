# Session state — 004-ressoa-dogfood-build / Separation of Business & Engineering Repositories

> **RESUME POINT (2026-11-11):**
> 1. **Spec 020 (`020-dynamic-doc-cap-scalability`) CONVERGIDA (100%):**
> - Expansão do orçamento de tamanho máximo de documento e capítulo de 93.636 bytes (~91.4 KiB) para 1.048.576 bytes (1 MiB / 1.024 KiB) em Rust (`crates/player/src/interruption.rs`, `crates/player/src/queue.rs`) e Kotlin (`android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`).
> - Viabiliza a reprodução contínua de notas extensas de PKMS (Obsidian/Logseq) e livros completos sem recusa prematura de `FormatUndeliverable`.
> - Propriedades de segurança, delimitação de memória em RAM via cache LRU de 32 MiB (`AudioChunkCache` - Spec 017) e Gates 1 a 8 de `./scripts/verify.sh` 100% preservados e verdes (`VERIFY_EXIT=0`).
>
> 2. **Spec 019 (`019-intent-filter-ingress`) CONVERGIDAS (100%):**
> - O "80/20" de maior impacto com menor esforço implementado: suporte completo a Android Intent Filters para `ACTION_VIEW` (`text/markdown`, `text/plain`, `text/x-markdown` e `*.md`) e `ACTION_SEND` (Share Sheet a partir de apps como Obsidian, Logseq e gerenciadores de arquivos).
> - Integração de ciclo de vida em `MainActivity.kt` (`onCreate` e `onNewIntent`) delegando com segurança para `DocIngest.handleIncomingIntent` sem qualquer lógica de áudio em Kotlin (pureza total do shell).
> - Todos os Gates 1 a 8 de `./scripts/verify.sh` rigorosamente verdes (`VERIFY_EXIT=0`).
>
> 2. **Bug `speed-button-ineffective-usability` CLOSED (VERIFIED):**
> - Eficácia Acústica da Velocidade: Multiplicador de velocidade (`playback_rate`) integrado em `GenerationConfig.speed` no `sherpa-onnx` (`crates/adapter-synthesis/src/synth.rs`). Re-ancoragem em tempo real (`reanchor_at`) disparada durante reprodução ativa (`Playing`) no bridge Tauri (`src-tauri/src/lib.rs`).
> - Usabilidade e Layout: Ilha flutuante (`.bottom-island`) refatorada em layout hierárquico em dois níveis, eliminando transbordamento de botões para fora da tela em dispositivos Android. Ribbon decorativo `wave-04-aurora-breeze` integrado ao cabeçalho.
> - Validação: Todos os Gates 1 a 8 de `./scripts/verify.sh` passaram (642 testes unitários/integração + auditoria de superfície C-H1..C-H9). Gate 9 e hardware real validados no Xiaomi Redmi Note 11 com APK universal debug via ADB.
> - Status dos artefatos: `assessment.md`, `fix.md` e `test.md` marcados como `closed (verified)`.
>
> 2. **Separação de Repositórios (Open Source vs Estratégia Privada MDMV):**
> - O repositório [`mdreader`](https://github.com/lfgranja/mdreader) é mantido 100% de código aberto (*open-source*), estritamente focado em engenharia de sistemas, especificações técnicas, testes determinísticos em Rust e o shell Android.
> - Criado e sincronizado o repositório privado na organização MDMV: [`MDMV-Laboratorio-de-Solucoes-Digitais/product-ressoa`](https://github.com/MDMV-Laboratorio-de-Solucoes-Digitais/product-ressoa).
> - Todos os documentos de negócio, inteligência de mercado, GTM, modelos de monetização, pareceres jurídicos e especificações comerciais foram transferidos para o repositório privado `product-ressoa` e removidos do repositório aberto.
> 2. **Documento Embutido de Demonstração (Deployable Sample):**
> - Arquivo em linguagem simples para o usuário mantido em `assets/samples/bem-vindo-ao-ressoa.md` (e espelhado nos assets do APK Android) para garantir o primeiro som (*time-to-first-sound*) sem atrito.

---

## Histórico Anterior

> **RESUME POINT (2026-10-24) — 005-adopt-design-ressoa, Phase 1 (Setup) COMPLETE (T001–T003).**
> Branch `005-adopt-design-ressoa` cut at dev HEAD `226c06c`
> (`226c06cdd211d5541f85926aa0e131ac12cc6f84`). T001 deviation disclosed: the
> tree was DIRTY at branch creation (33 pending clarify/plan/assessment
> artifacts from clarify runs + plan were never committed) — remediated ON the
> branch by 33 per-file commit-hook commits (tasks.md `edf6190` first, with
> the two pending refinements + the US3 parallel-example paths corrected to
> the child-module layout; then spec/plan/research/data-model/quickstart/3
> contracts/checklist; 20 clarify research notes; 5 assessment records; then
> memory `cdad44d`), giving the clean tree the task requires before the
> baseline measurement.
>
> **T002 baseline — three runs, all recorded** (`/home/luis/development/.mdreader-005-tr/logs/`):
> (1) run 1 (`baseline-2026-10-24.log`) RED at gate-5: gates 1–4 PASS,
> cargo deny advisories named RUSTSEC-2026-0385 (`ctutils` 0.4.3 via digest
> 0.11.3 ← hmac/sha1/sha2/pbkdf2 ← zip 9.0.5 ← (build) sherpa-onnx-sys).
> Provenance verified: NO lockfile commits since `9501bba` — the RED is a
> newly published advisory, not a tree change (zip 9.0.5 entered at
> `ea59333`, the latest-stable refresh). Fix `0c2b6a7`:
> `cargo update -p ctutils` → 0.4.5 (single package; 42 unchanged; fail-
> forward per constitution IV — no waiver, no auto-substitute; deny all-ok
> + audit 2 allowed warnings). (2) run 2 (`…-run2.log`): gate-5 green, then
> `verify.sh` died with a bash syntax error at:335 — an ORPHANED `fi`
> leftover from `8635535` (T164 waiver retirement), latent because the
> Oct-22 green runs predate that commit and run 1 aborted before the parser
> reached the line. Fix `9b9a539`: delete the stray token (`bash -n` clean;
> diff vs 9501bba = exactly the 57-line waiver removal, nothing else).
> (3) run 3 (`…-run3.log`) **GREEN, VERIFY_EXIT=0**: gate-1 format PASS ·
> gate-2 tests PASS (906 tests run: 906 passed, 2 skipped — the same
> inventory as the 004 attestation) · gate-3 documentation PASS · gate-4
> lint-drift-clippy PASS · gate-4 lint-drift PASS · gate-5 supply-chain
> PASS · gate-6 deny-list PASS · gate-7 closed-set PASS · gate-8-shell
> PASS · gate-8-full DEFERRED (owner=synthesis) — never a pass. Run 3 is
> the recorded green baseline of the feature.
>
> **T003 anchor audit — 19/19 PASS** (explore agent, per-anchor evidence):
> `enqueue_earcon` backtaint:511, FIFO `push_short_mix`:288-290
> (consume_into:318-342 pops front in lockstep with speech = the residual
> FIFO serialization D3 cites); `EarconKind`:10-26 with 7 variants
> (Play/Pause/SkipFwd/SkipBack/Warning/Failure/RateStep); `EventKind`
>:66/:69-88 exactly nine (SessionStart/SessionEnd/TransportChange/
> FocusTransition/OutputDevice/Failure/Refusal/Trim/Gap), no catch-all;
> status_view.html anchors all present (slates:21-22, white fill:26,
> --warning/--failure:29-30, shimmer gradient:400, @keyframes
> indeterminate:748-752, rgba tints:179-180, reasonBanner alert+polite
>:252 same element, «Seção: —»:234, speedDots aria-hidden:245,
> #progress:249 NO aria attrs, ZERO:focus-visible, ZERO
> prefers-reduced-motion, `infinite` only:402); status_view.rs 534 lines
> pure render; MediaMirror.kt constants block:701-744 (DISPLACED from the
> tasks':728-743 citation) with PROGRESS_FILL=0xFFFFFFFF:728 +
> setContentText guard:1419; frontendDist "../dist":6; verify.sh +
> expected-tests.txt (99 lines, no surface_audit/earcon_burst entries);
> logo-primary-concha-r.svg census-clean (0× #B9A6F5, 0× #5AD6A8);
> tokens.md §1 ledger:10-20 with harp #E8C46A:17; 004 contracts
> earcon.md/surface.md present; DESIGN-RESSOA CHANGELOG.md/VERSION ABSENT
> (as T014 expects); src-tauri/build.rs exists (auto-detected); NO
> surface_audit module yet; backend.rs has one #[cfg(test)] mod:664,
> proptest dev-dep:32 (not yet imported in src); constitution v1.12.0
> confirmed; diag contract_schema + prop_no_prose live under
> crates/diag/tests/ (writer.rs and scan.rs have NO inline tests).
>
> **Task-level bugs / adjudications recorded (dispatch inputs):**
> (i) T007(i)'s literal set claim is UNSATISFIABLE over contract A1#3's
> pinned slice:728-743 even post-T008 — that slice permanently carries the
> two admitted --signal-* tokens (FFD400:736, FF4D63:743) and the accent
> usage site (:1407 setColor(DESIGN_RESSOA_TITLE)) is OUTSIDE it; the ONLY
> DESIGN_RESSOA_ usage in MediaMirror.kt is:1407 (FILL/WARNING/FAILURE/
> SECTION/TRACK are declared but UNWIRED). Adjudication (contract wins):
> the accent twin asserts the fill token == ledger --ressoa-voice (C1:
> literal-white veto = today's NAMED RED) plus non-signal declared/used
> hexes ⇒ exactly {F2F0E6}, with FFD400/FF4D63 recognized as sanctioned
> --signal-* declarations; repeat verbatim in T007's commit summary.
> (ii) T005/T006's «Taxa: 1.0×» bench leak is ABSENT from the shipped
> dist/index.html (offline-synth bench, 4 948 B) — the S2 byte-identity
> mandate stands; the Taxa rationale no longer binds to present content.
> (iii) T011/T012 edit points: writer.rs has no inline tests — the
> contract_schema twins live in crates/diag/tests/. (iv) T021's
> versionFooter citation: the id exists only at:254 (:223 is the
>.version-footer class rule).
>
> NEXT: Phase 2 (T004 audit scaffolding → T005 RED served-identity → T006
> wiring), then the [P] agent waves (Agent B: accent.rs + MediaMirror.kt ·
> Agent C: crates/player + crates/diag).

> **RESUME POINT (2026-10-24) — 005-adopt-design-ressoa, clarify run 5 (batch mode).**
> User-directed protocol again (2 background agents per candidate: code dive +
> research note; batch answers at the end). 3 candidates (E1–E3). Today's date
> for all registrations: 2026-10-24.
> (a) Integrated autonomously (evidence-converged, veto reversers recorded in
> each bullet): **E1** — `--ressoa-` = FOCUS RING (`:focus-visible` on
> enumerated interactive controls); the «fluxo ativo» role is RESERVED
> non-painted for the full Lente; edits: FR-001 role map, US3 narrative
> («turquesa para o anel de foco»), SC-001 item (iii) exclusivity (off-role
> `#8FD8E8` paint = fail), batch-Q1 task gained item (5) (the 7 stale
> -progress rows: `tokens.md:16/:28/:47/:65`, `components.md:17`,
> `DESIGN-RESSOA.md:60/:85`, `preview.html:238`), C1 record annotated. Evidence:
> 0 occurrences of `#8FD8E8`/`` in audited files; 0 `:focus-visible` in
> `status_view.html`; recipe is the design dir's own (`tokens.md:20`).
> **E2** — CLOSED MOTION RULE: the indeterminate sliding pill (C5) is the ONLY
> continuous motion on the driving surface and freezes at a visible track
> position under `prefers-reduced-motion: reduce`; new SC-001 item (v) host
> check (media query present, zeros `animation`/`transform`, no new paint/element);
> SC-007 gains the freeze leg («Remover animações» on, announcements unchanged
> 10/10); FR-008 carries the rule clause; SC-006 pointer includes (iv)+(v).
> Yellow pulse TTL-bounded (3 s), failure = static red (no pulse), banner
> slide one-shot ≤200 ms, no other continuous loop. WCAG 2.2.2 strict reading
> + SCR40/C39 OS-setting as the sanctioned mechanism; census-clean (zero
> painted pixels). Historical note: the pulsing yellow dot does NOT exist in
> shipped code yet (US1 AC2 is the first implementation).
> Research notes: `specs/005-adopt-design-ressoa/research/clarify-e{1,2}-*.md`.
> (b) RESOLVED — maintainer ACCEPTED the recommendation ("accept the
> recommendations", 2026-10-24), batch E option A: **E3** — SC-002
> haptic side-channel disposition. DIVE FINDING (unchanged): haptics are DORMANT in the
> shipped binary (Rust always publishes `haptic=None` — `lib.rs:6047`; the
> 7-pattern table + `VibratorManager` in `MediaMirror.kt` never fire; no app
> off-switch; system haptic-feedback setting does NOT gate direct
> VibratorManager calls). Research: cross-modal redundancy fuses/confounds
> audio identification (Brewster segregation doctrine; ISO 7731 unimodal
> unit-of-test; two-pulse discrimination peaks at 50–700 ms ISI vs the 20 ms
> shipped gap; 10/15 ms buzzes below the ~50 ms perceivable floor).
> `research/clarify-e3-sc002-haptic-side-channel.md`.
> **INTEGRATED:** SC-002 haptic-disposition clause (audio gate sound-only;
> app haptics inert; administrator no-buzz check at the pilot; the haptic
> half stays in 004's T051-04 sounds-off condition, never this gate),
> B1 record annotated (patterns = Kotlin render table, UNWIRED —
> `haptic=None` published), Clarifications bullet E3, research-note
> correction addendum. ALL of E1–E3 resolved — spec is plan-ready.
> (c) Eliminated pre-spawn by direct evidence (never became candidates):
> DESIGN-RESSOA has NO haptic prescriptions (all «vibração» hits are visual
> metaphor) and NO soundscape/ambient audio content → Option A scope + closed
> inventory already govern; `--ressoa-hush` `#A9B8CE` = shipped section color
> (bit-exact, no reconciliation needed).
> Checklist re-validated after E3: 16/16 → 16/16, no marker changes. Spec is
> plan-ready. PLAN run complete (2026-10-24, same session): `plan.md` from
> the template (Summary / Technical Context / Constitution Check PASS — no
> violations, no Complexity rows / Project Structure / Post-Design re-check)
> + `research.md` (R1–R13) + `data-model.md` (RessoaToken 10-member lexicon,
> SurfaceInventory, NotificationSurface, EarconConfirmation 7 confirmations,
> RateLadder, ProgressIndication, MovementSpec, HapticDisposition, §VII diag
> extension, DesignDirState, ContrastPair) + `contracts/` {status-surface,
> earcon-mapping, surface-audit} + `quickstart.md` (host gates via the new
> `surface-audit` checks + device trials incl. the SC-002 protocol and the
> SC-007 freeze leg). `hooks: {}` — skipped per rules. Working tree still
> UNCOMMITTED (clarify + plan artifacts). NEXT: `/speckit.tasks`.

> **RESUME POINT (2026-10-24) — 005-adopt-design-ressoa, clarify run 4 (batch mode).**
> Ran with the user-directed protocol (2 background agents per candidate: code
> dive + research note; batch answers at the end). 5 candidates (D1–D5), ALL
> RESOLVED. (a) Integrated autonomously (evidence-converged): D1 — restyle is
> COLOR-ONLY (typography/spacing = design-dir prose, not `--ressoa-*` tokens;
> FR-001 clause + Session bullet); D4 — launcher icon explicitly OUT of scope
> (brand surface; regen-clobber; zero driving impact; Assumptions bullet).
> (b) Maintainer ACCEPTED the recommendations ("accept the recommendations")
> for the batch and all three were integrated in the same run: D2 — admit
> `#FFD400` `--signal-caution` / `#FF4D63` `--signal-failure` as role-locked
> signal paints (audited hex set 8→10; presence off-role = violation; never
> adjacent, 2.25:1; FR-001 + SC-001 + clarif bullet; values match shipped
> `status_view.html:29-30,179-180`; design-dir errata `tokens.md:48` /
> `components.md:18` ride the batch-Q1 DESIGN-RESSOA maintenance task); D3 —
> FR-009 truncate/replace «latest wins» sub-rule (earcon↔earcon; additive mix
> stays earcon↔narration-only; system Ostinato/Sforzando interrupt; typed
> truncation diagnostic counter; `short_mix` stays single-channel) + SC-002
> rapid-sequence sub-test (2–3 heterogeneous actions in 100–150 ms); D5 — the
> value-blind promise is now VERIFIED: SC-001 item (iv) semantics inventory
> (aria-labels, role="progressbar"+valuenow/valuetext, hidden role="status"
> live region for the ladder value, dots stay aria-hidden, reason banner
> re-typed from mismatched alert+polite to status) + NEW SC-007 TalkBack
> device trial (10/10 announcements; the spec's 7th success criterion).
> Research notes: `specs/005-adopt-design-ressoa/research/clarify-d{1..5}-*.md`.
> Checklist re-validated: 16/16 → 16/16, no marker changes. Spec is
> plan-ready. NEXT: `/speckit.plan`.


> **RESUME POINT (2026-10-22) — read this first.** Phase 17 is COMPLETE
> (T140–T163, all committed) and the final verification has RE-ATTESTED the
> tree at `9501bba` ("test(004): T163 model the FR-007 five-second cue in the
> canonical guard oracle") in the attestation commit `21892dd`
> ("verify(004): T095 final re-attestation (deterministic gate 2)"); the task
> markers are committed on top of it (`ce54aea`).
>
> **Landed in Phase 17 (24 tasks, all committed):** T140 `f30feca` (RED
> resume) · T141 `aa4ecf4` (resume restarts the stream) · T142 `59a12a2` (RED
> session-live) · T143 `97f0046` (session live from the first chunk) · T144
> `80803f5` (RED offer-dialog reachability) · T145 `88e4f2d` (offer dialog
> reachable) · T146/T154 `173b7d2` (DeviceRemoved diag contract row) · T147
> `3fc7934` (offer route read-only) · T148 `7bb8b25` (in-app Play functional) ·
> T149 `a651c61` (bounded resident queue) · T150/T153 `74fbb95` (surface
> contract record + shell contract paths) · T151 `d17da93` (comment-aware
> literal scanner) · T152 `8753164` (first consumed sample latency) · T155
> `07f59a2` (parked open re-check cadence) · T156 `fa04ca6` (purity-gate
> allowlist) · T157 `507a991`/`9786886`/`2a94589` (expected-tests sync) · T158
> `024d55e` (producer back-pressure) · T159 `5af0f33` (buffer bound in the diag
> record) · T160 `3e4fd0a` (resume divergence reachability) · T161 `1ac0ae3`
> (FR-003 first-audible observation restored) · T162 `b03f1a2`/`5e0acaf` (stale
> gate-2 comment) · T163 `9501bba` (the canonical-guard oracle repaired).
>
> **Final verification (the only thing that attests) — DETERMINISTIC, TWICE.**
> `scripts/verify.sh` was run TWICE on the attested tree, back to back, and
> BOTH runs exited **0** with a byte-identical per-gate result sequence — the
> determinism claim is DEMONSTRATED, not asserted:
> run 1 (03:27:34 – 03:43:03 -0400, 7 265 lines, `VERIFY_EXIT=0`) and run 2
> (03:50:02 – 04:05:58 -0400, 7 261 lines, `VERIFY_EXIT=0`); the exit status is
> the SCRIPT's (`PIPESTATUS[0]` under `bash -c`), and the two per-gate result
> sequences compare EMPTY under `diff`. Gates 1–7 + gate-8-shell PASS in both
> runs (906 tests run, 906 passed, 2 skipped — up from 902, T163's four new
> pins); `gate-8-full: DEFERRED (owner=synthesis)` (SC-044 open — NEVER a
> pass); `check-manifest.sh` and `check-shell-purity.sh` PASS in both runs.
> Standalone: clippy exit **0** (also as gate 4's fail-closed
> `lint-drift-clippy`), fmt exit **0**, nextest inventory **99 live / 99
> pinned** with the two-way diff empty in BOTH directions, `cargo deny` exit 0
> (`advisories ok, bans ok, licenses ok, sources ok`), `cargo audit` exit 0
> with the 2 allowed warnings. The RUSTSEC-2026-0381 waiver (Exceptions Log
> entry 13, PROVISIONAL, constitution v1.11.0) holds its mechanical bound: the
> COMMITTED lockfile carries exactly `webpki-roots` 0.26.11 + 1.0.10.
> Gate 9 `cargo tauri android build --target aarch64` exit 0 — APK sha256
> `2e3555f3…ece08f5`, AAB sha256 `c226ecc3…cc6d42b`, non-hollow (7/7 app
> classes in the dex). Logs of record:
> `/home/luis/development/.mdreader-t095-reattest-tmp/logs/` (`run1.log`,
> `run2.log`, `android.txt`, `clippy.txt`, `fmt.txt`, `nextest_list.txt`,
> `deny.txt`, `audit.txt`).
>
> **RESOLVED — the §3.1 flake is GONE.** The previously-disclosed
> intermittency (`player::prop_canonical_guard` red ~1 run in 40) was a STALE
> ORACLE, not a production defect, and T163 `9501bba` repaired the oracle with
> the production left byte-identical (`git diff --exit-code
> crates/player/src/rewind.rs` empty). The FR-007 5 s cue is now modelled, the
> `[cap, cap+5)` counterexample is pinned, the repair is mutation-proven
> (removing the cue turns 4 of 9 red), a 1 200 000-case equivalence sweep found
> zero disagreements, and a genuine proptest-recorded seed replaced the
> hand-copied hash that replayed no counterexample. T163's own 105 consecutive
> runs recorded 0 failures; all NINE `player::prop_canonical_guard` tests PASS
> in BOTH attested runs. Full detail in the attestation's §3.1.
>
> **Still open — device-only by design:** **T051** (the SC-001…SC-010 trials
> per `quickstart.md` §3 on the reference phone, Redmi 2201117TY) and **T052**
> (the on-device `diag scan` audit of the pulled JSONL). No trial sheet exists;
> both stay `[ ]` in `tasks.md` and the attestation explicitly does not claim
> them.
>
> **Next action, in order:**
> 1. T051 + T052 on the reference phone — the only un-attested criteria.
> 2. Do NOT push to `origin/main` without an explicit request (the branch is
> far ahead — 514 commits).
> 3. Optional maintainer housekeeping: `cargo deny` reports the
> RUSTSEC-2026-0381 ignore as `advisory-not-detected` (a warning, not a
> finding) because the graph now carries the patched 1.0.10; removing the
> two ignore records is a constitution exit-condition decision, not a
> verification-session change.
>
> Housekeeping: `git stash list` is empty; the working tree is clean. Note the
> host's `/tmp` is a 2.0 GB tmpfs that another project (a `dotnet build`
> fixture of `livingroots-smoke-harness`) fills to 100 % — use a disk-backed
> `TMPDIR` for cargo/Gradle and the scratch space.

Updated: 2026-10-17 (Wave B device verification GREEN + user-heard audio
confirmed; zip-9 substitution complete — patch, lock, governance v1.8.0;
verify.sh gates 1–4 PASS, gate-5 fixed; F1/F3 wiring IMPLEMENTED (wave-b4):
player is_drained + 500 ms playback-watch task feeds session clock from
backend clock, republishes at consumed-unit crossings, commands
document-end teardown; host-fallback + idempotency tests green; Kotlin
ShellHost.commandDocumentEnd landed; 5 new src-tauri tests + 3 player
tests pass; awaiting device build/install + F1 end-transition check;
upstream PR #4059 open).

## Position
- **Wave A (bootstrap) GREEN ON DEVICE**: `PlaybackService` foreground
 (`isForeground=true foregroundId=1`, channel `mdreader_media`, "Ressoa" holder,
 `POST_NOTIFICATIONS granted=true`). Committed `7ca4c1d`.
- **Wave B (Rust→Kotlin mirror publish) VERIFIED ON DEVICE (wave-b3)**:
 - R8 regression FIXED: explicit `-keep` (`b856bfc`, force-added
 `gen/android/app/proguard-rules.pro`) + ShellHost docstring (`728557b`);
 wave-b3 dex contains `publishMirror` (grep count 1), `BUILD_EXIT=0`.
 - APK sha256 `04ff6c7e1b8a9d77e92012cfe25dd6d217c8ed34949bfa8554970724d67ce56a`
 installed (`adb install -r`); verified with pid 18124 run (~22:04).
 - Start publish: notification title "Teste de bancada" + section
 "Teste de bancada" + determinate progress 0/2000 (2 units), category
 transport, flags 0x62 (non-dismissable while playing) — **no JNI errors**.
 - Pause publish: MediaSession `state=2`, flags 0x62→0x60 (ONGOING cleared →
 dismissable via pause, per design). Stop publish: `state=1`.
 - MediaSession: `actions=54` (play/pause/prev/next, no Stop), active.
- **FINDINGS (bench/dogfood flow; the mirror transport itself is sound)**:
 - F1 no end-of-audio detection: `play` (lib.rs:2165) enqueues the PCM,
 publishes once, returns; the backend exposes no drain signal
 (`backend.rs:300` is only `enqueue_pcm`); `dispatch_document_end` is
 reachable only via the Kotlin bridge intent, which cannot know (SC-035).
 Session stays Playing forever after the audio ends (2× confirmed:
 21:54 and 22:04 runs; notification frozen + non-dismissable). Intended
 wiring located (2026-10-16): Kotlin `Residency.documentEnd` and Rust
 `dispatch_document_end` both exist; the missing link is Rust drain
 detection + the JNI command that invokes the Kotlin teardown.
 - F2 bench cannot restart after Stop: `▶ Play` → "illegal transition from
 stopped via play" (UI FALHA); only relaunch replays. The failed attempt
 also requestStarts the paused stream (zombie track starved to 4265
 underruns at removal — seen in audio_flinger). Reclassified 2026-10-16:
 from-Stopped every verb is rejected by design (terminal; FR-032f
 "starting that document again requires opening the application") — the
 bench lacks a cold-start affordance; surface UX gap, not a product bug.
 - F3 single publish at start only: no mid-play/section-change updates
 (nothing calls `publish_current_state` during playback; the 20 s heartbeat
 only persists bookmarks) — matters for the T051-06 section-change state.
 Fix rides the same watcher: publish when `consumed_units_at(position)`
 changes.
 - **F1/F3 RESOLVED (wave-b4, 2026-10-17)**: `CpalBackend::is_drained`
 (player) + 500 ms playback-watch task (src-tauri) feeds the session
 clock from the backend clock, republishes at consumed-unit crossings,
 and commands the document-end teardown via
 `shell_jni::command_document_end_to_kotlin` → `ShellHost.commandDocumentEnd`
 → `Residency.documentEnd` → `dispatch_document_end`. Host fallback
 completes in-process; both paths idempotent. 5 src-tauri + 3 player
 tests green. Remaining: Android build/install + on-device F1 end-transition
 check + full verify.sh (gates 6–8 still pending).
 - F4 Stop re-posts nothing to the notification (content identical;
 MediaSession updates) — benign, noted.
 - Audio content flow CONFIRMED by the user's ears (pt-BR voice heard
 end-to-end in the bench runs). The MIUI `f`-counter freeze (~0.47 s while
 the mixer stayed fed, 0 underruns) was a log artifact, not a stall.
- **Gate 5**: zip-9 substitution COMPLETE — vendor `2177c67` (sha256
 `14bbeabb…`, one line zip 2→9), `[patch.crates-io]` + lock zip 9.0.4
 `02164b6`, ignores dropped + entry 11 deleted + entry 12 added, constitution
 v1.7.0 `8435260` (MSRV build-note: rustc ≥1.88), bzip2-1.0.6 admitted +
 v1.8.0 `9208578` (upstream PR #4059 keeps default features → admission
 survives the exit).
- Full `scripts/verify.sh` on the zip-9 tree: gates 1–4 PASS (775 tests);
 gate-5 FAILED pre-admission (libbz2-rs-sys bzip2-1.0.6) → admitted v1.8.0
 → targeted re-check green (`cargo deny` all-ok; `cargo audit` 0). Gates
 6–8 on this tree land with the next full run.
- T051 (human trials; needs picker = Wave C) + T052 pending.
- `gate-8-full: DEFERRED (owner=synthesis)` — unchanged (Exceptions Log 8).

## Next
1. **F1/F3 fix (wave-b4, tests first)**: (a) `crates/player` backend exposes
 `is_drained` (queue empty with speech consumed) + unit test; (b) src-tauri
 ~1 s watcher while Playing — drained → JNI `command_document_end` →
 `ShellHost` → `Residency.documentEnd` → `document_end` intent →
 `dispatch_document_end`; and `consumed_units_at` change →
 `publish_current_state`; (c) `shell_jni.rs` JNI entry + Kotlin `ShellHost`
 method. Then Android build + install + on-device check (2-segment doc:
 bar 0/2000 → 1000/2000 flip; `Completed` + notification reverted ~10 s).
2. Full `scripts/verify.sh` re-run (+ `scripts/expected-tests.txt` if new
 test targets) and wave-b4 commit.
3. Wave C — picker trigger (`DocIngest.pickDocument` / `handlePickResult`);
 then T051 sheets (human trials) + T052 diag scan (`run-as` fails on the
 release APK — needs debuggable variant or another pull route).

## Commits this session (004)
`c14e294` (T076 workspace clippy, this session) · `92129da` · `7ca4c1d` ·
`1849761` · `72f369e` · `18dd86a` · `7a25821` ·
`c6c3da4` (STATE) · `b856bfc` (R8 keep) · `728557b` (docstring) ·
`cc9164e` (STATE R8) · `2177c67` (vendor sherpa-onnx-sys) · `410a9b1`
(STATE wave-b3) · `02164b6` (patch + lock zip 9.0.4) · `8435260` (v1.7.0
governance) · `9208578` (v1.8.0 bzip2-1.0.6 admission).

## Verification summary
- T076 clippy (commit `c14e294`): `cargo clippy --workspace --all-targets --
 -D warnings` is clean for every file in this lane. Residual red is blocked
 on sibling lanes, NOT clippy debt: `src-tauri/src/lib.rs` (152 body + 20
 `multiple_crate_versions`), `tests/test_earcon_mirror.rs` (7, T074),
 `tests/integration/test_lockscreen_us4.rs` (8, T079). `cli.rs`/`surface.rs`
 committed via curated `update-index` blobs (sibling lane's rewrites left in
 worktree).
- Host: zip-9 tree gates 1–4 PASS (775 tests); gate-5 green after v1.8.0;
 6 mirror tests pinned.
- Device: Wave A + Wave B mirror path VERIFIED (start/pause/stop publishes;
 notification content + flags); F1–F4 bench findings recorded.
- Upstream: PR #4059 open (no upstream CI on PRs); zip 9 source-compatible.
- Brand / Design: Dupla Assinatura homologada (1º Concha Harmônica "R" primária,
 2º Ressonador Canônico selo motor). Arquitetura UI/UX multi-modo homologada:
 "A Lente Sinestésica" (P2) como Default Player; Cockpit Eyes-Free (P4) para
 condução; Terminal Suíço (P3) para sumário; Matriz Rust (P5) para diagnóstico.
 Assets PNG e SVG gerados via Storytold (PhotoCraft + VectorCraft + FigCraft).


T028: gate-9 build artifact confirmed: app-universal-release.apk (118MB, aarch64)

---

## 2026-11-18 — Decisões de governança confirmadas para mdreader

- **Aprovações em PRs (confirmado pelo Software Engineer):** `dev` permite merge com zero aprovações após os checks exigidos passarem; não exige review de CODEOWNER. `main` mantém pelo menos uma aprovação qualificada de alguém diferente do autor e review de CODEOWNER para paths próprios. `.github/CONTRIBUTING.md` foi atualizado no commit `0b2eef24`; configurações remotas do GitHub não foram alteradas nesta mudança.
- **Autoridade de CI/PR do alvo (confirmado):** GitHub Actions (`.github/workflows/ci.yml`) e `gh` são autoritativos para `mdreader`. Forgejo e seus workflows na `mdmv-dark-factory` são somente referência; nenhum workflow foi importado. HEAD da factory observado como fonte nesta iteração: `ce3d42132edb9466b75eab968be775ebdea38985`; nenhum arquivo da factory foi modificado ou copiado.
- **Gate 9 release-only (confirmado e IMPLEMENTADO em 2026-11-18):** gates Host 1–8 permanecem nos PRs; Gate 9 roda manualmente (`workflow_dispatch`) e em tags `v*`; ruleset `v* release gates` (id `27578490`, active) exige os contextos Host e Android em tags `v*`. PR #223 mesclado em `dev` (`568a4e9f`); CI do PR mostrou apenas `Host gates` (PASS 26m33s) e `Android build` = `skipping`, confirmando a mudança ao vivo. Detalhes em `specs/022-gate9-release-only/`.
- **Âncora SDD observada:** `specs/010-directory-reading` (`.specify/feature.json`). A decisão Gate 9 está aprovada, mas a escolha 80/20 e o ciclo SDD completo ainda precisam ser registrados após as auditorias de evidência dos Blocos 2–5.
- **Entrega documental em andamento:** PR #221 contém a atualização de `CONTRIBUTING.md`; seu estado atual de CI é [NV] até nova confirmação ao vivo. PR #220 teve Host e Android verdes na consulta de 2026-11-17 22:01:21 -04:00; estado posterior [NV].

## 2026-11-18 — Bloco 5: escolha 80/20 (ciclo Gate 9 release-only)

- **Critério:** impacto na missão × confiança na evidência ÷ esforço × risco (escala 1–5 cada; denominador declarado).
- **Ranque:** (1) Gate 9 release-only = (5×5)/(2×2) = **6,25**; (2) corrigir baselines do aparato SDD = (2×5)/(3×1) = 3,33; (3) implementar 013 TTFA completo = (5×3)/(5×4) = 0,75.
- **Escolha:** **Gate 9 release-only**. Decisão de governança já aprovada (STATE.md 2026-11-18); evidência determinística local: `.github/workflows/ci.yml:116-121` mostra Gate 9 rodando em todo PR/push, e `gh api repos/lfgranja/mdreader/rulesets` retorna vazio (sem ruleset de tags `v*`). CI do PR #222 verde (Host 9m47s, Android 27m29s) e merge em `dev` confirmado (`ff1b20e9`) — "elevar a CI" não é candidato nº1.
- **Evidência remota (verificada ao vivo em 2026-11-18):** ruleset `v* release gates` criado (id `27578490`) e conferido via `gh api`; PR #223 mesclado; remainder [NV]: execução real de `workflow_dispatch` em commit candidato e push de tag `v*` (não realizados — publicação de release fora de escopo).
- **Âncora:** `specs/022-gate9-release-only` (nova). HEAD da factory fonte dos aparatos SDD: `357e1c10bd4782bf5c3dad8666fe010539c25500` (mutou concorrentemente durante a sessão; factory somente leitura).

## 2026-11-18 — Verificação final (Bloco 8)

- **PRs mesclados:** #222 (recuperação WIP 013, `ff1b20e9`) e #223 (Gate 9 release-only, `568a4e9f`). Ambos com CI verde observada ao vivo.
- **Gates locais (PR #222):** fmt PASS; 16/16 testes focados; 1077/1077 workspace (`--profile ci`, PROPTEST_CASES=32); clippy exit 0; censo 136/136 diff limpo; bundles byte-idênticos.
- **Gates CI (PR #223):** Host gates PASS (26m33s); Android `skipping` — prova viva de que Gate 9 não roda em PR.
- **Stashes:** `stash@{0}` (042a6d84) promovido via PR #222 e descartado após merge confirmado; nenhum stash restante.
- **Worktrees/branches desta sessão:** removidas; ledger `done` com manifesto para #221/#220/#222/#223. Trabalho alheio (`fix/gate4-github-actions`, checkout compartilhado) intacto.
- **Factory fonte:** HEAD `357e1c10` (mutou durante a sessão); somente leitura; nenhum artefato transplantado.
- **[NV] remoto:** execução real de `workflow_dispatch` e push de tag `v*`; proteção efetiva do ruleset na criação de tag (payload verificado, aplicação real não exercitada).
- **Dívida:** spec 013 permanece parcial (ver `specs/013-ttfa-synthesis-pipeline/converge.md`); T232/T233 abertos; baselines do aparato SDD ausentes no alvo (checks informativos falham, não são o gate autoritativo).
- **Próximo 80/20 (provisório):** implementar 013 TTFA completo (score 0,75 na fila anterior) — depende de acesso a dispositivo 2201117TY e decisão sobre reabrir o ciclo.

## 2026-11-18 — Bloco 5: escolha 80/20 do ciclo 023

- **Critério:** impacto na missão × confiança na evidência ÷ esforço × risco (escala 1–5 cada).
- **Ranque:** T229/T230/T234 = (5×5)/(3×3) = **2,78**; coerência documental upstream = (1×5)/(2×1) = 2,50; T232/T233 = (5×4)/(5×4) = 1,00; 013 TTFA completo = (5×3)/(5×4) = 0,75.
- **Escolha:** **T229/T230/T234** — follow-ups abertos da spec 010: botão `btnPickDirectory`, listener `directory-selected` e controles de navegação de capítulos na view canônica. Evidência determinística: bridge já expõe e testa `pick_directory`/`next_chapter`/`previous_chapter` (`origin/dev:src-tauri/src/lib.rs:4495-4497,7491-7502`), mas `status_view.html` só tem handler opcional para `#btnPickDirectory` e nenhum botão/listener de capítulo.
- **Não exige autorização nova:** são follow-ups da spec 010 já aceita; mudança reversível, verificável em host. Sem dispositivo necessário. CI de `dev` verde em `3fcba2aa`, então "elevar a CI" não é candidato nº1.
- **Âncora:** `specs/023-directory-ui-controls`. Factory fonte: HEAD `fce55ca` (mutou nesta sessão; somente leitura).

## 2026-11-19 — Verificação final (Bloco 8, ciclo 023)

- **PR mesclado:** #225 (`6deebf81`), CI Host gates PASS (18m32s), Android `skipping` (Gate 9 release-only confirmado novamente).
- **Gates locais ciclo 023:** `cargo fmt --all -- --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo nextest run --workspace --profile ci` 1082/1082; censo 137/137 diff vazio; `PROPTEST_CASES=32 bash scripts/verify.sh` `VERIFY_EXIT=0`; cinco bundles HTML byte-idênticos (`dcefad7e…`).
- **TDD:** RED (2 falhas em botões de capítulo) → GREEN 5/5 no novo alvo `src-tauri::directory_ui_controls`.
- **Âncora:** `specs/023-directory-ui-controls`. Follow-ups 010 T229/T230/T234 fechados; T232/T233 permanecem abertos.
- **Factory fonte:** HEAD `fce55ca`, somente leitura; nenhum artefato transplantado.
- **[NV]:** verificação visual/interação em dispositivo; execução real de `workflow_dispatch` e de tag `v*`; aplicação efetiva do ruleset na criação de tag.
- **Dívidas remanescentes:** spec 013 TTFA parcial; T232/T233; Gate 8 full deferido; baselines do aparato SDD desalinhadas (informativas, não gate do alvo); duplicidade do número 020 em `specs/`.
- **Próximo 80/20:** reavaliar no próximo Bloco 5 — candidatos: coerência documental upstream (2,50), T232/T233 (1,00), 013 TTFA completo (0,75, depende de dispositivo).

## 2026-11-20 — Herança, sincronização e fechamento SDD (Blocos A–D)

- **Âncora:** `.specify/feature.json` = `specs/010-directory-reading`; HEAD `1d23b773`; merge-base com `origin/dev` = `b3e808ad` (24 commits integrados via merge `--no-ff`, conflitos resolvidos sem perda). Factory HEAD usado como fonte: `b1deab1` (somente leitura; nenhum artefato transplantado).
- **Bloqueio A:** trabalho sujo untracked/herdado commitado em `3159482` (19 arquivos: AGENTS.md, dist/index.html, shell_jni.rs, .specgate/, .specify/queue/, docs/governance/, guide-to-strict-rust.md, scripts/ci/, docs de estado). Backup integral em `/tmp/mdreader-full-20261120T153605`. Merge de `origin/dev` em `dev` sem conflitos.
- **Bloqueio B:** `vendor/sherpa-onnx-sys/build.rs` passou a usar o archive vendored (`.tar.bz2` 21,9 MB) no host linux/x64 quando não há env vars, com erro nomeado se ausente/corrompido; `cargo check -p adapter-synthesis` 0 erros sem env var e `bash scripts/verify.sh` gate 1–8 PASS com `VERIFY_EXIT=0` sem `SHERPA_ONNX_LIB_DIR`.
- **Bloqueio C:** 113 artefatos criados por 4 batches (assessments/validations/releases/checklists/spec-plan-tasks de 020-pockettts), 14 IDs duplicados corrigidos em 002/005/009/010, baselines `sdd-completeness` e `converge-append-only` sincronizados. Commit `1d23b77` (161 arquivos, 5811 inserções).
- **Bloqueio D:** `docs/PROGRESSO.md` + `docs/research/index.md` criados; `OBJETIVO.md`/`ROADMAP.md` atualizados com nota datada de 2026-11-20 (append-only).
- **Gates locais (saídas reais):** `scripts/verify.sh` `VERIFY_EXIT=0` (gates 1–8 + scaffold, 1082/1082 nextest); `sdd-completeness: PASS (spec_dirs=24 gaps=0)`; `work-order-dag: PASS (files=24 tasks=840 edges=0 cycles=0)`; `task-id-uniqueness: PASS (duplicates=0)`; `converge-append-only: PASS (files=23 baselined=23)`.
- **PRs:** nenhum aberto (`gh pr list --state open` vazio); #219/#220/#221/#225 todos `MERGED`. Nenhum merge/auto-merge/force-push feito nesta sessão.
- **[NV]:** CI remoto, proteções de branch, gate 9 Android, dispositivo/emulador, verificação visual. Nenhuma afirmação remota confirmada ao vivo além do estado `MERGED` dos PRs via `gh`.
- **Dívidas remanescentes:** 013 TTFA parcial (device proof [NV]); T232/T233; Gate 8 full deferido; duplicidade do número 020; injeção nenhuma detectada.
- **Próximo 80/20:** T232/T233 (score 1,00 — follow-ups de 010 já aceitos, verificáveis em host) ou coerência documental upstream (2,50). Reavaliar no próximo Bloco 5.

## 2026-11-20 — Ciclo 80/20 #024 (T233) e entrega via PR

- **Ranque:** T233 = (5×5)/(2×2) = 6,25 (GO, ~50-80 LOC, host-verificável); T232 = (4×4)/(5×5) = 0,64 (DEFER — novo worker de síntese, 200-350 LOC/4-6h, aceitação device-dependent [NV]). Escolha: **T233**.
- **Branch/feature:** `024-directory-resume-offer` via create-new-feature.sh (NNN de feature.json/check-prerequisites, não do branch). Cadeia SDD completa: assess (go) → specify → clarify → plan → checklist 23/23 → tasks T001-T015 → analyze (PASS, 5 desvios F1-F5 resolvidos na fonte) → implement → converge (append-only) → validate R4 → release R7 (v0.1.0-rc3 proposto).
- **Código:** `park_directory_resume`/`DirectoryResume` sem dead_code; payload `chapterFilename`/`storedOffsetSecs` no evento `directory-selected`; `route_offer` uma vez; JS lê payload e chama `refreshOffer()`; 5 bundles HTML byte-idênticos; teste comportamental no target já pinado + teste 023 T230 ajustado ao contrato com payload.
- **Gates locais finais:** `bash scripts/verify.sh` `VERIFY_EXIT=0` (gates 1-8 + check-shell-purity PASS; 1082/1082 nextest); fmt/clippy exit 0; test_directory_resume 4/4; directory_ui_controls 5/5; 4 checks SDD PASS (25 specs/0 gaps, 855 tasks/0 cycles, 0 dup, 24 converge baselined).
- **Commits/PRs:** `c817fa0` (23 arquivos) na branch 024; PR #228 aberto para dev (Host gates pending no momento da checagem). PR #227 (Bloqueios A-D) OPEN com **Host gates PASS** via `gh pr checks` (10m38s) e Android gate 9 `skipping`. Nenhum merge/auto-merge/force-push/delete nesta sessão.
- **[NV]:** CI final do PR #228, dispositivo Android, janela visual de 3s, TalkBack/foco, gate 9 em artifact release, proteções de branch. Nenhuma afirmação remota além do observado via `gh`.
- **Dívidas remanescentes:** T232 (DEFER, device-dependent), 013 TTFA parcial, Gate 8 full deferido, duplicidade do número 020, injeção nenhuma detectada.
- **Próximo 80/20:** T232 permanece DEFER até acesso a dispositivo; reavaliar coerência documental upstream (score 2,50 na fila anterior) como candidato host verificável.

## 2026-11-20 — Rodada 2: merges, dispositivo e spec 020 (gate de allowlist)

- **Estado:** branch `dev` @ `29cb287f` após `git merge --ff-only origin/dev`. PRs #227 e #228 **MERGED** via `gh pr merge --merge` (22:02:27Z e 22:02:43Z), nenhum PR aberto. Ordem #227 → #228 respeitada (#228 continha #227 como ancestral). Host gates `pass` observado via `gh pr checks` nos dois (10m38s e 7m54s); Android gate 9 `skipping` (release-only).
- **Gate autoritativo confirmado:** `scripts/verify.sh` (constituição: "single entrypoint"; CONTRIBUTING: gates 1-8 + gate 9 fora do script). Factory HEAD `b1deab1` somente leitura, não importada como gate. 4 checks SDD PASS no estado pós-merge: completeness (25 specs/0 gaps), DAG (855 tasks/0 cycles), uniqueness (0 dup), converge (24 baselined).
- **Dispositivo (evidência ADB passiva, sem mutação):** 2201117TY, Android 13/SDK 33/arm64-v8a, serial 53cf9aa7, `app.mdreader` v0.1.0 (versionCode 1000, minSdk 29, targetSdk 37), `.MainActivity` resolvível, **INTERNET AUSENTE** da lista de permissões. Sem serviço/MediaSession ativo; logcat -t 200 sem FATAL/ANR/exceção do app. Ações mutantes (install/am start/force-stop/input/grant/uiautomator/CDP) NÃO executadas — `play_to_audio_ms`, TTFA fim-a-fim, janela visual de 3s e TalkBack permanecem **[NV]**.
- **Spec 020 ("gate de allowlist"):** não existe gate de allowlist definido pela spec — termo é tag de proveniência (`research.md:46`). Gate real é NO-GO registrado: TTFA cold 9804-26113 ms vs ≤1551, warm 6690-10599 ms, RTF 2,035-6,565 > 1, VmHWM ~1,18-1,30 GB. Decisão: **DEFER** — manter NO-GO; integração seria `(4×1)/(5×5)=0,16` vs `(5×5)/(1×1)=25,0` para manter decisão documentada. T006: versão 1.13.8 confirmada localmente (Cargo.toml/Cargo.lock/PROVENANCE/build.rs), mas compatibilidade exata BOS/tokenização com ≥2.0.1 permanece **[NV]** — task mantida `[ ]`.
- **Gate 9 BLOCKED neste host:** JDK 25 instalado (exigido 17), NDK 29-beta3 (exigido 27.0.12077973), platforms android-34/36 (exigido 35), build-tools 36 (exigido 35), ANDROID_HOME/NDK_HOME ausentes. Rust target aarch64-linux-android e cargo-tauri 2.12.4 OK. Não foi instalado nada; alternativa é `workflow_dispatch`/tag `v*` no CI **[NV]**.
- **[NV] remoto:** aplicação efetiva de ruleset/proteções de branch, CI após merges, gate 9 em artifact release, qualidade prosódica/REENVIOS 020, termos de licença Kyutai.
- **Próximo 80/20:** nenhum candidato host-acionável com confiança alta permanece; T232 continua DEFER (200-350 LOC, device + build Android), 013 TTFA parcial, Gate 8 full deferido. Reavaliar se ambiente Android for provisionado.

## 2026-11-20 — Evidência device adicional (leitura passiva via run-as)

- `run-as app.mdreader id` → uid=10284(u0_a284), **DEBUGGABLE confirmado** (flags=DEBUGGABLE no dumpsys).
- `files/snapshots/`: 1 arquivo de 3.897 bytes (hash `c6c1f42f…`), conteúdo "Hamlet — Ato IV, Cena III" (pt-BR) — snapshot de documento persistido localmente.
- `bookmarks.redb`: 1.589.248 bytes (bookmarks/settings persistidos em redb).
- `diagnostic-record.jsonl`: **0 bytes** (sem diagnósticos gerados; app usado antes sem gravação de record — [NV] para comportamento de emissão).
- `voices/`: `.extracted`, `catalog.version` e 3 diretórios de voz Piper — `pt_BR-cadu-medium`, `pt_BR-faber-medium`, `pt_BR-jeff-medium` — mais `shared/`; catálogo fechado faber+cadu+jeff confirmado no dispositivo.
- **Interpretação:** evidência local direta de que o app roda 100% offline com vozes Piper embarcadas e snapshot persistido; nenhuma escrita no dispositivo foi feita. Continuam [NV]: TTFA/play_to_audio_ms, janela visual de 3s, TalkBack, gate 9 (bloqueado no host: JDK 25/NDK 29-beta3/platform 35 ausentes).
- **Correção documental:** `specs/010-directory-reading/tasks.md:144` T233 marcado `[x]` (ciclo 024/PR #228); bug `fastlane-distribution-artifacts` reconciliado em `fix.md`; `docs/PROGRESSO.md` atualizado com nota datada. Commit `a672f0b` publicado em dev.

## 2026-11-20 — CI remoto pós-merge (via gh api/gh run list)

- `gh run list --branch dev --limit 5`: run `44827145915` para `16f2c2a7` — **completed / success** (CI, push, 10m59s, 22:13:27Z). Resolve [NV] de CI no estado atual de `dev`.
- Runs anteriores (`44826940765`, `44826082890`, merges #227/#228) aparecem `cancelled` — cancelamento por concorrência de pushes rápidos na mesma branch; Não implicam falha de gates (o run final success cobre o HEAD atual). `gh api .../status` reporta `pending` no aggregate, mas os check-runs individuais do HEAD final estão success — [NV] para interpretacao de aggregate.

## 2026-11-20 — Rodada 3: debug hook ausente no APK instalado (somente leitura)

- `cmd package resolve-activity --brief -a app.mdreader.action.DEBUG_INGEST` → **"No activity found"**: o APK instalado (v0.1.0/1000) NÃO contém `DebugIngestActivity` do overlay debug (presente só no variant debug do generated Android project). Consequência: TTFA/play_to_audio_ms **não é mensurável** com o app instalado — exige build debug novo (gate 9), bloqueado no host (JDK 25/NDK 29-beta3/platform 35 ausentes).
- `diagnostic-record.jsonl` confirmado **0 bytes** via `run-as app.mdreader wc -c`.
- Gate 9 via CI (`workflow_dispatch`): tecnicamente viável, gera artefato `app-universal-debug-apk` (~214 MB, retenção 14 dias), mas **exige autorização explícita do SE** para disparar + instalar; nenhum dispatch feito. [NV] mantido.
- Nenhuma ação mutante no dispositivo (sem am start/input/install/grant) — 3 endpoints ADB confirmados como o mesmo serial 53cf9aa7.

## 2026-11-21 — Builds release + debug movidos para GitHub Actions (SUCESSO)

- **Motivação:** SE solicitou mover builds para CI para poupar CPU/RAM locais (host sem JDK 17/NDK 27/platform 35).
- **Mudanças no workflow:** novo job `android-release-build` (simétrico ao debug, `needs: host-gates`, guarda `workflow_dispatch || tag v*`), comando `cargo tauri android build --target aarch64` (Tauri CLI não aceita `--release`; release é o padrão), restore do keystore via secret `RELEASE_KEYSTORE_BASE64`, artifact `app-universal-release-apk`.
- **Correções de buildRelease necessárias (3 iterações, todas com evidência de log do CI):**
  1. `--release` inválido na CLI Tauri → removido.
  2. `:app:minifyUniversalReleaseWithR8` — "Optimized resource shrinking requires non-final IDs" → `android.nonFinalResIds=true` em `src-tauri/gen/android/gradle.properties`.
  3. `:app:lintVitalUniversalRelease` — `HardcodedDebugMode` em `AndroidManifest.xml:23` → removido `android:debuggable="true"` (variant debug já controla via `build.gradle.kts:39 isDebuggable=true`).
- **Commits:** `0f9859c` (job release + secret), `f60863c` (fix CLI), `901cabc` (fix R8), `e3d6425` (fix lint). Secret `RELEASE_KEYSTORE_BASE64` criado via `gh secret set` (keystore não versionado; nunca exposto em texto).
- **Dispatch final:** run `44844182975` (`workflow_dispatch` em `dev`) — **Host gates success + Android build (aarch64 debug) success + Android build (aarch64 release) success**. Artifacts: `app-universal-release-apk` 202.342.705 bytes (193 MB), `app-universal-debug-apk` 214.174.473 bytes (204 MB). [NV] verificação de conteúdo/hash dos APKs — apenas tamanho confirmado via API.
- **[NV] restante:** gate 9 local (host ainda sem toolchain), instalação dos APKs no dispositivo (não autorizada), verificação visual/TalkBack/TTFA.

## 2026-11-21 — Instalação do APK debug no device e política de builds por branch

- **Download/instalação:** artifact `app-universal-debug-apk` do run `44844182975` baixado (`/tmp/apk_debug/app-universal-debug.apk`, 265 MB, sha256 `20fd32ea…`). Assinatura incompatível com o app anterior → `adb uninstall` + `adb install` (dados preservados em `/tmp/device_backup/`: bookmarks.redb 1,6 MB + snapshot Hamlet 3,9 KB; restaurados via `/data/local/tmp` + `run-as cp`).
- **Evidência pós-instalação:** `app.mdreader` v0.1.0 (versionCode 1000, DEBUGGABLE), MediaSession `mdreader_media` registrada, sem FATAL/ANR no logcat. `DebugIngestActivity` **ainda ausente** no APK (overlay `src/debug/AndroidManifest.xml` não mesclado no build) — TTFA via hook permanece [NV].
- **Achado novo [NV]:** logcat mostra `Uncaught (in promise) event.listen not allowed. Permissions: core:event:allow-listen, core:event:default`. As capabilities no repo (`src-tauri/capabilities/default.json:7-9`) declaram ambas, mas o `tauri.conf.json` gerado em `src-tauri/gen/android/app/src/main/assets/` (2.424 bytes) contém apenas a chave `capabilities` sem as permissões embutidas — suspeita de capabilities não empacotadas no APK. Não corrigido nesta rodada (fora do escopo da instalação/validação).
- **Tentativas de ingestão:** `ACTION_VIEW` com `file://` e `content://media/external/file/1000851201` (após `am force-stop`) não carregou documento — log repete `get_status_view failed: no document loaded`. SAF picker aberto, mas direcionado ao Google Drive (não storage local). TTFA/`play_to_audio_ms` permanecem **[NV]**; `diagnostic-record.jsonl` = 0 bytes.
- **Política de builds alterada:** push em `dev` → Host gates + build debug; push em `main` → Host gates + `release-version` (SemVer automático: promove `-rc/-alpha/-beta` à base, senão incrementa PATCH) + builds debug e release. `workflow_dispatch` e tags `v*` preservados. Commit `a0cd741`; run `44849183497` disparado automaticamente pelo push em dev, confirmando a política. SemVer em main permanece [NV] até um push real em main.

## 2026-11-21 — TTFA medido no device e correção de capabilities

- **Caminho de ingestão descoberto:** `am start -n app.mdreader/.MainActivity -a android.intent.action.SEND -t text/plain --es android.intent.extra.TEXT "..."` (DocIngest.kt:759-780) ingere sem build novo e produz fingerprint novo (`StartNow`).
- **Evidência device (APK debug do CI, serial 53cf9aa7):** `diagnostic-record.jsonl` passou de 0 para 46 entries. `session_start` transport=playing; `transport_change action=play`; progressão de 0,08 s a ~9,2 s; `transport=completed` + `session_end` reason=completed; MediaSession `active=true`; nenhum FATAL/ANR.
- **TTFA medido: `play_to_audio_ms=12686`** (12,686 s) no entry `{"kind":"transport_change","position_secs":0.08072562358,"detail":"action=progress play_to_audio_ms=12686"}`. Acima do baseline 1551 ms e do SC-002 3000 ms — registro factual, não juízo de aprovação. [NV] comparação com a medição histórica de 2026-11-07 (11.497 ms) e com dispositivo de referência em condições controladas.
- **Causa raiz de `event.listen not allowed` confirmada:** `src-tauri/capabilities/` estava no `.gitignore:131`; `default.json` existia só no working tree → CI compilou ACL vazia. Correção: exceção estreita no `.gitignore` + versionamento de `default.json` (commit `c3d2447`, branch `fix/capabilities-device`, PR #229 aberto para dev). `gen/` continua ignorado. [NV] validação definitiva após reinstalar APK do CI com a correção.
- **Política de builds:** push em dev → debug; push em main → debug + release SemVer (validado em `44849645463`: debug success, release/semver skipped).

## 2026-11-21 — Verificação final do ciclo de capabilities

- PR #229 (`fix/capabilities-device`): CI run `44853226525` — Host gates **success**, Android build debug **success**, Android build release **success**, release-version skipped. 4 checks SDD PASS.
- Gate local `scripts/verify.sh` executado 2x na branch do PR; ambas as execuções abortadas por **SIGTERM externo** (limite de tempo do harness: 1007/1082 e 994/1082, sem FAIL funcional) — não classificado como RED. Evidência autoritativa: CI do PR (green).
- TTFA medido no device: `play_to_audio_ms=12686` via ACTION_SEND/EXTRA_TEXT; 46 entries de diagnóstico, sessão playing→completed, sem FATAL/ANR. 12686 ms excede baseline 1551 ms / SC-002 3000 ms.
- [NV] reinstalar APK com capabilities corretas e confirmar ausência de `event.listen not allowed`;昔日 baseline TTFA não revalidado; proteções de branch/ruleset.

## 2026-11-21 — Merge #229 e bloqueio de reinstalação no device

- PR #229 **MERGED** em dev (`bf7971b6`) com Host gates pass (run `44856655405`). Merge disparam run `44857393032`: Host gates success + Android debug build success. Release/SemVer skipped (esperado em push dev).
- APK debug do run `44857393032` baixado (`/tmp/apk_caps/app-universal-debug.apk`, 265 MB) para reinstalação com capabilities corretas.
- **BLOQUEIO GENUÍNO:** `adb install` falha com `INSTALL_FAILED_USER_RESTRICTED: Install canceled by user` — MIUI exige confirmação manual na tela (instalação via ADB). Tentativas com APK novo e APK antigo falharam. App `app.mdreader` está **AUSENTE** no dispositivo até o usuário autorizar a instalação na tela. Backup em `/tmp/device_backup/` (bookmarks.redb + snapshot Hamlet) e ambos APKs preservados em `/tmp/apk_caps/` e `/tmp/apk_debug/`.
- [NV] Confirmação de que `event.listen not allowed` desapareceu após reinstalação — exige intervenção humana no dispositivo.

## 2026-11-21 — Fechamento do [NV] capabilities + TTFA pós-merge

- Reinstalação do APK do run `44857393032` (com capabilities versionadas) bem-sucedida após confirmação manual do MIUI na tela (autorização do SE + usuário).
- **Verificação determinística:** `adb logcat -d | grep -c 'event.listen not allowed'` → **0**; log mostra `Publish listener registered`. BUG FECHADO.
- **TTFA pós-correção: `play_to_audio_ms=8827`** (8,827 s) via ACTION_SEND/EXTRA_TEXT + keyevent 126; 49 entries de diagnóstico, sessão playing→completed. Melhoria vs medição anterior 12686 ms (condições diferentes; [NV] comparação controlada). Ainda acima do baseline 1551 ms / SC-002 3000 ms — registro factual.
- Bookmarks (1,5 MB) restaurados no dispositivo; snapshot Hamlet não restaurado por permissão SELinux no destino ([NV] não impacta gates).
- dev @ `a4ed35eb`, limpo, nenhum PR aberto, CI pós-merge success (Host gates + debug).

## 2026-11-21 — Bug ACTION_VIEW Hamlet: latência extrema de primeiro áudio

- Usuário reportou: abriu "Hamlet trecho.md" e exibiu `document:1000848919.md` sem tocar. Evidência ADB confirmou sessão `doc=c6c1f42f…` com `progress position_secs=0.0` por ~183 s.
- Causa refinada: `action=buffer preroll_secs=60.000 preroll_cap_secs=60.000` — pre-roll atinge o teto antes do primeiro sample; `play_to_audio_ms=183089` (183 s). Nome exibido errado porque `queryName` (DocIngest.kt:638) retorna null para URI MediaStore e `effectiveUri` vira `content://media/external/file/1000848919`.
- Bug `playback-not-starting-after-file-selection/fix.md` reaberto com evidência (commit `5ded6e99` + atualização). Próximo 80/20: T232 pré-síntese de capítulo (dono natural da latência) — permanece DEFER (200-350 LOC, exige gate 9 + device).

## 2026-11-21 — Ciclo 025 (ACTION_VIEW display name) merged; validação device pendente

- PR #230 **MERGED** em dev (`177cf31e`), Host gates pass (run `44864740458`), CI pós-merge `44866299402`: Host gates success + debug build success. Release skipped (push dev).
- Ciclo SDD 025 completo: assess go, FR-025-001..005, T001-T012, analyze, converge, validate R4, release R7 (v0.1.0-rc4 proposto). 4 checks SDD PASS (26 specs/0 gaps, 867 tasks/0 cycles, 0 dup, 25 converge baselined).
- Código: `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` — fallback MediaStore.Files (`_id=?`) para `queryName`, rejeição de nomes numéricos, `effectiveUri=content://display_name/<nome>` sempre no ACTION_VIEW. Commit `64a75de`.
- Latência 183s (síntese integral do segmento) diagnosticada como **DEFER** (fora do ciclo; exige mudança de chunking/engine, T232/013).
- **BLOQUEIO GENUÍNO:** reinstalação no device falha com `INSTALL_FAILED_USER_RESTRICTED` (MIUI exige confirmação na tela). App `app.mdreader` está **AUSENTE**; APK 025 em `/tmp/apk_025/app-universal-debug.apk` (266 MB), APK anterior em `/tmp/apk_caps/`. Backup em `/tmp/device_backup/`.
- [NV] nome corrigido em device (Hamlet trecho.md) — exige confirmação manual da instalação no MIUI.

## 2026-11-21 — VALIDAÇÃO DEVICE do ciclo 025 (nome ACTION_VIEW) — PASS

- APK 025 instalado no device 2201117TY via `pm install -r /data/local/tmp/apk025.apk` (contornou `INSTALL_FAILED_USER_RESTRICTED` do MIUI).
- `am start -a android.intent.action.VIEW -d content://media/external/file/1000848919 -t text/markdown -f 0x1 app.mdreader/.MainActivity` (FLAG_GRANT_READ_URI_PERMISSION) → documento ingerido, `session_start doc=c6c1f42f…` (fingerprint Hamlet idêntico).
- Sem a flag, `SecurityException: app.mdreader has no access to content://media/external/file/...` — comportamento esperado do Android (a UI do app de arquivos concede a flag).
- **Verificação via WebView CDP (`Runtime.evaluate`):** `docName` = `"HAMLET trecho.md\n—Sem título"`; corpo começa com `HAMLET trecho.md`. Antes da correção: `document:1000848919.md`. **BUG CORRIGIDO E VERIFICADO EM DISPOSITIVO REAL.**
- Latência de 183s (síntese integral do segmento) permanece DEFER (fora do ciclo 025).

## 2026-11-21 — Ciclo 026 merged; validação TTFA em device pendente

- PR #232 **MERGED** em dev (`63817985`) com Host gates pass (`44904113013`) e CI pós-merge `44908647547` success (Host gates + debug build).
- Código: `src-tauri/src/lib.rs` — `PRIMER_BYTES=160`, `split_first_segment_primer`, `prepare_session_document(primer: bool)`, 2 testes em `ttfa_013_tests`.
- Gates locais: `scripts/verify.sh` **VERIFY_EXIT=0** (gates 1-8 + purity, **1084/1084** testes); fmt/clippy PASS; 4 checks SDD PASS (27 specs/0 gaps, 881 tasks/0 cycles, 0 dup, 26 baselined).
- Artefatos SDD completos: assess go, FR-026-001..006, T001-T014, analyze, converge, validate R4, release R7 (v0.1.0-rc5 proposto). Commit `f22fd98b` (15 arquivos).
- APK debug do merge baixado (`/tmp/apk_026/app-universal-debug.apk`, 266 MB). Instalação no device **BLOQUEADA** por `INSTALL_FAILED_USER_RESTRICTED` (MIUI exige confirmação na tela). App `app.mdreader` está ausente no dispositivo; APK também em `/data/local/tmp/apk026.apk`.
- **[NV]** medição de TTFA pós-correção no dispositivo — exige confirmação manual da instalação no MIUI + captura pareada (SC-TTFA-001 é relativo). 183s anterior permanece como baseline documentado, não como resultado do ciclo.
