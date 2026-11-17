# I1 Harness Decision — Debug Ingest Intent Parse Contract (Spec 012, T059/T013/T020)

**Status**: decided (host-contract fallback now; JVM extraction deferred)  
**Date**: 2026-11-06  
**Scope**: FR-002, FR-003, FR-007 debug-only intent parsing/normalization/validation semantics (T013/T020 traceability); no production ingest behavior change.

## Decision

**(B) Approved host-contract fallback now** via deterministic hook contract modes that encode/document pure debug-ingest intent parsing/normalization invariants.

This is chosen because the checked repo paths do not expose a host-testable Kotlin/JVM unit-test harness for `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt` parsing logic (`android-shell/src/test/` / visible Gradle source-set wiring absent in checked paths), and `src-tauri/gen/**` is non-canonical (CHK021/FR-011). Adding/generated test wiring under `gen/` or patching generated files is out of scope.

**Not chosen now**: **(A)** extract pure debug-only parsing/normalization/validation logic to a minimal host-testable Kotlin/JVM unit in maintained sources with runnable harness wiring. (A) remains the preferred long-term fidelity path if/when a maintained Android unit-test harness is added; it must not create a parallel production ingest path and must keep debug-only semantics bounded.

## Testable plan (RED → implementation, no CHK021)

1. **RED (this change, T059)**: add `scripts/tests/spec012/test_debug_ingest_intent_parse_contract.py` with failing Hypothesis/PBT/invariant tests expecting host-deterministic hook contract modes (no adb/device side effects), e.g.:
   - `--print-intent-parse-contract [--json]` — machine-readable parsing contract covering mode aliases, precedence, and error classes.
   - `--normalize-debug-ingest-intent --mode <m> [--path <p>] [--uri <u>] [--data-uri <d>] [--json]` — deterministic normalization/validation result for a supplied intent tuple.
2. **Implementation (separate T013/T020-linked wave, same hook/script owner as contract modes)**: extend `scripts/automate-t184-t226-hook.sh` with those contract modes only. Modes MUST be host-deterministic, parse flags early, take precedence over normal device automation, and emit stable JSON/human diagnostics. No device/adb/mkdir side effects in contract modes.
3. **Traceability**: T013 (document-mode JVM/contract normalization/precedence, FR-003/FR-007) and T020 (directory-mode aliases/validation, FR-002/FR-003/FR-007) remain open trace items. Host-contract tests provide RED/gate evidence for parsing semantics until (A) is available; they do **not** claim Kotlin unit-test execution.

## Authority boundary (explicit)

- **Contract validates**: debug-only intent parsing/normalization/validation semantics as specified for the debug ingest seam — normalized mode, deterministic precedence, missing-input/error classes, unknown mode handling, and parseable diagnostic shape.
- **Production ingest remains authoritative in Kotlin/Rust/JNI**: `DebugIngestActivity.handle()` / debug hook / `DocIngest` snapshot-fingerprint + JNI callback/manifest dispatch paths are the production-authority ingest flows (FR-004/FR-005, ASM-012-INGEST-SOT). The host-contract hook modes are a **spec/test contract mirror**, not a parallel production ingest path, not a substitute for device/runtime evidence, and not canonical Android behavior implementation.
- **No weakening**: does not weaken FR-002/FR-003/FR-007, does not patch or treat `src-tauri/gen/**` as canonical (CHK021/FR-011), and does not claim FR-010/SC-004, full T184/T226 completion, or UI/a11y residual closure.

## Invariants encoded by RED tests

- **Mode normalization**: `document|file|open_document` → document; `directory|dir|tree|open_directory` → directory; `uri` → uri (case-insensitive normalization as specified by seam contract).
- **Deterministic precedence (U1/FR-003/FR-007)**: in document/directory modes where `path` is present, `path` wins before `uri`/intent `data` (`--uri`/`--data-uri` in contract mode). This mirrors observed `DebugIngestActivity.handle()` behavior (T004/U1).
- **Missing inputs**: missing `path`/`uri`/`data` for an operation that requires a resolvable reference yields an explicit diagnosable `error:*` class (never silent pass, never `ok:*`).
- **Unknown mode**: yields explicit diagnosable `error:unknown-mode:*` (or contract-equivalent stable class containing `unknown-mode`).
- **Parseability**: results are machine-readable JSON in contract mode with stable status/class fields; no device side effects.
