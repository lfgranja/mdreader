"""Spec 012 FR-012 / FR-009 / SC-003 residual split contract tests (T011/T026/T027/T034).

TDD RED: reporting/update helpers and task-file invariants not fully implemented.
"""
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]
HOOK = REPO / "scripts" / "automate-t184-t226-hook.sh"
T184 = REPO / "specs/009-open-file-from-title/tasks.md"
T226 = REPO / "specs/010-directory-reading/tasks.md"
SMOKE = REPO / "specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt"
REQUIRED_RESIDUALS = {
    "T184-PROG",
    "T184-RES-SAF-CANCEL",
    "T184-RES-A11Y",
    "T226-PROG",
    "T226-RES-SAF-TREE",
    "T226-RES-TRANSITION",
    "T226-RES-RESUME-UI",
    "T226-RES-HARDWARE-CONTROLS",
    "T226-RES-INVALID-ENTRIES-UI",
    "T226-RES-A11Y-PICKER",
}
ALLOWED_STATUS = {"open", "verified", "accepted-risk", "deferred"}


def run_hook(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(HOOK), *args], cwd=REPO, text=True, capture_output=True, check=False)


def parse_json(proc: subprocess.CompletedProcess[str]) -> dict:
    return json.loads((proc.stdout or proc.stderr).strip())


def test_report_schema_marks_programmatic_scope_and_no_full_completion() -> None:
    proc = run_hook(["--print-report-schema", "--json"])
    data = parse_json(proc)
    assert data.get("full_parent_completion_claimed") is False
    assert data.get("scope_class") == "programmatic-non-ui"
    for k in ("preferred_location_or_channel", "effective_location_or_channel", "write_status", "residual_scope_note", "crash_anr_status"):
        assert k in data, f"missing FR-008/report field {k}"


def test_sc001_sc002_counters_and_formulae_present() -> None:
    proc = run_hook(["--print-report-schema", "--json"])
    data = parse_json(proc)
    counters = data.get("counters") or data
    for k in ("attempted", "precondition_blocked", "in_denominator", "explicit_within_budget", "inconclusive"):
        assert k in counters
    formulae = json.dumps(data).lower()
    assert "sc-001" in formulae and "sc-002" in formulae
    assert "95" in formulae or "0.95" in formulae


def test_residual_lifecycle_rules() -> None:
    proc = run_hook(["--print-residual-rules", "--json"])
    data = parse_json(proc)
    assert set(data.get("allowed_statuses") or []) == ALLOWED_STATUS
    ar = data.get("accepted_risk_requires") or []
    text = " ".join(ar).lower() if isinstance(ar, list) else str(ar).lower()
    for req in ("owner", "decision", "rationale", "scope", "evidence"):
        assert req in text
    assert data.get("accepted_risk_completes_parent_when_blocking") is False


def test_t184_t226_task_files_contain_required_split_ids_and_no_overclaim() -> None:
    for path, prog, residuals in [
        (T184, "T184-PROG", ["T184-RES-SAF-CANCEL", "T184-RES-A11Y"]),
        (T226, "T226-PROG", ["T226-RES-SAF-TREE", "T226-RES-TRANSITION", "T226-RES-RESUME-UI", "T226-RES-HARDWARE-CONTROLS", "T226-RES-INVALID-ENTRIES-UI", "T226-RES-A11Y-PICKER"]),
    ]:
        text = path.read_text(encoding="utf-8")
        assert prog in text, f"missing {prog} in {path}"
        for r in residuals:
            assert r in text, f"missing {r} in {path}"
        assert "full_parent_completion_claimed=false" in text.replace(" ", "") or "hook-only" in text.lower()


def test_androidtest_smoke_scaffold_is_static_contract_only() -> None:
    """T034: canonical smoke source must not claim manual SAF picker/UI traversal."""
    text = SMOKE.read_text(encoding="utf-8")
    assert "DebugIngest" in text
    lower = text.lower()
    assert "manual saf" in lower or "no manual" in lower
    assert "picker ui" in lower or "ui traversal" in lower
    assert "DebugIngestActivity" in text or "debug_ingest" in lower or "DEBUG_INGEST" in text
