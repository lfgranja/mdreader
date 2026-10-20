# T095 — Final verification attestation (feature 004-ressoa-dogfood-build)

> **What this file is.** The attestation T095 asks for: the recorded outputs of the
> host-checkable verification run — `scripts/verify.sh`, the lint gate, the
> nextest-target inventory, and the Android device artifact (gate 9) — each with
> its exit status and, where it produces one, a hash. It is evidence, not a
> claim: every number below is copied from a run whose log path is named.
>
> **What this file is NOT.** It is not a device acceptance record. The
> device-only success criteria (SC-001…SC-010 trials) and the device trials
> T051/T052 are **NOT attested by this host run** — see
> [Device-only criteria — not attested](#device-only-criteria--not-attested).
>
> Constitution Principle V ("do not declare success by implementer assertion")
> is the reason this file exists: the gates were run, and their own output is
> quoted.

**Date:** 2026-10-20
**Host:** Linux 7.2.9-201.fc44.x86_64 (x86_64) — rustc 1.98.1 / cargo 1.98.1
**Attested tree:** `3e8e25c` (`fix(004): T111 hoist cold-start position arithmetic
out of the shell vocabulary`) — the T111 fix landed before this run; the
shell-purity gate it repairs is part of `verify.sh` (gate: `check-shell-purity`),
so this run is the first green `verify.sh` on the post-T110 tree.
**Attestor:** the implementing session (T111/T095), running the gates as
`scripts/verify.sh` defines them.

---

## 1. `scripts/verify.sh` — exit status and per-gate table

**Command:** `RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock bash scripts/verify.sh 2>&1 | tee /tmp/opencode/t095_attest_verify2.txt`
**Exit status:** **0**
**Log path:** `/tmp/opencode/t095_attest_verify2.txt` (6 964 lines)

The exit status is the exit status of `scripts/verify.sh` itself (captured
through `PIPESTATUS[0]` under `bash -c`, so it is the script's status and not
the status of `tee`).

| # | Gate | Result | Evidence (from the log) |
|---|------|--------|-------------------------|
| 1 | `gate-1 format` | **PASS** | `cargo fmt --all -- --check` clean (log line 2) |
| 2 | `gate-2 tests` | **PASS** | `cargo nextest run --workspace` — *830 tests run: 830 passed (10 slow), 2 skipped*; live-vs-pinned target two-way diff clean (log lines 3–863) |
| 3 | `gate-3 documentation` | **PASS** | `cargo doc --workspace --no-deps` builds (log lines 864–1405) |
| 4 | `gate-4 lint-drift` | **PASS** | every workspace member declares `[lints] workspace = true`; the root lint tables equal the pinned T001 rigour set (log lines 1406–1407) |
| 5 | `gate-5 supply-chain` | **PASS** | `cargo deny check advisories licenses bans sources` + `cargo audit` (log lines 1408–6009) |
| 6 | `gate-6 deny-list` | **PASS** | no constitution-denied crate in any manifest or in `Cargo.lock` (log lines 6010–6011) |
| 7 | `gate-7 closed-set` | **PASS** | no catch-all `_ =>` match arm in `crates/*/src` (log lines 6012–6013) |
| 8a | `gate-8-shell pipeline` | **PASS** | `engine-core` handoff/stand-in/pitch/shape suites — *33 tests run: 33 passed*; `player::prop_transport_math` — *29 tests run: 29 passed* (log lines 6014–6091) |
| 8b | `gate-8-full` | **DEFERRED** | `gate-8-full: DEFERRED (owner=synthesis)` (log line 6092) — see [§1.1](#11-gate-8-full--deferred) |
| — | `check-manifest.sh` | **PASS** | `check-manifest: PASS (…/AndroidManifest.xml)` (log line 6095) |
| — | `check-shell-purity.sh` | **PASS** | `check-shell-purity: PASS (no forbidden call shapes; vocabulary confined to the T081 allowlist in type/getter/delegation positions; SC-057 closed sets and engine registry conform; spec→test traceability holds)` (log line 6963) |

Script's own closing line (log line 6964):

```
verify.sh: gates 1-8 complete — gate-8-shell PASS, gate-8-full DEFERRED (owner=synthesis)
```

### 1.1 gate-8-full — DEFERRED (never a pass)

The constitution's full gate 8 (byte budget < 5000 bytes/request, SSML
capability, sentence-level alignment) is **DEFERRED, owner=synthesis** — it is
**not** a pass and is not claimed as one here. Constitution Exceptions Log
entry 8 (`.specify/memory/constitution.md:887`) splits gate 8 for
`001-android-media-shell`: `verify.sh` reports `gate-8-shell: PASS` for the
feature-scoped handoff-conformance suites and `gate-8-full: DEFERRED
(owner=synthesis)` until the synthesis feature discharges it with SC-044.

**SC-044 remains open.** `specs/002-fr-024-synthesis-adapter/tasks.md` records
SC-044 as still `DEFERRED` (T039/T057/T076 attestation clauses: "`SC-044` still
`DEFERRED` per `constitution.md:643`"; "SC-044 stays deferred"). The 004
task text (T048) therefore requires `gate-8-full: DEFERRED
(owner=synthesis, reason=SC-044-open)` — which is what the run printed. The
script never prints a bare `gate-8: PASS` while those inputs are absent, and
this attestation never claims one.

---

## 2. Lint gate — clippy exit status

**Command:** `RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock cargo clippy --workspace --all-targets -- -D warnings`
**Exit status:** **0** (clean; no warnings, no errors)
**Log path:** `/tmp/opencode/t095_attest_clippy2.txt`

No lint level was weakened anywhere in this feature's history: gate 4's
lint-drift check compares the root `[workspace.lints]` tables against the pinned
T001 rigour set and passed (see §1), and the workspace-wide clippy run above is
green with `-D warnings`.

---

## 3. Test-target inventory and the two-way diff

**Command:** `cargo nextest list --workspace` (exit 0), reduced with the same
pipeline `verify.sh` gate 2 uses (`grep -E "::" | sed -E 's/ .*//' | sort -u`).

| Quantity | Value |
|----------|-------|
| Live nextest targets (distinct `package::target` / `package` entries) | **85** |
| Pinned targets in `scripts/expected-tests.txt` (distinct) | **85** |
| Missing (pinned but not live) | **0** — empty |
| Unexpected (live but not pinned) | **0** — empty |
| Two-way diff result | **IDENTICAL** — the live list and the pinned list are byte-for-byte equal after `sort -u` |

The inventory is non-empty (85 entries), satisfying T095's "non-empty" clause.
Both directions of the diff are empty, so neither a pinned-but-missing target
nor an unpinned-but-live target exists — the fail-closed two-way diff that
gate 2 runs has nothing to report.

---

## 4. Android device artifact (gate 9)

**Command:** `RUSTC_WRAPPER= CARGO_QUEUE_LOCK=/tmp/cargo-build.lock cargo tauri android build --target aarch64 2>&1 | tee /tmp/opencode/t095_attest_android2.txt`
**Exit status:** **0**
**Log path:** `/tmp/opencode/t095_attest_android2.txt` (159 lines)

| Artifact | Path | sha256 |
|----------|------|--------|
| APK (universal, release) | `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk` (118 384 850 bytes) | `d91cf7876a839fe70a20494905f6126ef2e8059b7619c1f9ff6ca9cc4d99a9a3` |
| AAB (universal, release) | `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab` | `623671f808bae081505f70236ab4550aa3697d931fa02efae30711fcb0ee3193` |

The build compiled the Rust side for the requested target
(`target/aarch64-linux-android/release/libsrc_tauri.so`, installed as the
`arm64-v8a` jniLibs entry) and produced the device artifact:

```
Finished 1 APK at:
    /home/luis/development/mdreader/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

The hash is of the artifact as built by this run, taken from the file on disk
after the build completed.

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
* **T052** — the device trial for the diagnostic-record acceptance pass.

No trial sheet is attached, because none was produced by this run. T095's task
text asks that the T051/T052 trial sheets be attached "if available" — they are
**not available**, and their absence is recorded here rather than papered over.
T051/T052 therefore remain open, and the SC-001…SC-010 criteria remain
un-attested.

---

## 6. Summary of what this attestation does and does not establish

**Established by this run (host-checkable, evidence-named):**

* `scripts/verify.sh` exits **0**; gates 1–7 and gate-8-shell **PASS**;
  `check-manifest.sh` and `check-shell-purity.sh` **PASS**.
* `gate-8-full` is **DEFERRED** (owner=synthesis, SC-044 open) — recorded as a
  deferral, never as a pass.
* `cargo clippy --workspace --all-targets -- -D warnings` exits **0**.
* `cargo fmt --all -- --check` is clean (gate 1).
* The nextest target inventory is **85** live / **85** pinned, two-way diff
  identical (empty in both directions).
* The workspace suite is green: **830 tests run, 830 passed, 2 skipped**.
* The Android device artifact builds (`--target aarch64`, exit **0**):
  `app-universal-release.apk`, sha256
  `d91cf7876a839fe70a20494905f6126ef2e8059b7619c1f9ff6ca9cc4d99a9a3`.

**Not established by this run:** the device-only criteria — SC-001…SC-010 and
the T051/T052 trials (§5). Those remain open and unattested.

**Scope note:** this attestation covers the host-checkable gates and the
artifact build only. It does not modify `spec.md`, `plan.md`, any gate, lint, or
test invariant, and it closes no device trial.
