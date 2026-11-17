# Contract: T184/T226 Residual Tracking (FR-012)

**Feature**: `012-debug-ingest-hook`  
**Covers**: FR-009, FR-012, SC-003  
**Status**: Normative for planning/tasks.

## Purpose

Allow trustworthy reporting of Spec 009 T184 and Spec 010 T226 progress without overstating coverage. Debug-hook automation may satisfy **programmatic non-UI ingest-path** scope only; native SAF picker UI/a11y behavior remains separately tracked.

## Rule

Programmatic pass => record `satisfied for exercised non-UI scope` with explicit evidence links and `full_parent_completion_claimed=false` (hook machine-readable residual rule: `full_parent_completion_claimed_from_hook_evidence=false`). Native SAF picker cancel and TalkBack/keyboard traversal of native picker/title UI remain `open` residual items (or `verified` only by manual/accessibility/device evidence, or `accepted-risk` only by explicit recorded decision).

**Required update locations (normative):** owning spec task artifacts `specs/009-open-file-from-title/tasks.md` (T184) and `specs/010-directory-reading/tasks.md` (T226), plus evidence/report paths produced for the run/campaign. No orphan ad-hoc-only evidence may claim T184/T226 status.

**Lifecycle/approval (FR-012):** statuses `open|verified|accepted-risk|deferred`. `verified` requires evidence refs appropriate to scope. `accepted-risk` requires designated owner, dated decision ref, rationale, scope boundary, and evidence where applicable; it does not satisfy full verification and does not auto-complete the parent while `blocks_full_completion=true`. `deferred` requires owner, decision ref, and rationale. Parent T184/T226 is fully complete only when all required `*-PROG` and required residual items are `verified` or explicitly re-scoped by a recorded decision (never implicit).

**Hook deterministic contract helper (host, no device side effects):** `./scripts/automate-t184-t226-hook.sh --print-residual-rules [--json]` emits the machine-readable lifecycle/parent-completion rules. Field names in that output (e.g., `allowed_statuses`, `accepted_risk_requires`, `verified_requires`, `deferred_requires`, `parent_completion_rule`, `full_parent_completion_claimed_from_hook_evidence`) MUST remain semantically aligned with this normative contract.

**Traceability/ownership (CHK032/CHK001/CHK002):** residual and evidence IDs are immutable (`RES-*`/`T184-*`/`T226-*`, `EV-012-*`, linked `FR/SC/TASK/ASM/DEP`). Missing trace links or hook-only full-completion claims are blocking findings. Ownership follows Spec 012 governance (Spec 012 maintainer + Android/shell maintainer; QA/evidence reviewer for FR-012 claims).
## T184 (Spec 009 — Open File From Title, quickstart Scenarios 1–4)

Parent task remains open until required scope is verified or explicitly re-scoped by owner decision.

| ID | Scope | Method | Can debug hook satisfy? |
|---|---|---|---|
| `T184-PROG` | Programmatic document ingest/routing through production path (fixture-based), explicit result/crash-ANR evidence | Debug hook + ADB/script | Yes, **only** for exercised non-UI scope |
| `T184-RES-SAF-CANCEL` | Scenario 3: user cancels/Back in native SAF picker while playing; playback/metadata unchanged | Manual/UI device validation | No |
| `T184-RES-A11Y` | Scenario 4: keyboard/focus-visible on `.meta-box`, Enter/Space opens SAF, TalkBack traversal/announcement/focus restoration across native picker/title UI as applicable | Manual accessibility validation | No |

## T226 (Spec 010 — Directory Reading, quickstart Scenarios 1–5)

| ID | Scope | Method | Can debug hook satisfy? |
|---|---|---|---|
| `T226-PROG` | Programmatic directory ingest/manifest-dispatch path, accepted chapter handling evidence for fixture, crash/ANR signals | Debug hook + ADB/script | Yes, **only** for exercised non-UI scope |
| `T226-RES-SAF-TREE` | Scenario 1 real SAF tree selection and TTFA/UI observation | Manual/device UI | No |
| `T226-RES-TRANSITION` | Scenario 2 gapless transition, visual/live-region updates | Manual/device UI + playback observation | No |
| `T226-RES-RESUME-UI` | Scenario 3 kill/reopen resume offer dialog and choices | Manual/device UI | No |
| `T226-RES-HARDWARE-CONTROLS` | Scenario 4 Bluetooth/lock-screen/MediaSession contextual skip | Manual/device hardware controls | No |
| `T226-RES-INVALID-ENTRIES-UI` | Scenario 5 resilience during real directory selection/playback UX | Manual/device UI (programmatic hook may provide supporting evidence only) | Supporting only; not full scenario proof |
| `T226-RES-A11Y-PICKER` | Native picker/directory selection accessibility, TalkBack/keyboard traversal/focus announcements | Manual accessibility validation | No |

## Evidence Record Minimum

**SC-003 reporting population:** all T184/T226 validation summaries produced for this feature/validation campaign. Every such summary MUST identify residual native system-picker UI/a11y cases, include evidence links for covered programmatic scope, and state `full_parent_completion_claimed=false` for hook-only coverage.

For each `*-PROG` item:
- correlation key/run identity (request/evidence id or deterministic run key);
- build/debug APK identity or build id + SHA/hash when available (`unavailable:<reason>` if not);
- device serial/model/SDK/ABI;
- exact fixture ref and mode;
- raw result (`ok:*`/`error:*`/`refused:*`);
- logcat/report/screenshot/crash-ANR evidence paths and evidence disposition (`preferred/effective path or channel`, `write_status`, reviewer-accessible location, retention scope tied to covered IDs/policy);
- fixture composition/expected filtering when claiming directory `chapters=<n>` pass (`ASM-012-CHAPTERS-INTERP`);
- statement: `Programmatic scope satisfied for exercised non-UI ingest path only. Native SAF picker UI/a11y residuals remain open.`
- `full_parent_completion_claimed=false`.

Manual a11y/device-system-control evidence is required for residual closure and cannot be replaced by hook evidence (`ASM-012-MANUAL-A11Y`, FR-009/FR-012).

## Suggested Markdown Shape (in owning spec tasks)

```md
- [ ] T184 [US1][US2][US3] Live on-device/manual validation of Scenarios 1–4... Not fully complete from debug-hook evidence alone (012/FR-012).
  - [x] T184-PROG Programmatic non-UI ingest path — satisfied for exercised scope. Evidence: <paths>.
  - [ ] T184-RES-SAF-CANCEL Native SAF cancel (Scenario 3) — open.
  - [ ] T184-RES-A11Y Keyboard/TalkBack/picker a11y (Scenario 4) — open.
```

Adapt IDs/wording to existing spec conventions, but preserve the split and the “not fully complete from debug-hook evidence alone” rule.
