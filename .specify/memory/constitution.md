<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-21
  Version change: 1.10.0 -> 1.11.0

  Bump rationale: MINOR. One governance record is CORRECTED: the
  Verification Gates paragraph that described the mechanical form of
  the lint-compliance pass as unwired now states the state that holds
  since T138. No principle is added, removed or redefined, no existing
  MUST is weakened, and no lint level, gate threshold or constraint is
  changed. Not PATCH (the record of what gate 4 mechanically enforces
  changes materially and a prior amendment's deferred item is
  superseded, which is not a wording, typo or non-semantic
  refinement), not MAJOR (no principle change, no MUST weakened).

  What changed:
    - Verification Gates: the paragraph that began "The mechanical
      form is not wired yet" is replaced. It now states that
      `scripts/verify.sh` gate 4 runs the lint-compliance pass itself
      -- `cargo clippy --workspace --all-targets -- -D warnings` -- as
      a fail-closed step (`running 4 lint-drift-clippy`, then a `fail`
      on a non-zero exit, then `pass 4 lint-drift-clippy`), so the lint
      SET is enforced by the gate itself and not only by the manual
      pre-attestation pass. The gate keeps checking the declared lint
      LEVELS as well: the clippy step is an ADDITION, not a
      replacement. The requirement that the moved lints are resolved in
      code -- never by weakening a lint level, never by a blanket
      `#[allow]`, never by an unreasoned `#[expect]` -- is unchanged
      and is retained.
    - Verification Gates, gate-4 row of the gate table: the row now
      names the clippy step the gate runs, so the table and the
      paragraph agree.
    - The v1.10.0 report's "Deferred / non-constitution" bullet, which
      carried the same statement in the present tense, is marked
      SUPERSEDED with a pointer to this report. The historical text
      beneath the marker is preserved: it remains a true record of
      what the v1.10.0 amendment itself did (it deliberately touched
      no gate script).
    - Provenance of the stale text: it was introduced by the v1.10.0
      toolchain-policy amendment (commit 43df07c, "docs(constitution):
      record the toolchain-stability policy"), which declined to touch
      a gate script for scope discipline and recorded the deferral
      explicitly. The deferral was discharged MECHANICALLY by T138,
      not by another policy change.

  Modified constraints:
    - Verification Gates: the lint-compliance mechanical-enforcement
      paragraph, and the gate-4 row of the gate table.
    - Nothing else: no principle, no MUST, no lint level, no gate
      threshold and no Exceptions Log entry is modified.

  Principles modified: none.
  Sections added: none.
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution:
    - No Exceptions Log entry: this is not a deviation from the
      reference stack.
    - The lint-compliance pass is still REQUIRED before a tree is
      re-attested after a toolchain bump. It is now ALSO a gate: the
      gate enforces the pass, it does not replace it, and the
      obligation is unchanged.
    - Reversal, stated so the owner can execute it: replace the
      Verification Gates paragraph beginning "The mechanical form is
      now wired" with the v1.10.0 text, verbatim -- "The mechanical
      form is not wired yet: `scripts/verify.sh` gate 4 today checks
      the declared lint levels and does not itself invoke clippy, so
      the pass is an attestation-time obligation, run by hand. Wiring
      clippy into gate 4 would make it mechanical; until it does, the
      clause above is what stands between a green attestation and a
      silently drifted lint set. This clause is reversible by deleting
      this paragraph -- no other gate, principle or constraint depends
      on it." Restore the gate-4 table row's "Command" cell to "root
      deny-tables intact; `[lints] workspace = true` in every member",
      delete the SUPERSEDED marker from the v1.10.0 report, and
      restore the footer to `**Version**: 1.10.0` (the "Last Amended"
      date and this report header stay as they are). Reversing this
      RECORD does not un-wire the gate: the mechanical enforcement
      lives in `scripts/verify.sh` (commit d23942b), and undoing that
      is `git revert d23942b`, not an edit to this file.

  Source evidence: commit d23942b ("fix(004): T138 gate the
  lint-compliance pass in verify.sh") wires the pass into gate 4 as
  step (d). The step is named `lint-drift-clippy` and is fail-closed:
  `running 4 lint-drift-clippy`; `if ! cargo clippy --workspace
  --all-targets -- -D warnings; then fail "gate-4 lint-drift: cargo
  clippy --workspace --all-targets -- -D warnings failed (the
  lint-compliance pass must exit 0; resolve the moved lints in code,
  never by weakening a lint level, a blanket #[allow] or an unreasoned
  #[expect])"; fi`; `pass 4 lint-drift-clippy`. The pre-existing checks
  of gate 4 -- (a) `[lints] workspace = true` in every member manifest,
  (b) the pinned lint-level set, (c) the intact
  `[workspace.dependencies]` table -- are unchanged, so the clippy step
  is an addition, not a replacement. Negative control (run on a COPY
  of the repository; the real tree was never modified): a `println!`
  injected into `crates/doc-structure/src/lib.rs` trips the denied
  `clippy::print_stdout`; the copy's `verify.sh` exits 1 with
  "verify.sh: FAIL: gate-4 lint-drift: cargo clippy --workspace
  --all-targets -- -D warnings failed"; the copy was reverted and
  deleted. Positive control: `bash scripts/verify.sh` exits 0 and gate
  4 reports the clippy step. The stale sentence being corrected is
  the one this report's "What changed" section quotes; its origin is
  the v1.10.0 report's "Deferred / non-constitution" bullet and the
  Verification Gates paragraph of the same amendment (commit 43df07c).
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-21
  Version change: 1.9.0 -> 1.10.0

  Bump rationale: MINOR. One governance clause is added (the
  toolchain-stability policy) and one new repo-root file is added
  (`rust-toolchain.toml`, naming the tracked channel). No principle
  is added, removed or redefined, no existing MUST is weakened, and
  the MSRV floor (1.85 for workspace code, rustc >= 1.88 for the
  full build) is unchanged. Not PATCH (new governance content), not
  MAJOR (no principle change).

  What changed:
    - Technology Stack gains a "Toolchain" entry: the workspace
      TRACKS the `stable` channel and does NOT pin a compiler
      version. The MSRV line stays the compatibility floor and is
      stated as such; the tracked channel can never contradict a
      floor, because `stable` only rises. A toolchain bump is
      EXPECTED to move the lint set, and a lint-compliance pass is
      REQUIRED before the tree is re-attested. The moved lints are
      resolved in code -- never by weakening a lint level, never by
      a blanket suppression.
    - Verification Gates gains a clause beside gate 4: a passing
      tree can become a failing one with NO source change, because
      the lint set moves with the toolchain. The lint-compliance
      pass is the required response, and it is the deterministic-gate
      concern Principle V exists for.
    - New repo-root `rust-toolchain.toml`: `channel = "stable"` plus
      `components = ["rustfmt", "clippy"]`. No version is pinned, so
      the file cannot contradict the MSRV relationship and cannot
      force a download of an exact toolchain on a host that lacks it.

  Modified constraints:
    - Technology Stack: new "Toolchain" entry.
    - Verification Gates: new clause beside gate 4.
    - rust-toolchain.toml: new file (repo root).

  Principles modified: none.
  Sections added: none.
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution:
    - SUPERSEDED by v1.11.0 (2026-10-21). The mechanical form of the
      lint-compliance pass IS now wired into `scripts/verify.sh` gate
      4 as a fail-closed step (T138, commit d23942b): gate 4 runs
      `cargo clippy --workspace --all-targets -- -D warnings` and
      FAILS the gate when it does not exit 0. The bullet below is
      preserved as the historical record of what THIS amendment did
      (it deliberately touched no gate script); its present-tense
      statement describes the state as of v1.10.0.
    - The mechanical form of the lint-compliance pass is NOT wired
      into `scripts/verify.sh`: gate 4 still checks the declared lint
      levels only and does not invoke clippy, so the pass is an
      attestation-time obligation, run by hand. Wiring clippy into
      gate 4 is a separate change and was deliberately NOT made here
      (scope discipline: this amendment is a policy record plus one
      new toolchain file, and no gate script was touched).
    - No Exceptions Log entry: this is not a deviation from the
      reference stack. `~/development/MDMV/projetos/crsdd-fabro`
      carries no `rust-toolchain.toml` and pins no compiler either;
      it records the toolchain it verified against (rustc 1.98.1) as
      documentation, which is the practice adopted here.
    - Reversal, stated so the owner can execute it: delete the
      "Toolchain" entry from the Technology Stack, delete the clause
      beside gate 4 in the Verification Gates, and delete the
      repo-root `rust-toolchain.toml`. No other governance rule
      depends on any of the three, and the MSRV floor is untouched
      either way.

  Source evidence: the `stable` toolchain was replaced on
  2026-10-20 22:19 (rustc 1.98.1 -> 1.99.1, clippy 0.1.98 ->
  0.1.99) and the repository pinned no rust-toolchain.toml, so the
  lint set moved under a byte-identical source tree: 86
  `unfulfilled_lint_expectations` (clippy 0.1.99 no longer lints the
  `==` that `assert_eq!`/`assert_ne!` generate inside their own
  macro expansion, so the `#[expect(clippy::float_cmp)]` attributes
  could no longer be fulfilled) plus 13 `clippy::assert_is_empty`.
  `cargo clippy --workspace --all-targets -- -D warnings` exited
  101 while `scripts/verify.sh` still exited 0, because clippy is
  supplementary to gates 1-8. T136 repaired it in code with no lint
  level weakened, no new suppression and no behaviour change
  (commits 8f12acf, 3b642d8, 0756fc3); T137 records the policy.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-20
  Version change: 1.8.0 -> 1.9.0

  Bump rationale: MINOR. One governance record is added
  (Exceptions Log entry 13, the PROVISIONAL waiver of
  RUSTSEC-2026-0381 for the `webpki-roots` 0.26 shim).
  No principle is added, removed or redefined, no
  existing MUST is weakened, and there is no
  architecture reversal. Not PATCH (new governance
  content), not MAJOR (no principle change).

  What changed:
    - Exceptions Log entry 13 APPENDED (PROVISIONAL):
      advisory RUSTSEC-2026-0381 / CVE-2026-111943
      (2026-10-19) is waived for `webpki-roots 0.26.11`.
      The flagged node is a pure semver shim — it ships
      NO trust anchors of its own (`pub use
      parent::TLS_SERVER_ROOTS;`) — while the anchors
      actually in use come from `webpki-roots` 1.0.10,
      which HAS removed the distrusted `emSign ECC Root
      CA - G3` root. Build-time only; nothing from the
      `ureq` / `rustls` chain links into any shipped
      artifact.
    - The waiver is version-bound: `scripts/verify.sh`
      gate 5 gains an assertion that the lockfile's
      `webpki-roots` version set is exactly
      `{0.26.11, 1.0.10}`. Any other version fails the
      gate, so the waiver cannot silently widen.
    - `deny.toml` gains a `[[advisories.ignore]]` block
      for RUSTSEC-2026-0381.
    - A new repo-root `.cargo/audit.toml` gains
      `[advisories] ignore = ["RUSTSEC-2026-0381"]`.
      Gate 5 runs BOTH `cargo deny check advisories` and
      `cargo audit`, and each tool reads its own ignore
      file.

  Modified constraints:
    - Exceptions Log: entry 13 appended.
    - deny.toml: new `[[advisories.ignore]]` block.
    - .cargo/audit.toml: new file (repo root).
    - scripts/verify.sh: gate 5 gains the `webpki-roots`
      version-set assertion.

  Principles modified: none.
  Sections added: none.
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution:
    - The `ureq` 3.x migration of the vendored
      `sherpa-onnx-sys` is NOT bundled into this waiver:
      every `ureq` 2.x requires `webpki-roots ^0.26`
      and only `ureq >= 3.1.0` requires `^1.0.0`, so
      moving that vendored crate's build-dependency is a
      SECOND deviation in it, requiring its own
      Exceptions Log entry and its own amendment.
    - Entry 13's exit condition: the `webpki-roots` 0.26
      shim leaving the graph — the vendored
      `sherpa-onnx-sys` build-dependency moving to
      `ureq` 3.x, or upstream publishing a 0.26.x
      release carrying the removal.
    - Re-evaluation trigger: whenever `webpki-roots` or
      `ureq` changes version in the lockfile.

  Source evidence: RUSTSEC-2026-0381 (CVE-2026-111943,
  dated 2026-10-19, category crypto-failure, patched set
  `>= 1.0.10`; advisory text: "Version 1.0.10+ removes
  this root"), from the RustSec advisory database.
  Lockfile path, as reported by `cargo deny check
  advisories` (2026-10-20): `adapter-synthesis` ->
  `sherpa-onnx` 1.13.8 -> vendored `sherpa-onnx-sys`
  1.13.8 (build) -> `ureq` 2.12.1 -> `webpki-roots`
  0.26.11. Shim fact: `webpki-roots` 0.26.11's ONLY
  dependency is `[dependencies.parent] version = "1"
  package = "webpki-roots"` and its `src/lib.rs` is
  `pub use parent::TLS_SERVER_ROOTS;` — it bundles no
  roots of its own. Removal fact: `webpki-roots` 1.0.9's
  `src/lib.rs` carries "emSign ECC Root CA - G3";
  1.0.10's does not. Tool results (2026-10-20, before
  the ignore records): `cargo deny check advisories`
  reports exactly this one vulnerability, and
  `cargo audit` reports the same one vulnerability
  (1 vulnerability, 2 allowed warnings) — each is
  silenced by its own tool's ignore record.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-16
  Version change: 1.7.0 -> 1.8.0

  Bump rationale: MINOR. One Licence Policy admission is
  added (bzip2-1.0.6, a permissive no-copyleft licence).
  No principle is added, removed or redefined and no
  existing MUST is weakened. Not PATCH (new governance
  content), not MAJOR (no principle change).

  What changed:
    - Licence Policy: bzip2-1.0.6 admitted to the closed
      allow list. Reason: the vendored `sherpa-onnx-sys`
      build-dependency `zip` 9 (entry 12) carries `bzip2` in
      its default feature set, resolving through `bzip2` 0.6
      to `libbz2-rs-sys` (`license = "bzip2-1.0.6"`). The
      licence is permissive (no copyleft; notice-and-
      attribution conditions only) and build-time-only
      (nothing from `zip` links into shipped artifacts).
      Admitted now because the upstream fix keeps the
      default feature set, so the crate returns to the
      graph after entry 12's exit.

  Modified constraints:
    - Licence Policy: bzip2-1.0.6 admitted.
    - deny.toml: licenses allow gains "bzip2-1.0.6".
    - vendor/sherpa-onnx-sys/PROVENANCE.md: the licence
      implication of the zip 9 default feature set recorded.

  Principles modified: none.
  Sections added: none.
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution:
    - none. Entry 12's exit condition is unchanged.

  Source evidence: `cargo deny check advisories licenses
  bans sources` (2026-10-16) — the pre-admission run
  rejected exactly one licence (libbz2-rs-sys 0.2.5,
  `license = "bzip2-1.0.6"`, reached as a build-dependency
  of zip 9.0.4 through the vendored sherpa-onnx-sys,
  entry 12); the post-admission run is green on all four
  checks.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-16
  Version change: 1.6.1 -> 1.7.0

  Bump rationale: MINOR. One governance record retires
  (Exceptions Log entry 11, its PROVISIONAL zip waiver now
  discharged by a mechanical fix) and one is added (entry 12,
  the vendored sherpa-onnx-sys override); the Technology Stack
  MSRV line is refined. No principle is added, removed or
  redefined, and no existing MUST is weakened. Not PATCH (new
  governance content plus the retirement of an exception
  record), not MAJOR (no principle change, no architecture
  reversal).

  What changed:
    - Exceptions Log entry 11 DELETED: RUSTSEC-2026-0367 /
      0368 / 0369 leave the dependency graph. The remediation
      in place instead of the waiver: `[patch.crates-io]`
      redirects `sherpa-onnx-sys` to a vendored copy of the
      published 1.13.8 crate carrying the single-line
      `zip = "9"` build-dependency fix; the lock resolves
      zip 9.0.4; the three deny.toml ignore blocks are
      deleted in the same commit. Upstream PR
      k2-fsa/sherpa-onnx#4059 stays open for the eventual
      crates.io release.
    - Exceptions Log entry 12 APPENDED (PROVISIONAL): the
      vendored override deviates from the crates.io-only
      source rule of the supply-chain gate and is recorded as
      accepted risk (build-time-only; provenance recorded).
    - Technology Stack: the Rust MSRV line now scopes 1.85 to
      workspace code and states the build-time requirement of
      rustc >= 1.88 while `zip` 9 sits in the build graph.

  Modified constraints:
    - Exceptions Log: entry 11 deleted; entry 12 appended.
    - deny.toml: the three `[[advisories.ignore]]` blocks for
      the zip advisories deleted (advisories no longer present
      in the graph).
    - Cargo.toml (root): new `[patch.crates-io]` stanza — the
      mechanical form of entry 12.
    - Technology Stack MSRV line refined (build-time 1.88).
    - Vendored tree added: `vendor/sherpa-onnx-sys/` with
      `PROVENANCE.md` (sha256, one-line diff, exit condition).

  Principles modified: none.
  Sections added: none.
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution:
    - Upstream PR k2-fsa/sherpa-onnx#4059 remains open; when a
      release carrying the fix lands, entry 12's own exit
      condition applies (bump `crates/adapter-synthesis`,
      delete the vendored tree + patch stanza + entry 12).

  Source evidence: Cargo.lock zip 9.0.4 resolved through the
  `sherpa-onnx-sys` path patch (source-crate sha256
  14bbeabb73f73f1c1f4a278af0ee9ae90a19f9727482027047fb9422fa14f8fe,
  recorded in vendor/sherpa-onnx-sys/PROVENANCE.md);
  `cargo deny check advisories` -> "advisories ok" with the
  three ignore blocks deleted (2026-10-16); commits 2177c67
  (vendor), 02164b6 (patch + lock), and the governance commit
  recording this report.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-12
  Version change: 1.4.0 -> 1.5.0

  Bump rationale: MINOR. Two governance records are added,
  neither of which adds, removes, or redefines a principle
  or weakens a MUST: (1) the Licence Policy admits
  CDLA-Permissive-2.0 — a permissive data license under
  which webpki-roots (transitive via the rustls/webpki
  chain the HTTPS client requires) ships Mozilla's CA root
  certificates as data; the admission extends an existing
  policy list with the same class of permissive data
  license already admitted (CC0-1.0, CC-BY-4.0); (2)
  Exceptions Log entry 10 waives advisory RUSTSEC-2024-0370
  (proc-macro-error 1.0.4, unmaintained, no known
  vulnerability, no CVE) — a compile-time-only proc-macro
  dependency of tauri's Linux gtk chain, none of whose
  code is linked into any shipped artifact. The Apache-2.0
  WITH LLVM-exception entry in deny.toml is a pure sync
  of the existing policy wording (the policy already
  admitted "Apache-2.0 (with LLVM exception)"; deny.toml
  lacked the exact SPDX expression). Reversible by deleting
  the licence sentence, the log entry, and the ignore
  block. Not MAJOR (no principle change, no MUST weakened),
  not PATCH (new governance content).

  Modified constraints:
    - Licence Policy: CDLA-Permissive-2.0 admitted to the
      closed allow list, with the webpki-roots data-license
      justification.
    - Exceptions Log: entry 10 appended (advisory waiver).
    - deny.toml: licenses allow gains "Apache-2.0 WITH
      LLVM-exception" (sync) and "CDLA-Permissive-2.0"
      (new admission); advisories gain the RUSTSEC-2024-0370
      ignore block mirroring entry 10.

  Principles modified: none.
  Sections added: none (one policy sentence, one log entry).
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution:
    - The stale footer version (1.3.0) is corrected to
      1.5.0 in the same commit; the 1.4.0 amendment had
      left it unsynced.
    - Re-evaluation trigger for entry 10: when tauri
      drops the proc-macro-error dependency.

  Source evidence: cargo deny check advisories licenses
  bans sources output on this workspace (2026-10-12):
  target-lexicon-0.12.16 "Apache-2.0 WITH LLVM-exception"
  rejected; webpki-roots 0.26.11 / 1.0.9
  "CDLA-Permissive-2.0" rejected; proc-macro-error
  1.0.4 RUSTSEC-2024-0370 unmaintained advisory.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-10
  Version change: 1.3.0 -> 1.4.0

  Bump rationale: MINOR. One new named constraint clause — the
  Workspace Dependency Portability and Supply-Chain Policy — is
  added to record the portability-first dependency-selection
  decision (prefer universal dependencies; use restricted
  dependencies only behind cfg-gating when they meet a
  measured, recorded exception bar). No principle is added,
  removed, or redefined; no existing MUST is weakened or
  replaced; the deviation from the previous unconditional-
  declaration default is explicitly reversible by deleting
  the clause. Not MAJOR (no principle change, no architecture
  reversal), not PATCH (new governance content, not a
  wording clarification of an existing clause).

  Modified constraints:
    - Workspace Dependency Portability and Supply-Chain
      Policy (new named section, inserted after the Tauri
      Platform Contract section): the three refinements of
      the portability-first policy, with the measured
      exception bar (>=10x, paired benchmark), the first-use
      declaration rule, the CI gate (cargo machete / cargo
      shear / cargo-check --workspace --all-targets), and the
      supply-chain gate (advisory-free, crates.io-only,
      build-script-free preference, with the undiagnosed
      build-time execution risk of build.rs documented).

  Principles modified: none.
  Sections added: one new named section — Workspace
  Dependency Portability and Supply-Chain Policy.
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution (from this amendment's
  conversation, not a governance change):
    - The precise tunable magnitude threshold (>=10x
      anchored at AAudio EXCLUSIVE 20ms vs ~205ms in
      DeviceAudioLatency docs) is set as a default; the
      maintainer may adjust it by amendment if a
      different domain's evidence demands.
    - The exact CI gate (blocking cargo-machete vs
      periodic cargo-udeps / cargo-shear) is left to
      the workspace's gate-infrastructure owner.
    - The specific device-farm / on-device trial plan
      for future cfg-gated platform behavior (audio
      focus / routing / lock-screen) is deferred to the
      001 on-device trial plan (SC-009 stage 2) and
      not required for this policy clause.

  Source evidence for the clause: specs/002-fr-024-synthesis-adapter/
  DEPENDENCY_PORTABILITY_RESEARCH.md (research note, 4 threads,
  cited, 2026-10-10); B2 repair commits on 001-android-media-shell
  (a90d2b6 manifest gate; 42b59d7 closed-set unification; 286009e
  snap_to_boundary cap-origin behavioral fix; session/lib.py
  lint repair at 663888b).
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-06
  Version change: 1.2.0 -> 1.3.0

  Bump rationale: **MINOR.** One new constraint clause is added to the Tauri Platform Contract
  (Audio Focus / Driving-Safety section) to record a deliberate deviation from Android's
  documented ducking best practice (ADR-001). No principle is added, removed, or redefined;
  no existing MUST is weakened or replaced; the deviation is explicitly reversible by deleting
  the clause. Not MAJOR (no principle change, no architecture reversal), not PATCH
  (new governance content, not a wording clarification).

  Modified constraints:
    - **Tauri Platform Contract — Audio Focus / Driving-Safety** (line ~466): a new
      bullet "Deliberate deviation from Android ducking best practice (ADR-001)" records that
      transient focus loss ducks to ~20% and continues rather than pausing, with the
      platform recommendation (`setWillPauseWhenDucked(true)`) cited by source, the
      product rationale (continuity over interruption in the commute) stated, and the
      reversibility affirmed (delete clause -> adopt default; no other governance change).

  Principles modified: none.
  Sections added: none (existing section extended; no new named section).
  Sections removed: none.
  Renamed sections: none.

  Deferred / non-constitution (from conversation, not this file):
    - Naming / brand decision (product title, domain, INPI, GitHub reserve): deferred to
      `/speckit.specify` or manual follow-up; not a governance change.
    - Monetization architecture (pricing tiers, Play Billing integration, COGS model):
      deferred; requires feature-level decision, not constitution amendment.
    - ADR-001 audit trail in Exceptions Log: should be added at next amendment cycle,
      not required for this clause insertion.

  Scope discipline: this amendment touches this file only. No crate, contract, gate script,
  or spec artifact was modified.
-->
<!--
  SYNC IMPACT REPORT (v1.2.0 — preserved)
  ===================
  Amended: 2026-10-01
  Version change: 1.1.0 -> 1.2.0

  Bump rationale: **MINOR.** One constraint section is materially rewritten (Tauri Platform
  Contract) to record a decision that changes who owns playback, and one additional constraint
  is added to it. No principle is added, removed, or redefined, and no MUST is weakened, so
  this is not MAJOR; it is not PATCH either, because the amendment reverses a v1.0.0
  architecture decision rather than clarifying its wording.

  Modified constraints:
    - **Tauri Platform Contract** — rewritten. Three changes, in order of weight:
      (a) **The application is now declared a modular monolith**: one Cargo workspace of focused
          crates with the Tauri app as one consumer, making Principle I structural rather than
          aspirational, and the Rust player is one crate in that workspace.
      (b) **Playback authority moves from Kotlin/Media3 to Rust.** At v1.0.0 the contract said
          "Media3 (Kotlin) owns playback, media session, notification, and audio focus. Rust owns
          synthesis." That is now reversed: Rust owns synthesis *and* PCM playback, via `cpal`
          (AAudio backend) or `oboe`. Media3 is no longer in the audio path.
      (c) **Kotlin is redefined as a thin platform shell** — the `Service`, the `MediaSession`,
          the notification, and the audio-focus request, and nothing else.

  Corrected framing, because v1.1.0 asserted more than the evidence supported. The v1.0.0
  contract implied that a Kotlin plugin was the only way to get a media layer. It is not. Two
  corrections were established by checking primary sources:

    - **Rust can drive Android audio directly.** `cpal` 0.18.2 ships an `aaudio` backend
      (`src/host/aaudio`, with `java_interface.rs` and the `ndk` crate) and lists
      `aarch64-linux-android` as a supported platform on docs.rs. `oboe` 0.6.1 (10.8M downloads)
      is the lower-level alternative. PCM playback in Rust is a supported path, not a workaround.
    - **Only the `Service` class is structurally forced to be Kotlin.** The Android
      `ActivityManager` instantiates a `Service` by reflecting on the class named in the
      manifest, so a pure-Rust service cannot exist. That single fact is the entire language
      boundary; everything else in the Kotlin surface is a design decision, and this amendment
      now labels it as one.

  The decision to move playback authority to Rust carries a named risk, recorded because it is
  the reason this rule is written as it is: Media3 expects to be the playback owner, and driving
  the audio track from Rust while a Kotlin `MediaSession` mirrors state creates two authorities
  over position and buffering unless the rule below is enforced.

  Added constraint, in the same section:
    - **Single playback authority.** Rust owns playback state and is the sole authority for
      position, buffering, and transport. The Kotlin `MediaSession` is a *mirror*, never a second
      source of truth. Two authorities over playback position produces a desynchronisation between
      the notification and the audio that is invisible in unit tests and audible only in a car —
      which is precisely the failure Principle VIII exists to prevent.

  Principles modified: none. Principle I is reinforced by this amendment, not changed by it.
  Sections added: none. Sections removed: none.

  Scope discipline: this amendment touches this file only.

  Deferred items: none.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Amended: 2026-10-01
  Version change: 1.0.0 -> 1.1.0
  Ratified: 2026-10-01 (unchanged — this amends an adopted constitution, it is not a new adoption)

  Bump rationale: **MINOR.** One principle gains a clause (XI), one new constraint section is
  added (TTS Engine Registry), and one existing constraint is materially extended (Audio Pipeline
  Contracts, alignment granularity). No principle is removed or redefined, and no MUST is
  weakened, so this is not MAJOR; it is not PATCH either, because it widens the set of engines
  the system may reach at runtime, which is a governance expansion rather than a wording fix.

  Modified principles:
    - **XI (Credential and Content Sovereignty)** — one clause added. The rule gains a named,
      revocable exception for one specific endpoint. Rationale below.
    - **VI (Engine-Neutral Core, Adapters at the Edges)** — *not modified*. The Edge adapter fits
      the existing trait contract as written; no carve-out was needed and none was taken. Recorded
      here because the fact that it fit without amendment is itself the finding.

  Modified constraints:
    - **Audio Pipeline Contracts** — the sentence-alignment paragraph is materially extended.
      It previously named the hard problem and settled on sentence-level as designed precision.
      It now records which engines can actually *deliver* that precision, which turns an
      aspiration into a testable requirement, and it corrects a prior uncertainty.

  Added sections:
    - **TTS Engine Registry** (Additional Constraints). A closed table of admissible engines with
      the evidence for each. This exists because the engine set was being chosen by conversation
      rather than by document, and an engine arriving or leaving was not currently a governance
      event.

  Removed sections: none.  Renamed principles: none.

  The XI exception, stated plainly because it is the substantive change. Principle XI forbids
  sending document content to a third-party synthesis provider without explicit per-provider
  revocable consent. Microsoft's Edge "read aloud" endpoint is **not a contracted provider**: it is
  undocumented, has no SLA, and is reached with a synthesised authentication token rather than a
  credential the user holds. An Edge adapter therefore does egress under a provider the user never
  chose. This is admitted as an exception rather than left to be discovered, for three reasons:

    1. It is a **fallback, not a default**. The constitution does not rank Edge as the primary
       cloud engine, so the user is never silently moved onto it.
    2. It is **consent-gated by the same mechanism** as every other cloud provider: explicit,
       per-provider, revocable. The exception waives the *contract*, not the *consent*.
    3. Its **failure mode is visible**. Principle VI already requires that a degradation of
       capability be observable; an adapter that cannot be reached MUST surface as an engine
       failure, not as silence, because silence is the failure this whole project exists to fix.

  Trigger: audit of two third-party TTS projects against the adopted constitution. The first,
  `pgmichael/wavenet-for-chrome`, confirmed the free-tier economics and the 5,000-byte limit and
  changed nothing. The second, `travisvn/openai-edge-tts`, surfaced the sentence-boundary fact
  below and forced this amendment. Both audits are recorded rather than discarded, because an
  audit that changes nothing is evidence too.

  Corrections to research recorded at v1.0.0, from the same audit:
    - **Google Cloud TTS `timepoints` are UNVERIFIED.** v1.0.0 left the word-offset capability
      unconfirmed. It remains unconfirmed; this document now says so explicitly instead of
      implying symmetry between providers.
    - **Edge emits `WordBoundary` and `SentenceBoundary`.** Verified in `edge-tts` source
      (`src/edge_tts/submaker.py`), which enumerates exactly those two event types. This is the
      only engine in the registry with a documented *sentence*-granularity boundary, and it is why
      the alignment contract below is written as a requirement rather than a hope.
    - **`msedge-tts` 0.4.0 (MIT OR Apache-2.0)** exists and is maintained (27.8k downloads,
      updated 2026-05-20). The Rust-friendliness of the Edge path is therefore not the liability
      it was assessed as. `edge-tts-rs` 0.1.3 has been dead since 2024-01-19 and is banned.

  Scope discipline: this amendment touches this file only. No template layer was written back,
  and no crate, contract, or gate script was modified by it.

  Deferred items: none. No placeholder tokens remain. The engine ranking within the registry, and
  the measured startup-time bound, remain feature-level decisions for `/speckit.specify`.
-->
<!--
  SYNC IMPACT REPORT
  ===================
  Version change: template (unresolved) -> 1.0.0
  Ratified: 2026-10-01 (initial adoption; the project had no constitution before this)

  Bump rationale: this is an **initial adoption**, not an amendment, so MAJOR/MINOR/PATCH
  does not apply. Every principle and section is new. The previous file was the unresolved
  Spec Kit scaffold, in which every value was an `[ALL_CAPS_IDENTIFIER]` placeholder.

  Principles added (11, all new):
    I.   Library-First, Thin Tauri Shell
    II.  Text I/O Protocol (CLI-First)
    III. Test-First (NON-NEGOTIABLE)
    IV.  Latest Stable Versions, Supply-Chain Gated
    V.   Deterministic Gates Over Judgment
    VI.  Engine-Neutral Core, Adapters at the Edges
    VII. Closed Sets, No Catch-Alls
    VIII.Driving-First (NON-NEGOTIABLE)
    IX.  Offline-First
    X.   Free-Tier Budgeted, Not Free-Hopeful
    XI.  Credential and Content Sovereignty

  Sections added:
    - All eleven Core Principles
    - Additional Constraints (Technology Stack, Dependency Deny-Lists, Licence Policy,
      Rust Code Rigor, Tauri Platform Contract, Driving-Safety Requirements,
      Audio Pipeline Contracts)
    - Development Workflow (Spec-Driven Development, Verification Gates, Exceptions Log)
    - Governance (Amendment Procedure, Versioning Policy, Compliance Review)

  Removed sections: none. This is the first constitution; the template carried no
  project-specific content to preserve.

  Deviations from the crsdd-fabro reference stack, each authorized by the sole maintainer
  (see Additional Constraints -> Exceptions Log). The reference stack is a **default, not an
  obligation**. The four that matter:

    - **Copyleft is permitted.** The reference bans GPL/AGPL/LGPL/EUPL because it ships a
      proprietary product to enterprises. This project is non-commercial and intended to be
      100% open source, and GPL-3.0 is the licence of `espeak-ng` (linked by sherpa-onnx) and
      of `piper1-gpl`. Excluding copyleft would have excluded the working offline engine.
      Revisiting this for adoption is a future amendment, not a current constraint.
    - **`ort` / `onnxruntime` are not banned.** The reference bans them to keep an LLM out of
      a code-migration pipeline. Here they are the inference runtime; banning them would ban
      the product. No LLM is in this system at all.
    - **Network clients are not banned.** The reference bans `reqwest`/`hyper`/`ureq`/`curl`
      to guarantee zero egress from an agent sandbox. There is no agent and no sandbox; an
      HTTPS client is this product's core capability. Egress control moves to Principle XI.
    - **`reqwest-tls` was removed from the deny-list draft.** It does not exist on crates.io.
      A named entry that does not exist is a fiction the gate cannot enforce.

  Research basis (verified 2026-10-01; every load-bearing claim below is sourced):
    - Google Cloud TTS: WaveNet is GA with no deprecation announcement, 4M chars/month free,
      USD 4/M after; the per-request content limit is **5,000 bytes** (not characters);
      Chirp3 is capped at 200 req/min; 100 concurrent streaming sessions per project.
    - `ort` `dist.tsv` ships a prebuilt `aarch64-linux-android` ONNX Runtime 1.30.0 with the
      NNAPI execution provider, SHA-256 verified and statically linked. No source build needed.
    - sherpa-onnx is Apache-2.0 and publishes arm64-v8a APKs for five Piper pt-BR voices.
      Piper pt-BR voice data is CC0 (`faber`, `cadu`, `jeff`); `edresson` is CC-BY-4.0 and is
      excluded on attribution grounds.
    - Kokoro-82M has three pt-BR voices, but sherpa-onnx's Kokoro build ships zh+en only, and
      Kokoro's pt-BR G2P requires espeak-ng. Hence **Piper forward** for the offline default.
    - Tauri v2 has **no** `mobile::update_manifest` API, so a foreground media service cannot be
      declared from a plugin. `tauri::ipc::Channel` does exist and is the sanctioned streaming path.

  Deferred items: none. No placeholder tokens remain. The `specs/` artifacts referenced by
  Development Workflow are created by `/speckit.specify`, not by this document.
-->
# mdreader Constitution

## Core Principles

### I. Library-First, Thin Tauri Shell
Every capability is a standalone Rust crate with one clear purpose, independently testable and
independently documented. The Tauri application is **one** consumer of those crates, never the
only one, and no crate may exist purely to organise other crates.

The engine MUST be drivable headless on Linux. Any behaviour reachable from the phone MUST also
be reachable from a command line, because that is what makes it testable in CI without an
emulator — a phone-only surface is a surface CI cannot see.

### II. Text I/O Protocol (CLI-First)
Every crate exposes its capability over text: arguments and stdin in, stdout out, errors to
stderr, never mixed. Machine-readable output is available via an explicit `--format json` flag
and MUST be the default for scripting; a human-readable format is offered alongside it.

The same protocol is how a crate is driven from the Tauri frontend over IPC, so the IPC path
and the CLI path exercise identical code. A capability that exists only behind IPC fails this
principle.

### III. Test-First (NON-NEGOTIABLE)
Tests are written, shown to fail, then implemented. Red-Green-Refactor is enforced by review,
not by tooling alone.

Test-first is non-negotiable rather than preferred because this project's value is entirely in
behaviour that is hard to verify by inspection: prosody boundaries, byte-budget chunking,
offset alignment between synthesised audio and Markdown source nodes, and audio-focus
transitions while a phone is locked. Reading the code does not tell you whether the reader stops
at a sentence boundary in a tunnel. The test suite is the only evidence that it does.

### IV. Latest Stable Versions, Supply-Chain Gated
Every dependency MUST be the **most recent stable release** available at the time it is added or
bumped. Stability means a non-pre-release version: not alpha, beta, release-candidate, nightly,
canary, or dev.

Departure from this rule is an exception, permitted only when a stated compatibility constraint
forces it, and each exception MUST record in the commit message: the exact version chosen, the
technical reason the stable alternative does not work, and the accepted risk. A preference,
scheduling convenience, or "it worked last time" is not a technical reason.

A supply-chain gate MUST fail closed. It MUST reject an active security advisory
(`cargo deny check advisories` plus `cargo audit`), MUST reject any source outside crates.io,
MUST reject a yanked version, and MUST verify the committed `Cargo.lock` on every build
(`--locked`). The gate MUST NOT auto-substitute one crate for another of similar name; a
failing crate is surfaced for human decision, because that substitution is the slopsquatting
pattern itself.

Rationale: pinning the newest stable keeps the known-CVE surface as small as the ecosystem
allows, and a fail-closed gate is what makes that policy safe to apply mechanically rather
than aspirationally.

### V. Deterministic Gates Over Judgment
Every decision that can be made deterministically MUST be made deterministically. Formatting,
lint level, test outcome, licence compliance, advisory status, dependency policy, and manifest
structure are decided by commands that exit zero or non-zero. No model and no reviewer opinion
is an input to whether the build is green.

Gates are fail-closed: a missing input fails its gate rather than being skipped. A gate that
passes when its input is absent is not a gate.

### VI. Engine-Neutral Core, Adapters at the Edges
TTS engines are reached through a Rust trait. No vendor SDK type, no HTTP response shape, no
SDK error code, and no provider-specific identifier may cross into the core. Implementations are
swappable and every provider-specific decision is confined to its adapter crate.

The intermediate spoken representation is engine-neutral **SSML**. Every adapter MUST declare its
SSML capability level, and degradation of prosody MUST be explicit and observable — never silent.
A heading that was rendered with a `<break>` and pitch adjustment under one engine, and flattened
to plain text under another, is a defect the user must be able to see, because the difference is
audible and it changes whether a document is pleasant to listen to.

SSML is chosen as the neutral form because the cloud engines honour it and the offline engines do
not, which makes the capability boundary unavoidable and therefore explicit rather than a
surprise discovered in the car.

### VII. Closed Sets, No Catch-Alls
Error types, provider identifiers, stage verbs, capability levels, and policy enums are closed
sets. Every path is explicit and auditable. No `Other`, `Internal`, `Unknown`, or catch-all arm
may appear on a closed-set enum.

Adding a variant is a breaking change requiring coordinated edits across affected files, a
semver bump, and a documentation update. The rationale is that a catch-all arm converts an
unhandled condition into a silently wrong one, and in an app whose failure mode is "went quiet
in the tunnel", a silent branch is worse than a failed build.

### VIII. Driving-First (NON-NEGOTIABLE)
The primary context is a moving vehicle. **No required interaction may demand looking at the
screen.** Every operation reachable while driving — play, pause, resume, next section, previous
section, adjust rate, change voice — MUST be reachable from the lock screen or notification
without unlocking the device and without a tap sequence that requires visual confirmation.

Playback MUST continue with the screen off, MUST survive the device being locked, and MUST
survive an interrupted network connection without stopping mid-sentence.

This principle exists because it is the failure the project was created to fix. The tools
replaced so far either have no Android version, cannot read local files, or take long enough to
start that they are already a distraction when they finally do. Driving-safety is therefore a
functional requirement here, not a UX preference, and it is the reason the notification and
media-session surface is built first rather than last.

### IX. Offline-First
The engine MUST produce audible speech with no network connectivity. A design that degrades to
silence when the network drops is non-conforming.

The offline path is `sherpa-onnx` running Piper pt-BR voice models. Cloud synthesis is an
enhancement layered on top, never the sole path. Rationale: tunnels, rural roads, and parking
structures are exactly where a car-commute reader is used, and a dependency that needs signal
cannot serve that case at all.

### X. Free-Tier Budgeted, Not Free-Hopeful
No build and no run may require a paid tier. Quotas, per-request content limits, and
per-character prices are recorded in a versioned table in the repository, re-verified against
provider documentation, and changed only by amendment.

Requests MUST be chunked below the smallest documented per-request limit, measured in **bytes**
rather than characters, and a test MUST enforce that budget. The 5,000-byte Google limit is
exceeded by roughly 2,000 bytes by a single mid-sized Portuguese paragraph, so this is not a
theoretical concern.

The default cloud voice MUST be Google Cloud TTS WaveNet while that family exists and its free
quota holds. When the quota ends or the family is retired, the system MUST escalate through the
remaining free options in order of quality-per-free-character, and finally MUST choose explicitly
between low-quality local synthesis and a paid high-quality API — surfacing that choice to the
user as a decision, never making it silently on their behalf and never spending money without
explicit opt-in.

Rationale: "free" is a property that decays. WaveNet is GA today with no deprecation announced
and the largest free tier of any option, but the constitution states a trigger and an escalation
order rather than a permanent preference, so that its eventual retirement is a planned state
transition instead of a break.

### XI. Credential and Content Sovereignty
No API key, token, or secret may appear in the repository, in a committed file, or in the
application binary. Credentials enter through the platform keystore under user control, or
through a user-operated proxy. This is non-negotiable because a key compiled into a distributed
APK is extractable in minutes, which is why the Chrome extension being replaced needed a backend
project of its own.

Document content MUST NOT reach a third-party synthesis provider without explicit, per-provider,
revocable user consent. No telemetry, no usage analytics, and no crash reporting to any external
service ships enabled by default.

**One named exception, and it is narrow.** The Microsoft Edge "read aloud" endpoint is not a
contracted provider: it is undocumented, has no SLA, and authenticates with a synthesised token
rather than a credential the user holds. An Edge adapter is therefore admissible as an **evaluated
fallback** under the same consent gate as any other cloud engine — explicit, per-provider,
revocable — while never being ranked as a default. The exception waives the *contract*, not the
*consent*, and it does not permit silent substitution: an Edge adapter that cannot be reached MUST
surface as an engine failure rather than as silence.

Rationale: this records a real property of that endpoint instead of leaving it to be discovered
mid-car-commute, and it keeps the consent boundary intact where it matters — the user's choice of
provider — while admitting that one endpoint has no contract behind it.

## Additional Constraints

### Technology Stack
- **Language**: Rust, Edition 2024, MSRV 1.85 for workspace code (inherited from the reference
  stack); the full build additionally requires rustc >= 1.88 — the `zip` 9 build-script dependency
  of `sherpa-onnx-sys` (Exceptions Log entry 12)
- **Toolchain**: the workspace **tracks the `stable` channel** and does **not** pin a compiler
  version. `rust-toolchain.toml` names that channel and the components the gates need
  (`rustfmt`, `clippy`), so the toolchain the gates run under is a property of the repository
  rather than of the host's rustup default, which may be an older pinned toolchain (this host
  carries 1.94.0, 1.98.0 and nightly alongside `stable`). The **MSRV line above is the
  compatibility floor and is unchanged by this clause** — it states the oldest compiler the
  workspace supports (1.85 for workspace code, rustc >= 1.88 for the full build), not the
  compiler the workspace builds with; a tracked `stable` channel can never contradict a floor,
  because `stable` only rises. A toolchain bump on the tracked channel is **expected to move the
  lint set**: a new clippy release may add a lint, change a lint's reach, or retire one, so a
  tree that was green can become red with **no source change at all**. Every toolchain bump
  therefore requires a **lint-compliance pass** before the tree is re-attested — see
  "Verification Gates". The moved lints are resolved **in code**, never by weakening a lint level
  in `Cargo.toml` or `clippy.toml` and never by a blanket suppression (Rust Code Rigor). A hard
  version pin is deliberately **not** adopted: it would freeze the compiler *and* the lint set, so
  new lint coverage would never arrive and a "green" tree would be green only against a frozen
  toolchain; it would also force a download of that exact toolchain on any host that lacks it,
  and it would sit in tension with Principle IV's latest-stable posture. This clause is
  reversible by deleting this entry, deleting the repo-root `rust-toolchain.toml`, and adopting a
  hard version pin instead — no other governance rule changes.
- **Build**: Cargo workspace; every member crate declares `[lints] workspace = true`
- **App shell**: Tauri **2.12.1** (latest stable on the 2.x line as of 2026-10-01).
  `3.0.0-alpha.3` is available and is **not** to be taken: Principle IV permits pre-release only
  for compatibility, and stability here is a policy choice, not a forced exception.
- **Frontend**: Svelte 5 (Runes mode)
- **IPC**: `tauri::ipc::Channel` for Rust-to-frontend streaming. Note that
  `InvokeBody::Streaming` does not exist in Tauri 2, so streaming responses are built on
  channels or successive invocations, never assumed.
- **Native bridge**: custom Kotlin plugins via `PluginApi::register_android_plugin` and
  `PluginHandle::run_mobile_plugin`. There is no `registerService`; see the Tauri Platform
  Contract below.
- **Markdown**: `pulldown-cmark` (0.13.x). Its event stream carries source ranges, which is what
  makes boundary-to-source offset mapping possible.
- **Segmentation**: `unicode-segmentation` (UAX #29), with a pt-BR abbreviation guard list,
  because UAX #29 splits on the dot in `Dr.` and on the dot in `1.5`.
- **Inference**: `ort` (2.0.0-rc.13) with the NNAPI execution provider, plus `sherpa-onnx`
  (1.13.8) for the TTS pipeline. `ort` being an RC is a recorded exception under Principle IV:
  the Android arm64 prebuilt with NNAPI is only published on the RC line.
- **Errors**: `thiserror`, closed-set enums. **Observability**: `tracing`. **CLI**: `clap`
  derive. **Serialisation**: `serde`. **State**: `redb` for the read position bookmark.

Extending this list is an amendment, not a dependency-resolution decision.

### Dependency Deny-Lists
This project's deny-list is **not** the reference project's. Each entry is a named crate with a
reason that is still true here, and every name was verified to exist on crates.io on 2026-10-01.
A name that does not exist is removed rather than left as fiction.

**Deny (risk-based, not technology-based):**

| Crate | Reason |
|---|---|
| `tts-rs` | Slopsquat profile: generic name, created 2026-02, 755 downloads, never updated. Legitimate functionality, no adoption. |
| `onnx` | Frozen since 2018 (5.4k downloads), predates the `ort` ecosystem. |
| `sherpa` | Frozen since 2017 (12k downloads), unrelated tool predating `sherpa-onnx`. |
| `candle`, `burn`, `tch` | ML training/inference frameworks outside scope; large supply-chain surface. |
| `llm`, `llama-cpp-2`, `llama-rs`, `rust-bert` | Generic LLM shims and runtimes; no LLM in this system. |
| `openai`, `async-openai`, `openai-api` | LLM provider clients; Principle XI forbids unconsented egress. |
| `langchain`, `langchain-rust` | Orchestration frameworks outside scope. |
| `edge-tts-rs` | Dead since 2024-01-19 (0.1.3, 5.4k downloads); superseded by `msedge-tts`. |

The list is closed. Adding an entry requires a reason that survives the "is this true for this
project" question, and removal requires the same test. `sherpa-onnx`, `ort`, `espeak-rs`, and
`msedge-tts` are explicitly **not** banned: each is a verified, actively maintained dependency or
the sanctioned protocol client for this project.

### Licence Policy
Copyleft is **permitted**. `deny.toml` allows MIT, Apache-2.0 (with LLVM exception),
CDLA-Permissive-2.0, BSD-2/3, ISC, Zlib, bzip2-1.0.6, MPL-2.0, Unicode-3.0, GPL-3.0,
LGPL-3.0, AGPL-3.0, CC0-1.0, and CC-BY-4.0. Copyleft is
admitted because `espeak-ng` is GPL-3.0 and is linked by `sherpa-onnx`; excluding it would have
excluded the working offline engine. CDLA-Permissive-2.0 is admitted because
`webpki-roots` (transitive via the rustls/webpki chain the HTTPS client requires)
ships Mozilla's CA root certificates as data under it — a permissive data-sharing
license with no copyleft and no attribution obligation beyond the license notice,
consistent with the permissive data licenses already admitted (CC0-1.0, CC-BY-4.0);
the crate is data, not executable logic. bzip2-1.0.6 is admitted because
`libbz2-rs-sys` — reached as `zip` 9's `bzip2` default feature, the vendored
`sherpa-onnx-sys` build-dependency (Exceptions Log entry 12) — carries no copyleft
and only notice-and-attribution conditions, and the crate is build-time only;
nothing from `zip` links into shipped artifacts. The admission outlives entry 12:
the upstream fix (PR #4059) bumps the dependency version only, default features
kept.

**Two axes, not one.** Crate licence and model-weight licence are audited separately, because
they are different obligations. A permissive engine does not make its weights distributable:
Piper pt-BR voice `edresson` is CC-BY-4.0 and carries an attribution obligation that the CC0
voices (`faber`, `cadu`, `jeff`) do not, so the default set is the CC0 three. Weight licences MUST
be recorded per voice, not per engine, because the Piper repository holds both kinds.

Because the project is non-commercial and intended to be fully open source, copyleft
contamination of the shipped app is acceptable rather than a risk to be mitigated. Distribution
stays sideloaded and GitHub-Released for now; F-Droid (own repo first, public submission only if
wider adoption is wanted) and the Play Store are later amendments. F-Droid accepts GPL-3.0 and
AGPL-3.0, so the licence choice does not block that path.

### Rust Code Rigor
- `unsafe_code = "deny"`, `missing_docs = "deny"`, `missing_debug_implementations = "deny"`,
  `unreachable_pub = "deny"`, `unused_results = "deny"`
- `clippy::all = deny`, `clippy::pedantic = deny`, `clippy::nursery = warn`, `clippy::cargo = warn`
- `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`,
  `print_stdout`, `print_stderr` — all deny. No `println!`/`dbg!`; use `tracing`
- `allow_attributes` and `allow_attributes_without_reason` — deny. The only sanctioned
  suppression form is `#[expect(lint, reason = "auditable rationale")]`
- `wildcard_dependencies` and `multiple_crate_versions` — deny
- Panics in library code are defects. Failure is a typed error, not an unwind.

Never weaken `Cargo.toml` or `clippy.toml` to silence a lint. A lint that cannot be satisfied is
resolved in code or, if genuinely unavoidable, via `#[expect]` with a written reason.

### Tauri Platform Contract
The application is a **modular monolith**: one Cargo workspace of focused crates, and the Tauri
application is one consumer of them rather than the only one (Principle I). The audio path is
split so that **Rust owns synthesis and playback**, and a deliberately thin Kotlin surface owns
only what the Android platform requires a Java or Kotlin type for.

- **Kotlin is confined to the platform shell.** The Android `Service`, the `MediaSession`, the
  notification, and the audio-focus request live in Kotlin. Everything else — parsing, segmentation,
  SSML assembly, synthesis, and **PCM playback** — lives in Rust. This is a recorded architectural
  decision, not a platform impossibility: Kotlin calls into Rust over `PluginHandle::run_mobile_plugin`
  and Rust drives playback through `cpal` (AAudio backend) or `oboe`.
- **Single playback authority.** Rust owns the playback state and is the sole authority for
  position, buffering, and transport. The Kotlin `MediaSession` is a *mirror* of that state, never a
  second source of truth. Two authorities over playback position is the specific failure this rule
  exists to prevent: it produces a desynchronisation between the notification and the audio that
  is invisible in testing and audible only in a car.
- **Foreground service**: `tauri-plugin` exposes **no** `mobile::update_manifest` API — only
  `update_info_plist` for iOS. A `<service>` with `android:foregroundServiceType="mediaPlayback"`
  therefore **cannot** be declared from a plugin and MUST be added by editing
  `AndroidManifest.xml`/Gradle directly. That edit MUST be re-verified on every Tauri upgrade.
- **The Service is Kotlin for a structural reason.** The Android `ActivityManager` instantiates a
  `Service` by reflecting on the class named in the manifest, so a pure-Rust service cannot exist.
  This is the only language boundary in the audio path that the platform forces.
- **Plugin lifecycle**: `register_android_plugin` binds to the Activity and its Kotlin constructor
  takes `(Landroid/app/Activity;)V`. There is no `registerService`. The `Service` MUST
  self-bootstrap from the `Application` or the Activity, which is a non-standard lifecycle and
  interacts with Android 14+ restrictions on starting a foreground service from the background.
- **Required components**: `FOREGROUND_SERVICE` and `FOREGROUND_SERVICE_MEDIA_PLAYBACK`
  permissions; a `MediaSession`; `AudioFocusRequest` with `AUDIOFOCUS_GAIN`; handling of
  `ACTION_AUDIO_BECOMING_NOISY` so that unplugging headphones pauses rather than playing
  out of a speaker.
- **Deliberate deviation from Android ducking best practice (ADR-001 — Audio Focus).**
  The Android documentation (`developer.android.com/media/optimize/audio-focus`) states
  that automatic ducking *"isn't useful when playing spoken content, such as in an audio
  book app"* and recommends `setWillPauseWhenDucked(true)` (pause rather than duck) for
  spoken-word apps. This project deliberately chooses the opposite: transient focus loss
  (`AUDIOFOCUS_LOSS_TRANSIENT` / `AUDIOFOCUS_LOSS_TRANSIENT_MAY_DUCK`) causes the
  document to duck to ~20% of normal volume and continue reading, never pausing, with
  rewind-on-resume of 5% capped at 30 s (restarting at the sentence boundary the cursor
  lands in). Rationale: a brief navigation prompt or notification chime must not chop a
  sentence mid-flow during a commute; the product's contract with the driver is
  continuity, not interruption. The platform recommendation is noted, the deviation is
  conscious, and it is reversible by removing this clause and adopting the platform
  default without changing any other governance rule. See the Exceptions Log entry
  for the audit trail.
- **Android Auto is out of scope for now.** Auto requires a browsing-capable
  `MediaLibraryService` and has no TTS category, so a Markdown reader does not qualify. This is
  recorded as a deliberate deferral, not an oversight.

### TTS Engine Registry
The admissible engine set is closed. Adding or removing an engine is an amendment, not a
dependency-resolution decision, because an engine arriving or leaving changes what the user hears
and what leaves the device.

| Engine | Tier | Boundary events | SSML | Admissible |
|---|---|---|---|---|
| `sherpa-onnx` + Piper pt-BR (`faber`, `cadu`, `jeff`) | offline default | none | none | **Yes** — Principle IX |
| Google Cloud TTS WaveNet | cloud default | `timepoints` UNVERIFIED | full | **Yes** — Principle X |
| Google Cloud TTS Neural2 / Chirp 3 HD / Studio | cloud escalation | UNVERIFIED | full / partial | Yes, ranked by free quota |
| Amazon Polly Neural (speech marks) | cloud escalation | word-level | full | Yes, ranked by free quota |
| Azure Speech Neural | cloud escalation | word-level | full | Yes, ranked by free quota |
| Microsoft Edge read-aloud (via `msedge-tts`) | evaluated fallback | **`WordBoundary` + `SentenceBoundary`** | none | **Yes, under the XI exception** |
| Kokoro-82M pt-BR | offline alternative | none | none | Deferred — sherpa-onnx ships zh+en only |

Ranking within the cloud tiers belongs to `/speckit.specify`. What this document fixes is the
*closed set* and the rule that capability differences are declared rather than assumed.

Two findings that changed the registry and are recorded so they are not re-litigated:

- **Sentence-granularity alignment is achievable today, but only via Edge.** Verified in
  `edge-tts` source (`src/edge_tts/submaker.py`), which accepts exactly `WordBoundary` and
  `SentenceBoundary`. Google's `timepoints` field is **UNVERIFIED** — not confirmed to exist in the
  REST response, and absent from the Gemini TTS docs. The alignment contract below therefore
  states sentence-level as a requirement of the *pipeline*, not as an assumption that every engine
  can satisfy it.
- **The Rust path to Edge is maintained.** `msedge-tts` 0.4.0 (MIT OR Apache-2.0, 27.8k downloads,
  updated 2026-05-20). `edge-tts-rs` 0.1.3 has been dead since 2024-01-19 and is banned. Note that
  the widely-starred `travisvn/openai-edge-tts` is a **Python/Docker wrapper around** `edge-tts`,
  not a protocol implementation: it exposes the OpenAI-compatible `/v1/audio/speech` shape and its
  SSE stream carries only `speech.audio.delta` and `speech.audio.done`, discarding the boundary
  events. An adapter MUST talk to the endpoint directly and MUST NOT proxy through that wrapper,
  because proxying loses the only capability the engine uniquely offers.

### Driving-Safety Requirements
Playback MUST continue with the screen off and while the device is locked. All driving-time
controls MUST be reachable from the lock screen or notification (Principle VIII). Rate and voice
changes MUST be reachable without unlocking. The app MUST NOT require a network connection to
begin or continue playback (Principle IX).

Audio MUST start within a bounded time from the control action, so that a tunnel does not turn a
play button into a guessing game. That bound is a measured requirement to be fixed by the first
`/speckit.specify`, not an open question.

### Audio Pipeline Contracts
Markdown is parsed to an AST carrying source ranges; filtered by a spoken-content policy (code
blocks, images, and tables are skipped or announced, not read verbatim); segmented into sentences
with a pt-BR abbreviation guard; normalised for spoken numbers and currency; assembled into
engine-neutral SSML; synthesised per chunk under the byte budget; and aligned back to source
nodes for highlighting.

Alignment is the known hard problem. Synthesis normalises text (`1.5` becomes "um ponto cinco"),
so word events cannot map one-to-one onto source offsets. The mitigation is to map at **sentence or
node granularity**, and to treat that as the designed precision rather than a defect to be
eliminated.

The pipeline MUST therefore be capable of sentence-level alignment to source nodes, and a test
MUST exercise it. Only the Edge endpoint is currently known to supply sentence boundaries; the
other engines supply none, or supply word events whose presence is unverified. An adapter MUST
declare which granularity it can deliver, and the pipeline MUST degrade to a coarser, declared
granularity when an adapter cannot deliver sentence alignment — observably, per Principle VI, never
by silently highlighting nothing.

No mature pt-BR number-to-words crate exists; this normaliser is written in this project and is
covered by tests.

## Development Workflow

### Spec-Driven Development
Every feature is a `specs/NNN-<short-name>/` directory containing `spec.md` (what), `plan.md`
(how), `tasks.md` (dependency-ordered), `checklists/` (requirements as tests), and `contracts/`
(JSON Schema for wire formats). This layout is inherited from the reference stack.

Voice selection, engine ranking within the TTS Engine Registry, Markdown-to-speech policy, and
the startup-time bound are feature-level decisions. The constitution fixes the closed set of
admissible engines, the escalation order, and the consent boundary; it does not fix the ranking,
which belongs to `/speckit.specify`.

### Verification Gates
`scripts/verify.sh` is the single entrypoint, mirroring the reference project's `ci.sh` ordering.
Exit status is zero if and only if every gate passes.

| # | Gate | Command | Checks |
|---|---|---|---|
| 1 | format | `cargo fmt --all -- --check` | rustfmt-clean |
| 2 | tests | `cargo nextest run --workspace` | unit, integration, E2E |
| 3 | documentation | `cargo doc --workspace --no-deps` | public API documented |
| 4 | lint-drift | root deny-tables intact; `[lints] workspace = true` in every member; `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (fail-closed step `lint-drift-clippy`) | rigour not quietly weakened; the lint SET is clean under the tracked toolchain, so a toolchain bump that moves it fails the gate; the lint-compliance pass below is REQUIRED before re-attestation |
| 5 | supply-chain | `cargo deny check advisories licenses bans sources` + `cargo audit` | Principle IV |
| 6 | deny-list | no denied crate in any manifest or in `Cargo.lock` | Principle IV |
| 7 | closed-set | no `_ =>` in `crates/*/src/` | Principle VII |
| 8 | pipeline | byte-budget test under 5,000 bytes/request; SSML capability declared per adapter; sentence-level alignment exercised | Principles VI, X |
| 9 | android-build | `cargo tauri android build --target aarch64` | the app actually compiles for the device |

**A toolchain bump moves the lint set, and the tree must be re-earned.** Gate 4 checks the lint
*levels* declared in the root `[workspace.lints]` tables, and those do not move with the
toolchain. The lint *set* — which lints exist, and which code they reach — does. A `stable`
release can add a lint (`clippy::assert_is_empty`), change a lint's reach (clippy 0.1.99 stopped
linting the `==` that `assert_eq!` generates inside its own macro expansion), or retire one, so
**a tree that passed every gate can fail with no source change at all**. That is the
deterministic-gate concern Principle V exists for, and it has exactly one required response: on
every toolchain bump, a **lint-compliance pass** (`cargo clippy --workspace --all-targets -- -D
warnings`, exit 0) MUST run and MUST be clean **before the tree is re-attested**. The pass
resolves the moved lints **in code** — never by weakening a lint level in `Cargo.toml` or
`clippy.toml`, never by a blanket `#[allow]`, and never by an `#[expect]` without a written
reason (Rust Code Rigor). The recorded instance is the 2026-10-20 `stable` bump (rustc 1.98.1 ->
1.99.1, clippy 0.1.98 -> 0.1.99), which turned a byte-identical tree red with 86
`unfulfilled_lint_expectations` and 13 `clippy::assert_is_empty` diagnostics; T136 repaired it in
code and T137 records the policy. Gate 5 is unaffected by the movement of the lint set, and the
tracked channel cannot break it either: the build-time floor it depends on (rustc >= 1.88, the
`zip` 9 build dependency of `sherpa-onnx-sys`) is a floor that `stable` only rises above.

The mechanical form is now wired: `scripts/verify.sh` gate 4 runs the lint-compliance pass itself —
`cargo clippy --workspace --all-targets -- -D warnings` — as a fail-closed step (`running 4
lint-drift-clippy`, then `fail` on a non-zero exit, then `pass 4 lint-drift-clippy`), so the lint
*set* is enforced by the gate itself, not only by the manual pre-attestation pass: a `stable` bump
that moves the lint set now fails a deterministic gate (Principle V) instead of depending on a
person remembering to run the pass. The step fails closed — a missing toolchain, a clippy that
cannot run and a non-zero exit are all a FAILED gate, never a skipped one; there is no `|| true`,
no "the tool is missing -> pass" branch and no skip flag. The gate keeps checking the declared lint
levels as well — the clippy step is an addition, not a replacement — and the moved lints are
resolved **in code**, never by weakening a lint level in `Cargo.toml` or `clippy.toml`, never by a
blanket `#[allow]`, and never by an `#[expect]` without a written reason (Rust Code Rigor). Wired
in T138 (commit `d23942b`); verified by a negative control on a copy of the tree, where a
`println!` injected into `crates/doc-structure/src/lib.rs` trips the denied `clippy::print_stdout`
and the copy's `verify.sh` exits 1. This paragraph records the state; the enforcement itself lives
in `scripts/verify.sh` gate 4, and deleting this paragraph changes no gate, principle or constraint.

Gate 9 exists because gates 1–8 are all satisfiable on a host that cannot produce an Android
artifact, which would let a workspace pass every deterministic check while having no working app.

Human review remains a requirement at phase boundaries, where a decision needs judgement rather
than a command.

### Exceptions Log
The reference stack (`~/development/MDMV/projetos/crsdd-fabro`) is the **default, not an
obligation**. Deviations are permitted and are recorded here so they are decisions rather than
drift. Each was authorized by the sole maintainer on 2026-10-01.

1. **Copyleft permitted** (deviates from the licence allow-list). Reason: `espeak-ng` is GPL-3.0
   and is linked by `sherpa-onnx`; the project is non-commercial and open source.
2. **`ort`/`onnxruntime` permitted** (deviates from the LLM framework deny-list). Reason: it is
   this project's inference runtime; no LLM is involved.
3. **Network clients permitted** (deviates from the network deny-list). Reason: an HTTPS client is
   the product; egress control is Principle XI.
4. **`ort` 2.0.0-rc.13 accepted** (deviates from stable-only). Reason: the Android arm64 prebuilt
   with the NNAPI provider is published only on the RC line. Re-evaluate when 2.0.0 stable ships.
5. **`cargo machete` not inherited.** Its unused-dependency check would flag legitimate
   dev-only and test-only crates. It applies from the first release onward.
6. **Android Auto deferred.** See Tauri Platform Contract.
7. **Edge read-aloud endpoint admitted as a fallback.** See Principle XI. It is a documented
   exception to the consent rule, not a ranked default, and the adapter speaks the protocol
   directly rather than through the OpenAI-compatible Python wrapper.
8. **Gate 8 split for `001-android-media-shell`.** Constitution gate 8 (byte-budget + SSML + sentence-alignment) cannot pass while FR-024 places synthesis in another feature. `verify.sh` reports `gate-8-shell: PASS` (handoff shape + granularity-declared + 22050 Hz request) and `gate-8-full: DEFERRED (owner=synthesis)`; no build claims constitution gate 8 PASS until the synthesis feature discharges it with SC-044.

9. **Workspace Dependency Portability and Supply-Chain Policy (1.4.0)** — deviations
   from the unconditional-declaration default (previous implicit standard) to
   a portability-first dependency-selection rule recorded in the constitution.
   Author: maintainer (2026-10-10). Reason: the T036 B2 repair showed the
   unconditional `ndk = "0.9"` declaration (unused at declaration time,
   rust-mobile/ndk#80 compile_error! on every non-Android target) broke all
   host builds of `crates/player`. The clause requires: universal preference;
   restricted dependencies only behind `[target.'cfg(...)'.dependencies]`
   with a documented justification; a measured exception bar (>=10x paired
   benchmark or exclusive API); first-use declaration; CI gate (cargo machete);
   supply-chain fail-closed (advisory-free). Reversible by deleting the clause
   (no architecture reversal; no existing MUST weakened; only a new named
   constraint section added). Source evidence: specs/002-fr-024-synthesis-adapter/
   DEPENDENCY_PORTABILITY_RESEARCH.md (4 threads, cited, 2026-10-10); B2
   repair commits `a90d2b6`, `42b59d7`, `286009e` on 001-android-media-shell.
10. **`proc-macro-error` advisory waived (RUSTSEC-2024-0370, 2026-10-12)** —
    deviates from the advisory-free supply-chain gate (Principle IV). Author:
    maintainer (2026-10-12), accepted in the commit that records it. Reason:
    `proc-macro-error` 1.0.4 is a compile-time-only proc-macro dependency of
    tauri's Linux gtk chain (via gtk-rs build machinery); none of its code is
    linked into any shipped artifact — proc macros execute at compile time only.
    The advisory is "unmaintained" status (maintainer unreachable, no commits
    for 2 years, no releases for 4 years) with no known vulnerability and no
    CVE; the crate's API is frozen; removal requires forking tauri's gtk chain.
    Risk accepted as compile-time-only supply-chain surface. Re-evaluate when
    tauri drops the dependency. Mechanical form: `[[advisories.ignore]]` in
    `deny.toml` with the id and this reason.

12. **`sherpa-onnx-sys` vendored override (zip 9, 2026-10-16)** — deviates from the
    crates.io-only source rule of the supply-chain gate (Principle IV). Author:
    maintainer (2026-10-16), accepted in the commit that records it — a human
    decision on the flagged crate, and not a substitution: the vendored tree is the
    same crate name and version (1.13.8) from the crates.io artifact, not a
    lookalike. Reason: the published `sherpa-onnx-sys` 1.13.8 pins the vulnerable
    `zip 2.x` (the now-retired entry 11); the fix is verified source-compatible
    (`DOCS_RS=1 cargo check`, exit 0), proposed upstream, and not yet released —
    waiting would leave RUSTSEC-2026-0367 / 0368 / 0369 in the graph. Mechanical
    form: a vendored copy of the published crate at `vendor/sherpa-onnx-sys/` with
    exactly one line changed in its manifest (`zip` 2 -> 9, a build-dependency),
    wired via `[patch.crates-io]` in the root `Cargo.toml`; the source `.crate`
    sha256, the diff and the exit condition are in
    `vendor/sherpa-onnx-sys/PROVENANCE.md`. Risk accepted as build-time-only: `zip`
    is a build-script dependency of the sys crate and nothing from it links into
    shipped artifacts. Build-time note: `zip` 9 needs rustc >= 1.88 (Technology
    Stack MSRV line). PROVISIONAL — exit condition: a released `sherpa-onnx`
    shipping `zip >= 9`; then bump the dependency in
    `crates/adapter-synthesis/Cargo.toml`, delete `vendor/sherpa-onnx-sys/` and the
    `[patch.crates-io]` stanza, and delete this entry.

13. **`webpki-roots` 0.26.11 advisory waived (RUSTSEC-2026-0381 /
    CVE-2026-111943, 2026-10-20)** — deviates from the advisory-free supply-chain
    gate (Principle IV). Author: maintainer (2026-10-20), accepted in the commit that
    records it. Exact version: `webpki-roots` 0.26.11. Reason (why the version cannot
    simply be bumped): the flagged node is a pure compatibility shim — its ONLY
    dependency is `[dependencies.parent] version = "1" package = "webpki-roots"` and its
    `src/lib.rs` is `pub use parent::TLS_SERVER_ROOTS;`, so it ships NO root certificates
    of its own. The trust anchors actually in use come from `webpki-roots` 1.0.10, which
    HAS removed the distrusted `emSign ECC Root CA - G3` root (the advisory's own text:
    "Version 1.0.10+ removes this root"). The patched set is `>= 1.0.10`; the 0.26 line
    has NO patched release, and every `ureq` 2.x requires `webpki-roots ^0.26` (only
    `ureq >= 3.1.0` requires `^1.0.0`). The 0.26.11 node is therefore version-flagged
    while bundling nothing. Reachability: build-time only — `adapter-synthesis` ->
    `sherpa-onnx` 1.13.8 -> vendored `sherpa-onnx-sys` 1.13.8 (build-dependency `ureq`
    "2.12") -> `ureq` 2.12.1 -> `webpki-roots` 0.26.11; nothing from `ureq`, `rustls`,
    `rustls-webpki` or `webpki-roots` links into any shipped artifact. Risk accepted as
    build-time-only supply-chain surface: the trust anchors in use are already patched at
    1.0.10, and the residual is a version-number artifact of a shim. Mechanical form
    (what makes this waiver version-bound): `scripts/verify.sh` gate 5 asserts that the
    lockfile's `webpki-roots` version set is exactly `{0.26.11, 1.0.10}`; any other
    version — an unpatched 1.x, a reappearance of an unpatched 0.26.x, or the shim
    leaving the graph — FAILS the gate, so the waiver cannot silently widen. Tooling
    form: a `[[advisories.ignore]]` block for RUSTSEC-2026-0381 in `deny.toml`, plus
    `[advisories] ignore = ["RUSTSEC-2026-0381"]` in a new repo-root `.cargo/audit.toml`
    — gate 5 runs BOTH `cargo deny check advisories` and `cargo audit`, and each tool
    reads its own ignore file. PROVISIONAL — exit condition: the `webpki-roots` 0.26
    shim leaving the graph, i.e. the vendored `sherpa-onnx-sys` build-dependency moving
    to `ureq` 3.x (which requires `webpki-roots ^1.0.0`), or upstream publishing a
    0.26.x release carrying the removal; then delete both ignore records and this entry.
    Re-evaluate whenever `webpki-roots` or `ureq` changes version in the lockfile.

Any new deviation is added here in the same commit that introduces it, and is treated as a
minor amendment.

## Governance

### Amendment Procedure
An amendment is a proposed change to this document carrying: the intended outcome, the reason, and
the affected principles and constraints. It MUST be recorded as a Sync Impact Report at the top of
this file, stating the version change, the bump rationale, the modified principles, and any
deferred items.

This project has a single maintainer, who is also its sole user. An amendment is proposed by an
agent or by the maintainer, and is accepted when the maintainer explicitly accepts it. The
two-party approval rule that suits a multi-maintainer repository does not apply here and is not
imposed as ceremony. Acceptance is recorded in the commit that carries the amendment, so that it
is visible in history rather than asserted inside the document.

A proposal that changes a NON-NEGOTIABLE principle, the licence policy, or the
driving-safety requirement MUST be stated explicitly as such, so that accepting it is a
deliberate act rather than an incidental edit.

### Versioning Policy
MAJOR for a backward-incompatible governance change: removing or redefining a principle, or
weakening a MUST. MINOR for a new principle or section, or a material expansion of guidance.
PATCH for clarification, wording, typo fixes, and non-semantic refinements.

Rationale: the value of this document is that a reader can tell which changes matter, so a
version number that does not distinguish them is worse than none.

### Compliance Review
Every feature's Phase 0 MUST attest conformance to this constitution before research begins, and
name any exception rather than silently proceeding. Every amendment MUST be recorded in the
Exceptions Log when it is a deviation from the reference stack. Every new dependency MUST pass
gates 5 and 6 before it is used, and every model-weight licence MUST be recorded per voice before
the voice is offered.

Where this constitution and a convenience conflict, this constitution wins and the convenience is
either changed or proposed as an amendment. Where this constitution and the reference stack
conflict, the maintainer decides, and the decision is logged.

**Version**: 1.11.0 | **Ratified**: 2026-10-01 | **Last Amended**: 2026-10-21