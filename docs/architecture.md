# Architecture

Grounded in the `001-android-media-shell` feature branch: workspace
`Cargo.toml`, the crates under `crates/`, `src-tauri/`, and
`.specify/memory/constitution.md` (v1.5.0). Every requirement token
(FR-### / SC-###) refers to `specs/001-android-media-shell/spec.md`.

## Shape: a Cargo workspace of focused crates

The application is a modular monolith (constitution, Tauri Platform
Contract; Principle I). The workspace has seven members:

| Crate | Binary | Owns |
|---|---|---|
| `crates/engine-core` | `engine-core` | The closed vocabulary (`closed.rs`), the `Engine` trait (`trait_.rs`), the fallback selector (`fallback.rs`), the declared handoff shape (`format.rs`), and the two stand-in engines (`standin_a.rs`, `standin_b.rs`) |
| `crates/player` | `player` | The playback session: the closed verb set and transition table (`cli.rs`), the position clock (`position.rs`), rate ladder (`rate.rs`), skip/rewind math (`skip.rs`, `rewind.rs`), the engine-failure recovery machine (`recovery.rs`), focus classification (`focus.rs`), output-device routing (`routing.rs`), and the notification-surface resolver (`surface.rs`) |
| `crates/session` | `session` | Bookmarks and settings in `redb` (`store.rs`), the `blake3` content fingerprint (`fingerprint.rs`), and the resume-offer machine (`offer.rs`, `lib.rs`) |
| `crates/diag` | `diag` | The append-only JSONL diagnostic-record writer (`writer.rs`) and the conformance scanner (`scan.rs`) |
| `crates/doc-structure` | `doc-structure` | The ordered heading list and the FR-038 re-anchor resolver (`lib.rs`) |
| `crates/adapter-synthesis` | `adapter-synthesis` | The FR-024 synthesis adapter registered in the `offline_default` tier only (`reg.rs`): document segmentation (`chunk.rs`), synthesis with the embedded Piper pt-BR `faber` voice (`synth.rs`), format/delivery checks (`format_check.rs`), and the failure taxonomy re-export (`failure.rs`) |
| `src-tauri` | (Tauri app) | One consumer of the crates above: the IPC bridge (`src/lib.rs`), the permission-state gate (`src/permissions.rs`), the shared reason shapes (`src/surface.rs`), the engine-failure surface (`src/failure.rs`), and the minimal status view (`src/status_view.rs`) |

Every crate is a library plus a thin `main.rs`; the Tauri app is one
consumer among several, never the only one (Principle I).

## The authority split (FR-002 / FR-003 / FR-042 / SC-035)

Rust owns synthesis **and** PCM playback. `crates/player` is the sole
authority for transport state, position, rate, buffering, pre-roll, and
transport decisions; it advances position only on audible progress
(`PlaybackSession::on_audible_progress` ignores progress while not
`Playing`, FR-005). The Kotlin/Tauri surface contains no playback logic:
it owns the `Service`, the `MediaSession` (a **mirror** — it renders the
state Rust publishes, it never computes position), the notification, and
the `AudioFocusRequest`. The bridge is structural, not incidental: the
Kotlin `PlaybackService` self-bootstraps from the Application/Activity
and reaches Rust through `register_android_plugin` + `run_mobile_plugin`
(`src-tauri/src/lib.rs`); the Android manifest is edited directly (the
`mobile::update_manifest` API does not exist in Tauri 2 — see the
constitution's Tauri Platform Contract).

A second independent source of truth for state is prohibited (FR-003): the
state mirror the shell consumes (`player::surface::MirrorState`) carries
exactly `transport`, `position_secs`, and `progress`, all read from the
session's authoritative snapshot.

## Principle II: one code path, CLI-drivable

Every crate resolves through the same functions whether the caller is a
notification, the lock screen, a CLI, or IPC (`crates/*/src/cli.rs`).
Arguments and stdin in; the success payload on stdout; the failure
diagnostic on stderr — never mixed. `--format json` is the default for
scripting, with `--format human` selectable alongside it (a closed
format set in `diag`; the `player` CLI parses the flag ahead of the
subcommand).

Verified invocations (workspace `target/debug`):

```sh
# player — mirror of the IPC verbs
player play                 # → {"transport":"Playing","position_secs":0,"rate":R1_0,"engine_id":"rust-player"}
player position             # → {"transport":"Paused","position_secs":0}
player rate-up              # rate moves one rung on the closed ladder

# engine-core — trait operations as contract checks
engine-core capabilities    # → the closed ssml_levels / alignment_granularities vocabulary
engine-core set-rate --rate 1.5
engine-core stream --at 12.5 --budget-secs 5.0   # or: echo 12.5 | engine-core stream --budget-secs 5.0

# session — bookmarks, settings, resume offers (the redb path is required: --db <path>)
session --db s.db get doc.txt
session --db s.db put doc.txt --offset-secs 12.5 --first-yield-ms 100 --end-kind paused
session --db s.db rate get
session --db s.db offer show doc.txt
session --db s.db foreground on|off

# diag — record conformance
diag scan record.jsonl

# doc-structure — headings and the FR-038 re-anchor point
doc-structure headings headings.json        # headings.json = [{"position_secs":0.0,"title":"Intro"}, …]
doc-structure reanchor headings.json --at 75.0

# adapter-synthesis — the offline_default adapter's capabilities
adapter-synthesis capabilities
adapter-synthesis format-check --rate-hz 22050 --channels 1 --width-bits 32
adapter-synthesis chunk --input doc.md      # segment metadata only, never segment content (FR-045)
```

Rejected inputs are named errors with non-zero exit status, never silent
clamps — e.g. `session --db s.db rate set 3.0` fails with `rate must be
one of 0.75 | 1.0 | 1.25 | 1.5 | 1.75 | 2.0 | 4.0, got 3`, and
`put … --end-kind stopped` fails with `the terminal session end stopped
deletes the stored entry instead of storing one (FR-032 / FR-032b)`.

## Closed sets (Principle VII / FR-044)

The governed closed sets live in **one** declaration,
`crates/engine-core/src/closed.rs`; every consumer matches exhaustively
(no `_ =>` arm in `crates/*/src` — constitution gate 7). Extending a set
is a coordinated edit across every file that enumerates it, with a
semantic-version bump and a documentation update. The sets:

- `PlaybackRate` — seven rungs `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]`,
  wire form the JSON number (FR-019).
- `TransportState` — `playing | paused | stopped | completed | interrupted`
  (FR-020).
- `FocusOutcome` — `duckable_transient | silence_demanding | permanent_loss`;
  `regained` is a separate direction event, not a fourth outcome.
- `EngineError` — `engine_unreachable | engine_init_failed | rate_mismatch |
  channel_mismatch | device_refused | device_lost | start_refused` (FR-023/
  FR-035).
- `SessionEnd` — `paused | stopped | completed | interrupted` — one closed
  set of four values shared with the diagnostic record; `playing` is a
  transport state and never a session-end reason (FR-044).
- `EngineTier` — `offline_default | cloud_default | cloud_escalation |
  test_double | evaluated_fallback`, with the single authoritative
  ascending fallback sequence `EngineTier::ORDER` (FR-023c).
- The closed verb set every surface maps to — `player::cli::ShellVerb`
  (fifteen members; see `docs/ipc.md`).

## Engine handoff shape (FR-033 / FR-035 / FR-043 / SC-034)

The engine delivers **mono `f32` PCM at exactly 22 050 Hz**
(`engine-core/src/format.rs`: `SAMPLE_RATE = 22050`, `CHANNELS = 1`,
`SAMPLE_WIDTH_BITS = 32`). The numeric value is fixed by the
specification, not by planning. Rate is a pitch-preserving time-stretch
**inside the engine**; a rate change never alters the delivered sample
rate. Any deviation (rate, channel count, sample width) surfaces as a
named `EngineError` (`rate_mismatch` / `channel_mismatch`) — never a
silent resample or downmix. Pre-roll is bounded at 60 s outstanding
(`MAX_PREROLL_SECS`); position never advances on buffered-only audio.

## The TTS Engine Registry (constitution, "TTS Engine Registry")

The admissible engine set is closed; adding or removing an engine is a
constitution amendment, not a dependency-resolution decision. Every tier
except `test_double` refers to a registry engine by name and version:

| Engine | Tier | Admissible |
|---|---|---|
| `sherpa-onnx` + Piper pt-BR (`faber`, `cadu`, `jeff`) | `offline_default` | Yes — Principle IX (offline-first) |
| Google Cloud TTS WaveNet | `cloud_default` | Yes — Principle X (consent-gated, revocable, never a silent default) |
| Google Cloud TTS Neural2 / Chirp 3 HD / Studio; Amazon Polly Neural; Azure Speech Neural | `cloud_escalation` | Yes, ranked by free quota |
| Microsoft Edge read-aloud (via `msedge-tts` 0.4.0) | `evaluated_fallback` | Yes, under the Principle XI exception — last, never a default, consent-gated, speaks the endpoint directly (never proxied through the OpenAI-compatible Python wrapper, which discards the boundary events) |

`test_double` is the sole registry exception (2026-10-05 clarification):
it is restricted to the `standin_a` / `standin_b` fixtures used for the
SC-051 failover drills and is admissible only while no non-`test_double`
adapter is registered (see `docs/fallback-order.md`). At v1 the only
registered adapter is `adapter-synthesis` at `OfflineDefault`
(`crates/adapter-synthesis/src/reg.rs`: `REGISTERED_TIERS =
[EngineTier::OfflineDefault]`), so only `offline_default` is selectable
(`EngineTier::is_selectable`).

## Verification gates (constitution, "Verification Gates")

`scripts/verify.sh` is the single entrypoint; exit status is zero if and
only if every gate passes:

| # | Gate | Command |
|---|---|---|
| 1 | format | `cargo fmt --all -- --check` |
| 2 | tests | `cargo nextest run --workspace` |
| 3 | documentation | `cargo doc --workspace --no-deps` |
| 4 | lint-drift | root deny-tables intact; `[lints] workspace = true` in every member |
| 5 | supply-chain | `cargo deny check advisories licenses bans sources` + `cargo audit` |
| 6 | deny-list | no denied crate in any manifest or in `Cargo.lock` |
| 7 | closed-set | no `_ =>` in `crates/*/src/` |
| 8 | pipeline | byte-budget test under 5,000 bytes/request; SSML capability declared per adapter; sentence-level alignment exercised |
| 9 | android-build | `cargo tauri android build --target aarch64` |

Gate 9 exists because gates 1–8 are satisfiable on a host that cannot
produce an Android artifact. For this feature, gate 8 is split
(constitution Exceptions Log entry 8): `verify.sh` reports
`gate-8-shell: PASS` (handoff shape + granularity declared + 22 050 Hz
request) and `gate-8-full: DEFERRED (owner=synthesis)` until the synthesis
feature discharges the byte-budget and sentence-alignment clauses with
SC-044.

## Rust rigour (constitution, "Rust Code Rigour")

Every workspace member declares `[lints] workspace = true`. The deny
tables reject `unsafe`, `missing_docs`, `missing_debug_implementations`,
`unreachable_pub`, and `unused_results`; `clippy::all` and `clippy::pedantic`
are denied, `nursery` and `cargo` warned. `unwrap` / `expect` / `panic` /
`todo` / `dbg` / `println` are denied — errors flow through `thiserror`
types and `tracing`. The only suppression is `#[expect(lint, reason =
"...")]`; `Cargo.toml` and `clippy.toml` are never weakened to silence a
lint. Supply chain: latest stable only (`ort` 2.0.0-rc.13 is the sole
pre-recorded exception, Exceptions Log entry 4), `--locked` with a committed
`Cargo.lock`, crates.io-only sources, no secrets in the repo or binary.
