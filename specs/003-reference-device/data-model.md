# Data Model: Reference Device Realignment

**Feature**: `003-reference-device` | **Date**: 2026-10-11

No runtime entities. The "entities" are documentation records the amendment creates or re-judges.
Each maps to its spec Key Entity and to the contract (`contracts/amendment-record.md`) site that governs it.

## 1. Reference Device (spec Key Entity)

| Field | Value |
|---|---|
| Designation | Redmi 2201117TY (Redmi Note 11) |
| SoC | Snapdragon 680 (`bengal`) |
| CPUs | 4× Cortex-A73 + 4× Cortex-A53 |
| RAM | ~5.8 GB |
| OS / API | Android 13 / API 33 |
| Normative for | 001 FR-027, FR-019c, all "10 out of 10 trials" criteria |

**Validation rules** (from FR-001 / SC-001 / SC-003):
- Exactly one definition across `001/plan.md` and `002/spec.md`, both naming this phone.
- No `A78`-class reference-device text remains in `001/plan.md`.
- Replaceable only by another possessed device via a new amendment (spec Edge Cases).

**State transitions**: hypothetical class → possessed phone (this amendment). Future replacement → new realignment feature, same shape.

## 2. Trial Verdict (spec Key Entity)

| Field | Value |
|---|---|
| Subject | Any on-device 10/10-trial run on the reference phone |
| Bound applied | ≤10 s cold-start (device-coupled criteria); device-independent bounds unchanged |
| Recording rule | Verdict recorded as-is against the amended bound; no device-class asterisk (US1/AC2) |
| History rule | Past measurements preserved verbatim; re-judgment stated explicitly, never by editing old rows |

**Validation rules** (from SC-002):
- The 9 s × 5 cold-start measurement carries a final, un-asterisked verdict under the reconciled bound.
- No on-device criterion in the repo is normed to a device nobody possesses (SC-003).

## 3. Amendment Record (FR-003 / FR-004 carrier)

| Field | Value |
|---|---|
| Sites | `001/plan.md` Reference-device paragraph; `001/plan.md` Testing latency site; `001/plan.md` Performance Goals site; `002/spec.md:117` divergence note |
| Acceptance | Maintainer-accepted, dated, rationale recorded in the implementing commit |
| Preservation | Supersede-by-note; no in-place rewrite of prior text or past verdicts |
| Precedent shape | 002 amendment `d5d6b45` |

**Validation rules**:
- Commit message carries acceptance + date + rationale (single-user project; possessed device beats hypothetical class) + preserved-history pointer.
- 002 annotation closed by follow-up note, original text intact.
