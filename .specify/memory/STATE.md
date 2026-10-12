# Session state — 001-android-media-shell (US4/US5 complete; all gates GREEN)

Updated: 2026-10-12 (wave 3 landed; US4 + US5 gates attested).

## Position
- Wave 1 LANDED: T057, T061c, T065, T073, T058–T060
  (`57bb525`), T075 (`247ecbb`).
- Wave 2 LANDED: T064 Residency.kt (`0ad77cc`) +
  shell-ipc.md residency intent set; T074 diag scanners +
  CLI (`1f3d72f`); T065b status view (`99e71c4`); T063
  session CLI (`aa3e10f`); T076 recovery machine
  (`4b9f78b`); T061 surface resolver (`97030af`).
- Wave 3 LANDED: T062 MediaMirror.kt (`3349fcf` —
  7 accepted deviations adjudicated in the commit msg,
  purity gate PASS, Kotlin compile carried to T086);
  T078 failure.rs (`6da537d` — closed 1:1 taxonomy
  mapping, T065 getters single-source, T076 decision
  carried verbatim; 38/38 src-tauri).
- Docs checkoffs: T062+T078 (`cc0a420`), T063+T074
  (`9833aa7`), T061+T076 (`125d99c`).
- Gates: **T066 US4 GREEN** (`0f900eb` — US4 suites
  97/97; expected-tests.txt regenerated 27→47 entries:
  44 package::target + 3 bare lib-binary entries,
  decision recorded); **T079 US5 GREEN** (`a02d0d2` —
  US5 suites 61/61; expected-tests.txt re-checked
  identical, no diff to commit).
- Workspace: `cargo nextest run --workspace` — **525
  passed (14 slow), 1 skipped**. Baseline before US4/US5
  impl: 299+1.
- All five story gates GREEN: T031, T041, T048, T066,
  T079.

## Next
1. T081 (in flight): check-manifest.sh + check-shell-purity.sh
   hardening (minSdk 29, 9-file allowlist, SC-057 registry
   check, spec→test traceability).
2. T084 host gates: `./scripts/verify.sh` full green run
   (gates 1–7 PASS + gate-8-shell PASS + gate-8-full
   DEFERRED) — the authoritative attestation run; gate 9
   (cargo tauri android build --target aarch64) is
   SDK-blocked on this host (ANDROID_HOME unset).
3. T085 quickstart validation; T086 on-device (hardware-
   blocked, document carried-forward); T087 VERIFICATION.md
   attestation. ONLY T084–T087 attest success.

## Phase 8 landed
- T080 (c824bfc): verify.sh gates 1–8 runner (two-way
  target diff, lint-drift both directions, deny-list
  name-boundary scan, comment-stripped closed-set scan,
  gate-8-shell conformance mapping, gate-8-full DEFERRED
  reporting) + src-tauri [lints] workspace = true
  (build.rs doc, lib.rs run()/register_android_plugin()
  docs, AppState Debug, .expect → unwrap_or_else).
- T082 (e41c3dd): docs/architecture.md, ipc.md,
  fallback-order.md, diagnostic-record.md (873 lines).
- T083 (79277ae): tests/device/README.md + trials.md
  (48 trial procedures at 10/10 + routing sections).
- Governance (6d655bb): constitution v1.4.0 → v1.5.0 —
  CDLA-Permissive-2.0 admitted (webpki-roots data
  license), Apache-2.0 WITH LLVM-exception synced into
  deny.toml, RUSTSEC-2024-0370 waived (Exceptions Log
  entry 10: compile-time-only proc macro in tauri's gtk
  chain, no CVE). cargo deny all ok; cargo audit exit 0
  (2 quality advisories as allowed warnings: entry-10
  waiver + glib RUSTSEC-2024-0429, Linux-gtk-chain-only,
  absent from the Android artifact).

## Watch-items
- Kotlin compile verification SDK-gated (ANDROID_HOME
  unset, kotlinc absent) — T062/T064 written to
  established mirror patterns (Residency.kt idiom).
- `cargo clippy` workspace-wide fails on pre-existing
  root-Cargo.toml lint-group priority errors (clippy
  1.98 `nursery`/`cargo`, Cargo.toml:26/27) +
  engine-core cli.rs:220/327/389, trait_.rs:212 —
  T080 scope; nextest gates unaffected.
- src-tauri/Cargo.toml lacks `[lints] workspace =
  true` — T080 lint-drift gate; test whether adding it
  breaks the build before committing (pre-existing
  `.expect` at src-tauri/src/lib.rs:97 would violate
  `expect_used`).
- nextest 0.9.133: `--workspace-type all` unsupported
  — use `--workspace`; stop-on-first-failure per
  binary — verify per-target for complete results.
- proptest 1.11: `prop_assert_eq!` moves operands —
  reused operands need `.clone()`/`.as_str()` at the
  assert site.
- Commit discipline: stage ONLY task-owned files
  explicitly; never `git add -A` while parallel agents
  are mid-edit. Agents never commit and never edit
  tests.
