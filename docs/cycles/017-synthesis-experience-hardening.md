# Cycle 017 — Synthesis Experience Hardening & Convergence

- **Date**: 2026-11-10
- **Type**: 80/20 cycle (Pareto: close the implemented synthesis batch, defer
  the un-executable one, prove the tree with the deterministic gates)
- **Governance**: `.specify/memory/constitution.md` v2.0.0; dark-factory SDD
  flow (`constitution → intake → research → define → shape → decide →
  specify → clarify → plan → checklist → analyze → tasks → implement →
  converge → validate → release`); autonomy contract (decide by default,
  escalate only for explicit authorization or incapacity).
- **Research**: `findings/2026-11-10-next-cycle-research.md` (multilingual,
  primary-source, 2026-11-10).

## 1. The 80/20 decision

The project is ~94% delivered; the remaining work is mostly device-gated.
The highest-impact autonomous slice is to **close the synthesis experience
batch that is already implemented but unconverged**, and to **defer the one
feature that cannot be executed autonomously and carries legal exposure**.

| Decision | Rationale |
|---|---|
| Converge 014 (symbol normalization) | Implemented; research confirmed the approach and found 9 gaps; closed them. |
| Converge 015 (semantic prosody) | Implemented; conservative speed map is the only per-segment lever; converged with invariants. |
| **Defer 016 (voz Lula)** | No model; LGPD/STJ/TSE/Google Play exposure; constitution pins one voice; low 80/20. |
| Document 013 (TTFA) | Worker implemented; `SC-TTFA-001` remains device-blocked; no fabricated proof. |
| Keep copyleft policy | Constitution already admits GPL-3.0 (`espeak-ng` linked by `sherpa-onnx`); research's "release-blocking" framing is a known, accepted tradeoff, not a new blocker. |

## 2. Work executed

### 2.1 Spec 014 — `normalize.rs` hardened

Closed the research gaps (links/images, blockquotes, strikethrough, raw HTML,
intraword `a*b`, pipe-table rows, dead code, collision-resistant
placeholders). Added `crates/adapter-synthesis/tests/normalize_invariants.rs`
(segment metadata invariance, determinism, protected zones, non-blanking).

### 2.2 Spec 015 — prosody converged

`prosody.rs` + `GenerationConfig.speed` wiring; conservative map
(`Code` 0.85, `Heading`/`Strong` 0.90, `Emphasis`/`Quote`/`List` 0.95,
others 1.00). Added `crates/adapter-synthesis/tests/prosody_invariants.rs`.

### 2.3 Spec 016 — deferred

`specs/016-voz-lula/decision.md` records the `defer` verdict and the
conditions for any future pursuit (consent, licence, legal review,
constitution amendment, synthetic-content disclosure). Placeholder asset tree
removed (it broke `exactly_one_bundled_voice_directory`).

### 2.4 Contract-test reconciliation (013 rename fallout)

The previous 013 work renamed production functions and moved the
first-chunk enqueue into the demand-driven producer, which silently broke
four source-scanning contract tests. Each was reconciled to the new
architecture rather than weakened:

- `test_play_to_audio_measured` — `STREAM_START` now tracks
  `start_backend_for_generation`.
- `test_producer_back_pressure` — the handover scan now tracks
  `enqueue_chunk_for_generation`.
- `test_session_live_from_first_chunk` — `ENQUEUE` tracks the renamed
  enqueue, and the liveness analysis now follows the synthesis helper's
  callers (the obligations sit in `play_document` /
  `prepare_session_document`).
- `play_document` was refactored so the single synthesis call site lives
  in `stream_synthesis_and_land`, and the playing landing + surface publish
  run BEFORE the first chunk (T143 / FR-002), restoring the liveness
  contract the 013 refactor had inverted.

### 2.5 In-flight reconciliation

- `tauri.conf.json` `beforeBuildCommand` made portable (relative `cwd`).
- `shell_jni.rs` `eprintln!` → `tracing::error!` (lint `print_stderr`).
- `status_view.html` / `dist/index.html`: `btnPickDirectory` null guard (the
  element does not exist; the unguarded listener threw).
- `MirrorState::is_false` clippy `trivially_copy_pass_by_ref` expectation;
  moved above the test module (`items_after_test_module`).
- `src-tauri` test code clippy cleanup (dead test fn removed, `expect` →
  `?`/`into_inner`, `significant_drop_tightening`, `cast_possible_truncation`).
- `.cargo/config.toml` restored to `clang` (the canonical linker; verified
  present and building).
- Test census updated (`scripts/expected-tests.txt`, 128 → 132 targets).

## 3. Gate evidence

To be filled by the authoritative run (see `converge` records):

- Gate 1 format / Gate 2 nextest + census / Gate 3 docs / Gate 4 clippy /
  Gate 5 supply chain / Gate 6 deny-list / Gate 7 closed-set /
  Gate 8 pipeline — host-checkable via `./scripts/verify.sh`.
- Gate 9 (Android build) is device-checkable and unchanged by this cycle.

## 4. Follow-ups (not in this cycle)

1. **Device-gate 013** (`SC-TTFA-001`): controlled `play_to_audio_ms`
   measurement on the reference device.
2. **Spec 010 T226**: full device verification of directory reading.
3. **Android 16 accessibility**: no action needed — the codebase uses the
   WebView `aria-live` region (`src-tauri/src/status_view.html`), not the
   deprecated `announceForAccessibility` (verified 2026-11-10).
4. **Honest highlighting copy**: word-level read-along is impossible with
   sherpa-onnx (no word timestamps); target segment/sentence highlighting.
5. **Competitive copy audit**: do not claim unverified competitor gaps
   (e.g. Speechify Markdown support).
6. ** sherpa-onnx 2.x evaluation**: only if the copyleft posture is amended;
   not a current blocker.
