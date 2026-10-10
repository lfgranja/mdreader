# Clarification Batch — 002-fr-024-synthesis-adapter checklists
Run date: 2026-10-07. Backgrounded review agents: 4 checklist reviews (CHK024/CHK014/CHK028/CHK031) + 3 research agents (resume boundaries, parser fallback, gate-check standards). All completed.

---

Question: For very short interruptions (e.g., 1 s paused duration), does the "nearest completed paragraph/heading boundary" snap dominate over the FR-015 rewind formula (min(5%×paused, 30 s)), yielding ~0.05 s rewind then a full-boundary snap? And is the boundary-selection rule (nearest at or before yield vs. "containing the 5%-back point") defined unambiguously for yields near a boundary?

Why does it matter: The interaction determines the effective resume point and replay cost for the most common interruption regime (brief focus loss / notification glance). If "rewind" is interpreted as a hard magnitude, a 1 s pause replays ~0.05 s; if "snap" dominates, it can replay a full paragraph (up to ~30 s). The spec's two-phase sequence (rewind then snap, spec.md:59) implies snap dominates, but the precedence is not stated, and FR-015's raw-cursor degradation branch (001/spec.md:611-613) conflicts with 002's paragraph-snap when the adapter declares `AlignmentGranularity::None`. A missing de-minimis/floor rule also means the proportional term can degenerate to near-zero with no defined behavior.

Findings:
- Event Segmentation Theory (Zacks & Swallow 2007; Zacks et al. 2007 Psych Bulletin) and resumption-lag studies (Monk et al. 2008 JEP:Applied; Labonté et al. 2021 Scientific Reports) support boundary-level resume (paragraph/heading) over word/sentence for interruptions > few seconds; for very short interruptions context stays active, so fine-grained resume is tolerable, but the boundary-vs-magnitude precedence is not tested directly in media players.
- Spec.md:59 defines ordered composition: cursor back min(5%×paused,30s) → snap to nearest completed paragraph/heading boundary containing that point; cap overrides snap only when boundary >30 s. This means snap dominates magnitude; 30 s is ceiling over boundary, not over rewind.
- RESUME_ALIGNMENT_RESEARCH.md:109-118 (repo-verified) states `None` declared → snap to paragraph/heading; hard invariant resume ≥ word-start; keep min(5%,30s) as rehearsal context — confirming snapshot dominates.
- CLARIFICATION_BATCH_RESOLVED.md:13 (APPROVED 2026-10-07) requires applying a two-part FR-015 rewrite (not yet edited; underlying edits still pending). Until applied, FR-015's raw-cursor degradation vs 002's paragraph snap remain in conflict.
- CHK028 subagent (completed) found 4 ambiguities: (1) no de-minimis/floor, (2) "nearest at or before yield" vs "containing 5%-back point" divergence from CHK007 paraphrase, (3) degradation-branch conflict shared with CHK014, (4) 30 s cap interpretation at `None` granularity lacks timing mechanism.

Options:
- Recommended: Keep error-close (no de-minimis threshold); apply CLARIFICATION_BATCH_RESOLVED.md:13 plus a short-pause addendum to spec.md:59 stating explicitly that the proportional term selects the boundary and the snap dominates magnitude (effective rule: start of paragraph/heading containing yield−min(5%×paused,30s)). Document that for very short pauses the snap can replay a full paragraph (≤30 s cap). Amend FR-015's degradation so `None` + adapter parser mapping = paragraph/heading snap (not raw cursor); reserve raw cursor only where no parser boundary exists. Fix CHK007 paraphrase to match "containing the 5%-back point". Add synthetic SC-027 case (1 s pause, known paragraph boundaries → current-paragraph start, ≤30 s).
- Reason for recommendation: Matches the spec's two-phase sequence, aligns with EST + resumption-lag evidence (boundary is dominant lever, not 0.05 s replay), respects fail-closed supply-chain and constitution gate-8 split (deferred discharge, preserved mechanism), and fixes the unapplied edit already approved.
- Alternative (not recommended): Add a de-minimis floor (e.g., if 5% < 1 s, use 1 s rewind) — contradicts boundary-dominance rationale and adds an arbitrary constant unsupported by literature.

Status of other 3 checklist items:
- CHK024 (mechanism-gate): [x] — reviewed sound, sequence defined, prerequisites anchored; minor citation drift only.
- CHK014 (segment-resume consistency): [x] — no contradiction; deferral uniform across 8 sites.
- CHK031 (dependency/fallback): [x] (conditional) — grep 0 verified; status and procedural fallback documented; requires 2 micro-edits (FR-024a clause + citation fix adapter-handoff.md:2.3→5,98,205) before fully locked.
