# Session state — 004-ressoa-dogfood-build (shell bring-up wave)

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
  - F1 no end-of-audio detection: `play()` (lib.rs:2165) enqueues the PCM,
    publishes once, returns; the backend exposes no drain signal
    (`backend.rs:300` is only `enqueue_pcm`); `dispatch_document_end` is
    reachable only via the Kotlin bridge intent, which cannot know (SC-035).
    Session stays Playing forever after the audio ends (2× confirmed:
    21:54 and 22:04 runs; notification frozen + non-dismissable). Intended
    wiring located (2026-10-16): Kotlin `Residency.documentEnd()` and Rust
    `dispatch_document_end` both exist; the missing link is Rust drain
    detection + the JNI command that invokes the Kotlin teardown.
  - F2 bench cannot restart after Stop: `▶ Play` → "illegal transition from
    stopped via play" (UI FALHA); only relaunch replays. The failed attempt
    also requestStart()s the paused stream (zombie track starved to 4265
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
   `is_drained()` (queue empty with speech consumed) + unit test; (b) src-tauri
   ~1 s watcher while Playing — drained → JNI `command_document_end` →
   `ShellHost` → `Residency.documentEnd()` → `document_end` intent →
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
`92129da` · `7ca4c1d` · `1849761` · `72f369e` · `18dd86a` · `7a25821` ·
`c6c3da4` (STATE) · `b856bfc` (R8 keep) · `728557b` (docstring) ·
`cc9164e` (STATE R8) · `2177c67` (vendor sherpa-onnx-sys) · `410a9b1`
(STATE wave-b3) · `02164b6` (patch + lock zip 9.0.4) · `8435260` (v1.7.0
governance) · `9208578` (v1.8.0 bzip2-1.0.6 admission).

## Verification summary
- Host: zip-9 tree gates 1–4 PASS (775 tests); gate-5 green after v1.8.0;
  6 mirror tests pinned.
- Device: Wave A + Wave B mirror path VERIFIED (start/pause/stop publishes;
  notification content + flags); F1–F4 bench findings recorded.
- Upstream: PR #4059 open (no upstream CI on PRs); zip 9 source-compatible.
- Brand / Design: Briefing de logo v2.0.0 aprovado pelo usuário (tríade acústica
  Sona: Azul #8FD8E8, Verde #5AD6A8, Roxo #B9A6F5); assets SVG gerados em
  DESIGN-SONA/assets/logo/; brand.md e tokens.md sincronizados.

