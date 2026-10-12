# Session state — 001-android-media-shell (Phase 8 COMPLETE — T084–T087 attested)

Updated: 2026-10-12 (verification wave landed; ONLY T084–T087
attest success).

## Position
- Phase 8 hardening wave LANDED: T080 (c824bfc),
  T081 (46a2fed), T082 (e41c3dd), T083 (79277ae);
  minSdk-29 restoration (c8991b1); governance
  v1.5.0 (6d655bb); check-offs (466000b).
- **T084 VERIFICATION GREEN** (b06efc8): `./scripts/verify.sh`
  exit 0 — gates 1–7 PASS + gate-8-shell PASS +
  gate-8-full DEFERRED (owner=synthesis); gate 9
  `cargo tauri android build --target aarch64` exit 0
  → app-universal-debug.apk (270 MB, arm64-v8a/
  libsrc_tauri.so) + .aab.
- **T085 quickstart validation** (8839aa5): every
  host-checkable step PASS (§1 host gates, §2 manifest
  + shell-purity + gate 9, §4 host legs); per-step
  record in tasks.md.
- **T086 human-review residue adjudication** (68bc18d):
  all 149 allowlisted occurrences are type/getter/
  delegation/doc-comment/test positions — zero
  computations; on-device trials DEFERRED (hardware).
- **T087 VERIFICATION.md** (this commit): per-criterion
  verdict table for all 59 SCs — 6 host-PASS, 3
  record-scan PASS (SC-005/012/019 with T068b canary
  evidence), 2 PARTIAL (SC-010, SC-037 host legs PASS,
  device legs deferred), 47 trial-SC deferred + FR-026,
  SC-044 + gate-8-full DEFERRED (owner=synthesis).
- **Gate-9 regression found + fixed** (ff6da8b): the
  post-realignment debug APK's dex carried no app
  classes (hollow artifact — no MainActivity). Cause:
  the alpha-generated app module applied no Kotlin
  plugin while android.builtInKotlin=false, and T028b's
  android-shell source-set wiring had never been
  applied. Fixed per the tauri-cli 2.12.1 template
  (KGP application + JvmTarget + compileOptions, host-
  local gen edits) + android-shell sourceSet wiring;
  MediaMirror.kt CATEGORY_MEDIA (nonexistent) →
  CATEGORY_TRANSPORT. Re-verified: both APK variants
  contain MainActivity + PlaybackService + all five
  mirrors; verify.sh re-run exit 0 (525/525, 1 skip).
- **Ledger reconciled** (2c2724b): T001–T018, T028a-
  stub, T028b checked off post-hoc (deliverables
  re-verified present); T055b adjudicated NOT REQUIRED
  (SC-015 passed; contingency never triggered).
- Workspace: `cargo nextest run --workspace` — **525
  passed (7 slow), 1 skipped**; `cargo nextest run
  -p diag` 26/26; `prop_surface` 11/11; `prop_pitch`
  9/9.

## Next
1. On-device trials (T-01–T-48, each 10/10) + FR-026
   offline start + SC-051 failover drill + the SC-010
   locked visual check + SC-037 manual-absence walk +
   the §3 record pulls — on a CI host with an API-29+
   device (procedures + recording sheets ready in
   tests/device/trials.md; verdicts tabulated in
   VERIFICATION.md §5).
2. SC-044 real-engine re-verification — owner: the
   synthesis feature (002-fr-024-synthesis-adapter),
   which also owns gate-8-full and the §9 observation
   (the bridge's read-only `diagnose` voice-asset
   inventory command sits outside the shell-IPC closed
   verb set — document or fold in).
3. FR-024a spec defect (dangling cross-reference,
   spec.md:283 — no 001 definition, owned by 002):
   define the requirement in 001 or remove the mention.

## Phase 8 landed
- T080 (c824bfc): verify.sh gates 1–8 runner +
  src-tauri [lints] workspace = true fix chain.
- T081 (46a2fed): check-manifest.sh (minSdk 29 across
  three declaration forms) + check-shell-purity.sh
  (5 sections; 149-occurrence residue inventory;
  SC-057 registry check with the rust-player exemption;
  three-tier traceability: 135 resolved, 1 exempted
  (FR-024a), 0 unresolved). Negative tests 14/14.
- T082 (e41c3dd): docs/architecture.md, ipc.md,
  fallback-order.md, diagnostic-record.md (873 lines).
- T083 (79277ae): tests/device/README.md + trials.md
  (48 trial procedures + routing S-1–S-6).
- Governance (6d655bb): constitution v1.5.0 supply-chain
  sync.

## Watch-items
- Gate-9 toolchain: the alpha-CLI-generated gen/android
  carried AGP 9.0.0 + KGP 2.1.0 (incompatible — KGP
  2.1.0 cannot coexist with AGP 9's built-in Kotlin
  extension); realigned to the tauri-cli 2.12.1
  template's tested combination (AGP 9.3.1 + KGP 2.2.10
  + Gradle 9.6.1 + android.builtInKotlin=false +
  android.newDsl=false) — host-local, git-ignored gen/
  only; pinned tauri = 2.12.1 untouched. The app
  module now ALSO applies org.jetbrains.kotlin.android
  (template parity — without it, builtInKotlin=false
  leaves the :app module with no Kotlin compiler and
  the APK ships no app classes) and compiles
  android-shell/src/main/kotlin as a main sourceSet
  (T028b wiring, ff6da8b).
- `cargo clippy` workspace-wide fails on pre-existing
  root-Cargo.toml lint-group priority errors (clippy
  1.98 `nursery`/`cargo`, Cargo.toml:26/27) +
  engine-core cli.rs:220/327/389, trait_.rs:212 —
  pre-existing, outside the verify.sh gate set; nextest
  gates unaffected.
- nextest 0.9.133: `--workspace-type all` unsupported
  — use `--workspace`; filter by `--test <target>`
  (positional name filters do not match integration-test
  binary names).
- proptest 1.11: `prop_assert_eq!` moves operands —
  reused operands need `.clone()`/`.as_str()` at the
  assert site.
- Commit discipline: stage ONLY task-owned files
  explicitly; never `git add -A` while parallel agents
  are mid-edit. Agents never commit and never edit
  tests.
