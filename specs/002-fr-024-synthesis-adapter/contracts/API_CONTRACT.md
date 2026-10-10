# Contract: API Contract — Adapter Declaration-Consumption (`FR-024c` / T068)

**Branch**: `002-fr-024-synthesis-adapter` | **Status**: Bound (Phase 10 / T068 completed)
**Source of truth**: `contracts/adapter-capabilities.md` (§A `:154-157` — `spec.md:144` vs `FR-024c` conflict recorded); `spec.md:97` (`declared_capabilities()`); `constitution.md:299-312` (`T016` mandate — trait not yet built, declaration unconditional against existing shapes only); `plan.md` (adapter-only scope, Option A, no cloud / second engine / SSML).

---

## 1. Purpose

Bind the adapter-to-player declaration-consumption interface (`FR-024c`) to the existing `DeclaredFormat` + `EngineError` contract shapes (`format.rs`, `closed.rs`, `engine-core/src/closed.rs`) **without inventing phantom `trait_` methods** (`format()`, `fail()` — neither exists; `grep '\btrait\b'` = 0; `grep 'declared_capabilities'` = 0 in workspace). The adapter declares locally (`AdapterCapabilities`, `SsmlLevel`, `AlignmentGranularity`, `SsmlDelivery`, `ResumeAnchor`) and the player branches explicitly on those enum variants (not string comparison; closes set at compile time — Principle VII).

---

## 7. Synthesis Engine Binding (§7 / `API_CONTRACT.md:390-410` — updated to built behaviour)

- **Pinned dependency**: `sherpa-onnx` `=1.13.8` (`constitution.md:412-413`; registry pin; latest stable; `--locked`; `Cargo.lock` committed). Exception 4 (`ort` 2.0.0-rc.13) is **not** invoked — `sherpa-onnx-sys` build script fetches the prebuilt static archive bundling ONNX Runtime + espeak-ng + piper_phonemize; one bundled runtime, no second RC dependency, preserving exception 4's intent with lower risk.
- **Voice data**: Piper pt-BR `faber` (CC0) embedded at `crates/adapter-synthesis/assets/voices/pt_BR-faber-medium/`; blake3 provenance in `PROVENANCE.md`; load once per process (`OnceLock`); failed load cached and re-reported with current segment's echoed position (`SC-009` stage 3 / mechanism-preservation verified on host).
- **Output contract**: mono `f32` @ **22050 Hz** (engine delivers at declared rate; device format conversion is `player`'s responsibility, never adapter's silent resample/downmix); rate/channel/width mismatch = named `EngineError` (`RateMismatch`, `ChannelMismatch` — closed 7-member set, zero `_ =>`).
- **Failure surface (named, not silent)**: every failure path names a member of `EngineError` via `failure.rs:125-138` (`InitFailed` for voice/init corruption / load failure; `Unreachable` for process death / OOM / kill / interior-NUL; `RateMismatch` for delivered-rate deviation; `DeviceRefused` / `DeviceLost` / `StartRefused` for device-side outcomes). No `AdapterFailed`; `AdapterFailed` explicitly excluded per `failure.rs:12` (`//! no AdapterFailed`).

---

## 10. Adapter Declaration-Consumption Amendment (`:451` — amended, not suppressive)

**Prior state (`API_CONTRACT.md` before this edit)**: sections 7 and 10 deferred this binding to `003-contract-edit`; the deferral was **not** granted by `spec.md` (only sentence-level resume, `SC-044`, consent deferred); the contract contradicted `FR-024c` by omitting the obligation.

**Amendment (this commit)**:

- `:451` — **amended to bind**, not suppressed. The adapter's declaration-consumption interface (`declared_capabilities()` → `plan_for()`) is now required and implemented (`crates/player/src/consume.rs`; `adapter-synthesis/src/consume.rs`; `adapter-synthesis/src/capabilities.rs`).
- The declaration is unconditional (`constitution.md:304-305`; `FR-024c` normative); the adapter does not wait for `T016` trait design (`trait_.rs` still absent — `constitution.md:299-312` mandate preserved, not violated, because declaration is adapter-local against existing shapes).
- The player branches explicitly on `AdapterCapabilities::OfflineDefault` (sole selectable per `SC-051`; `test_double` inadmissible — no reference in `player/src/`); for `(SsmlLevel::None, AlignmentGranularity::None)` the consumption yields `(SsmlDelivery::PlainText, ResumeAnchor::ParagraphHeading)` (identity mapping verified in `consume.rs`; total over 3×3 pairs — all enumerated arms, zero catch-all).
- `spec.md:144` (adapter-only scope excluding `player` transport/position/bookmark logic) governs adapter-side work only; `FR-024c` governs the declaration-consumption interface specifically; the conflict is resolved per `contracts/adapter-capabilities.md` §A (`FR-024c` governs the branch; `spec.md:144` excludes transport/position — not the consumption interface).
- `SC-009` benchmark remains a **named blocker** (stage 2 blocked; reference device required per `001/plan.md:50-53`; never silent pass / silent deferral — `SC-009-GATE-RECORD.md:42-131`); this contract does not alter that verdict.
- `SC-051` (`test_double` non-selectable) preserved: adapter declares only `OfflineDefault`; player selects only `OfflineDefault`; no second selectable engine registered (`constitution.md:524` — `offline_default` boundary events = none, SSML = none).

---

## 2. Closed-Set Enforcement (Principle VI / VII)

- `EngineError` variants (7 members): `InitFailed`, `Unreachable`, `RateMismatch`, `ChannelMismatch`, `DeviceRefused`, `DeviceLost`, `StartRefused`. Zero `_ =>` in `crates/adapter-synthesis/src/` and `crates/player/src/consume.rs` (only pre-existing `player/src/focus.rs` / `position.rs` — not expanded, not introduced by adapter).
- Suppression policy: only registered `#[expect(lint, reason="...")]` allowed (`failure.rs:120`, `reg.rs:55`, `format_check.rs:130`, plus adapter additions `audio_chunk.rs:239`, `synth.rs:246`, `engine-core/src/closed.rs:70`). No new suppression weakens the gate.

---

## 3. References

- `contracts/adapter-capabilities.md` (§A — conflict record `:154-157`)
- `spec.md:97` (`declared_capabilities()` interface); `spec.md:89` (`SC-051` / `test_double` exclusion)
- `constitution.md:299-312` (`T016` mandate); `:304-305` (declaration unconditional); `:412-413` (dependency pin); `:524` (registry — `offline_default` = none SSML / none boundary)
- `plan.md:23,80,84` (dependencies / embedded voice / delivery); `crates/player/src/consume.rs` (wired branch, verified)
- `SC-009-GATE-RECORD.md` (`:42-131`; stage 2 BLOCKED — named blocker preserved)
