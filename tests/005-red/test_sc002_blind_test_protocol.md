# SC-002 Blind-Test Protocol — RED scaffold (NOT EXECUTED)

Status: RED / DEFERRED — protocol scaffold defined; execution deferred (device unavailable).
Owner: device-build. Feature: `005-adopt-design-sona` (Option-A subset). Section ref: `quickstart.md` §3.3; contract ref: `contracts/earcon-mapping.md` E3 / E5 / E6.
File: `tests/005-red/test_sc002_blind_test_protocol.md`
Created: 2026-10-27 (Rem-T203).

## RED assertion (TDD first, before any execution)

> SC-002 protocol is DEFINED (structure, 7 types, trial counts, trap %, CSV format, risky pairs, replication, E3 line, pilot buzz-check mandatory). Execution is DEFERRED — device `Redmi 2201117TY` (API 33) unavailable (per T030 / tasks.md T215 / T203); NO CSV artifact has been produced; NO trial has been executed; no source edit to `crates/player` / `MediaMirror.kt` required for protocol-only task.

No fabricated CSV, no fabricated per-attempt scores, no fabricated replication session.

---

## Source requirements (from quickstart.md §3.3 + contracts/earcon-mapping.md E3/E5/E6)

- 7 trial types, randomized constant stimulus (no two equal consecutively).
- ≥15 trials per type → ≥105 total trials.
- 5–10% trap attempts (counted, recorded).
- Familiarization 5–10 min WITH feedback before session.
- Rapid-sequence sub-test: 2–3 heterogeneous actions within 100–150 ms, per-attempt identification.
- Per-attempt row → confusion-matrix CSV artifact + direction-of-skip sub-score (per E6).
- Channel = media speaker (parked car); headphone secondary allowed.
- Replication session: ≥24 h later on same device/channel; re-score; compare.
- Risky pairs (systematic confusion = measured amendment evidence, never silent patch):
  - Allegro-trás × Andante (skip-back confused with pause)
  - Unison × Allegro-frente (rate-step confused with skip-forward)
- Haptic-disposition line (E3 mandatory): rigid mount, hands off body, admin buzz-check at pilot. Recorded per trial.
- Pilot buzz-check mandatory (second person ONLY if no rigid mount improvised — admin can self-perform otherwise; per quickstart §3).
- Confusion matrix artifact format (CSV): rows = predicted, columns = actual, cell = count; direction-of-skip sub-score per cell; header names = 7 type names (Adagio / Andante / Allegro-frente / Allegro-trás / Unison / Ostinato / Sforzando, matching E1 mapping).

---

## Trial types (7 — matches E1 / E6 / quickstart §3.3)

| # | Protocol label | Earcon (E1) | Action | Trials / type (min) | Trap % range | Notes |
|---|---|---|---|---|---|---|
| 1 | Play(Adagio) | Adagio | play → playback starts | ≥15 | 5–10% | Primary stimulus; no feedback in session |
| 2 | Pause(Andante) | Andante | pause → playback pauses | ≥15 | 5–10% | Risk: confused with Allegro-trás |
| 3 | SkipFwd(Allegro-frente) | Allegro-frente | skip-forward | ≥15 | 5–10% | Risk: confused with Unison |
| 4 | SkipBack(Allegro-trás) | Allegro-trás | skip-back | ≥15 | 5–10% | Risk: confused with Andante |
| 5 | Rate(Unison) | Unison | rate-step | ≥15 | 5–10% | Risk: confused with Allegro-frente |
| 6 | Warning(Ostinato) | Ostinato | warning state | ≥15 | 5–10% | System earcon (interrupt per E2) |
| 7 | Failure(Sforzando) | Sforzando | failure state | ≥15 | 5–10% | System earcon; static red banner; no pulse on failure (T016) |

Total minimum: 7 × 15 = 105 trials. Target session: ~105–120 with 5–10% traps = 5–12 traps total (counted separately from the 105).

---

## Trap attempts (5–10% of session)

- Definition: stimulus presented that intentionally does NOT match the expected earcon (e.g., action triggers but wrong channel / truncated emission / suppression attempt — all record the typed diagnostic per E3/E4, never silence-without-registration).
- Counted per trial; reported in CSV column `trap` (0/1) and in the trial-sheet count.
- Must NOT exceed 10% (would invalidate constant-stimulus assumption) and MUST reach at least 5% (otherwise trap evidence is absent). Session target: ~7–10 trap attempts across 105 trials ≈ 7–9%.

---

## Confusion-matrix artifact (CSV — TEMPLATE ONLY; not produced)

File path (when executed, per quickstart §3 / contracts/earcon-mapping.md E6): `tests/005-red/SC002_confusion_matrix.csv` — NOT CREATED.

Template header (rows = predicted, cols = actual):

```csv
predicted\\actual,Adagio,Andante,Allegro-frente,Allegro-trás,Unison,Ostinato,Sforzando
Adagio,0,0,0,0,0,0,0
Andante,0,0,0,0,0,0,0
Allegro-frente,0,0,0,0,0,0,0
Allegro-trás,0,0,0,0,0,0,0
Unison,0,0,0,0,0,0,0
Ostinato,0,0,0,0,0,0,0
Sforzando,0,0,0,0,0,0,0
```

Plus column `trap` (0/1), column `trial_index`, column `replication_batch` (1 = initial, 2 = ≥24h), column `direction_of_skip_subscore` (per E6; to be computed from off-diagonal direction), column `haptic_disposition_E3` (0 = not recorded; 1 = rigid mount + hands-off + buzz-check recorded), column `rapid_sequence_flag` (0/1 for 2–3 heterogeneous within 100–150 ms).

Per-attempt list (minimum fields, per E6 / quickstart §3): trial_index, timestamp_utc, session_type (familiarization / randomized / rapid-sub), action_triggered, identified_by_listener, correct (0/1), confusable_pair_flag (Allegro-trás×Andante | Unison×Allegro-frente | none), trap_flag, latency_ms (action→first audible; FR-010 ≤100 ms — measured, not assumed), replication_batch.

---

## Haptic-disposition line (E3 — mandatory per trial)

Per E3 and E6, recorded per trial, not per session:
- Mount: rigid (no hand-held — eliminates body-conduction noise); if improvised, second-person admin required at pilot.
- Hands: off body / not touching device during stimulus; no proprioceptive cue to response.
- Pilot buzz-check: mandatory before session start; admin self-performs if mount is rigid (no second person required).
- Per-trial notation in CSV column `haptic_disposition_E3` = 1 when all three confirmed; 0 = missing (trial invalid, not counted toward pass/fail).
- No suppression of signals (E3: suppression prohibited — every triggered earcon sounds; truncation is a temporal consequence registered by typed diagnostic counter, not a silence).

---

## Replication (≥24 h between batches)

- Batch 1 (initial): randomized constant stimulus ≥105 trials; confusion matrix CSV produced; risky pairs flagged; haptic E3 line recorded.
- Batch 2 (replication): ≥24 h later, same device / same channel (media speaker, parked); same 7-type randomized sequence (different order, no overlap with batch 1 sequence); re-score; compare matrices.
- Pass criterion: zero systematic confusions per type in both batches; any systematic confusion on the two risky pairs (Allegro-trás×Andante; Unison×Allegro-frente) is MEASURED evidence that authorizes a 004-contract amendment — recorded, never silently patched.

---

## Deferral record (required — never silent pass)

```
DEFERRED (owner=device-build; device Redmi 2201117TY unavailable, API 33);
SC-002 blind-test trial NOT EXECUTED; protocol scaffold preserved
(per T030 / tasks.md T215 / T203); no CSV artifact produced; no per-attempt
scores fabricated; replication session not run; haptic E3 line not recorded
in-session (only defined here in scaffold); pilot buzz-check not performed.
Remediation path: when device available, execute 105-trial session per this
protocol; write CSV; record E3 per trial; replicate ≥24h; report systematics.
```

---

## Verification evidence (Phase 3 — this session)

- [x] `tests/005-red/` directory exists.
- [x] `tests/005-red/test_sc002_blind_test_protocol.md` exists (this file).
- [x] Content lists exactly 7 trial types.
- [x] Per-type ≥15 trials specified (total ≥105).
- [x] Trap % specified (5–10%).
- [x] Confusion-matrix CSV format described (rows=predicted, cols=actual, counts + sub-score + E3 + replication column).
- [x] Risky pairs listed (Allegro-trás×Andante; Unison×Allegro-frente).
- [x] E3 haptic-disposition line mentioned (mandatory per trial: rigid mount, hands off body, pilot buzz-check).
- [x] Replication ≥24h noted.
- [x] RED assertion present: protocol defined, execution deferred, no CSV produced.
- [x] Deferred reason stated (Redmi 2201117TY / API 33 unavailable; owner=device-build; per T030/T215).
- [ ] CSV artifact produced — INTENTIONALLY NOT EXECUTED (DEFERRED).
- [ ] Trial passed — NOT CLAIMED (deferred only).

No edit to `crates/player/src/backend.rs`, `MediaMirror.kt`, or status-view source — protocol-only task per scope.
