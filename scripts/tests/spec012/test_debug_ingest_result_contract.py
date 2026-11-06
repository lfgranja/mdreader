"""Spec 012 debug ingest result/corroboration/FR-008 contract tests (T009/T014/T021).

TDD RED: validates parseable result grammar and evidence invariants; expected to fail
where script/contract helpers are not yet implemented.
"""
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
HOOK_SCRIPT = REPO / "scripts" / "automate-t184-t226-hook.sh"
RESULT_RE = re.compile(
    r"^(ok:document:.+|ok:uri:.+|ok:directory:.+:chapters=\d+|error:[^:]+:.+|refused:not-debug)$"
)
FR008_FIELDS = {
    "fixture_reference",
    "operation",
    "outcome",
    "build_identity",
    "device_identity",
    "diagnostic_capture",
    "crash_anr_status",
    "residual_scope_note",
    "preferred_location_or_channel",
    "effective_location_or_channel",
    "write_status",
    "reviewer_accessible_location",
    "retention_scope",
}


def run_hook(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(HOOK_SCRIPT), *args], cwd=REPO, text=True, capture_output=True, check=False)


@given(
    st.sampled_from(["document", "uri", "directory"]),
    st.text(alphabet="abcdefghijklmnopqrstuvwxyz0123456789/_-.", min_size=1, max_size=40),
    st.integers(min_value=0, max_value=500),
)
@settings(max_examples=60, deadline=None)
def test_valid_result_prefixes_parse(mode: str, ref: str, chapters: int) -> None:
    if mode == "document":
        s = f"ok:document:/{ref}"
    elif mode == "uri":
        s = f"ok:uri:file:///{ref}"
    else:
        s = f"ok:directory:/{ref}:chapters={chapters}"
    assert RESULT_RE.match(s), s


@given(st.text(min_size=0, max_size=80).filter(lambda s: not RESULT_RE.match(s)))
@settings(max_examples=60, deadline=None)
def test_invalid_result_strings_are_not_explicit_success(s: str) -> None:
    # invariant: non-matching strings cannot be treated as explicit pass evidence
    assert not s.startswith("ok:"), f"unexpected ok prefix slipped through generator: {s!r}"


@pytest.mark.parametrize("alias", ["file", "open_document", "DOCUMENT", "dir", "tree", "open_directory"])
def test_alias_matrix_normalization_contract(alias: str) -> None:
    # RED: hook script must expose deterministic normalization/reporting per contract.
    proc = run_hook(["--help"])
    assert proc.returncode == 0
    text = (proc.stdout + proc.stderr).lower()
    assert alias.lower() in text or "mode" in text, "script must document or enforce normalized aliases"


def test_hook_reports_corroboration_and_fr008_fields_in_json_mode() -> None:
    proc = run_hook(["--help", "--json"])
    # RED until implemented: help/json contract should be machine-readable or script provides --print-contract/--json behavior.
    data = json.loads((proc.stdout or proc.stderr).strip())
    assert data.get("scope_class") == "programmatic-non-ui"
    missing = FR008_FIELDS - set(data.keys())
    assert not missing, f"missing FR-008/report fields: {sorted(missing)}"
    assert data.get("write_status") in {"written", "unavailable"} or str(data.get("write_status", "")).startswith("failed:")


def test_directory_chapters_count_is_explicit_and_filtering_aware() -> None:
    proc = run_hook(["--print-contract"])
    data = json.loads((proc.stdout or proc.stderr).strip())
    d = data.get("directory_result") or {}
    assert "chapters" in json.dumps(d)
    assert "expected_filtering" in json.dumps(d) or "fixture_composition" in json.dumps(d)


def test_precondition_blocked_is_not_counted_as_explicit_pass() -> None:
    proc = run_hook(["--classify-outcome", "blocked:fixture-inaccessible"])
    data = json.loads((proc.stdout or proc.stderr).strip())
    assert data["status"] == "blocked"
    assert data["counts_toward_sc002_denominator"] is False
    assert data["explicit_within_budget"] is False


def test_inconclusive_is_not_explicit_until_rerun_resolves() -> None:
    proc = run_hook(["--classify-outcome", "inconclusive:missing-authoritative-channel"])
    data = json.loads((proc.stdout or proc.stderr).strip())
    assert data["status"] == "inconclusive"
    assert data["explicit_within_budget"] is False
