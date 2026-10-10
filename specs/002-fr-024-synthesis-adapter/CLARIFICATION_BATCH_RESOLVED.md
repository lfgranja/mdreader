# Clarification batch — decisions recorded (2026-10-07)

Source: batch from 002-fr-024 synthesis adapter review (3 review agents + 4 /research agents).
Status: APPROVED by user (re-confirmed in follow-up).
Edit scope: checklist [x] applied; underlying spec edits noted below for editor.

## Decisions applied (checklist)
- audio-safety.md CHK014, CHK036 → [x] (measurement defined; assumption validated; citation fixed)
- mechanism-gate.md CHK012, CHK014 → [x] (invariant consistent; citation corrected; editorial rewording approved)
- FR-015b mis-cite, SC-027 ID collision → not checked until spec edits complete (CHK016/CHK021 remain open)

## Underlying spec edits required (not yet edited — editorial / contract-level)
1. 002/spec.md:59 — rewrite FR-015 snippet to two-part rule (cursor 5% + boundary snap; cap overrides only when boundary > 30 s); document None adapter uses parser-derived paragraph/heading (degradation branch never fires).
2. 002/spec.md:116 / data-model.md:73 / adapter-handoff.md:107 — correct FR-015b → FR-004/SC-007 for replay ≤30 s.
3. 002/spec.md:76 / adapter-handoff.md:4 / contracts — state explicitly: new engine uses adapter's DocumentSegment split points; adapter never re-splits.
4. constitution.md citations across feature — 304-308 → 377-391 (§XI consent); 138→139 (line drift); add SC-044 prerequisite chain (writer/UI/verb/storage/T075 → state → selectable).
5. SC-009 measurement definition — full pipeline from Play to transducer; adapter-output→speaker as diagnostic sub-measure.
6. adapter-handoff.md:177 — reword mechanism description to "preserved as invariant (active fallback deferred to SC-044)".
7. adapter-handoff.md §2.1 — declare empty-state (no segments → nothing delivered) and partial-load (available segments only, resume on available split points) as v1 behavior.
8. AudioChunk immutability + single authority — immutable after delivery; redb bookmark authoritative; adapter never second source.
