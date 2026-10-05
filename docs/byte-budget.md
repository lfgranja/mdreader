# Byte-Budget (Principle X)

| Item | Value | Source / Evidence | Change rule |
|---|---|---|---|
| Google Cloud TTS smallest documented per-request limit | 5,000 bytes | Audit 2026-10-05 (see constitution amendment v1.2.0) | Only amendment changes a number |
| Offline default per-request limit | No limit | Feature operates offline-first (FR-015b / FR-026) | N/A |
| Pre-roll bound | ≤60 s engine-delivered audio | FR-015a / pipeline property | Not a byte budget |

**Enforcement status:** Deferred to synthesis feature (FR-024). `SC-050` verification deferred; `SC-050b` references this file by path. Only an amendment to `.specify/memory/constitution.md` may change any number.
