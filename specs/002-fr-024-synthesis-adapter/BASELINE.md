# Baseline — host gates BEFORE FR-024 implementation (evidence for V1/V2)

**Captured**: 2026-10-08 | **Commit**: `4993f5c` (API contract) | **Scope of this file**: attribute every gate failure to either FR-024 or to a defect that already exists at `HEAD`.

Rule 5 of the implementation protocol: **only the verification tasks (V1–V3) attest success**. This file records the starting state so that a verification failure can be attributed honestly instead of being mistaken for a regression introduced by this feature.

## Environment constraints (not fixable from inside the repo)

| Constraint | Evidence | Consequence |
|---|---|---|
| `libasound2-dev` (`alsa.pc`) absent, no passwordless `sudo` | `pkg-config --exists alsa` → false; `sudo -n true` → fails; `alsa-sys-0.4.0/build.rs:13` panics with `Package 'alsa' was not found` | `crates/player` **cannot be compiled on this host** (`cpal 0.18.2` → `alsa-sys`). Any `--workspace` gate that builds `player` (including `cargo nextest run --workspace` and `cargo clippy --workspace`) is unrunnable here. Pre-recorded in `scripts/verify.sh:4-6` as expected; gate 9 (`cargo tauri android build --target aarch64`) is the authoritative artifact per constitution gate 8 / `scripts/verify.sh`. |
| `deny.toml` uses the **cargo-deny v1** schema; installed binary is **cargo-deny 0.20.2 (v2)** | `cargo deny check bans` → `error[unexpected-keys] … expected: […]` for `[advisories] deny`, and `[sources] require-registry-index` | `cargo deny check advisories licenses bans sources` (constitution gate 5, `constitution.md:612`) **cannot run at all**. Governance config, outside FR-024 scope; migrating it is a constitution-adjacent change that needs its own decision. |

## Pre-existing compile failures at `HEAD` (present before any FR-024 test/core edit)

Verified with `git status --short`: neither `crates/session/**` nor `crates/diag/**` nor `deny.toml` is modified in the working tree, so these failures pre-date this feature.

### 1. `crates/session` — 10 errors, lib + lib-test

```
error: missing documentation for a variant      (×8)  crates/session/src/lib.rs:34,35,42,43,44,45 …
error: missing documentation for …               (×1)
error: unused result of type `std::option::Option<AccessGuard<'_, (f64, u64, u32, u8)>>`
                                                    crates/session/src/lib.rs:54:9  `table.remove(fp)?;`
```
**Note (2026-10-09, C1-A):** historical snapshot — the error text above is preserved byte-for-byte as the compiler emitted it at commit `4993f5c`; the bookmark tuple is now `(f64, u64, u8)` because the C1-A v1 ruling removed the `sentence_idx` member (`u32`) from the bookmark value (coordinated edit recorded in the note below), so the snapshot's `(f64, u64, u32, u8)` is stale, not current state.

Workspace denies `missing_docs` and `unused_results` (`Cargo.toml:16-21`). `crates/session` is the `redb` bookmark store — FR-024 **must not modify it** (`spec.md` Assumption 5; API_CONTRACT §10).

**Note (2026-10-09, C1-A):** the v1 ruling of clarification C1-A (`CLARIFICATION_BATCH_RESOLVED.md:13` — resume snaps to the nearest completed paragraph/heading boundary at or before the yield point; sentence-level resume is deferred to v2) required a one-time coordinated edit (closed as T050 in `tasks.md`, 2026-10-09) to `crates/session` (`sentence_idx` removed from the bookmark value; `lib.rs:9` now `TableDefinition<&[u8; 32], (f64, u64, u8)>`) together with 001's `data-model.md`, `contracts/storage-schema.md` and `tasks.md`; the constraint above otherwise stands. A Sync Impact Report under the constitution's Amendment Procedure (`constitution.md` §Governance) is owed for this ruling but has not been written — the orchestrator must decide that separately.

### 2. `crates/diag` — 2 test targets fail to compile

```
error[E0432]: unresolved import `diag::writer`: could not find `writer` in `diag`
  crates/diag/tests/contract_schema.rs:68:11
  crates/diag/tests/contract_schema_negative.rs:40:11
```
A `diag::writer` module the tests import does not exist. FR-024 **must not modify** `crates/diag` — and must not add a `diag` `kind` at v1 (`R8`; `spec.md:26`).

### 3. Workspace `clippy::cargo` metadata warnings, every crate

```
warning: package `player` is missing `package.description` metadata
warning: package `player` is missing `either package.license or package.license_file` metadata
warning: package `player` is missing `package.repository` metadata
warning: package `player` is missing `package.readme` metadata
warning: package `player` is missing `package.keywords` metadata
warning: package `player` is missing `package.categories` metadata
   …identical set for `session`, `engine-core`, `doc-structure`, `diag`
```
`clippy::cargo` is `warn` at the workspace level (`Cargo.toml:27`), and V2 runs `cargo clippy -- -D warnings`, which promotes these to errors. `crates/adapter-synthesis/Cargo.toml` was given the missing fields in commit `6c95eee`; the other four crates are out of FR-024 scope.

## Reachable verification scope

| Gate | Workspace-wide | Reachable scope in this environment |
|---|---|---|
| V1 `cargo nextest run` | blocked by `player` (alsa) and by `session`/`diag` compile errors | `cargo nextest run -p adapter-synthesis -p engine-core -p doc-structure` |
| V2 `cargo fmt --all -- --check` | reachable | workspace-wide |
| V2 `cargo clippy --workspace -- -D warnings` | blocked by `player` (alsa) + the `clippy::cargo` metadata warnings in four foreign crates | `cargo clippy -p adapter-synthesis --all-targets -- -D warnings`, plus `-p engine-core -p doc-structure` for the contract surface this crate depends on |
| V2 `cargo deny check …` | blocked (v1/v2 schema mismatch, above) | not reachable |
| V3 checklist review | reachable | workspace-wide, static |

## Reporting rule adopted for this feature

A V1/V2/V3 outcome is reported as **two numbers**: the result over the reachable scope, and an explicit itemised list of the workspace-scope failures above with their commit-level provenance. No pre-existing defect is silently fixed, and none is silently absorbed into FR-024's result.

## Sources

- `crates/player/Cargo.toml:7` (`cpal = "0.18.2"`), `scripts/verify.sh:4-6`
- `deny.toml:4` (`[advisories] deny`), `deny.toml:44` (`require-registry-index`)
- `.specify/memory/constitution.md:612` (gate 5 supply-chain), `:603` (`verify.sh` is the single entrypoint), `:614` (gate 7 closed-set), `:615` (gate 8 byte-budget)
- `specs/002-fr-024-synthesis-adapter/API_CONTRACT.md` §10 (out of scope), §0 (global rules)