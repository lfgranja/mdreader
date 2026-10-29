# The diagnostic record

Grounded in `specs/001-android-media-shell/contracts/diagnostic-record.schema.json`
(the authority for this page) and `crates/diag/` — the append-only
JSONL writer (`src/writer.rs`, T073) and the conformance scanner
(`src/scan.rs`, T074). Every requirement token (FR-### / SC-###)
refers to `specs/001-android-media-shell/spec.md`.

The record is the application's private, append-only audit trail:
**event contents are kinds, positions, and failure names — never
document text** (FR-039a, FR-045, SC-056). It is never
transmitted (FR-040) and has no in-app surface (FR-039a); it is
read by the CLI and by CI, nothing else (SC-056).

## Shape: one JSON object per line

One `RecordEvent` per line, serialized with `serde_json`. Four
fields are required on every event; `additionalProperties: false`
makes the property set closed — an unknown key fails validation:

- `v` — record version, `const` **1**
- `ts_ms` — event timestamp, milliseconds since the Unix epoch
  (integer ≥ 0)
- `kind` — one of the closed eleven-member event-kind set
- `position_secs` — playback position in seconds (number ≥ 0)

The closed event kinds (`diag::writer::EventKind`):

| `kind` | Carries |
|---|---|
| `session_start` | — |
| `session_end` | `transport` + `session_end_reason` |
| `transport_change` | `transport` |
| `focus_transition` | `focus` |
| `output_device` | `device` |
| `failure` | `error` |
| `refusal` | `error` |
| `trim` | `trim` |
| `gap` | — (storage-exhaustion marker) |
| `earcon_truncation` | truncation counter (latest-wins, E2.2) |
| `earcon_emission_failure` | `error` (typed emission failure, E4) |

The closed value sets (wire-identical to the `engine_core::closed`
registry — one list, never a second drifting list, FR-044):

- `transport`: `playing | paused | stopped | completed | interrupted`
- `session_end_reason`: `paused | stopped | completed | interrupted`
  — the closed-4 set; `playing` is a transport state and is never a
  session-end reason (FR-044)
- `focus`: `duckable_transient | silence_demanding | permanent_loss |
  regained` (the three `FocusOutcome` outcomes plus the `FocusRegained`
  direction event, identical wire strings)
- `error`: `engine_unreachable | engine_init_failed | rate_mismatch |
  channel_mismatch | device_refused | device_lost | start_refused`
  (the seven-member engine-failure taxonomy)
- `engine_tier`: `offline_default | cloud_default | cloud_escalation |
  test_double | evaluated_fallback` (ascending fallback order; the
  Edge endpoint stays last and is never a default)
- `device`: `{direction, device_id}` with `direction ∈
  appeared | disappeared` and a non-empty `device_id`

Conditional requirements (the schema's `allOf`):

- an entry carrying `engine` must also carry `engine_tier` (the
  C1 pairing invariant); with `engine_tier = test_double`,
  `engine` must be one of the two declared stand-ins
  (`standin_a`, `standin_b`) and is exempt from constitution
  registry resolution (FR-023c, SC-057)
- a `trim` entry must carry `trim: {dropped_events, cutoff_ts_ms}`
  with `dropped_events ≥ 1`
- a `session_end` entry must carry `transport` — restricted to the
  closed-4 set, never `playing` — and `session_end_reason`
- a `failure` or `refusal` entry must carry `error`
- `supersedes_ts_ms` marks the timestamp an entry supersedes, where
  it applies

## The no-prose invariant (FR-039a / FR-045)

The optional `detail` field is a bound, non-prose `key=value`
string: at most **200 characters**, matching
`^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$`
— one or more space-separated `key=value` pairs. Document text
never enters the record: no prose, no titles, no section content.
Example of a conforming `detail`:
`"engine_id=sherpa-onnx-piper-pt-br tier=offline_default"`.

## The writer: dual bounds, never silent (FR-039 / FR-039b)

`diag::writer::DiagnosticWriter` opens (creating if absent) the
record in app-private storage and enforces two bounds **on open
and on every append**:

- **Retention** — `RETENTION_DAYS = 30`: events older than the
  30-day window, measured backwards from the newest retained
  event, are trimmed oldest-first.
- **Size** — `SIZE_CAP_BYTES = 10 MiB` (`10 * 1024 * 1024`):
  the oldest retained events are dropped until the retained
  serialized length is within the cap.

Every drop is accounted by a **trim marker** line
(`kind: "trim"`, carrying `trim.dropped_events` and
`trim.cutoff_ts_ms`) flushed to disk **before** the retained
entries it describes, so the on-disk line order is
`[trim markers][retained entries]` and a shrink is never silent.
Trim and gap markers are metadata, not retained entries
(`retained_entries()` counts only record events).

Writes are best-effort: a storage-full append failure records a
**gap marker** (`kind: "gap"`) and returns `Ok`, so playback is
never interrupted (FR-039b); `gap_count()` reports the markers
since open.

## Position keying (FR-036 / FR-036a)

`position_secs` in the record is the playback position in
seconds. The *stored* resume position it audits is keyed by the
document's **blake3-256 content fingerprint** (`session::fingerprint`,
T059): the same bytes yield the same 32-byte hash regardless of
path, and a 1-bit change yields a different hash. An edit is
therefore a different document — no resume (FR-036a) — and a
`Stop` or document end clears the stored entry (FR-032 / FR-032b /
FR-032f).

## The scanner: seven conformance violations (SC-005, SC-056)

`diag::scan::scan_record` makes one read-only pass over the
record (the same file the writer produces) and reports every
violation the per-entry schema cannot express. The violation set
is closed — exactly seven kinds, no `Other` catch-all:

| Violation | Rule |
|---|---|
| `prose_in_detail { ts_ms }` | `detail` present but does not match the non-prose `key=value` pattern — document text in the record (FR-039a, FR-045, SC-056) |
| `detail_too_long { ts_ms, len }` | `detail` longer than 200 characters (FR-045) |
| `failure_without_error { ts_ms }` | a `failure` entry with no `error` (FR-045) |
| `refusal_without_error { ts_ms }` | a `refusal` entry with no `error` (FR-045) |
| `playing_without_advance { ts_ms }` | a `transport_change` to `playing` with no later entry in the same session at a strictly greater `position_secs` and no later intervening entry (SC-005) |
| `session_end_with_playing { ts_ms }` | a `session_end` entry carrying `transport: "playing"` (FR-044: the closed-4 session-end set never includes `playing`) |
| `engine_without_tier { ts_ms }` | an entry carrying an `engine` with no `engine_tier` (the C1 pairing invariant) |

The SC-005 verdict is computed per session segment (a
`session_end` closes its session inclusively, a `session_start`
opens the next). A `playing` entry conforms when a later
same-session entry sits at a **strictly greater** `position_secs`,
or when an entry **intervenes** — a non-playing `transport`, a
`permanent_loss` focus transition, or a `failure` — explaining
why playback stopped advancing. The report is oldest-first, in
the record's on-disk line order.

Empty lines are ignored; a non-empty line that is not a valid
`RecordEvent` is a corrupt record, not a conformance question,
and fails the scan.

## CLI (Principle II)

The scanner is the `diag` binary's single capability. Verified
invocations (workspace `target/debug/diag`; `--format json` is
the default, `--format human` selects the plain form):

```sh
# A conforming record — session start, playing with advance,
# a deliberate pause ending the session:
diag scan rec.jsonl
# → {"violations":[]}   (exit 0)

# A failure entry with no error:
diag scan rec-bad.jsonl
# → {"violations":[{"failure_without_error":{"ts_ms":1}}]}   (exit 0)

# The same report in the human-readable plain form:
diag scan --format human rec-bad.jsonl
# → capability: scan
#   violation-count: 1
#   failure_without_error: ts_ms=1
```

Violations are a **report, not an error**: the scan exits 0 even
when it finds violations — the payload on stdout is the whole
finding; a non-zero exit means the record could not be read or a
line was not a valid event. A stall (a `playing` entry with no
later advance) and a `session_end` at `playing` are both caught
in one pass:

```sh
diag scan rec-stall.jsonl
# → {"violations":[{"playing_without_advance":{"ts_ms":9}}]}
```
