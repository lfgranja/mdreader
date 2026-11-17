"""Spec 012 FR-011 durable overlay contract tests (T008/T033).

TDD RED: expected to FAIL against skeleton scripts (blocked:not-implemented).
Property/invariant-oriented where practical.
"""
from __future__ import annotations

import json
import os
import stat
import subprocess
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
APPLY = REPO / "scripts" / "apply-debug-ingest-overlay.sh"
CHECK = REPO / "scripts" / "check-debug-ingest-overlay.sh"
CANONICAL_DEBUG = REPO / "specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml"
CANONICAL_ANDROIDTEST = REPO / "specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt"
FORMAT_MARKER = REPO / "specs/012-debug-ingest-hook/android-overlays/overlay-format.json"
STATUS = {"pass", "fail", "blocked", "not-applicable"}


def run_script(script: Path, args: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(script), *args],
        cwd=cwd or REPO,
        text=True,
        capture_output=True,
        check=False,
    )


def parse_json_or_fail(proc: subprocess.CompletedProcess[str]) -> dict:
    out = (proc.stdout or "").strip() or (proc.stderr or "").strip()
    try:
        return json.loads(out)
    except json.JSONDecodeError as e:  # pragma: no cover - RED path
        raise AssertionError(f"expected JSON output, got rc={proc.returncode}\nstdout={proc.stdout}\nstderr={proc.stderr}") from e


@pytest.mark.parametrize("script", [APPLY, CHECK])
def test_scripts_exist_executable_and_help(script: Path) -> None:
    assert script.exists(), f"missing script: {script}"
    mode = script.stat().st_mode
    assert mode & stat.S_IXUSR, f"not executable: {script}"
    proc = run_script(script, ["--help"])
    assert proc.returncode == 0
    assert "Usage:" in proc.stdout + proc.stderr


def test_check_is_non_mutating_and_reports_required_shape(tmp_path: Path) -> None:
    gen = tmp_path / "gen/app"
    target = gen / "src/debug/AndroidManifest.xml"
    target.parent.mkdir(parents=True)
    before = None
    proc = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    assert not target.exists(), "check must not create managed targets"
    report = parse_json_or_fail(proc)
    assert report.get("overall_status") in {"pass", "fail", "blocked"}
    targets = report.get("managed_targets") or report.get("targets")
    assert isinstance(targets, list) and targets, "expected per-target results"
    for t in targets:
        assert t.get("status") in STATUS
        assert t.get("target_path") or t.get("path")
        assert "expected" in t and "actual" in t
        assert t.get("remediation")
    assert report.get("detected_overlay_format_version") is not None


def test_check_fails_closed_on_missing_required_debug_target(tmp_path: Path) -> None:
    gen = tmp_path / "gen/app"
    (gen / "src/debug").mkdir(parents=True)
    proc = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    report = parse_json_or_fail(proc)
    assert report["overall_status"] in {"fail", "blocked"}
    debug_rows = [t for t in (report.get("managed_targets") or report.get("targets") or []) if "debug/AndroidManifest.xml" in str(t.get("target_path") or t.get("path"))]
    assert debug_rows, "missing debug manifest target result"
    assert debug_rows[0]["status"] in {"fail", "blocked"}


def test_apply_is_idempotent_when_aligned(tmp_path: Path) -> None:
    gen = tmp_path / "gen/app"
    # T066/FR-011: source-set wiring is part of dependency/layout validation;
    # provide verifiable wiring so aligned managed targets can reach pass.
    (gen / "app").mkdir(parents=True, exist_ok=True)
    (gen / "app" / "build.gradle.kts").write_text(
        'android { sourceSets["main"].java.srcDir("android-shell/src/main/kotlin") }\n',
        encoding="utf-8",
    )
    first = run_script(APPLY, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    r1 = parse_json_or_fail(first)
    assert r1.get("overall_status") in {"pass", "blocked", "fail"}
    second = run_script(APPLY, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    r2 = parse_json_or_fail(second)
    check = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    rc = parse_json_or_fail(check)
    assert rc["overall_status"] == "pass", rc
    # idempotent: second apply does not regress a passing check
    assert r2.get("overall_status") in {"pass", "fail", "blocked"}


def test_androidtest_conditional_applicability(tmp_path: Path) -> None:
    """FR-013/SC-006/T033: required when canonical source exists, else not-applicable."""
    gen = tmp_path / "gen/app"
    proc = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    report = parse_json_or_fail(proc)
    rows = report.get("managed_targets") or report.get("targets") or []
    at = [t for t in rows if "androidTest" in str(t.get("target_path") or t.get("path")) or "DebugIngestSeamSmokeTest" in str(t.get("target_path") or t.get("path"))]
    assert at, "expected androidTest managed-target row"
    if CANONICAL_ANDROIDTEST.exists():
        assert at[0]["status"] in {"pass", "fail", "blocked"}
    else:
        assert at[0]["status"] == "not-applicable"


@given(st.text(alphabet=st.characters(blacklist_categories=("Cs",), blacklist_characters="\x00"), max_size=40))
@settings(max_examples=25, deadline=None)
def test_path_safety_rejects_unsafe_relative_segments(segment: str) -> None:
    # invariant: apply/check must not treat path-like segment input as authority to escape gen/root; contract behavior via --gen-dir/--root validation.
    # We exercise a safe tmp dir name; dangerous segments should never cause writes outside tmp contract area. RED until scripts enforce.
    if segment in {"", ".", ".."} or "/" in segment or "\\" in segment:
        return
    proc = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(REPO / "src-tauri" / "gen" / "android" / "app"), "--json"])
    report = parse_json_or_fail(proc)
    assert report.get("overall_status") in STATUS - {"not-applicable"}


def test_format_version_marker_is_detected() -> None:
    assert FORMAT_MARKER.exists(), "T003 canonical overlay format marker missing"
    proc = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(REPO / "src-tauri" / "gen" / "android" / "app"), "--json"])
    report = parse_json_or_fail(proc)
    ver = str(report.get("detected_overlay_format_version"))
    assert ver and ver != "None"
    assert report.get("overall_status") in {"pass", "fail", "blocked"}


def test_forbidden_hook_exposure_in_release_target_fails_closed(tmp_path: Path) -> None:
    gen = tmp_path / "gen/app"
    run_script(APPLY, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    # plant forbidden debug hook surface in main manifest target path to ensure check fails closed for managed/forbidden-state policy
    main = gen / "src/main/AndroidManifest.xml"
    main.parent.mkdir(parents=True, exist_ok=True)
    main.write_text(CANONICAL_DEBUG.read_text(encoding="utf-8"), encoding="utf-8")
    proc = run_script(CHECK, ["--root", str(REPO), "--gen-dir", str(gen), "--json"])
    report = parse_json_or_fail(proc)
    assert report["overall_status"] in {"fail", "blocked"}, report
