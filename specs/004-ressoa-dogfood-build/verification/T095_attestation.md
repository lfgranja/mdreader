# T095 — Final verification attestation (feature 004-ressoa-dogfood-build)

> **What this file is.** The attestation T095 asks for: the recorded outputs of the
> host-checkable verification run — `scripts/verify.sh`, the lint gate, the
> nextest-target inventory, the supply-chain tools, and the Android device artifact
> (gate 9) — each with its exit status and, where it produces one, a hash. It is
> evidence, not a claim: every number below is copied from a run whose log path is
> named.
>
> **What this file is NOT.** It is not a device acceptance record. The
> device-only success criteria (SC-001…SC-010 trials) and the device trials
> T051/T052 are **NOT attested by this host run** — see
> [Device-only criteria — not attested](#5-device-only-criteria--not-attested).
>
> **This run re-attests the Phase-16 tree.** The previous attestation
> (`9841cd0`) covered the pre-Phase-16 tree `3e8e25c`. That attestation is
> superseded; the tree it named no longer exists. The Phase-16 work it did not
> cover is listed with its commit hash in [§7](#7-the-phase-16-work-this-run-attests).
>
> **One gate in this run is NOT green — see [§2.1](#21-clippy--workspace---all-targets---d-warnings--exit-101).**
> `cargo clippy --workspace --all-targets -- -D warnings` exits **101** on this
> tree. The cause is identified, dated, and shown to be **toolchain drift, not a
> Phase-16 regression**. It is reported here rather than omitted, because an
> attestation that records only what passed is not an attestation.
>
> Constitution Principle V ("do not declare success by implementer assertion")
> is the reason this file exists: the gates were run, and their own output is
> quoted.

**Date:** 2026-10-21
**Host:** Linux 7.2.11-200.fc44.x86_64 (x86_64) — Fedora Linux 44 — rustc 1.99.1 / cargo 1.99.1 / clippy 0.1.99
**Attested tree:** `51b20c928173739102191286437ebb882020b00a` (`fix(004): T135 bound the webpki-roots shim waiver with a deterministic gate`) — the tree HEAD at run time. The tree was **clean** (`git status --porcelain` empty) before and after the run.
**Attestor:** the verification session for T095, running the gates as `scripts/verify.sh` defines them.
**Prior attestation:** `9841cd0` (`verify(004): T095 final verification attestation`, 2026-10-20 09:45 -0400) — covered tree `3e8e25c`; **superseded** by this file.

---

## 1. `scripts/verify.sh` — exit status and per-gate table

**Command:**
```
RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 TMPDIR=/home/luis/development/.mdreader-t095-tmp \
  bash scripts/verify.sh 2>&1 | tee /home/luis/development/.mdreader-t095-tmp/verify.log | tee /tmp/opencode/t095_final2_verify.txt
```
**Exit status:** **0**
**Log path:** `/tmp/opencode/t095_final2_verify.txt` (7 069 lines) — byte-identical durable mirror at `/home/luis/development/.mdreader-t095-tmp/verify.log` (both md5 `ce163f50bd397244d713e644d79d42f0`).

The exit status is the exit status of `scripts/verify.sh` itself, captured through
`PIPESTATUS[0]` under `bash -c`, so it is the script's status and not the status
of `tee`.

| # | Gate | Result | Evidence (from the log) |
|---|------|--------|-------------------------|
| 1 | `gate-1 format` | **PASS** | `cargo fmt --all -- --check` clean (log line 2) |
| 2 | `gate-2 tests` | **PASS** | `cargo nextest run --workspace` — *847 tests run: 847 passed (16 slow), 2 skipped*; live-vs-pinned target two-way diff clean (log lines 3–889; summary line 888) |
| 3 | `gate-3 documentation` | **PASS** | `cargo doc --workspace --no-deps` builds (log lines 890–1436) |
| 4 | `gate-4 lint-drift` | **PASS** | every workspace member declares `[lints] workspace = true`; the root lint tables equal the pinned T001 rigour set (log lines 1437–1438) |
| 5 | `gate-5 supply-chain` | **PASS** | `cargo deny check advisories licenses bans sources` → `advisories ok, bans ok, licenses ok, sources ok` (log line 6039); `cargo audit` → `warning: 2 allowed warnings found` (log line 6060); the RUSTSEC-2026-0381 waiver bound asserted (log lines 6061) |
| 6 | `gate-6 deny-list` | **PASS** | no constitution-denied crate in any manifest or in `Cargo.lock` (log lines 6062–6063) |
| 7 | `gate-7 closed-set` | **PASS** | no catch-all `_ =>` match arm in `crates/*/src` (log lines 6064–6065) |
| 8a | `gate-8-shell pipeline` | **PASS** | `engine-core` handoff/stand-in/pitch/shape suites — *33 tests run: 33 passed* (log line 6106); `player::prop_transport_math` — *29 tests run: 29 passed* (log line 6142) |
| 8b | `gate-8-full` | **DEFERRED** | `gate-8-full: DEFERRED (owner=synthesis)` (log line 6144) — see [§6](#6-gate-8-full--deferred-never-a-pass) |
| — | `check-manifest.sh` | **PASS** | `check-manifest: PASS (/home/luis/development/mdreader/src-tauri/gen/android/app/src/main/AndroidManifest.xml)` (log line 6147) |
| — | `check-shell-purity.sh` | **PASS** | `check-shell-purity: PASS (no forbidden call shapes; vocabulary confined to the T081 allowlist in type/getter/delegation positions; SC-057 closed sets and engine registry conform; spec→test traceability holds)` (log line 7068) |

Script's own closing line (log line 7069):

```
verify.sh: gates 1-8 complete — gate-8-shell PASS, gate-8-full DEFERRED (owner=synthesis)
```

### 1.0 Environmental conditions of this run (disclosed)

Two host conditions were corrected before the run; neither changed a gate, a lint
level, or a test invariant, and both are recorded here so the log can be read
honestly.

* **Scratch space.** The first two attempts failed for **environmental**, not
  product, reasons. Attempt 1 failed with `redb storage failure: I/O error: No
  space left on device (os error 28)` — `/tmp` is a **2.0 GB tmpfs that was at
  100 %**, one 1.4 GB orphaned `cargo install` temp dir (`/tmp/cargo-installXBsk3G`,
  no process holding it) accounting for most of it. Attempt 2 failed with
  `No such file or directory (os error 2)` at `/home/luis/.cache/mdreader-verify-tmp/.tmpXXXXXX`
  — the shared scratch parent was deleted mid-run by a concurrent sibling
  verification session in the same repository. The orphaned temp dir was removed,
  and this run used a **run-unique, disk-backed** `TMPDIR`
  (`/home/luis/development/.mdreader-t095-tmp`) so no other session could remove
  it. The failing attempts' logs are kept at
  `/tmp/opencode/t095_final2_verify_enospc.txt` (attempt 1) and are not the
  attestation evidence. **No gate, lint, test, assertion or threshold was
  changed, relaxed or skipped to reach the green run.**
* **Rebuild determinism.** The `player` package was `cargo clean -p player`-ed and
  the whole workspace re-linted from cold before the clippy measurement in
  §2.1, so the clippy result there is not a cached no-op.

---

## 2. Lint and inventory gates

### 2.1 `cargo clippy --workspace --all-targets -- -D warnings` — exit **101**

**Command:**
```
RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 TMPDIR=/home/luis/development/.mdreader-t095-tmp \
  cargo clippy --workspace --all-targets --keep-going -- -D warnings
```
**Exit status:** **101** (FAIL) · **Log path:** `/tmp/opencode/t095_final2_clippy_keepgoing.txt`
**Reproduced with:** `RUSTC_WRAPPER=` (no sccache) **and** with sccache enabled
plus a forced recompile of the affected targets — `/tmp/opencode/t095_final2_clippy.txt`
and `/tmp/opencode/t095_final2_clippy_sccache.txt` — and again after
`cargo clean -p player` (cold build of the whole `player` package):
`/tmp/opencode/t095_final2_clippy_clean.txt`. The failure is deterministic.

**Inventory (101 lint errors across 4 packages):**

| Lint | Count | Where |
|------|-------|-------|
| `unfulfilled_lint_expectations` on `#[expect(clippy::float_cmp, reason = "…")]` | **86** | `crates/player/tests/**`, `crates/session/tests/prop_resume.rs` |
| `clippy::assert_is_empty` (new `clippy::pedantic` lint) | **15** | `crates/player/src/{cli,position,backend,earcon}.rs`, `src-tauri/src/{surface,failure}.rs`, `crates/engine-core/src/fallback.rs` |

Failing compilation units: `player` (lib test + 16 test targets), `engine-core`
(lib test), `session` (test `prop_resume`), `src-tauri` (lib test).

**Diagnosis — toolchain drift, not a Phase-16 regression:**

1. The clippy diagnostics link `https://rust-lang.github.io/rust-clippy/rust-**1.99.0**/…`.
   The installed toolchain is `rustc 1.99.1 / cargo 1.99.1 / clippy 0.1.99`.
   The prior attestation (`9841cd0`, 2026-10-20 09:45 -0400) ran on **rustc
   1.98.1**.
2. The `stable` toolchain was **replaced at 2026-10-20 22:19:58 -0400** — about
   12.5 hours *after* the prior attestation — and the repository pins **no**
   `rust-toolchain.toml`, so `rustup` tracks `stable` and the lint set moved
   underneath the tree.
3. `clippy::assert_is_empty` is new in this clippy release and now fires on
   `assert!(!x.is_empty())` in `crates/player/src/**` and `src-tauri/src/**`;
   the `float_cmp` behaviour change makes the pre-existing
   `#[expect(clippy::float_cmp, …)]` suppressions **unfulfilled**.
4. **The Phase-16 diff does not touch any of it.** `git diff 3e8e25c HEAD --
   crates/player/` shows one file only — `crates/player/tests/prop_first_audio_3s.rs`
   (T112). The offending `crates/player/src/{cli,position,backend,earcon}.rs`,
   `src-tauri/src/{surface,failure}.rs` and `crates/engine-core/src/fallback.rs`
   are byte-identical to the prior attested tree, and so are
   `crates/player/tests/prop_rewind.rs` (sha256
   `fc9e945a60ef7fb490969d4c0e06cacab08fd883faa89856e95fed3da3115b7f` at both
   `3e8e25c` and `51b20c9`) and
   `crates/player/tests/prop_resume_boundary.rs`.
5. The `#[expect(clippy::float_cmp, …)]` attributes were introduced by `db73577`
   (`refactor(004): satisfy the workspace clippy rigour set in the test suites`),
   which **is an ancestor of the prior attested tree** `3e8e25c`. The prior
   attestation's clippy exit 0 therefore did not survive the toolchain upgrade.

**Why the constitution's gates still pass.** The constitution's *Verification
Gates* table (`constitution.md:934`) numbers exactly nine gates — format, tests,
documentation, lint-drift, supply-chain, deny-list, closed-set, pipeline,
android-build — and `scripts/verify.sh` is the single entrypoint for gates 1–8.
**Clippy is not a numbered gate**: it is the mechanism behind the *Rust Code
Rigor* lint levels, which gate 4 enforces by comparing the root
`[workspace.lints]` tables against the pinned T001 set (gate 4 **PASS**). The
unfulfilled expectations are stale *suppressions* of a lint that no longer fires
— the opposite of "rigour quietly weakened" (gate 4's actual subject) — but they
do leave `cargo clippy --workspace --all-targets -- -D warnings` red.

**Status: OPEN, and NOT repaired by this attestation.** The repair belongs in
code (drop the stale `#[expect(clippy::float_cmp, …)]` attributes and replace the
15 `assert!(!x.is_empty())` sites with `assert!(!x.is_empty())`-equivalent
`assert_ne!` forms) or in a toolchain pin — **both outside this session's
scope**, which is read-only verification plus the two evidence commits. No lint
level, `Cargo.toml` or `clippy.toml` was weakened, and no suppression was
removed or added, to reach the green result reported above.

### 2.2 `cargo fmt --all -- --check` — exit **0**

**Command:** `RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 cargo fmt --all -- --check`
**Exit status:** **0** (clean — no diff) · **Log path:** `/tmp/opencode/t095_final2_fmt.txt`
This is the same command gate 1 runs (log line 2, `gate-1 format: PASS`).

### 2.3 Nextest target inventory and the two-way diff

**Command:** `cargo nextest list --workspace` (exit **0**), reduced with the same
pipeline `verify.sh` gate 2 uses (`grep -E "::" | sed -E 's/ .*//' | sort -u`).

| Quantity | Value |
|----------|-------|
| Live nextest targets (distinct `package::target` / `package` entries) | **88** |
| Pinned targets in `scripts/expected-tests.txt` (distinct) | **88** |
| Missing (pinned but not live) | **0** — empty |
| Unexpected (live but not pinned) | **0** — empty |
| Two-way diff result | **IDENTICAL** — `cmp` reports the live list and the pinned list byte-for-byte equal after `sort -u` |

The inventory is non-empty (88 entries), satisfying T095's "non-empty" clause.
Both directions of the diff are empty, so the fail-closed two-way diff that gate 2
runs has nothing to report.

---

## 3. Supply chain

**Commands and exit statuses:**

| Command | Exit | Log path |
|---------|------|----------|
| `cargo deny check advisories licenses bans sources` | **0** | `/tmp/opencode/t095_final2_deny.txt` (also inside the gate-5 log, lines 1439–6039) |
| `cargo audit` | **0** | `/tmp/opencode/t095_final2_audit.txt` (also inside the gate-5 log, lines 6040–6060) |

`cargo deny` closes with:

```
advisories ok, bans ok, licenses ok, sources ok
```

`cargo audit` loads 1348 advisories, scans 548 crate dependencies, and closes
with `warning: 2 allowed warnings found` — the **two allowed warnings** remain
and are unchanged:

| Crate | Version | Advisory | Status |
|-------|---------|----------|--------|
| `proc-macro-error` | 1.0.4 | RUSTSEC-2024-0370 (`unmaintained`) | allowed (constitution Exceptions Log entry 10) |
| `glib` | 0.18.5 | RUSTSEC-2024-0429 (`unsound`) | allowed |

### 3.1 The RUSTSEC-2026-0381 waiver — constitution Exceptions Log entry 13 (PROVISIONAL, constitution v1.9.0)

The advisory **RUSTSEC-2026-0381 / CVE-2026-111943** (2026-10-19) is waived for
`webpki-roots` **0.26.11** by **constitution Exceptions Log entry 13**
(PROVISIONAL), recorded in constitution **v1.9.0** (`constitution.md:1026-1055`,
amendment commit `71d9f36`).

* **Why the version cannot simply be bumped.** The flagged node is a pure
  compatibility shim: its only dependency is
  `[dependencies.parent] version = "1" package = "webpki-roots"` and its
  `src/lib.rs` is `pub use parent::TLS_SERVER_ROOTS;`, so it ships **no root
  certificates of its own**. The trust anchors actually in use come from
  `webpki-roots` 1.0.10, which **has** removed the distrusted
  `emSign ECC Root CA - G3` root (the advisory's own text: "Version 1.0.10+
  removes this root"). The patched set is `>= 1.0.10`; the 0.26 line has **no**
  patched release, and every `ureq` 2.x requires `webpki-roots ^0.26` (only
  `ureq >= 3.1.0` requires `^1.0.0`).
* **Reachability.** Build-time only — `adapter-synthesis` → `sherpa-onnx` 1.13.8
  → vendored `sherpa-onnx-sys` 1.13.8 (build-dependency `ureq` "2.12") → `ureq`
  2.12.1 → `webpki-roots` 0.26.11. Nothing from `ureq`, `rustls`,
  `rustls-webpki` or `webpki-roots` links into any shipped artifact.
* **The lockfile bump** (`9701d98`, `fix(deps): bump rustls-webpki 0.103.16 and
  webpki-roots 1.0.10 (RUSTSEC-2026-0381/0382)`) — lockfile-only, no manifest
  change: `rustls-webpki` **0.103.15 → 0.103.16** (clears **RUSTSEC-2026-0382**,
  distrusted emSign intermediates; `rustls` 0.23.46 requires `^0.103.14`) and
  `webpki-roots` **1.0.9 → 1.0.10** (clears the 1.0.9 instance of
  RUSTSEC-2026-0381 and removes the distrusted root from the bundled anchors).
  The committed `Cargo.lock` carries `rustls-webpki` **0.103.16** (verified
  above).
* **The mechanical bound — the waiver cannot silently widen.** `scripts/verify.sh`
  **gate 5** asserts that the committed lockfile's `webpki-roots` version set is
  **EXACTLY `{0.26.11, 1.0.10}`**. The lockfile is **parsed, not grepped** (an
  `awk` over the `[[package]]` blocks reads each block's `name`/`version` pair),
  so the 0.26.11 block's own `dependencies` array — which literally contains the
  text `"webpki-roots 1.0.10"` — can never be mistaken for a package entry. An
  added version, a different version, or the shim leaving the graph altogether
  all **fail the gate closed**.
* **The bound, verified independently on this run** (same parse, run against the
  committed `Cargo.lock` of tree `51b20c9`):

  ```
  webpki-roots 0.26.11
  webpki-roots 1.0.10
  ```

  which is exactly the expected set — **the assertion holds**, and gate 5 passed
  it inside the run (log line 6061).
* **Tooling form.** A `[[advisories.ignore]]` block for RUSTSEC-2026-0381 in
  `deny.toml`, plus `[advisories] ignore = ["RUSTSEC-2026-0381"]` in a new
  repo-root `.cargo/audit.toml` — gate 5 runs BOTH tools and each reads its own
  ignore file.
* **Bound implementation commit:** `51b20c9` (`fix(004): T135 bound the
  webpki-roots shim waiver with a deterministic gate`), which also records a
  negative control: on a repo copy with the patched `webpki-roots` drifted
  1.0.10 → 1.0.9, `cargo deny` and `cargo audit` still pass (the waiver would
  have silently widened) while the new gate-5 assertion **fails**, naming the
  offending version 1.0.9.
* **PROVISIONAL exit condition.** The `webpki-roots` 0.26 shim leaving the graph
  — i.e. the vendored `sherpa-onnx-sys` build-dependency moving to `ureq` 3.x
  (which requires `webpki-roots ^1.0.0`), or upstream publishing a 0.26.x
  release carrying the removal; then both ignore records and entry 13 are
  deleted. Re-evaluate whenever `webpki-roots` or `ureq` changes version in the
  lockfile.

---

## 4. Android device artifact (gate 9)

**Command:**
```
RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 TMPDIR=/home/luis/development/.mdreader-t095-tmp \
  JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=/home/luis/development/.mdreader-t095-tmp/gradle-tmp \
  cargo tauri android build --target aarch64
```
**Exit status:** **0**
**Log path:** `/tmp/opencode/t095_final2_android.txt` (durable mirror:
`/home/luis/development/.mdreader-t095-tmp/android.log`)

The build compiled the Rust side for the requested target
(`target/aarch64-linux-android/release/libsrc_tauri.so`, installed as the
`arm64-v8a` jniLibs entry) and produced both device artifacts:

```
    Finished 1 APK at:
        /home/luis/development/mdreader/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk

    Finished 1 AAB at:
        /home/luis/development/mdreader/src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab
```

| Artifact | Path | Size | sha256 |
|----------|------|------|--------|
| APK (universal, release) | `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk` | 118 395 410 B | `33a1b0aeeacf8c9b7ea1f6b1abf093038deee94442f856c2c422e2a3b558195b` |
| AAB (universal, release) | `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab` | 86 272 347 B | `965a87b3a83d38d171eff711b5f50f44f3aa29765819718998f473a4d8c97950` |

Both hashes are of the artifacts as built by this run, taken from the files on
disk after the build completed.

> The first attempt at this gate failed with `java.io.IOException: Não há espaço
> disponível no dispositivo` in `:app:packageUniversalReleaseBundle` — the Gradle
> daemon's `java.io.tmpdir` is `/tmp`, the 2.0 GB tmpfs, which was full. Freeing
> the tmpfs and pointing the JVM's `java.io.tmpdir` at the disk-backed scratch
> directory fixed it. The failure was in the packaging step's scratch space, not
> in the artifact's content.

### 4.1 Non-hollow artifact check (the dex carries the app's classes)

Gate 9 attests that the build exits zero; it does **not** attest that the app's
classes are inside the artifact. That is the hollow-APK regression class of
failure (`ff6da8b`), so the produced APK's dex was inspected directly:

```
$ unzip -Z1 app-universal-release.apk | grep -E '^classes.*\.dex$'
classes.dex

$ unzip -p app-universal-release.apk 'classes*.dex' | strings | grep -F '<class>;'
```

| Required Dalvik type | Present in the dex |
|----------------------|-------------------|
| `Lapp/mdreader/MainActivity;` (launcher activity) | **present** |
| `Lapp/mdreader/PlaybackService;` (resident foreground service) | **present** |
| `Lapp/mdreader/FocusMirror;` | **present** |
| `Lapp/mdreader/MediaMirror;` | **present** |
| `Lapp/mdreader/Residency;` | **present** |
| `Lapp/mdreader/RoutingMirror;` | **present** |
| `Lapp/mdreader/TauriActivity;` (generated Tauri plumbing / dex-parse canary) | **present** |

**Result: 7 / 7 present — the artifact is NOT hollow.** (This is the same class
set that `scripts/check-apk-classes.sh` asserts; that script points at the
*debug* APK, so the release artifact was checked directly with the equivalent
`unzip -p … 'classes*.dex' | strings | grep -F` form.)

---

## 5. Device-only criteria — not attested

**This host run does NOT attest the device-only criteria.** Nothing below is
claimed, implied, or discharged by this attestation:

* **SC-001 … SC-010** — the on-device trial success criteria (10/10 trials for
  five-minute continuous playback, headphone-disconnect pause ≤ 1 s, permitted-
  interruption ducking to 15–25 % gain, lock-screen transport controls,
  zero false `transport: "playing"` diagnostic entries, foreground-service
  declaration check, resume after unexpected process termination, blind-test
  locked trials, ≤ 3 s press-to-first-audible, and zero document body text in
  the notification / lock screen / app UI). Each requires a physical device
  run with a trial sheet; none was performed in this run.
* **T051** — the device trial for the lock-screen / focus / routing / ≤ 3 s
  play-to-audio behaviour.
* **T052** — the on-device diagnostic-record acceptance audit.

All of them require the **reference phone (Redmi 2201117TY)**; no phone was
attached to this host. No trial sheet is attached, because none was produced by
this run. T095's task text asks that the T051/T052 trial sheets be attached "if
available" — they are **not available**, and their absence is recorded here
rather than papered over. T051/T052 therefore **remain open**, and the
SC-001…SC-010 criteria remain **un-attested**.

---

## 6. `gate-8-full` — DEFERRED (never a pass)

The constitution's full gate 8 (byte budget < 5000 bytes/request, SSML
capability, sentence-level alignment) is **DEFERRED, owner=synthesis** — it is
**not** a pass and is not claimed as one here. Constitution Exceptions Log
entry 8 (`.specify/memory/constitution.md:975`) splits gate 8 for
`001-android-media-shell`: `verify.sh` reports `gate-8-shell: PASS` for the
feature-scoped handoff-conformance suites and `gate-8-full: DEFERRED
(owner=synthesis)` until the synthesis feature discharges it with SC-044.

**SC-044 remains open.** `specs/002-fr-024-synthesis-adapter/tasks.md` records
SC-044 as still `DEFERRED` (T039/T057/T076 attestation clauses: "`SC-044` still
`DEFERRED` per `constitution.md:643`"; "SC-044 stays deferred"). The 004 task
text (T048) therefore requires `gate-8-full: DEFERRED (owner=synthesis,
reason=SC-044-open)` — which is what the run printed (log line 6144). The script
never prints a bare `gate-8: PASS` while those inputs are absent, and this
attestation never claims one.

---

## 7. The Phase-16 work this run attests

Every Phase-16 task below is attested by the **same run** recorded above: its
commit is in the history of tree `51b20c9`, its behaviour is covered by the 847
green tests of gate 2, and its artefacts are inside the gate-9 APK.

| Task | What it landed | Commit(s) |
|------|----------------|-----------|
| T112 | RED-FIRST rewrite of `crates/player/tests/prop_first_audio_3s.rs` against a **real** production seam (the old test was a tautology over a self-written event list) | `244e652` |
| T113 | ≤3 s first-audible path: enqueue and play the **first** synthesized segment, then synthesize/enqueue the remainder | `24aab82` |
| T114 | RED-FIRST property for the explicit-resume cold start surviving the offer-window expiries | `07e8e34` |
| T115 | Latch the `explicit_resume` requirement across offer-window expiries; keep the pending open parked until the reader resumes | `cf32cf0` |
| T116 | RED-FIRST contract test for the in-app resume offer surface (`src-tauri/tests/test_offer_dialog.rs`) | `db30345` |
| T117 | Render the 3-second continue-vs-start-over dialog on the single in-app screen and wire both choices (`resume_at_stored` / `start_over`); keep the dialog on the reopen path only | `05e2de1`, `5bacea4` |
| T118 | RED-FIRST contract test for the lock-screen explicit resume control (`src-tauri/tests/test_lockscreen_resume_control.rs`) | `7c66939` |
| T119 | Add the lock-screen resume action/callback (render-only, zero computation) and admit the plain-play verb to the parked-open resolution | `05f4306`, `503e6ec` |
| T120 | Render the pt-BR progress label on the lock-screen surface | `17e4e0c` |
| T121 | Record per-section progress entries in the diagnostic record at the consumed-unit crossing | `9d1e845` |
| T122 | Amend `contracts/diagnostic-trace.md` to state the N:1 product-reason → error-taxonomy mapping, mirrored in the feature directory | `b21ccf7` |
| T123 | Correct the stale `frontend_mirror` doc comment | `2485126` |
| T124 | Close the superseded task text (T071/T072/T080/T085/T086) with notes pointing at T089/T090/T099 — no ID renumbered or deleted | `00c0b50` |
| T125 | Resolve the untracked artifacts claimed by T107 (`.github/dependabot.yml`, the stray nested duplicate, orphaned cargo-vet state) | `f44e01f` |
| T126 | Hoist the second inline offset arithmetic to T111's `assert_offset` helper | `c6bf8be` |
| T127 | Replace the English title fallback on the lock screen | `78b7bb1` |
| T128 | Add the INTERNET-absence assertion to `scripts/check-manifest.sh` (the gate now matches the T106 claim) | `30b0415` |
| T129 | Translate the lock-screen action labels to pt-BR | `ded005f` |
| T130 | Translate the resident-pause notification strings to pt-BR | `63b0d95` |
| T131 | Adjudicate the resident stop affordance: the resident-paused surface is a **distinct** surface outside FR-002's lock-screen prohibition; `contracts/surface.md` amended accordingly | `fbfe42f` |
| T132 | Pair the T131 contract amendment with `spec.md` FR-002 per `spec.md:146` | `4c1f9f7` |
| T133 | Pair the resident carve-out with `data-model.md` | `52c8758` |
| T134 | Complete the resident carve-out coordination in `plan.md` and `spec.md` Key Entities | `c1e8750` |
| T135 | Supply-chain waiver + its mechanical bound: the lockfile bump, the constitution Exceptions Log entry 13, and the gate-5 `webpki-roots` version-set assertion | `9701d98`, `71d9f36`, `51b20c9` |

### 7.1 The RED → GREEN evidence trail

The four red-first test commits and their green counterparts, in order. The RED
state of each is recorded in the red commit itself, and the GREEN state is
witnessed by the same gate-2 run that produced *847 tests run: 847 passed*.

| Pair | RED commit (test, recorded failing) | GREEN commit (implementation) | RED evidence in the commit |
|------|-------------------------------------|-------------------------------|----------------------------|
| T112 → T113 | `244e652` `test(004): T112 first-audio property red-first` | `24aab82` `feat(004): T113 stream the first chunk so the reading is audible at once` | The commit body states the observed RED order — `synthesize → synthesize → synthesize → enqueue(pcm) → play`, the first enqueue landing **after the LAST synthesis** — and names the stale comment at `src-tauri/src/lib.rs:3759` that T113 had to delete. |
| T114 → T115 | `07e8e34` `test(004): T114 explicit-resume expiry red-first` | `cf32cf0` `fix(004): T115 latch the explicit-resume requirement across offer-window expiries` | The commit body quotes the RED output verbatim: `left: NoOffer, right: ExplicitResume` on expiry 2, minimal failing input `(StoredPosition { offset_secs: 0.0, first_yield_ms: 0, end_kind: Interrupted }, 2)`, `Summary: 4 tests run: 3 passed, 1 failed`. It also names the defective site (`crates/session/src/offer.rs:340-363`, `src-tauri/src/lib.rs:1791-1816`) and the co-owned test that pinned the defect (`prop_bookmark_lifecycle.rs:516-521`). |
| T116 → T117 | `db30345` `test(004): T116 in-app offer dialog red-first` | `05e2de1` `feat(004): T117 render the in-app resume offer dialog on the single screen` | The commit **adds** `src-tauri/tests/test_offer_dialog.rs` (472 lines) and pins the target in `scripts/expected-tests.txt` — a test that did not exist before, asserting the two-choice contract that the surface did not implement. *Its commit body is empty: unlike T112/T114/T118, T116's RED transcript is not written into the commit message; the RED evidence for this pair is the added failing test plus the implementation commit that follows it.* |
| T118 → T119 | `7c66939` `test(004): T118 lock-screen resume control red-first` | `05f4306` `feat(004): T119 add the lock-screen explicit resume control` | The commit body names the two RED assertions (the mirror's forwarding vocabulary carries the plain-play verb and nothing else; the bridge's parked-open resolution admits only `ResumeAtStored`/`StartOver`) and the two GREEN guards (no Stop control and no prohibited field on the published payload; an unexpected end leaves the reading waiting). It states: "No production change: T119 owns the implementation." |

All four RED test artifacts are present in the attested tree and are covered by
the green run:

| Artifact | Present in tree `51b20c9` |
|----------|----------------------------|
| `crates/player/tests/prop_first_audio_3s.rs` | yes |
| `crates/session/tests/prop_explicit_resume_survives_expiry.rs` (+ its checked-in proptest failure seed) | yes |
| `src-tauri/tests/test_offer_dialog.rs` | yes |
| `src-tauri/tests/test_lockscreen_resume_control.rs` | yes |

---

## 8. Summary of what this attestation does and does not establish

**Established by this run (host-checkable, evidence-named):**

* `scripts/verify.sh` exits **0**; gates 1–7 and gate-8-shell **PASS**;
  `check-manifest.sh` and `check-shell-purity.sh` **PASS**.
* `gate-8-full` is **DEFERRED** (owner=synthesis, reason=SC-044 open) — recorded
  as a deferral, **never** as a pass.
* `cargo fmt --all -- --check` is clean (exit **0**; gate 1).
* `cargo nextest list --workspace` is non-empty — **88** live targets vs **88**
  pinned in `scripts/expected-tests.txt`, two-way diff empty in **BOTH**
  directions (`cmp` byte-identical).
* The workspace suite is green: **847 tests run, 847 passed (16 slow), 2
  skipped**; gate-8-shell's own suites: 33/33 (`engine-core`) and 29/29
  (`player::prop_transport_math`).
* The supply chain is clean: `cargo deny check advisories licenses bans sources`
  exit **0** (`advisories ok, bans ok, licenses ok, sources ok`); `cargo audit`
  exit **0** with the **2 allowed warnings** (`proc-macro-error` /
  RUSTSEC-2024-0370, `glib` / RUSTSEC-2024-0429) unchanged.
* The RUSTSEC-2026-0381 waiver (constitution Exceptions Log entry 13,
  PROVISIONAL, constitution v1.9.0) is in force and **its mechanical bound
  holds**: the committed `Cargo.lock` carries **exactly** `webpki-roots` 0.26.11
  and 1.0.10, with `rustls-webpki` at 0.103.16 after the `9701d98` lockfile bump.
* The Android device artifact builds (`--target aarch64`, exit **0**):
  `app-universal-release.apk`, sha256
  `33a1b0aeeacf8c9b7ea1f6b1abf093038deee94442f856c2c422e2a3b558195b`;
  `app-universal-release.aab`, sha256
  `965a87b3a83d38d171eff711b5f50f44f3aa29765819718998f473a4d8c97950`;
  **not hollow** — 7/7 required app classes present in the dex.
* All Phase-16 tasks T112–T135 are landed and are attested by this run (§7).

**NOT green in this run — reported, not hidden:**

* `cargo clippy --workspace --all-targets -- -D warnings` exits **101** (§2.1):
  86 unfulfilled `#[expect(clippy::float_cmp, …)]` suppressions and 15
  `clippy::assert_is_empty` hits, across `player`, `engine-core`, `session` and
  `src-tauri`. Diagnosed as **clippy 1.99 toolchain drift** (the `stable`
  toolchain was replaced at 2026-10-20 22:19 with rustc/clippy 1.99.1; the
  repository pins no `rust-toolchain.toml`), **not** a Phase-16 regression — the
  affected sources are byte-identical to the prior attested tree `3e8e25c`.
  **Status: OPEN.** Not repaired here (read-only scope), and no lint level was
  weakened to hide it.

**Not established by this run:** the device-only criteria — SC-001…SC-010 and
the T051/T052 trials (§5). Those require the reference phone (Redmi 2201117TY),
remain open, and are explicitly not claimed by this host run.

**Scope note:** this attestation covers the host-checkable gates and the
artifact build only. It does not modify `spec.md`, `plan.md`, `contracts/`, the
constitution, or any code file; it does not weaken any gate, lint, or test
invariant; and it closes no device trial.
