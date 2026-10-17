# Session state — 004-ressoa-dogfood-build (shell bring-up wave)

Updated: 2026-10-16 (wave-b3 installed + Wave B device verification GREEN with
findings; zip vendor landed, patch pending; gate 5 provisionally waived,
constitution v1.6.1; upstream PR #4059 open).

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
    21:54 and 22:04 runs; notification frozen + non-dismissable).
  - F2 bench cannot restart after Stop: `▶ Play` → "illegal transition from
    stopped via play" (UI FALHA); only relaunch replays. The failed attempt
    also requestStart()s the paused stream (zombie track starved to 4265
    underruns at removal — seen in audio_flinger).
  - F3 single publish at start only: no mid-play/section-change updates
    (nothing calls `publish_current_state` during playback; the 20 s heartbeat
    only persists bookmarks) — matters for the T051-06 section-change state.
  - F4 Stop re-posts nothing to the notification (content identical;
    MediaSession updates) — benign, noted.
  - Audio content flow unverified by machine (MIUI `f` counters freeze
    ~0.47 s while the mixer stays fed, 0 underruns) — awaiting user's ears.
- **Gate 5**: constitution 1.6.1 (entry 11 PROVISIONAL); `deny.toml` ignores
  367/368/369; upstream issue #4058, fork `lfgranja` commit `5821f23`, PR
  #4059. Vendor landed `2177c67` (`vendor/sherpa-onnx-sys/` + PROVENANCE,
  crate sha256 `14bbeabb…`, one line zip 2→9). **Patch NOT yet applied**:
  `[patch.crates-io]` + `cargo update` → zip 9.0.4, then drop the 3 ignores +
  entry 11, add entry 12, bump 1.7.0 MINOR (build MSRV note: 1.88).
- Gates 1–4 + 8-shell PASS on the Wave B tree; manifest + shell-purity PASS.
- T051 (human trials; needs picker = Wave C) + T052 pending.
- `gate-8-full: DEFERRED (owner=synthesis)` — unchanged (Exceptions Log 8).

## Next
1. **Zip-9 patch application**: `[patch.crates-io]` + `cargo update` (lock zip
   9.0.4) + `cargo deny check advisories` without ignores; constitution 1.7.0
   (entry 11 deleted, entry 12 added); gates re-run.
2. **F1/F2 bench fixes** (end-of-drain detection → `Complete`; restart
   affordance) before T051 is attempted on the bench.
3. Wave C — picker trigger (`DocIngest.pickDocument` / `handlePickResult`).
4. Full `scripts/verify.sh`; then T051 sheets to the user, T052 diag scan.
   Note: `run-as` fails (release APK not debuggable) — the T052 pull needs a
   debuggable variant or another route.

## Commits this session (004)
`92129da` · `7ca4c1d` · `1849761` · `72f369e` · `18dd86a` · `7a25821` ·
`c6c3da4` (STATE) · `b856bfc` (R8 keep) · `728557b` (docstring) ·
`cc9164e` (STATE R8) · `2177c67` (vendor sherpa-onnx-sys).

## Verification summary
- Host: gates 1–4 + 8-shell green on the Wave B tree; 6 mirror tests pinned.
- Device: Wave A + Wave B mirror path VERIFIED (start/pause/stop publishes;
  notification content + flags); F1–F4 bench findings recorded.
- Upstream: PR #4059 open (no upstream CI on PRs); zip 9 source-compatible.
