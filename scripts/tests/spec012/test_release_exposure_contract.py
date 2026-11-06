"""Spec 012 FR-010 release/non-debug exposure contract tests (T010).

TDD RED against skeleton script.
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
SCRIPT = REPO / "scripts" / "check-debug-ingest-release-exposure.sh"
IN_SCOPE = [
    "DebugIngestActivity",
    "DebugIngestHook",
    "app.mdreader.action.DEBUG_INGEST",
]


def run_script(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(SCRIPT), *args], cwd=REPO, text=True, capture_output=True, check=False)


def parse_json(proc: subprocess.CompletedProcess[str]) -> dict:
    raw = (proc.stdout or proc.stderr).strip()
    return json.loads(raw)


def test_help_exposes_modes_and_fail_closed_contract() -> None:
    proc = run_script(["--help"])
    assert proc.returncode == 0
    text = proc.stdout + proc.stderr
    assert "--mode" in text
    assert "static|runtime|full" in text
    assert "blocked" in text.lower()


@pytest.mark.parametrize("mode", ["static", "runtime"])
def test_non_full_modes_never_satisfy_fr010(mode: str) -> None:
    proc = run_script(["--mode", mode, "--json"])
    data = parse_json(proc)
    assert data["mode"] == mode
    assert data["satisfies_fr010_sc004"] is False
    assert data["overall"] in {"fail", "partial", "blocked-not-evaluated", "pass-non-full"}
    assert data["overall"] != "pass" or data["satisfies_fr010_sc004"] is False


def test_full_mode_blocked_without_required_inputs() -> None:
    proc = run_script(["--mode", "full", "--json"])
    data = parse_json(proc)
    assert data["mode"] == "full"
    assert data["overall"] == "blocked-not-evaluated"
    assert data["satisfies_fr010_sc004"] is False
    b = data.get("blocked_not_evaluated") or {}
    assert b.get("missing_inputs")
    assert "attempted_modes" in b


def test_full_mode_requires_d1_d4_and_identity_binding(tmp_path: Path) -> None:
    apk = tmp_path / "fake.apk"
    apk.write_bytes(b"not-a-real-apk")
    proc = run_script(["--mode", "full", "--apk", str(apk), "--serial", "fake-serial", "--json"])
    data = parse_json(proc)
    for d in ("d1_scope", "d2_static_absence", "d3_runtime_negative", "d4_repeatable"):
        assert d in data, f"missing {d}"
    if data.get("overall") == "pass":
        assert data["satisfies_fr010_sc004"] is True
        assert data.get("artifact_identity")
        assert data.get("install_identity")
        assert all(data[d].get("status") == "pass" for d in ("d1_scope", "d2_static_absence", "d3_runtime_negative", "d4_repeatable"))


def test_class_presence_note_required_when_class_scan_runs(tmp_path: Path) -> None:
    apk = tmp_path / "fake.apk"
    apk.write_bytes(b"x")
    proc = run_script(["--mode", "static", "--apk", str(apk), "--json"])
    data = parse_json(proc)
    if data.get("class_scan_ran"):
        assert "class_presence_note" in data and "insufficient" in data["class_presence_note"].lower()


@given(st.sampled_from(IN_SCOPE))
@settings(max_examples=10, deadline=None)
def test_in_scope_identifiers_are_reported(identifier: str) -> None:
    proc = run_script(["--print-scope", "--json"])
    data = parse_json(proc)
    scope = json.dumps(data)
    assert identifier in scope
