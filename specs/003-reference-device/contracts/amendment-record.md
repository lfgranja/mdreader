# Contract: Amendment Record

**Feature**: `003-reference-device` | **Date**: 2026-10-11

Exact edit sites and preservation rules for the implementing phase. A checker (human or script)
confirms each site reads as specified after `/speckit.implement` lands.

## Site A — `specs/001-android-media-shell/plan.md`, Reference-device paragraph (FR-001)

- **Current text** (superseded, not deleted from history — git preserves it): *"mid-range arm64, 4x Cortex-A78-class cores, 6 GB RAM or equivalent, document already parsed. All "10 out of 10 trials" latency criteria (SC-009, SC-054, SC-050-idle-resume) are measured on this class of device; a faster device passing is conforming, a slower device failing is not evidence of failure."*
- **New text MUST**: name Redmi 2201117TY (Snapdragon 680 `bengal`, 4× Cortex-A73 + 4× Cortex-A53, ~5.8 GB RAM, Android 13 / API 33) as normative for FR-027, FR-019c, and all "10 out of 10 trials" criteria.
- **New text MUST NOT**: retain the A78 class, the "6 GB or equivalent" class wording, or the slower-device-dismissal rule (it is what US1/AC2 removes).
- **Check**: `grep -c "A78" specs/001-android-media-shell/plan.md` → `0` for reference-device meaning; `grep "2201117TY" specs/001-android-media-shell/plan.md` → ≥1.

## Site B — `specs/001-android-media-shell/plan.md`, device-coupled bounds (FR-002)

Two sites, both reconciled to **≤10 s cold-start on the reference phone**:

1. **Testing paragraph**: *"latency (≤3 s play-to-audio)"* → *"latency (≤10 s cold-start play-to-audio)"* (10/10 trials; endpoint at transducer; pre-roll a future improvement, not a gate obligation).
2. **Performance Goals**: *"Audible audio ≤3 s after Play (mid-range arm64, parsed document)"* → *"Audible audio ≤10 s after Play on the reference phone (cold start)"*.
- **Untouched** (out of scope per spec Assumptions): headphone-pull pause ≤1 s; skip ±5 s; rate-change ≤2 s; pre-roll ≤60 s drain; replay-after-kill ≤30 s; transient rewind 5%-capped-at-30 s.
- **Check**: both sites read `≤10 s`; the three device-independent bounds still read `≤1 s` / `±5 s` / `≤2 s`.

## Site C — `specs/002-fr-024-synthesis-adapter/spec.md`, divergence note (FR-004)

- **Current text** (`:117`, preserved verbatim): *"Divergence noted: `001/plan.md:50-53` still names the A78 class (001-owned; alignment is a 001 follow-up, not done here)."*
- **Required**: follow-up note appended below it (not an edit of it) stating the condition no longer holds, naming the 001 amendment commit, and dating the closure.
- **Check**: original sentence intact; closure note present with commit hash + date.

## Site D — Amendment commit message (FR-003)

- **MUST carry**: maintainer acceptance, date (2026-10-11), rationale (single-user project; possessed device beats hypothetical class), preserved-history pointer (prior verdicts untouched; re-judgment of 9 s × 5 stated explicitly), and the 002-precedent shape (`d5d6b45`).
- **MUST NOT**: rewrite any past measurement row in `SC-009-GATE-RECORD.md` or elsewhere.

## SC-002 re-judgment input

- Source measurement: 9 s × 5 cold-start (recorded FAIL under the ≤3 s bound).
- After this amendment the verdict is re-judged against ≤10 s and recorded un-asterisked in the implementing phase (PASS under the relaxed bound, stated explicitly at implementation time).
