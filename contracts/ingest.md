# Contract: Document Ingest (picker → snapshot → BLAKE3)

FR-018 + FR-008 + FR-010. Synthesis reads only the transient snapshot; the persisted URI is a freshness grant, never an audio input.

## Flow

1. App fires Kotlin `ACTION_OPEN_DOCUMENT` (`text/markdown`, `text/plain`, `FLAG_GRANT_READ_URI_PERMISSION | FLAG_GRANT_PERSISTABLE_URI_PERMISSION`).
2. On pick: `takePersistableUriPermission()` best-effort in `try/catch(SecurityException)` (failure still reads the one-shot grant, re-prompts next open) → `ContentResolver.openInputStream(uri)` → read bytes once.
3. `fingerprint = blake3_256(bytes)`; write `filesDir/snapshots/<hex>.{md,txt}` (transient, session-scoped); record `(uri, fingerprint, snapshot_path)`.
4. Rust `fingerprint()` → parse (`adapter_synthesis::chunk::split_document`) → `store.get(fingerprint)` → match = resume-offer path (FR-009), mismatch = start `0.0`, no offer.
5. Synthesis input = snapshot bytes only. Trim snapshot on new pick / Stop / document-end.

## Failure contract (closed, no catch-all)

- `SecurityException` / `FileNotFoundException` on re-read (moved/renamed/revoked) → re-prompt same picker; user re-picks → hash-match restore (SC-005).
- Persist-grant failure → one-shot read proceeds; next open re-prompts.
- Non-`.md`/`.txt` pick → refused before synthesis (picker filter + Rust extension check + error `FormatUndeliverable`).
- No user-visible library or import step; reopening a moved/renamed document goes through the same picker path.

## CLI surface (Principle II)

`session` binary gains `ingest <snapshot-file>` (hash + store lookup → `{fingerprint, resume: bool, offset}` JSON); Tauri IPC calls the same functions. Kotlin owns intent + stream + snapshot write; Rust owns hash/parse/lookup/synthesis.
