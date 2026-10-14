# Contract: Resume Algorithm (interruption → at/just-before point of interruption)

**FR-007**. Defines the behavioral rules for computing the resume position after an interruption. The measurement (`replay_secs` in `contracts/diagnostic-trace.md`) is the observed output; this contract specifies the algorithm that produces it.

## Scope

Applies to every resume path: interruption replay (≤30 s cap), kill/power-loss replay (≤60 s cap), and deliberate-pause resume (3 s dialog → continue from stored position). The stored position is always the last-spoken cursor, never buffered-ahead.

## Algorithm

1. **Containing-sentence-start snap-back**: From the last-spoken position, find the sentence boundary at or before the cursor. The sentence boundary is the start of the containing sentence (the sentence that was playing when the interruption occurred).

2. **Context cue (fixed 5 s further backward)**: From the sentence start, move further backward by exactly 5 seconds of content. The cue distance is selected to land the listener before the sentence that was interrupted, providing context without excessive replay.

3. **Total backward cap**: The total backward distance from the interruption point MUST be ≤30 s for interruption replay and ≤60 s for kill/power-loss replay. If the computed position exceeds the cap, clamp to the cap.

4. **Never mid-word**: If the computed position falls mid-word, snap forward to the next word boundary. The resume position MUST NOT be mid-word.

5. **Never doc start**: If the computed position is before the document start (position 0.0), clamp to 0.0. The resume position MUST NOT be negative.

6. **Degrade fallback**: If sentence-start detection fails (e.g., segmentation error, empty document), fall back to the raw last-spoken cursor, never doc start. The degrade path is logged in the diagnostic record.

## Atomic write

The persisted cursor MUST be the last-spoken position (never buffered-ahead) in a single atomic transaction (`BookmarkStore::put`). The resume position is computed at resume time from the persisted cursor; it is NOT pre-computed and stored.

## Measurement

`replay_secs` is measured as the difference between the resume position and the interruption/kill position, logged in the diagnostic record per `contracts/diagnostic-trace.md`. The measurement verifies the cap is met; it does not define the algorithm.

## CLI surface (Principle II)

`player` binary: `resume --position <secs> --interruption-type <interruption|kill>` computes the resume position from the given inputs and emits the result as JSON; Tauri IPC calls the same function.
