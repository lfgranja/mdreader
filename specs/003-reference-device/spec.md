# Feature Specification: Reference Device Realignment

**Feature Branch**: `003-reference-device`

**Created**: 2026-10-11

**Status**: Draft

**Input**: User description: "Divergência com 001/plan.md (que ainda diz A78) registrada como follow-up do 001 — trocar para o meu aparelho atual"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Gate verdicts measured on the phone in hand (Priority: P1)

The maintainer runs any on-device gate trial (latency, lock-screen, focus, routing, failover) and the verdict is judged against the phone they actually own — Redmi 2201117TY — instead of a hypothetical mid-range class that exists nowhere in this project.

**Why this priority**: Every "10 out of 10 trials" criterion in `001-android-media-shell` is currently normed to a device class (4× Cortex-A78, 6 GB) no one on this project possesses. Verdicts against it are either unmeasurable or, when measured on the real phone, dismissed by the "slower device failing is not evidence of failure" rule — which makes the gates decorative. A reference the maintainer can actually run is the precondition for every other on-device gate meaning anything.

**Independent Test**: Open `specs/001-android-media-shell/plan.md`, read the Reference-device paragraph, and confirm it names the Redmi Note 11 characteristics; run any one 10/10-trial criterion on that phone and record a verdict that counts (pass or fail, no device-class asterisk).

**Acceptance Scenarios**:

1. **Given** the 001 plan's Reference-device paragraph, **When** a reader reads it, **Then** it names Redmi 2201117TY (Snapdragon 680, 4× Cortex-A73 + 4× A53, ~5.8 GB RAM, API 33) instead of the A78 class.
2. **Given** a 10/10-trial latency run on that phone, **When** all 10 trials complete, **Then** the verdict is recorded as-is against the amended bound, with no "slower device" dismissal.
3. **Given** the 002 SC-009 amendment (≤10 s on the same phone, commit `d5d6b45`), **When** both specs are read together, **Then** device identity and cold-start bound agree (no spec says A78 while the other says Redmi).

---

### User Story 2 — No silent drift between the two specs (Priority: P2)

A reader comparing `001/plan.md` and `002/spec.md` finds one device story, with the 001-side divergence explicitly closed rather than left as a recorded-but-open follow-up.

**Why this priority**: The 002 amendment (`d5d6b45`) deliberately left `001/plan.md:50-53` untouched and recorded the divergence. Until this feature lands, the repo asserts two reference devices at once — the exact "second drifting authority" class the constitution exists to prevent.

**Independent Test**: Grep both specs for the reference-device definition and confirm they name the same phone; confirm the 002 "Divergence noted" annotation is resolved or explicitly superseded.

**Acceptance Scenarios**:

1. **Given** the 002 amendment note pointing at the 001 divergence, **When** this feature lands, **Then** the note's condition (001 still says A78) no longer holds, and a follow-up commit records the closure.

---

### Edge Cases

- What happens to past verdicts recorded under the A78 class (e.g. the SC-009 9 s FAIL-under-≤3 s history)? → Preserved as history; re-judged only where the amended bound changes the verdict, with the re-judgment stated explicitly, never by editing old measurements.
- What happens if the maintainer replaces their phone? → A new realignment feature, same shape as this one; the class always names a possessed device, never a hypothetical one.
- How does this interact with `002/spec.md:115` (already amended)? → The 002 text is the template: same phone, same ≤10 s cold-start reasoning. This feature brings 001 into agreement with it.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The 001 plan's Reference-device paragraph MUST name the maintainer's current phone (Redmi 2201117TY: Snapdragon 680 `bengal`, 4× Cortex-A73 + 4× A53, ~5.8 GB RAM, Android 13 / API 33) as the normative class for FR-027, FR-019c, and all "10 out of 10 trials" criteria, replacing the A78-class text.
- **FR-002**: The 001 plan's device-dependent numeric bounds MUST be reconciled to ≤10 s cold-start, consistent with the 002 amendment (maintainer choice Q1:B, 2026-10-11): the "latency (≤3 s play-to-audio)" 10/10-trial bound and the "Audible audio ≤3 s after Play" performance goal both become ≤10 s cold-start on the reference phone. Pre-roll becomes a future improvement, not a gate obligation.
- **FR-003**: The amendment MUST be recorded with maintainer acceptance, date, rationale (single-user project; possessed device beats hypothetical class), and preserved history — the same amendment discipline as the 002 change (`d5d6b45`).
- **FR-004**: The 002 "Divergence noted" annotation MUST be closed by a follow-up note (not rewritten history) once 001 agrees.

### Key Entities *(include if feature involves data)*

- **Reference Device**: The possessed phone all on-device verdicts are judged against — Redmi 2201117TY, Snapdragon 680, ~5.8 GB RAM, API 33. Replaces a hypothetical class; itself replaceable only by another possessed device via a new amendment.
- **Trial Verdict**: A pass/fail recorded against the amended bound on the reference phone — counts as-is, with no device-class asterisk.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reader finds exactly one reference-device definition across `001/plan.md` and `002/spec.md`, and both name the Redmi Note 11.
- **SC-002**: The 9 s × 5 cold-start measurement has a final, un-asterisked verdict under the reconciled bound (PASS if the bound is relaxed to cover it, FAIL-with-mandated-adaptation if the bound is kept).
- **SC-003**: No on-device criterion in the repo is normed to a device nobody possesses.

## Assumptions

- The maintainer's current and only gate device is the Redmi 2201117TY (no second phone, no device farm, no emulator-as-reference).
- The 002 amendment (`d5d6b45`: ≤10 s cold start on this phone) is accepted precedent, not under re-review here.
- `001/plan.md` Performance Goals ("Audible audio ≤3 s after Play") and Testing ("latency (≤3 s play-to-audio)" 10/10 trials) are the only other numeric bounds coupled to the device class; headphone-pull ≤1 s, skip ±5 s, and rate-change ≤2 s are device-independent interaction guarantees and out of scope.
- Constitution needs no amendment (it fixes no device class and no numeric latency bound).
