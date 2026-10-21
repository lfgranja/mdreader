# T095 — Final re-attestation (feature 004-ressoa-dogfood-build)

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
> [§6 Device-only criteria — not attested](#6-device-only-criteria--not-attested).
>
> **This run re-attests the tree at `19a1467`.** It supersedes two prior
> attestations: `99c801c` (which covered the tree at `51b20c9`) and `9841cd0`
> (which covered the pre-Phase-16 tree `3e8e25c`). Between `51b20c9` and this
> tree the Phase-16 work list grew by T136–T139, all of which is listed with its
> commit hash in [§7](#7-the-phase-16-work-this-run-attests).
>
> **Every gate in this run is green.** Unlike the prior attestation (which
> recorded `cargo clippy --workspace --all-targets -- -D warnings` exiting 101 on
> clippy-1.99 toolchain drift), the lint-compliance pass now exits **0**, and it
> is no longer a manual pre-attestation step: T138 (`d23942b`) wired it into
> `scripts/verify.sh` gate 4 as a **fail-closed** step, so it is enforced by the
> gate itself — see [§4](#4-the-toolchain-stability-record).
>
> Constitution Principle V ("do not declare success by implementer assertion")
> is the reason this file exists: the gates were run, and their own output is
> quoted.

**Date:** 2026-10-21 (run window 02:33 – 02:54 -0400)
**Host:** Linux 7.2.11-200.fc44.x86_64 (x86_64) — Fedora Linux 44 (KDE Plasma Desktop Edition) — `rustc 1.99.1 (58cf8f9c3 2026-10-13)` / `cargo 1.99.1 (5f94df478 2026-08-27)` / `clippy 0.1.99 (58cf8f9c35 2026-10-13)`; active rustup channel `stable-x86_64-unknown-linux-gnu` (also installed on the host: `nightly`, `1.94.0`, `1.98.0`).
**Attested tree:** `19a14678663362a23e888fae3e2e36ed8440274e` (`docs(constitution): correct the stale gate-4 lint-compliance record`) — the tree HEAD at run time. The tree was **clean** (`git status --porcelain` empty) before the run and empty after it.
**Attestor:** the verification session for T095 — a read-only verification session running the gates as `scripts/verify.sh` defines them. It wrote no application code, no spec, no plan, no contract, no constitution text and no gate script; the only commits it produces are this attestation and the task markers derived from it.
**Prior attestations:** `99c801c` (`verify(004): T095 final verification attestation`, tree `51b20c9`) — **superseded** by this file; `9841cd0` (tree `3e8e25c`) — **superseded** twice over.
**Toolchain in force:** the repo-root `rust-toolchain.toml` (added by T137, `43df07c`) names the tracked channel `stable` with components `rustfmt` and `clippy`; no compiler version is pinned. See [§4](#4-the-toolchain-stability-record).

---

## 1. `scripts/verify.sh` — exit status and per-gate table

**Command:**
```
RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 \
  bash -c 'bash scripts/verify.sh 2>&1 | tee /tmp/opencode/t095_reverify.txt; echo "VERIFY_SH_EXIT=${PIPESTATUS[0]}"'
```
**Exit status:** **0**
**Log path:** `/tmp/opencode/t095_reverify.txt` (7 057 lines) — durable mirror at
`/home/luis/development/.mdreader-t095-reverify/verify.log` (both md5
`27757eca14fe3ad0cfbe08c8eec279b7`).

The exit status is the exit status of `scripts/verify.sh` itself, captured through
`PIPESTATUS[0]` under `bash -c`, so it is the script's status and not the status
of `tee`.

| # | Gate | Result | Evidence (from the log) |
|---|------|--------|-------------------------|
| 1 | `gate-1 format` | **PASS** | `cargo fmt --all -- --check` clean (log line 2) |
| 2 | `gate-2 tests` | **PASS** | `cargo nextest run --workspace` — *847 tests run: 847 passed (2 slow), 2 skipped* (log line 864); live-vs-pinned target two-way diff clean in BOTH directions (log lines 3–865) |
| 3 | `gate-3 documentation` | **PASS** | `cargo doc --workspace --no-deps` builds (log lines 866–1412) |
| 4 | `gate-4 lint-drift` | **PASS** | every workspace member declares `[lints] workspace = true`; the root lint tables equal the pinned T001 rigour set; **step (d)** `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (log lines 1413–1418) |
| 4d | `gate-4 lint-drift-clippy` | **PASS** | `running 4 lint-drift-clippy` → `Finished \`dev\` profile [unoptimized] target(s) in 7.50s` → `PASS` (log lines 1414–1417) — the fail-closed clippy step T138 added |
| 5 | `gate-5 supply-chain` | **PASS** | `cargo deny check advisories licenses bans sources` → `advisories ok, bans ok, licenses ok, sources ok` (log line 6027); `cargo audit` → `warning: 2 allowed warnings found` (log line 6048); the RUSTSEC-2026-0381 waiver bound asserted (the assertion is silent on success — the gate's `PASS` at log line 6049 is only reached if the committed lockfile's `webpki-roots` version set is exactly `{0.26.11, 1.0.10}`) |
| 6 | `gate-6 deny-list` | **PASS** | no constitution-denied crate in any manifest or in `Cargo.lock` (log lines 6050–6051) |
| 7 | `gate-7 closed-set` | **PASS** | no catch-all `_ =>` match arm in `crates/*/src` (log lines 6052–6053) |
| 8a | `gate-8-shell pipeline` | **PASS** | `engine-core` handoff/stand-in/pitch/shape suites — *33 tests run: 33 passed* (log line 6094); `player::prop_transport_math` — *29 tests run: 29 passed* (log line 6130) |
| 8b | `gate-8-full` | **DEFERRED** | `gate-8-full: DEFERRED (owner=synthesis)` (log line 6132) — see [§5](#5-gate-8-full--deferred-never-a-pass) |
| — | `check-manifest.sh` | **PASS** | `check-manifest: PASS (/home/luis/development/mdreader/src-tauri/gen/android/app/src/main/AndroidManifest.xml)` (log line 6135) |
| — | `check-shell-purity.sh` | **PASS** | `check-shell-purity: PASS (no forbidden call shapes; vocabulary confined to the T081 allowlist in type/getter/delegation positions; SC-057 closed sets and engine registry conform; spec→test traceability holds)` (log line 7056) |

Script's own closing line (log line 7057):

```
verify.sh: gates 1-8 complete — gate-8-shell PASS, gate-8-full DEFERRED (owner=synthesis)
```

### 1.0 Environmental conditions of this run (disclosed)

* **Scratch space.** `/tmp` is a 2.0 GB tmpfs that was at 65 % (736 MB free) during
  this run — the same constraint that broke an earlier attempt recorded in the
  prior attestation. The gate suite itself did not need it, and the Android
  build (which does) was pointed at a **run-unique, disk-backed** scratch
  directory (`TMPDIR=/home/luis/development/.mdreader-t095-reverify`,
  `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=/home/luis/development/.mdreader-t095-reverify/gradle-tmp`),
  so the Gradle packaging step's scratch space was never on the tmpfs. **No gate,
  lint level, test, assertion or threshold was changed, relaxed or skipped to
  reach the green run.**
* **Artifact freshness.** The first gate-9 invocation of this run returned exit 0
  but left the APK/AAB files untouched (gradle found its packaging tasks
  up-to-date against the artifacts the previous session had written at 01:07 /
  01:19). To make the hashes unambiguously attributable to *this* run, the two
  output files — both **git-ignored** build outputs under `src-tauri/gen/`, not
  tracked files — were deleted and the gate was re-run. The second invocation
  repackaged both artifacts from scratch (APK mtime `2026-10-21 02:53:22`,
  AAB mtime `2026-10-21 02:54:19`) and the resulting sha256 values are
  **byte-identical** to the ones the prior attestation recorded for tree
  `51b20c9` — an independent reproducibility witness. No tracked file was
  touched by either invocation; `git status --porcelain` was empty afterwards.

---

## 2. Lint and inventory gates

### 2.1 `cargo clippy --workspace --all-targets -- -D warnings` — exit **0**

**Command:**
```
RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 \
  cargo clippy --workspace --all-targets -- -D warnings
```
**Exit status:** **0** (PASS) · **Log path:** `/tmp/opencode/t095_reverify_clippy.txt`
(durable mirror `/home/luis/development/.mdreader-t095-reverify/clippy.txt`)

```
   Compiling src-tauri v0.1.0 (/home/luis/development/mdreader/src-tauri)
    Finished `dev` profile [unoptimized] target(s) in 6.70s
```

The same command is gate 4's fail-closed step (log lines 1414–1417) and it also
exited 0 inside the gate suite. This is the command the prior attestation
recorded as exiting **101** (86 unfulfilled `#[expect(clippy::float_cmp)]`
suppressions + 15 `clippy::assert_is_empty` hits). It is now clean — the repair
was made **in code** by T136 (`8f12acf`, `3b642d8`, `0756fc3`), which changed no
`Cargo.toml`, no `clippy.toml` and no lint level: `git show --stat` on all three
commits shows **no manifest or lint-config file touched**, only Rust sources.

### 2.2 `cargo fmt --all -- --check` — exit **0**

**Command:** `RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 cargo fmt --all -- --check`
**Exit status:** **0** (clean — no diff, output file 0 bytes) · **Log path:** `/tmp/opencode/t095_reverify_fmt.txt`
(durable mirror `/home/luis/development/.mdreader-t095-reverify/fmt.txt`)
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
runs has nothing to report. Raw list: `/tmp/opencode/t095_reverify_nextest_list.txt`
(durable mirror `nextest_list.txt`).

---

## 3. Supply chain

**Commands and exit statuses:**

| Command | Exit | Log path |
|---------|------|----------|
| `cargo deny check advisories licenses bans sources` | **0** | `/tmp/opencode/t095_reverify_deny.txt` (durable mirror `deny.txt`; also inside the gate-5 log, lines 1420–6027) |
| `cargo audit` | **0** | `/tmp/opencode/t095_reverify_audit.txt` (durable mirror `audit.txt`; also inside the gate-5 log, lines 6028–6048) |

`cargo deny` closes with (its log line 4608):

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

### 3.1 The RUSTSEC-2026-0381 waiver — constitution Exceptions Log entry 13 (PROVISIONAL, constitution v1.11.0)

The advisory **RUSTSEC-2026-0381 / CVE-2026-111943** (2026-10-19) is waived for
`webpki-roots` **0.26.11** by **constitution Exceptions Log entry 13**
(PROVISIONAL). The entry was added by amendment commit `71d9f36`
(`docs(constitution): add PROVISIONAL Exceptions Log entry 13 for the
webpki-roots shim waiver`) and is **still in force in constitution v1.11.0** —
the tree attested here.

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
  independently on this run).
* **The mechanical bound — the waiver cannot silently widen.** `scripts/verify.sh`
  **gate 5** asserts that the committed lockfile's `webpki-roots` version set is
  **EXACTLY `{0.26.11, 1.0.10}`**. The lockfile is **parsed, not grepped** (an
  `awk` over the `[[package]]` blocks reads each block's `name`/`version` pair),
  so the 0.26.11 block's own `dependencies` array — which literally contains the
  text `"webpki-roots 1.0.10"` — can never be mistaken for a package entry. An
  added version, a different version, or the shim leaving the graph altogether
  all **fail the gate closed**.
* **The bound, verified independently on this run** (same parse, run against the
  committed `Cargo.lock` of tree `19a1467`):

  ```
  webpki-roots 0.26.11
  webpki-roots 1.0.10
  ```

  `diff` against the expected set `0.26.11` / `1.0.10` is **empty** — exactly
  the expected set — and gate 5 passed the same assertion inside the run
  (log line 6049).
* **Tooling form.** A `[[advisories.ignore]]` block for RUSTSEC-2026-0381 in
  `deny.toml`, plus `[advisories] ignore = ["RUSTSEC-2026-0381"]` in a new
  repo-root `.cargo/audit.toml` — gate 5 runs BOTH tools and each reads its own
  ignore file. `cargo deny` prints the ignore record with its written reason
  (its log line 4608 and the preceding block).
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

## 4. The toolchain-stability record

### 4.1 Constitution v1.10.0 — the Toolchain clause

The constitution's **Technology Stack → Toolchain** clause (`.specify/memory/constitution.md:910-929`,
added by amendment commit `43df07c`, constitution **v1.10.0**) records:

* the workspace **tracks the `stable` channel** and does **not** pin a compiler
  version; the repo-root `rust-toolchain.toml` names that channel and the two
  components the gates need (`rustfmt`, `clippy`), so the toolchain the gates run
  under is a property of the **repository** rather than of the host's rustup
  default (this host carries `stable`, `nightly`, `1.94.0` and `1.98.0`);
* the **MSRV line is the compatibility floor and is unchanged by the clause** —
  MSRV 1.85 for workspace code, rustc >= 1.88 for the full build (the `zip` 9
  build-script dependency of `sherpa-onnx-sys`) — because a tracked `stable`
  channel can never contradict a floor, as `stable` only rises;
* a toolchain bump on the tracked channel is **expected to move the lint set**:
  a new clippy release may add a lint, change a lint's reach, or retire one, so
  "a tree that was green can become red with **no source change at all**";
* **every toolchain bump therefore requires a lint-compliance pass before the
  tree is re-attested** — the pass resolves the moved lints **in code**, never by
  weakening a lint level in `Cargo.toml` or `clippy.toml` and never by a blanket
  suppression;
* a hard version pin is deliberately **not** adopted (it would freeze the
  compiler *and* the lint set, force a download on hosts that lack it, and sit in
  tension with Principle IV's latest-stable posture); the clause is reversible by
  deleting the entry, deleting `rust-toolchain.toml`, and adopting a pin.

The repo-root `rust-toolchain.toml` landed with that amendment and is present in
the attested tree (`channel = "stable"`, `components = ["rustfmt", "clippy"]`).

### 4.2 Constitution v1.10.0 — "A toolchain bump moves the lint set, and the tree must be re-earned"

The **Verification Gates** paragraph (`.specify/memory/constitution.md:1163-1179`)
states the required response to a bump: on every toolchain bump, a
**lint-compliance pass** (`cargo clippy --workspace --all-targets -- -D warnings`,
exit 0) MUST run and MUST be clean **before the tree is re-attested**; the moved
lints are resolved in code, never by weakening a lint level, never by a blanket
`#[allow]`, and never by an `#[expect]` without a written reason. It records the
**triggered instance** explicitly:

> The recorded instance is the 2026-10-20 `stable` bump (rustc 1.98.1 -> 1.99.1,
> clippy 0.1.98 -> 0.1.99), which turned a byte-identical tree red with 86
> `unfulfilled_lint_expectations` and 13 `clippy::assert_is_empty` diagnostics;
> T136 repaired it in code and T137 records the policy.

### 4.3 T138 (`d23942b`) — the pass is now a deterministic, fail-closed gate

Before T138 the policy existed but was not mechanically enforced. `scripts/verify.sh`
gate 4 (lint-drift) checked only the **declared lint LEVELS**; it never invoked
clippy, so the drift the policy names would have failed only the manual
pre-attestation pass, never a gate.

`d23942b` (`fix(004): T138 gate the lint-compliance pass in verify.sh`) gave gate 4
a **step (d)** that runs the lint-compliance pass **inside the gate** and fails
the gate when it does not exit 0. The step fails **closed**: a missing toolchain,
a clippy that cannot run and a non-zero exit are all a **FAILED** gate — there
is no `|| true`, no "the tool is missing -> pass" branch and no skip flag. The
pre-existing checks — (a) `[lints] workspace = true` in every member manifest,
(b) the pinned lint-level set compared against the T001 rigour set, (c) the
intact root `[workspace.dependencies]` table — are **unchanged**, so the clippy
step is an **addition, not a replacement**. No lint level was weakened and no
`#[allow]`/`#[expect]` was added; only `scripts/verify.sh` changed.

The gate's own commit records a **negative control**: on a *copy* of the repo, a
`println!` injected into `crates/doc-structure/src/lib.rs` trips the denied
`clippy::print_stdout` and the copy's `verify.sh` exits 1 with
`verify.sh: FAIL: gate-4 lint-drift: cargo clippy --workspace --all-targets -- -D warnings failed`.

Constitution **v1.11.0** (`19a1467`) then corrected the stale paragraph that
still described the mechanical form as unwired; its SYNC IMPACT REPORT records the
correction as **MINOR** — one governance record corrected, no principle added,
removed or redefined, no MUST weakened, no lint level or gate threshold changed.

### 4.4 The recorded bump and the compliance pass it required

| Fact | Value |
|------|-------|
| The bump | 2026-10-20 22:19:58 -0400 — `stable` replaced: **rustc 1.98.1 → 1.99.1**, **clippy 0.1.98 → 0.1.99** |
| Why it turned a fixed tree red | the lint **set** (which lints exist, which code they reach) moves with the toolchain; the lint **levels** do not — the 2026-10-20 bump retired the reach of `#[expect(clippy::float_cmp)]` (86 unfulfilled expectations) and added `clippy::assert_is_empty` (13–15 hits) |
| The compliance pass required by the constitution | `cargo clippy --workspace --all-targets -- -D warnings`, exit 0, before re-attestation |
| Was it performed? | **YES** — T136 (`8f12acf` player crate, `3b642d8` session crate, `0756fc3` the `assert_is_empty` sites) cleared every unfulfilled expectation and every `assert_is_empty` hit **in code**, touching **no** `Cargo.toml` and **no** `clippy.toml` and weakening **no** lint level |
| Is it enforced now? | **YES** — gate 4 step (d), fail-closed (`d23942b`, T138), described in constitution v1.11.0 (`19a1467`, T139) |
| Result on this run | **exit 0**, inside the gate suite (log line 1417) and standalone (§2.1) |

The toolchain in force for this run was **rustc 1.99.1 / clippy 0.1.99** — i.e.
the post-bump toolchain — and the tree is clean under it. The lint-compliance
pass the constitution required has been performed and is now gate-enforced.

---

## 5. `gate-8-full` — DEFERRED (never a pass)

The constitution's full gate 8 (byte budget < 5000 bytes/request, SSML
capability, sentence-level alignment) is **DEFERRED, owner=synthesis** — it is
**not** a pass and is not claimed as one here. Constitution Exceptions Log
entry 8 (`.specify/memory/constitution.md:1222`) splits gate 8 for
`001-android-media-shell`: `verify.sh` reports `gate-8-shell: PASS` for the
feature-scoped handoff-conformance suites and `gate-8-full: DEFERRED
(owner=synthesis)` until the synthesis feature discharges it with SC-044.

**SC-044 remains open.** `specs/002-fr-024-synthesis-adapter/tasks.md` records
SC-044 as still `DEFERRED` (T039: "`SC-044` still `DEFERRED` per
`constitution.md:643`"; T047 stage 4: "`SC-044` stays `DEFERRED`"; T057: "SC-044
stays `DEFERRED` with all seven discharge prerequisites restated"; T076: "SC-044
correctly stays `DEFERRED`"). The 004 task text (T048) therefore requires
`gate-8-full: DEFERRED (owner=synthesis, reason=SC-044-open)` — which is what the
run printed (log line 6132). The script never prints a bare `gate-8: PASS` while
those inputs are absent, and this attestation never claims one.

---

## 6. Device-only criteria — not attested

**This host run does NOT attest the device-only criteria.** Nothing below is
claimed, implied, or discharged by this attestation:

* **SC-001 … SC-010** — the on-device trial success criteria (30-minute locked
  commute at every speed with 10/10 ≤ 3 s press-to-first-audible, 10/10 per
  interruption class, 10/10 blind per confirmation incl. rate-step sounds-on/off,
  10/10 resume matrix + replay caps, 10/10 purity, 10/10 dismissal-refused-while-playing,
  permitted-interruption ducking to 15–25 % gain, lock-screen transport controls,
  zero false `transport: "playing"` diagnostic entries, foreground-service
  declaration check, resume after unexpected process termination, and zero
  document body text in the notification / lock screen / app UI). Each requires a
  physical device run with a trial sheet; none was performed in this run.
* **T051** — the on-device SC-001…SC-010 trials per `quickstart.md` §3 on the
  **reference phone only** (Redmi 2201117TY).
* **T052** — the on-device diagnostic-record acceptance audit (`diag scan` on the
  pulled JSONL: schema conformance, negative control, zero prose, `error` on
  every failure/refusal, `Trim`/`Gap` markers sane).

All of them require the **reference phone (Redmi 2201117TY)**; no phone was
attached to this host. No trial sheet is attached, because none was produced by
this run. T095's task text asks that the T051/T052 trial sheets be attached "if
available" — they are **not available**, and their absence is recorded here
rather than papered over. **T051 and T052 therefore remain open, and the
SC-001…SC-010 criteria remain un-attested.** Both tasks stay `[ ]` in
`tasks.md` — they are device-only by design and are not host-verifiable.

---

## 7. Android device artifact (gate 9)

**Command:**
```
RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock CARGO_QUEUE_TIMEOUT=180 \
  TMPDIR=/home/luis/development/.mdreader-t095-reverify \
  JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=/home/luis/development/.mdreader-t095-reverify/gradle-tmp \
  cargo tauri android build --target aarch64
```
**Exit status:** **0**
**Log path:** `/tmp/opencode/t095_reverify_android.txt` (161 lines) — durable mirror
`/home/luis/development/.mdreader-t095-reverify/android.log` (both md5
`48f6b12ea1568773ff24aebffbe46edb`).

The build compiled the Rust side for the requested target
(`target/aarch64-linux-android/release/libsrc_tauri.so`, installed as the
`arm64-v8a` jniLibs entry) and produced both device artifacts:

```
    Finished 1 APK at:
        /home/luis/development/mdreader/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk

    Finished 1 AAB at:
        /home/luis/development/mdreader/src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab
```

| Artifact | Path | Size | mtime | sha256 |
|----------|------|------|-------|--------|
| APK (universal, release) | `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk` | 118 395 410 B | 2026-10-21 02:53:22 -0400 | `33a1b0aeeacf8c9b7ea1f6b1abf093038deee94442f856c2c422e2a3b558195b` |
| AAB (universal, release) | `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab` | 86 272 347 B | 2026-10-21 02:54:19 -0400 | `965a87b3a83d38d171eff711b5f50f44f3aa29765819718998f473a4d8c97950` |

Both hashes are of the artifacts **as produced by this run**, taken from the files
on disk after the repackaging described in §1.0. They are byte-identical to the
hashes the prior attestation recorded for tree `51b20c9`, which is an
independent reproducibility witness: the same source tree packages to the same
bytes.

The APK's native payload is the arm64 set (the build target was `aarch64`):

```
lib/arm64-v8a/libonnxruntime.so
lib/arm64-v8a/libsherpa-onnx-c-api.so
lib/arm64-v8a/libsherpa-onnx-cxx-api.so
lib/arm64-v8a/libsherpa-onnx-jni.so
lib/arm64-v8a/libsrc_tauri.so
```

### 7.1 Non-hollow artifact check (the dex carries the app's classes)

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

## 8. The Phase-16 work this run attests

Every Phase-16 task below is attested by the **same run** recorded above: its
commit is in the history of tree `19a1467`, its behaviour is covered by the 847
green tests of gate 2, and its artefacts are inside the gate-9 APK.

| Task | What it landed | Commit(s) |
|------|----------------|-----------|
| T112 / T113 | RED-FIRST rewrite of `crates/player/tests/prop_first_audio_3s.rs` against a **real** production seam (the old test was a tautology over a self-written event list) → the ≤ 3 s first-audible path: enqueue and play the **first** synthesized segment, then synthesize/enqueue the remainder | `244e652`, `24aab82` |
| T114 / T115 | RED-FIRST property for the explicit-resume cold start surviving the offer-window expiries → latch the `explicit_resume` requirement across offer-window expiries; keep the pending open parked until the reader resumes | `07e8e34`, `cf32cf0` |
| T116 / T117 | RED-FIRST contract test for the in-app resume offer surface (`src-tauri/tests/test_offer_dialog.rs`) → render the 3-second continue-vs-start-over dialog on the single in-app screen and wire both choices (`resume_at_stored` / `start_over`); keep the dialog on the reopen path only | `db30345`, `05e2de1`, `5bacea4` |
| T118 / T119 | RED-FIRST contract test for the lock-screen explicit resume control (`src-tauri/tests/test_lockscreen_resume_control.rs`) → add the lock-screen resume action/callback (render-only, zero computation) and admit the plain-play verb to the parked-open resolution | `7c66939`, `05f4306`, `503e6ec` |
| T120 | Render the pt-BR progress label on the lock-screen surface (Kotlin `MediaMirror` parses and renders `progressLabel`) | `17e4e0c` |
| T121 | Record per-section progress entries in the diagnostic record at the consumed-unit crossing | `9d1e845` |
| T122 | Amend `contracts/diagnostic-trace.md` to state the N:1 product-reason → error-taxonomy mapping, mirrored in the feature directory | `b21ccf7` |
| T123 | Correct the stale `frontend_mirror` doc comment | `2485126` |
| T124 / T125 | Close the superseded task text (T071/T072/T080/T085/T086) with notes pointing at T089/T090/T099 — no ID renumbered or deleted → resolve the untracked artifacts claimed by T107 (`.github/dependabot.yml`, the stray nested duplicate, orphaned cargo-vet state) | `00c0b50`, `f44e01f` |
| T126 | Hoist the second inline offset arithmetic to T111's `assert_offset` helper | `c6bf8be` |
| T127 | Replace the English title fallback on the lock screen | `78b7bb1` |
| T128 | Add the INTERNET-absence assertion to `scripts/check-manifest.sh` (the gate now matches the T106 claim) | `30b0415` |
| T129 | Translate the lock-screen action labels to pt-BR | `ded005f` |
| T130 | Translate the resident-pause notification strings to pt-BR | `63b0d95` |
| T131–T134 | Adjudicate the resident stop affordance (the resident-paused surface is a **distinct** surface outside FR-002's lock-screen prohibition) and pair the `contracts/surface.md` amendment with `spec.md` FR-002, `data-model.md`, `plan.md` and `spec.md` Key Entities | `fbfe42f`, `4c1f9f7`, `52c8758`, `c1e8750` |
| T135 | Supply-chain waiver + its mechanical bound: the lockfile bump, the constitution Exceptions Log entry 13, and the gate-5 `webpki-roots` version-set assertion | `9701d98`, `71d9f36`, `51b20c9` |
| T136 | Restore `cargo clippy --workspace --all-targets -- -D warnings` to exit 0 under rustc 1.99.1 / clippy 0.1.99 — the unfulfilled `#[expect(clippy::float_cmp)]` suppressions cleared in the player and session crates, and the `clippy::assert_is_empty` sites rewritten as `assert_eq!`/`assert_ne!` **in code**, with no manifest or lint-config change | `8f12acf`, `3b642d8`, `0756fc3` |
| T137 | Record the toolchain-stability policy: constitution v1.10.0's Technology Stack "Toolchain" clause + the repo-root `rust-toolchain.toml` (channel `stable`, components `rustfmt` + `clippy`; no version pinned; MSRV relationship stated) | `43df07c` |
| T138 | Wire the constitution's lint-compliance pass into `scripts/verify.sh` gate 4 as a **fail-closed** step (`lint-drift-clippy`), keeping the pinned lint-level checks and weakening no level | `d23942b` |
| T139 | Correct the stale constitution paragraph (constitution v1.11.0, Sync Impact Report: MINOR, record corrected, no principle/MUST changed) so the constitution states that gate 4 *does* run clippy | `19a1467` |

### 8.1 The RED → GREEN evidence trail

The four red-first test commits and their green counterparts, in order. The RED
state of each is recorded in the red commit itself, and the GREEN state is
witnessed by the same gate-2 run that produced *847 tests run: 847 passed*.

| Pair | RED commit (test, recorded failing) | GREEN commit (implementation) | RED evidence in the commit |
|------|-------------------------------------|-------------------------------|----------------------------|
| T112 → T113 | `244e652` `test(004): T112 first-audio property red-first` | `24aab82` `feat(004): T113 stream the first chunk so the reading is audible at once` | The commit body states the observed RED order — `synthesize → synthesize → synthesize → enqueue(pcm) → play`, the first enqueue landing **after the LAST synthesis** — and names the stale comment at `src-tauri/src/lib.rs:3759` that T113 had to delete. |
| T114 → T115 | `07e8e34` `test(004): T114 explicit-resume expiry red-first` | `cf32cf0` `fix(004): T115 latch the explicit-resume requirement across offer-window expiries` | The commit body quotes the RED output verbatim: `left: NoOffer, right: ExplicitResume` on expiry 2, minimal failing input `(StoredPosition { offset_secs: 0.0, first_yield_ms: 0, end_kind: Interrupted }, 2)`, `Summary: 4 tests run: 3 passed, 1 failed`. It also names the defective site (`crates/session/src/offer.rs:340-363`, `src-tauri/src/lib.rs:1791-1816`) and the co-owned test that pinned the defect (`prop_bookmark_lifecycle.rs:516-521`). |
| T116 → T117 | `db30345` `test(004): T116 in-app offer dialog red-first` | `05e2de1` `feat(004): T117 render the in-app resume offer dialog on the single screen` | The commit **adds** `src-tauri/tests/test_offer_dialog.rs` and pins the target in `scripts/expected-tests.txt` — a test that did not exist before, asserting the two-choice contract that the surface did not implement. *Its commit body is empty: unlike T112/T114/T118, T116's RED transcript is not written into the commit message; the RED evidence for this pair is the added failing test plus the implementation commit that follows it.* |
| T118 → T119 | `7c66939` `test(004): T118 lock-screen resume control red-first` | `05f4306` `feat(004): T119 add the lock-screen explicit resume control` | The commit body names the two RED assertions (the mirror's forwarding vocabulary carries the plain-play verb and nothing else; the bridge's parked-open resolution admits only `ResumeAtStored`/`StartOver`) and the two GREEN guards (no Stop control and no prohibited field on the published payload; an unexpected end leaves the reading waiting). It states: "No production change: T119 owns the implementation." |

All four RED test artifacts are present in the attested tree and are covered by
the green run:

| Artifact | Present in tree `19a1467` |
|----------|----------------------------|
| `crates/player/tests/prop_first_audio_3s.rs` | yes |
| `crates/session/tests/prop_explicit_resume_survives_expiry.rs` (+ its checked-in proptest failure seed) | yes |
| `src-tauri/tests/test_offer_dialog.rs` | yes |
| `src-tauri/tests/test_lockscreen_resume_control.rs` | yes |

---

## 9. Summary of what this attestation does and does not establish

**Established by this run (host-checkable, evidence-named):**

* `scripts/verify.sh` exits **0**; gates 1–7 and gate-8-shell **PASS**;
  `check-manifest.sh` and `check-shell-purity.sh` **PASS**.
* `gate-8-full` is **DEFERRED** (owner=synthesis, reason=SC-044 open) — recorded
  as a deferral, **never** as a pass.
* `cargo fmt --all -- --check` is clean (exit **0**; gate 1).
* `cargo clippy --workspace --all-targets -- -D warnings` exits **0** — both
  standalone (§2.1) and as gate 4's fail-closed step `lint-drift-clippy`
  (log line 1417). The clippy-1.99 drift that the prior attestation recorded as
  exit 101 is repaired **in code** (T136) and is now **gate-enforced** (T138).
* `cargo nextest list --workspace` is non-empty — **88** live targets vs **88**
  pinned in `scripts/expected-tests.txt`, two-way diff empty in **BOTH**
  directions (`cmp` byte-identical).
* The workspace suite is green: **847 tests run, 847 passed (2 slow), 2
  skipped**; gate-8-shell's own suites: 33/33 (`engine-core`) and 29/29
  (`player::prop_transport_math`).
* The supply chain is clean: `cargo deny check advisories licenses bans sources`
  exit **0** (`advisories ok, bans ok, licenses ok, sources ok`); `cargo audit`
  exit **0** with the **2 allowed warnings** (`proc-macro-error` /
  RUSTSEC-2024-0370, `glib` / RUSTSEC-2024-0429) unchanged.
* The RUSTSEC-2026-0381 waiver (constitution Exceptions Log entry 13,
  PROVISIONAL, constitution v1.11.0) is in force and **its mechanical bound
  holds**: the committed `Cargo.lock` carries **exactly** `webpki-roots` 0.26.11
  and 1.0.10, with `rustls-webpki` at 0.103.16 after the `9701d98` lockfile bump.
* The Android device artifact builds (`--target aarch64`, exit **0**):
  `app-universal-release.apk`, sha256
  `33a1b0aeeacf8c9b7ea1f6b1abf093038deee94442f856c2c422e2a3b558195b`;
  `app-universal-release.aab`, sha256
  `965a87b3a83d38d171eff711b5f50f44f3aa29765819718998f473a4d8c97950`;
  **not hollow** — 7/7 required app classes present in the dex.
* All Phase-16 tasks T112–T139 are landed and are attested by this run (§8).
* The toolchain-stability record is complete and enforced: constitution v1.10.0's
  Toolchain clause (`43df07c`), the lint-compliance pass performed and clean
  (T136), the pass wired into gate 4 as a fail-closed step (T138, `d23942b`), and
  the constitution record corrected to match (T139, `19a1467`, v1.11.0).

**NOT established by this run:**

* The device-only criteria — **SC-001…SC-010** and the **T051 / T052** trials
  (§6). Those require the reference phone (Redmi 2201117TY), remain open, and
  are explicitly not claimed by this host run. No trial sheet exists.
* `gate-8-full` (§5) — DEFERRED to the synthesis feature with SC-044 open.

**Scope note:** this attestation covers the host-checkable gates and the
artifact build only. It does not modify `spec.md`, `plan.md`, `contracts/`, the
constitution, `scripts/**`, or any code file; it does not weaken any gate, lint,
or test invariant; and it closes no device trial.
