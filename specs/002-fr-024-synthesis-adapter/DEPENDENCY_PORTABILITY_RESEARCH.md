# Dependency Portability Policy — Research Note

> **Scope: workspace-level.** Although filed under `specs/002-fr-024-synthesis-adapter/`
> (where it originated — the T036 B2 repair), this note informs an amendment to
> `.specify/memory/constitution.md` and binds every feature, 001 included.
>
> **Policy under refinement** (proposed 2026-10-10): *when choosing between a
> dependency that runs only on one restricted environment (Android/iOS/web/desktop)
> and one that runs universally, prefer the universal one; use the restricted
> dependency only behind conditional compilation (`[target.'cfg(...)'.dependencies]`)
> when it has clear and overwhelming advantages.*
>
> **Method (2026-10-10):** four parallel research agents; web search + fetch of
> primary sources (official Cargo/Rust documentation, RFCs, project docs) and
> scientific publications (IEEE/ACM/arXiv; ICSE, FSE, ESEM, EMSE, SPLC, ASE,
> USENIX Security), prioritizing 2018–2026. One agent's websearch partially
> failed (HTTP 401) mid-session; its claims were verified via webfetch against
> the fetched primary sources and Semantic Scholar metadata instead.

## Thread A — the "no universal alternative" case and verification of cfg-gated code

**Cargo semantics (Cargo Book, `specifying-dependencies.html`, 2026).** Two gating
forms exist: `[target.'cfg(...)'.dependencies]` (cfg syntax with `not`/`any`/`all`,
evaluated against `rustc --print=cfg --target X`) and `[target.<triple>.dependencies]`.
The same gating works for `dev-dependencies` and `build-dependencies` (build-deps
match the *host* triple). `cfg(feature=…)`, `debug_assertions`, `test` and
`proc_macro` are **silently ignored** inside target sections — features must use
`[features]` + `optional = true`. RFC 1361 (rust-lang/rfcs#1361) introduced
target-specific dependencies precisely to stop unconditional platform deps.

**What gating guarantees — and what it does not.** A gated dep resolves and builds
only when the predicate matches the `--target` (Cargo Book). **Not guaranteed:**
that the dep actually supports that target (validation exists only as the
*unimplemented* RFC 3759 "supported-targets", 2025, rust-lang/rfcs#3759), that
gated code compiles elsewhere, or that it is ever exercised. Consequence: the
declaration itself must carry its justification (see Thread B, gate 6).

**Real-world conventions.** Mozilla's Android experiment (2017) gates `jni`;
Tauri 2.x gates `jni` (android), `embed_plist`/`objc2-app-kit` (macos), and
`cfg(any(target_os = "android", all(target_vendor = "apple", not(target_os = "macos"))))`
for reqwest/rustls; cargo itself gates `libc`/`cargo-credential-*` by `cfg(unix)`,
`cfg(target_os = "linux")`, `cfg(windows)` (Cargo 1.86 dev-cycle blog, 2025).

**Verifying code that cannot run on host CI.**
- *Compile gate:* `cargo check`/`cargo build --target <triple>` per declared triple
  in CI; `cargo-hack` (taiki-e) automates feature/target matrices; enable rustc
  `--check-cfg` / the `unexpected_cfgs` lint.
- *Execution where a Linux target exists:* `cross-rs` runs `cross test` under QEMU
  user-mode (caveat: QEMU bugs cause false failures; FreeBSD unsupported); the
  Embedded Rust Book documents QEMU + a `.cargo/config` `runner`; Codethink (2024)
  documents QEMU-in-CI parallelism.
- *On-device-only behavior* (audio routing, audio focus, lock-screen): host CI
  **cannot** execute it — requires on-device trials or a device farm (Firebase Test
  Lab, 2016; AWS Device Farm) before release, not per-PR.

**Literature (verified via Semantic Scholar).** Agh, Azamnouri & Wagner, "Software
product line testing: a systematic literature review," *Empirical Software
Engineering*, 2024 (doi:10.1007/s10664-24-10516-x). Cabral, Cohen & Rothermel,
SPLC 2010. Neto et al., *Journal of Systems and Software* 2011 (mapping study).
Kästner et al., "Toward Variability-Aware Testing," FOSD 2012
(doi:10.1145/2377816.2377817). Medeiros, Kästner, Ribeiro, Gheyi & Apel, "A
Comparison of 10 Sampling Algorithms for Configurable Systems," ICSE 2016
(doi:10.1145/2884781.2884793). Ahmed, Dev & Wei, "Variability-Aware Fuzzing,"
ICSE 2026. Abal, Brabrand & Wasowski, "Variability bugs in the Linux kernel,"
ASE 2014. The SPL discipline treats the cfg matrix as a product line: test closed
sets exhaustively, sample combinations otherwise (Medeiros ICSE'16), and document
the sampling.

## Thread B — operationalizing the exception bar ("clear and overwhelming advantages")

**Overhead is not inherent — it is bridge-architecture-dependent, so the bar must
be measured, not asserted.** Oliveira et al. (EASE 2023, doi:10.1145/3593434.3593487)
measured Flutter/React Native/Ionic vs native Android across 10 CPU-intensive
benchmarks: in 5/10 a framework version *beat* native (up to −81% energy, −83%
time). Biørn-Hansen et al. (EMSE 2020, doi:10.1007/s10664-020-09827-6) found
Cordova bridges historically ~2× slower, up to 20× for file/GPS access, yet
"certain cross-platform frameworks can perform equally well or even better than
native on certain metrics", and prescribe *well-defined technical requirements for
deliberate selection*. Huber & Demetz (ICSOFT 2019, doi:10.5220/0007838000400048):
multi-platform UI interactions cost ~2× CPU and 14–28× memory.

**Anchor for "overwhelming":** Android's own docs quantify AAudio EXCLUSIVE/MMAP at
~20 ms vs ~205 ms latency (~10×) — a platform-exclusive API, and even Google wraps
it (Oboe falls back AAudio→OpenSL ES) (developer.android.com/ndk/guides/audio/aaudio/aaudio).

**The universal-wrapper pattern is the industry default, not the exception.** cpal
selects AAudio/CoreAudio/WASAPI/ALSA/Web Audio per platform with optional backends
behind feature flags; mio selects epoll/kqueue/IOCP via cfg ("uses the 'best'
implementation possible"); wgpu covers Vulkan/Metal/D3D12/WebGL2 and is the core of
WebGPU in Firefox/Servo/Deno; Rust `std` enforces a platform-abstraction layer
(RFC 1868 portability lint; `tools/tidy/src/pal.rs`). The SEI ATAM catalog lists
the portability tactics: *minimize platform dependencies; isolate platform-specific
code behind APIs/adapters; use portability-supporting technology*
(sei.cmu.edu/library/the-architecture-tradeoff-analysis-method/).

**Selection science supports a gated default.** Practitioner studies (Chase et al.
2024, 24 professionals, 6 selection patterns; ICSE-SEIP 2020, doi:10.1145/3368089.3409711,
26 factors; EMSE 2023, doi:10.1007/s10664-023-10288-w, ISO/IEC 20741 model) show
selection is systematic Search→Compare→Review→Integrate→Maintain with gating. SPL
research (Berger et al., 128 models) shows conditional compilation (Kconfig) is the
canonical variability mechanism — exactly "universal default + restricted variant
behind cfg".

**Proposed operational definition — ALL gates must pass:**
1. **Exclusivity (verifiable):** no universal crate exposes the required API surface.
   Method: dated crates.io keyword search + docs.rs full-text search, query and result
   logged in the decision record. If a universal crate qualifies → exception denied.
2. **Measured magnitude:** ≥10× improvement on a primary attribute
   (latency/energy/startup/memory), or a capability that cannot be approximated
   (e.g. MediaSession routing). Method: paired benchmark on the target platform,
   ≥3 repetitions, median reported, device/OS recorded.
3. **Isolation:** dependency reachable only via `#[cfg]`/Cargo target-specific entry
   or opt-in feature; the universal path compiles and passes tests on all other
   targets (CI matrix) — mirrors cpal/mio/RFC 1868.
4. **Fallback:** a universal fallback (alternate backend or graceful degradation)
   always exists — mirrors Oboe/AAudio fallback; the build never hard-requires the
   restricted environment.
5. **Supply-chain gate:** advisory-free, crates.io-only, actively maintained
   (constitution fail-closed); failure denies the exception regardless of measured
   advantage.
6. **Recorded & re-verified:** decision record with gates 1–5 as evidence;
   re-verified on dependency or platform upgrades (SPL variability-model discipline).
   Plus (Thread A): a comment on the gated entry naming the rejected universal
   alternative and the advantage — Cargo cannot validate target support (RFC 3759
   unimplemented).

## Thread C — a declared-but-unused dependency is debt against the build

**Cost is quantified.** Weeraddana et al. (FSE 2024, dl.acm.org/doi/10.1145/3660823):
across 20,743 dependency-update commits in 1,487 npm projects, **55.88%** of
dependency-update CI build time is triggered solely by unused dependencies (median
project: 54.45% of its dependency-related build time); bots generate 89.12% of that
waste, Dependabot alone 74.52%.

**Prevalence.** Soto-Valero et al. (EMSE 2021, doi:10.1007/s10664-020-09914-8): of
723,444 Maven dependency relationships, **75.1% are bloated** (2.7% direct, 15.4%
inherited, 57% transitive); dependency counts could fall to ¼. A 2024 Concordia
thesis (spectrum.library.concordia.ca/id/eprint/993232) found unused deps in **79%**
of npm projects.

**Attack surface scales with dependency count.** Arafat (arXiv, Dec 2025,
arxiv.org/abs/2512.14739): mean transitive amplification 24.70× (Maven), 4.32×
(npm) — and **0% of Cargo projects exceed 10×**. ML-SCAR (JSAER 2024; 1.1M
packages/47K pipelines): 82% of popular projects import ≥1 CVE'd package; 29%
inherit CVSS≥9 CVEs ≥3 hops deep. Mir et al. (2023, arXiv:2301.07972; 3M Maven
packages): only 31% of vulnerable projects have vulnerable *direct* deps —
transitive bloat multiplies exposure. Dependency confusion (Birsan 2021,
CVE-2021-24105) hit 35+ organizations; package confusion has 13 mechanism
categories over 1,200+ documented attacks (USENIX Security 2023,
usenix.org/system/files/usenixsecurity23-neupane.pdf).

**Tooling trade-offs.** `cargo-machete` (2022): regex scan, rust-lang/rust in
1.08 s, CI exit codes — blind to macro/build-script usage (false positives).
`cargo-udeps`: compile-based, accurate, needs nightly, misses doc-tests, masks
transitively-used deps, per-target blind spots. `cargo-shear`: rust-analyzer
static parse, no compile, one run covers all platforms, `--deny-warnings`/JSON for
CI. `depcheck` (npm), `fawltydeps` (Python, 2023). rustc's `unused_crate_dependencies`
is allow-by-default due to multi-target false positives.

**The motivating failure mode, confirmed.** `ndk-sys` unconditionally emits
`compile_error!("android-ndk-sys only supports compiling for Android")` on every
non-Android target (rust-mobile/ndk#80, 2020; docs.rs host build logs). An
unconditional `ndk` declaration therefore breaks *all* host builds — with zero
consumer. This is exactly the T036 B2 case.

**Recommendations for the clause:**
1. **CI gate:** `cargo machete --with-metadata` fails the build on unused deps;
   record known false positives in `[workspace.metadata.cargo-machete] ignored`
   (an ignore suppressing nothing is itself debt — cargo-shear flags redundant ignores).
2. **First-use rule:** a dependency is declared in the same commit that first
   imports it; enforce with `cargo shear --deny-warnings` (static, cross-platform,
   no compile).
3. **Platform-scoped declaration:** platform-restricted deps live only under
   `[target.'cfg(...)'.dependencies]` — never unconditional; that alone would have
   prevented the B2 case.
4. **Host-green invariant:** `cargo check --workspace --all-targets` on a host
   target must stay green; a cfg-gated dep with no consumer cannot break it, an
   unconditional one will.
5. **Nightly spot-check:** schedule `cargo +nightly udeps` (or Cargo's unstable
   `lints.cargo.unused_dependencies`) periodically — not a blocking gate, given
   per-target/doc-test blind spots.

## Thread D — policy mechanics: what "universal" means and how to enforce it

**Resolver behavior (Cargo Book, `resolver.html`, 2026).** `[target]` deps resolve
"as-if all platforms are enabled" (Cargo.lock covers every target); `resolver = "2"`
skips their features for non-built targets. RFC 1361 notes ungated deps cause
"excessive downloads, excessive compilations".

**"Universal" is matrix-relative.** On `wasm32-unknown-unknown` `std::fs` always
errors and `std::thread::spawn` panics (Rust platform-support docs, 2026); gate
with `cfg(all(target_family = "wasm", target_os = "unknown"))`. "Universal" must
therefore be defined operationally: *compiles and tests on every triple in the
workspace's declared build matrix* (here: host + `aarch64-linux-android`; a wasm
target would join only if declared), verified per-target in CI (`cargo check
--target X` for each, or cargo-hack).

**Enforcement stack.** `cargo tree --workspace --target all --all-features`
(inspection), cargo-deny (bans/sources), cargo-audit (RustSec), cargo-hack
(matrices), and a versioned build-matrix declaration. Supply-chain context: Rust
Foundation 2025 Technology Report (Trusted Publishing live, TUF crate signing
underway); the August 2026 arrayref incident (malicious code executed via a
dependency's `build.rs` at compile time) — prefer no `build.rs`; build-time
execution is the highest-risk supply-chain class.

**Embedded portability precedent.** embedded-hal's portability-trait pattern
(M+N traits instead of M·N implementations; docs.rust-embedded.org/book/portability,
2024) is the architectural form of "universal default": the abstraction is the
dependency, platforms plug in behind it. Ferrocene qualifies only named targets
(Linux/QNX/bare-metal) — restricted environments need explicit qualification.

## The elaborated refinements (final form)

**Refinement 1 — "no alternative" is necessity, not advantage.** Platform-exclusive
APIs with no universal equivalent (AAudio EXCLUSIVE, MediaSession routing,
ActivityManager reflection) are a *separate* path from the advantage path: both end
in cfg-gating, but necessity is discharged by a logged exclusivity search (Thread B,
gate 1), advantage by a paired benchmark (gate 2). Because Cargo cannot validate
that a gated dep supports the target (RFC 3759 unimplemented) and cannot guarantee
gated code is ever exercised, each gated dependency carries: the narrowest cfg
predicate, a comment naming the rejected universal alternative, a merge gate
(`cargo check` + clippy per declared triple, `--check-cfg`), execution evidence
appropriate to the behavior class (QEMU user-mode for logic; on-device trials or a
device farm for platform-integration behavior — audio routing, focus, lock-screen —
before release, not per-PR), and re-verification on upgrade. The cfg matrix is a
product line (Kästner 2012): closed sets tested exhaustively, combinations sampled
and documented otherwise (Medeiros ICSE'16).

**Refinement 2 — the exception bar, operationalized.** All six gates of Thread B
must pass (exclusivity verified by logged search; ≥10× measured magnitude or
non-approximable capability via paired benchmark ≥3 reps; isolation with the
universal path green across the CI matrix; universal fallback always present;
supply-chain fail-closed; recorded and re-verified). The ≥10× figure is anchored
on the AAudio 20 ms vs 205 ms reference and is the one tunable parameter a
maintainer may adjust — but never below "measured", because overhead claims without
benchmarks are unreliable (EASE 2023: frameworks beat native in 5/10 benchmarks).

**Refinement 3 — declare at first use, and let CI enforce it.** A dependency is
declared in the commit that first imports it (first-use rule, `cargo shear
--deny-warnings`); `cargo machete --with-metadata` is a blocking CI gate with
false positives recorded in `[workspace.metadata.cargo-machete] ignored`;
platform-restricted deps are *only* ever declared under `[target.'cfg(...)'.dependencies]`;
`cargo check --workspace --all-targets` stays green on host (host-green invariant);
`cargo +nightly udeps` runs as a scheduled spot-check. The B2 case is the canonical
violation: an unconditional, consumer-less, platform-restricted declaration that
broke every host build.

## Open questions for the maintainer

1. **Landing mechanism:** constitution amendment (v1.2.0 → v1.3.0, requiring a Sync
   Impact Report header + Exceptions Log entry in the same commit per the amendment
   ceremony) vs a standalone ADR referenced by the constitution.
2. **Tunable parameters:** the ≥10× magnitude threshold (Thread B anchor: AAudio);
   the blocking-vs-scheduled split for machete (blocking) vs udeps (scheduled).
3. **Build matrix declaration:** formalize the workspace's declared target matrix
   (host + aarch64-linux-android today) as the definition of "universal".
4. **On-device trial scope:** which cfg-gated behaviors (audio routing, focus,
   lock-screen) require pre-release on-device trials vs QEMU-sufficient logic —
   001's trial suite already covers this; the policy should point at it.
