# Concept: Reference-Device Verification — Emulador Structural Layer (001/003)

- **Slug**: ref-device-emulator
- **Created**: 2026-10-29
- **Recommended option**: Option A — Small structural-verification layer (`VERIF-EMULATOR`) for `003-reference-device`, with explicit boundary that physical device remains required for `gate-8-full`. Not Option B (status quo) because evidence confirms emulator viability for structural checks and the 80/20 switch to 003 requires a verified path; not Option C (full CI/attestation) because appetite exceeds benefit given empty `tests/`, unconfirmed `verify.sh`, 51 dirty files, interrupted `cargo check`, and 6 open gaps.

## Options

### Option A — Small: `VERIF-EMULATOR` structural layer (recommended, conditional)
- **Sketch**: Document and optionally script a structural verification path for `003-reference-device` using the official Android SDK Emulator (API 29+, local/free). The layer verifies: manifest `<service android:foregroundServiceType="mediaPlayback">`, `MediaSessionService` lifecycle, `MediaStyle` notification content (title + section + progress, no body/rate/voice), focus split (`MAY_DUCK` ~20% vs permanent loss + 3 s reason, per `spec.md` clarifications), rate ladder UI, diag JSONL write/read, resume by `blake3` fingerprint, service teardown at document end. Explicitly does NOT claim audio output verification (mono f32 @22050 Hz → device format via `cpal`/`AAudio`), `SC-009` ≤3 s benchmark, real telephony/alarm interruption, Bluetooth/car routing, or 10/10 on-device trials — those remain physical-only. Output: updated `003-reference-device/spec.md`, `plan.md`, `data-model.md`; optionally `specs/003-reference-device/verification/emulator.md`; `ROADMAP.md` keeps `VERIF-EMULATOR` + `gate-8-full` DEFERRED.
- **Appetite**: small (days, not weeks). No new crate; no `crates/*/src` edits; no `engine-core` contract changes; only documentation + possibly a local script (`scripts/verify-emulator.sh` or `tests/` stub) that runs `check` / manifest validation / notification inspection via AVD. Uses existing `clap` + `thiserror` + `tracing` patterns if any script is written.
- **Trade-offs**: Wins — creates a documented structural-verification path for `003` without physical-device bottleneck; enables earlier structural validation of `001` prerequisites; respects `AGENTS.md` (zero playback logic in Kotlin — emulation is structural only); uses free SDK, no supply-chain change. Sacrifices — does NOT close `gate-8-full`; requires maintaining two verification paths (emulator + physical); does not produce `tests/` for `003`; potential false confidence if boundary is not clearly documented (mitigated by explicit `VERIF-EMULATOR` label and `problem.md` definition). Risk — if 6 open gaps (emulator type, sub-vs-replace, SC-044 interaction, success criteria, CI/hardware, assessment-vs-spec-note) are not resolved, the layer may become ambiguous.
- **Rabbit holes**: (1) Defining exact AVD config to match reference device (API 29+ vs API 34 FGS restrictions); (2) Automating notification inspection / focus state read from AVD (needs `adb` + possibly UI-automation, not just framework); (3) Maintaining `verify.sh` parity when `verify.sh` gates evolve; (4) Whether `cleanroom-core` should receive verification logic (currently `0 .rs` — empty; putting verification there would require filling it, which is out of scope); (5) Confusion between `VERIF-EMULATOR` and `gate-8-full` — must be explicit.

### Option B — Small: Status Quo / Physical-Only (defensible, low-effort)
- **Sketch**: Document explicitly that `003-reference-device` requires a physical `API 29+` device connected to `adb devices`; update `spec.md`/`plan.md` with that constraint; close `ref-device-emulator` assessment with "not needed / physical required"; keep `ROADMAP.md` with `gate-8-full` DEFERRED; do not build or document any emulator layer. No new artifacts beyond the decision record.
- **Appetite**: small (days, mostly documentation + `decision.md` + possibly `spec.md` note).
- **Trade-offs**: Wins — zero maintenance, zero false-confidence risk, no AVD/CI/hardware cost, no ambiguity between structural and full verification. Sacrifices — does not provide any intermediate verification path; `003` remains blocked on physical-device access; does not accelerate `001` convergence; the user's explicit research ask remains unanswered in implementation (though answered in `research.md` with evidence).
- **Rabbit holes**: (1) If physical device is unavailable or unreliable (current `bg_14` interruption suggests instability), verification stalls; (2) `verify.sh` remains a single-point failure with no structural preview.

### Option C — Medium: Full Emulator CI + Attestation (not recommended unless gaps resolve)
- **Sketch**: Create `tests/003-reference-device/`, `scripts/verify-emulator.sh`, CI job running AVD with full gate checklist (manifest + service + notification + focus + diag + rate + resume + teardown), generate `VERIFICATION.md` for emulator, maintain attestation artifact (`V3_ATTESTATION` style). Would require filling `tests/` for 003, possibly using `proptest` (already workspace dependency) for property-based checks. Could eventually claim structural gate pass from emulator.
- **Appetite**: medium (weeks — requires AVD setup, CI integration, test writing, attestation, and maintenance of a second verification path alongside physical).
- **Trade-offs**: Wins — most complete answer to the user's ask; could produce a reusable `VERIF-EMULATOR` framework; would close `003` structural verification with attestation. Sacrifices — high effort for a layer that evidence says is only structural; risk of false equivalence with physical gate; requires resolving all 6 gaps first; conflicts with `cleanroom-core` empty state (no clean home for verification logic); might delay `005` tasks (currently 80/20 switched) and not unblock `SC-044` (002 deferred) directly.
- **Rabbit holes**: (1) AVD setup across environments (dev, CI) is non-trivial; (2) Audio testing in AVD does not verify device-format conversion; (3) Focus interruption testing requires simulating telephony/alarm — impossible in AVD; (4) If `verify-emulator.sh` diverges from `verify.sh`, maintenance burden grows; (5) `cleanroom-core` must be addressed to host verification cleanly, or verification scatters.

## Recommendation

**Option A — Small `VERIF-UMULATOR` structural layer**, executed only if the 6 open gaps from `research.md` are resolved (especially: emulator type confirmed = Android SDK AVD; success criteria = structural-only attestation document; CI/hardware budget confirmed; clarification that this is sub-layer not replacement for `gate-8-full`).

**Rationale tied to goals and metrics:**
- Goal G1 (define whether layer is feasible) — evidence already supports structural feasibility; the question is not "can it work structurally" but "should we document it and with what boundary." Option A answers this precisely.
- Goal G2 (preserve `gate-8-full` as physical-only) — Option A's boundary is explicit; Option C risks blurring this; Option B avoids the question entirely but leaves the structural gap unaddressed.
- Success metric: `003` has a documented `VERIF-EMULATOR` note with explicit exclusions, not a false claim of equivalence.
- Evidence: `research.md` shows 5 evidence-against points (no external demand, maintenance, cleanroom empty, doesn't unblock 001, physical still required); however, the structural feasibility is confirmed (docs + StackOverflow + workflow). The smallest viable response is a bounded documentation layer, not a full CI build.
- If any of the 6 gaps resolve negatively (e.g., CI budget unavailable, user confirms emulator is not useful, or `gate-8-full` must remain solely physical with no intermediate layer), fall back to **Option B** (document physical-only, close assessment with evidence).

## Out of Scope (for recommended Option A)

- Any claim that emulator verifies audio output (`cpal`/`AAudio` at 22050 Hz), `SC-009` ≤3 s, Bluetooth/car routing, real telephony/alarm interruption, or 10/10 on-device trials — physical device remains the only path for these.
- Any change to `SC-044` (consent / cloud TTS) — `002` adapter stays `offline_default`; `SC-044` stays `DEFERRED`.
- Any change to `engine-core` contract (rate ladder, `EngineError` variants, `DeclaredFormat`, `AlignmentGranularity`) — adapter delivers mono f32 @ 22050 Hz; engine contracts unchanged.
- Any new crate (do not fill `cleanroom-core` for this layer; do not create `crates/verif-emulator`); verification artifacts live in `specs/003-reference-device/verification/` or as docs.
- Any change to `Tauri` version (`2.12.1` pinned) or manifest-edit mechanism (direct XML edit).
- Any source changes to `player`, `session`, `diag`, `adapter-synthesis`, `engine-core`, or `src-tauri` beyond what is needed for structural verification (and likely none).
- Any claim of `verify.sh` PASS at `030` (current interrupted `bg_14`); structural layer does not require `verify.sh` to pass, only that `verify.sh` gates remain physical-only.
- Any external user-facing feature (no UI, no consumer-visible change; this is a verification/quality-layer addition, not a product feature).

## Assumptions to Validate (before `/speckit.assess.decide` or `/speckit.specify`)

- **Assumption 1**: The user (or verification maintainer) has access to an Android SDK AVD (API 29+) on the development/CI environment, and can run it without prohibitive memory/GPU cost. If false → Option A fails; fall back to B.
- **Assumption 2**: The 6 open gaps from `research.md` are resolvable with a brief clarification (not a multi-week study). If false → Option A's small appetite is underestimated; must either reduce scope further or choose B.
- **Assumption 3**: The `003-reference-device` team accepts a structural-only verification layer that does not replace `gate-8-full`; no false-equivalence claim is expected from attestation. If false → Option A risks misrepresentation; must document more aggressively or choose B.
- **Assumption 4**: `cleanroom-core` does not need to be filled to host verification logic; verification artifacts can live in `specs/003/` or `scripts/` directly. If false → Option A requires `cleanroom-core` fill (adds appetite, not small).
- **Assumption 5**: The `005-adopt-design-sona` branch (`0c985ee`, 51 dirty, interrupted `cargo check`) can absorb a small documentation/script addition without destabilizing the active feature; no conflict between `VERIF-EMULATOR` artifacts and `DESIGN-SONA` token integration. If false → delay until 005 converges.
