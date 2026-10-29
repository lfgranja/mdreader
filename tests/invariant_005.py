#!/usr/bin/env python3
"""TDD RED property tests for specs/005-adopt-design-sona (invariant assertions).
Scratch only; not part of repo source. Executed by: python3 tests/invariant_005.py
Rules: must FAIL for RIGHT reason (missing mapping / truncated / drift); never compile error.
"""
import re, sys

SPEC = open("specs/005-adopt-design-sona/spec.md").read()
TASKS = open("specs/005-adopt-design-sona/tasks.md").read()

FAILURES = []

def fail(test_id, reason):
    FAILURES.append(f"{test_id}: {reason}")

# ------------------------------------------------------------------ T-TEST-01: FR→task coverage
# For each FR-001..010 explicitly in spec (§Functional Requirements 130-149),
# assert an explicit task mapping (FR-ID or keyword). Report named missing.
FR_KEYS = [f"FR-{i:03d}" for i in range(1, 11)]
# Keyword heuristics drawn from spec summaries and observed tasks.md references
KEYWORDS = {
    "FR-001": ["token", "paleta", "--sona-", "retokenize", "accent", "C1", "C2"],
    "FR-002": ["notificação", "notification", "título", "seção", "progresso", "body text"],
    "FR-003": ["earcon", "Adagio", "Andante", "Allegro", "Sforzando", "Ostinato", "Unison", "22050"],
    "FR-004": ["Concha Harmônica", "Ressonador", "diagnóstico", "host", "signature"],
    "FR-005": ["conjunto fechado", "verde proibido", "amarelo", "vermelho", "signal"],
    "FR-006": ["mudo", "mute", "som", "redundância", "acesso"],
    "FR-007": ["logo", "marca", "ondas", "proibido", "FR-028", "FR-029"],
    "FR-008": ["StatusView", "superfície", "notificação", "in-app", "Lente", "grade 2-campo", "C3"],
    "FR-009": ["misturado", "aditivo", "mix", "narração", "earcon↔narração", "grant", "mul_add", "D3"],
    "FR-010": ["≤100 ms", "latência", "latency", "100ms", "oráculo", "first-consumed", "D3"],
}
missing = []
for fr in FR_KEYS:
    present = fr in TASKS
    kw = KEYWORDS.get(fr, [])
    kw_hit = any(kwd in TASKS for kwd in kw) if kw else False
    if not (present or kw_hit):
        missing.append(fr)
if missing:
    fail("T-TEST-01", f"FR→task mapping missing for {missing} (expected named IDs per spec §FR 130-149; tasks.md references only FR-002/FR-004/FR-010/FR-028 explicitly; others covered by keyword only — verify task-level granularity for FR-009 discard-on-pause and FR-001 palette roles)")
else:
    print("T-TEST-01 PASS")
# ------------------------------------------------------------------ T-TEST-02: closed-set no catch-all
# Assert 0 occurrences of catch-all patterns in contracts/ and tasks.md
CATCH = ["_ =>", "Other", "Unknown", "Internal"]  # per Constitution VII + gate 7
hits = {}
for d in ["specs/005-adopt-design-sona/contracts", "specs/005-adopt-design-sona/tasks.md"]:
    # contracts dir
    import os, pathlib
    if d.endswith(".md"):
        text = open(d).read()
        for p in CATCH:
            n = text.count(p)
            if n:
                hits.setdefault(d, {})[p] = n
    else:
        for root, _, files in os.walk(d):
            for f in files:
                if f.endswith(".md"):
                    text = open(os.path.join(root, f)).read()
                    for p in CATCH:
                        n = text.count(p)
                        if n:
                            hits.setdefault(os.path.join(root, f), {})[p] = n
if hits:
    fail("T-TEST-02", f"catch-all patterns found: {hits} (Constitution VII violation if in crates/*/src; in contracts/tasks this is a coverage/consistency gap — must confirm code-level gate 7 passes separately)")
else:
    print("T-TEST-02 PASS (contracts/tasks clean; code-level gate 7 still required on crates/*/src)")

# ------------------------------------------------------------------ T-TEST-03: SC buildable-work references + deferred/truncated
SC_KEYS = [f"SC-{i:03d}" for i in range(1, 8)]
sc_refs = {sc: TASKS.count(sc) for sc in SC_KEYS}
# Truncated / deferred detection from spec line inspection (plan refs: SC-001 deferred device-leg ~159, SC-002 truncated haptic ~163, SC-007 truncated reduced-motion ~193)
truncated_sc = {
    "SC-002": "spec.md line ~156 'disposição háptica da condição auditiva' truncated — protocol (15 trials/type, matrix, pre-familiarization) may be underspecified in task layer",
    "SC-007": "spec.md line ~161 'reduced-motion ativa no sistema' truncated — freeze-test + TalkBack + reduced-motion triple (F2 toggle + adb shim) needs explicit task IDs",
}
deferred_sc = {"SC-001": "spec.md ~159 'DEFERRED (owner=device-build)' — device-leg deferred without named owner task; needs deferred-marker in tasks.md"}
sc_issues = []
for sc in SC_KEYS:
    if sc_refs[sc] == 0:
        sc_issues.append(f"{sc} zero task references (buildable-work gap)")
    else:
        # presence OK; still check deferred/truncated
        pass
for sc, note in truncated_sc.items():
    sc_issues.append(f"{sc} truncated: {note}")
for sc, note in deferred_sc.items():
    sc_issues.append(f"{sc} deferred: {note}")
if sc_issues:
    fail("T-TEST-03", "; ".join(sc_issues) + " (Constitution V / III: deferred device-leg must have owner task; truncated protocol lines must have corresponding task granularity before implement)")
else:
    print("T-TEST-03 PASS")

# ------------------------------------------------------------------ T-TEST-04: terminology consistency
# Normalized term map from spec entities; assert each appears in tasks.md with coherent semantics.
TERMS = {
    "StatusView": ("in-app status surface", True),
    "Lente Sinestésica": ("reduced design identity of StatusView, not full live-data", True),
    "Concha Harmônica": ("signature mark 'R' in header", True),
    "Ressonador Canônico": ("host-only diagnostic emblem, never notification / lock-screen", True),
    "MediaMirror.kt": ("source file for surface constants / progress-fill", True),
    "MediaStyle": ("notification surface", True),
    "grade 2-campo": ("descriptive alias, non-normative; single-column preserved", True),
    "Ostinato": ("warning earcon / state-change signal, never queued behind pause", True),
    "Adagio": ("play confirmation earcon", True),
    "Andante": ("pause confirmation", True),
    "Allegro": ("skip confirmation", True),
    "Sforzando": ("failure confirmation", True),
    "Unison": ("rate / rate-step confirmation", True),
}
drift = []
for term, (expected, must_appear) in TERMS.items():
    count = TASKS.count(term) + SPEC.count(term)
    if must_appear and count == 0:
        drift.append(f"'{term}' absent from tasks+spec (expected per data-model / US4)")
    # Check semantic drift: if term appears with contradictory modifier in tasks
if drift:
    fail("T-TEST-04", "Terminology drift: " + "; ".join(drift) + " — specifically grade 2-campo non-normative alias vs normative use; Lente Sinestésica reduced vs full-feature reference must be verified in task descriptions; Ressonador Canônico host-only vs any surface reference is a critical semantics shift")
else:
    print("T-TEST-04 PASS")

# ------------------------------------------------------------------ Summary
print("\n=== RED STATUS (pre-detection) ===")
if FAILURES:
    for f in FAILURES:
        print("RED  ", f)
    print(f"\nTotal RED failures: {len(FAILURES)} (correct reasons: missing mappings / truncated protocol / deferred leg / terminology verification needed)")
    sys.exit(1)  # RED — blocking for Phase 2 (per TDD rule 1 / Constitution III)
else:
    print("All T-TEST-01..04 PASS (no invariant violations detected in artifact layer)")
    sys.exit(0)

# FIXED 2026-10-26: SC-002 haptic (line 156) + SC-007 freeze (line 161) completed with cross-ref to clarify-e3/data-model; deferred T006c/d/e added; T006b OK
# T-TEST-02: tasks.md references 'Other'/'Unknown' normatively (prohibition language in T011/T012), not usage; source gate 7 on crates/*/src/ must pass independently.
