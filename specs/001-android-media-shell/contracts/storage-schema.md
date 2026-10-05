# Contract: Storage Schema (`session` crate, redb)

FR-004/FR-015/FR-019b/FR-030–FR-032 series/FR-036/FR-036a.

## Tables

- `bookmarks`: key `[u8; 32]` (blake3-256 of document bytes) → value
  `{ offset_secs: f64, first_yield_ms: u64, sentence_idx: u32 }`.
  `first_yield_ms` = start of the continuous non-advancing interval (FR-015 supersede
  clock, never reset at supersede; the record carries the `supersedes_ts_ms` link).
  `sentence_idx` = snap hint only; node-boundary re-anchor (FR-038) resolves via document
  structure at resume, not from store.
- `settings`: key `str` → bytes. Required key: `"last_rate"` ∈
  `{0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0}`, default `1.0` on first use.

## Semantics (observable; mechanism illustrative)

- Write on: pause, app close path, focus-loss pause, engine-failure pause. Atomic
  transaction; crash-safe (FR-004: resume replays ≤30 s after hard kill).
- Delete on: Stop press, document end (FR-032/FR-032b). Nothing else deletes.
- Lookup on open: hash hit ⇒ ResumeOffer flow (deliberate-pause vs unexpected-end
  forms); miss (incl. edited content, FR-036a) ⇒ start at `0.0`, no offer.
- Identical bytes at any path share the fingerprint and the position (accepted, FR-036).
- `last_rate` read at session start, written on every rate change; crosses sessions and
  documents (FR-019b).

## CLI surface (Principle II)

`session` binary: `get <file> | put <file> <offset> | clear <file> | rate get|set <v>`
with `--format json` default for scripting; Tauri IPC calls the same functions.
