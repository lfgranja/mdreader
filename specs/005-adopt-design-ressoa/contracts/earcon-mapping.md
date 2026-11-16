# Contract — Earcon Mapping & Burst/Protocol Dispositions

**Feature**: `005-adopt-design-sona` | **Date**: 2026-10-24 | Extends: `specs/004-ressoa-dogfood-build/contracts/earcon.md` (the acoustic GRAMMAR authority — band 300–2500 Hz, per-earcon params; NEVER duplicated here) and `contracts/surface.md`. This contract fixes name→action mapping, channel/burst policy, the rapid-sequence sub-test, the trial-sheet schema, and the two typed diagnostic kinds this feature adds.

## E1. Mapping (closed — `data-model.md` §EarconConfirmation is the table)

play→Adagio · pause→Andante · rate-step→Unison · skip-forward→Allegro-frente · skip-back→Allegro-trás · warning→Ostinato · failure→Sforzando. Implementation may reference only these names; Crescendo/Aria and any re-pitch are out of scope. The design-dir rows (post-R1 alignment) carry the SAME mapping — a divergence between dir and this contract fails the alignment task, not the code.

## E2. Channel policy (invariants, host-testable)

1. Additive mix earcon↔narração ONLY, full gain on the duck-exempt short channel; narration is never ducked/paused/delayed/suppressed to accommodate an earcon.
2. Burst = truncate/substitute **latest wins**: an in-flight USER earcon (Adagio/Andante/Allegro-frente/Allegro-trás/Unison) is cut — tail dropped in `enqueue_earcon` BEFORE the append — so every triggered action keeps audible confirmation and FR-010 (≤100 ms, first consumed sample to the newest earcon) survives rajadas.
3. SYSTEM earcons (Ostinato/Sforzando) interrupt any in-flight user confirmation and fire immediately at the state change — never queued, dropped, or delayed behind a paused state.
4. The pause drop-and-refire rule is UNTOUCHED and applies to the user-initiated set only.
5. The single `short_mix` channel remains single (`mul_add` per slot; no second channel, no pre-mix chain).

## E3. Truncation vs suppression (the legal boundary)

Truncation is a temporal consequence registered by a **typed diagnostic counter** (an extension of the 004 record's closed kind set — coordinated edit + docs, no catch-all variant); suppression as POLICY stays prohibited: every triggered signal sounds its earcon.

## E4. Emission failure (edge case #72)

Synthesis error or short-channel unavailability: the transport action STILL proceeds and announces; narration is never interrupted by the failure; AT MOST one re-emission at the next render space; then a typed diagnostic event (`failure`, never silence-without-registration); confirmation equivalences ride the redundant channels (visual state + haptics when wired) — no new error surface toward the driver.

## E5. SC-002 rapid-sequence sub-test

2–3 heterogeneous actions triggered within 100–150 ms, per-attempt identification expected (trial-level); systematic confusion on the risk pairs (Allegro-trás×Andante; Unison×Allegro-frente) is MEASURED evidence that authorizes a 004-contract amendment (never a silent edit); single-stimulus rolling is superseded by this sub-test whenever bursts occur (D3).

## E6. Blind-test trial sheet (SC-002; human, not a unit test)

Mandatory fields: 7 types (play/pausa/salto-frente/salto-trás/taxa/aviso/falha) × ≥15 attempts randomized constant-stimulus (no two equal consecutively; no in-session feedback); 5–10% trap attempts recorded and countable; familiarization 5–10 min WITH feedback before the session; per-attempt row → confusion matrix artifact (CSV) + direction-of-skip sub-score; channel = media speaker, parked car (headphone secondary); ≥24 h replication session; **haptic-disposition line** (mandatory: rigid mount, hands off body, admin buzz-check at pilot — E3). Sheet lives in `quickstart.md` §3; the matrix artifact rides the repo/adb-pull like the other trial artifacts.

## E7. Latency

FR-010's ≤100 ms is measured action→first consumed short-mix sample (004's T152 technique, host oracle) and re-checked on-device per `quickstart.md` §3.5. The warning-path exception clause (R8) needs device data + explicit amendment before any loosening.
