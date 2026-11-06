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


# -----------------------------------------------------------------------------
# T063 RED: FR-010 release-exposure mechanization invariants (Spec 012).
# These tests intentionally fail against the skeleton/placeholder script until
# static inspection, runtime negative invocation, and full D1-D4 mechanization
# bind real evidence (not "not fully proven"/blocked fake-pass shapes).
# -----------------------------------------------------------------------------

T063_IN_SCOPE_IDENTIFIERS = tuple(IN_SCOPE)
T063_PLACEHOLDER_MARKERS = (
    "not fully proven",
    "not proven",
    "not performed",
    "placeholder",
    "skeleton",
)


def _write_contract_shaped_manifest(path: Path, *, include_debug_hook: bool = False) -> Path:
    # T063 RED helper: representative merged/packaged manifest fixture (not authoritative APK bytes).
    hook_bits = (
        '\n    <activity android:name="app.mdreader.DebugIngestActivity" android:exported="true">'
        '\n      <intent-filter><action android:name="app.mdreader.action.DEBUG_INGEST"/></intent-filter>'
        "\n    </activity>"
        if include_debug_hook
        else ""
    )
    path.write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n'
        '<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="app.mdreader">'
        f"{hook_bits}\n"
        "</manifest>\n",
        encoding="utf-8",
    )
    return path


def _write_contract_shaped_merger_report(path: Path, *, include_debug_hook: bool = False) -> Path:
    # T063 RED helper: representative manifest merger report fixture.
    lines = [
        "Manifest merger report",
        "variant: universalRelease",
        "package: app.mdreader",
    ]
    if include_debug_hook:
        lines.extend(
            [
                "ADDED from debug overlay: DebugIngestActivity",
                "ADDED intent-filter action app.mdreader.action.DEBUG_INGEST",
            ]
        )
    else:
        lines.append("No debug ingest overlay materialized for release variant")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def _write_representative_apk(path: Path) -> Path:
    # T063 RED helper: fake-but-contract-shaped release artifact path; D2 must still
    # inspect provided manifest/merger/report inputs and bind provenance, not claim proof from bytes.
    path.write_bytes(b"PK\x03\x04T063-representative-release-apk-fixture")
    return path


def _assert_tool_provenance_bound(data: dict, *, required_tools: tuple[str, ...] = ()) -> None:
    # T063 RED invariant: evidence must bind tool/path/provenance for requested inspection/invocation.
    provenance = data.get("tool_provenance")
    assert isinstance(provenance, dict) and provenance, "T063: tool_provenance must be a bound mapping"
    for tool in required_tools:
        rec = provenance.get(tool)
        assert isinstance(rec, dict), f"T063: tool_provenance missing record for {tool}"
        assert "path" in rec or "available" in rec, f"T063: {tool} provenance must expose path/availability"


def _assert_static_inspection_mechanized(data: dict, *, expect_paths: list[Path]) -> None:
    # T063 RED invariant: --mode static with representative inputs must perform/record actual
    # merged/packaged manifest, APK-AAB, and merger-report inspection for in-scope identifiers.
    d2 = data.get("d2_static_absence")
    assert isinstance(d2, dict), "T063: d2_static_absence required"
    summary = json.dumps(d2, sort_keys=True).lower()
    assert not any(marker in summary for marker in T063_PLACEHOLDER_MARKERS), (
        "T063: static D2 must record actual inspection evidence, not placeholder 'not fully proven'"
    )
    assert d2.get("status") in {"pass", "fail"}, "T063: static D2 must resolve from actual inspection"
    inspection_flag = d2.get("inspection_performed", data.get("inspection_performed"))
    inspected = (
        d2.get("inspected_artifacts")
        or d2.get("artifact_inspections")
        or d2.get("inspections")
        or data.get("inspected_artifacts")
    )
    findings = d2.get("identifier_findings") or d2.get("identifier_scan") or data.get("identifier_findings")
    assert inspection_flag is True, "T063: static inspection must set inspection_performed=true"
    assert isinstance(inspected, (dict, list)) and inspected, (
        "T063: static inspection must bind inspected merged/packaged manifest/APK-AAB/merger-report artifacts"
    )
    inspected_text = json.dumps(inspected, sort_keys=True)
    for p in expect_paths:
        assert str(p) in inspected_text, f"T063: inspected artifacts must bind path {p}"
    assert isinstance(findings, (dict, list)) and findings, (
        "T063: static inspection must record per-identifier scan findings"
    )
    findings_text = json.dumps(findings, sort_keys=True)
    for ident in T063_IN_SCOPE_IDENTIFIERS:
        assert ident in findings_text, f"T063: static inspection missing in-scope identifier finding for {ident}"
    tool_bound = (
        d2.get("tool_path")
        or d2.get("tool")
        or d2.get("tools_used")
        or d2.get("inspection_tool_provenance")
        or data.get("inspection_tool_provenance")
    )
    assert tool_bound, "T063: static inspection must bind tool/path/provenance used for artifact inspection"


@given(include_debug_hook=st.booleans())
@settings(max_examples=6, deadline=None)
def test_t063_red_static_mode_requires_actual_artifact_inspection_with_representative_inputs(
    tmp_path_factory: pytest.TempPathFactory, include_debug_hook: bool
) -> None:
    """T063 RED: --mode static with valid/representative release artifact inputs must perform/record
    actual merged/packaged manifest/APK-AAB/merger-report inspection for in-scope identifiers and bind
    tool/path/provenance (not just "not fully proven" placeholder). Fail-closed invariants remain.
    """
    tmp_path = tmp_path_factory.mktemp("t063_static")
    apk = _write_representative_apk(tmp_path / "universalRelease.apk")
    manifest = _write_contract_shaped_manifest(
        tmp_path / "AndroidManifest.merged.xml", include_debug_hook=include_debug_hook
    )
    merger = _write_contract_shaped_merger_report(
        tmp_path / "manifest-merger-universalRelease-report.txt", include_debug_hook=include_debug_hook
    )
    proc = run_script(
        [
            "--mode",
            "static",
            "--apk",
            str(apk),
            "--merged-manifest",
            str(manifest),
            "--merger-report",
            str(merger),
            "--variant",
            "universalRelease",
            "--json",
        ]
    )
    data = parse_json(proc)
    assert data["mode"] == "static"
    assert data["satisfies_fr010_sc004"] is False
    assert data.get("overall") != "pass" or data["satisfies_fr010_sc004"] is False
    artifact_identity = data.get("artifact_identity") or {}
    assert artifact_identity.get("apk_path") == str(apk), "T063: static artifact identity must bind APK path"
    assert artifact_identity.get("apk_sha256"), "T063: static artifact identity must bind APK hash"
    assert str(manifest) in json.dumps(artifact_identity, sort_keys=True), "T063: merged manifest path must be bound"
    assert str(merger) in json.dumps(artifact_identity, sort_keys=True), "T063: merger report path must be bound"
    _assert_tool_provenance_bound(data)
    _assert_static_inspection_mechanized(data, expect_paths=[apk, manifest, merger])
    if include_debug_hook:
        assert data.get("d2_static_absence", {}).get("status") == "fail", (
            "T063: static inspection of hook-present representative inputs must fail D2"
        )


@given(serial=st.text(min_size=3, max_size=40).filter(lambda s: s.strip() and "\n" not in s and "\x00" not in s))
@settings(max_examples=6, deadline=None)
def test_t063_red_runtime_mode_requires_negative_adb_invocation_evidence_shape(serial: str) -> None:
    """T063 RED: --mode runtime with required inputs must exercise explicit negative ADB invocation path
    contract (injectable runner/fake harness acceptable if script supports it; else required command
    evidence shape blocks fake pass) and verify no hook success/check-caused crash-ANR evidence fields.
    """
    proc = run_script(
        [
            "--mode",
            "runtime",
            "--serial",
            serial,
            "--package",
            "app.mdreader",
            "--component",
            "app.mdreader/.DebugIngestActivity",
            "--action",
            "app.mdreader.action.DEBUG_INGEST",
            "--json",
        ]
    )
    data = parse_json(proc)
    assert data["mode"] == "runtime"
    assert data["satisfies_fr010_sc004"] is False
    assert data.get("overall") in {"fail", "blocked-not-evaluated", "partial", "pass-non-full"}
    d3 = data.get("d3_runtime_negative")
    assert isinstance(d3, dict), "T063: d3_runtime_negative required"
    _assert_tool_provenance_bound(data, required_tools=("adb",))
    install = data.get("install_identity") or {}
    assert install.get("serial") == serial, "T063: runtime install identity must bind requested serial"
    assert install.get("package_id") == "app.mdreader"
    assert d3.get("invocation_attempted") is True, (
        "T063: runtime must record an explicit negative ADB invocation attempt or fail closed with mechanized evidence"
    )
    command_evidence = d3.get("adb_command") or d3.get("command") or d3.get("commands") or d3.get("invocation_command")
    assert command_evidence, "T063: runtime must bind explicit ADB/component/action invocation command evidence"
    command_text = json.dumps(command_evidence, sort_keys=True)
    assert serial in command_text or serial in json.dumps(d3, sort_keys=True)
    assert "DebugIngestActivity" in command_text or "app.mdreader/.DebugIngestActivity" in json.dumps(d3, sort_keys=True)
    assert "DEBUG_INGEST" in command_text or "DEBUG_INGEST" in json.dumps(d3, sort_keys=True)
    assert "hook_success_detected" in d3, "T063: runtime D3 must explicitly report hook_success_detected"
    assert d3.get("hook_success_detected") is not True, "T063: negative runtime check must not report hook success"
    crash_field = (
        d3.get("crash_anr_detected")
        if "crash_anr_detected" in d3
        else d3.get("check_caused_crash_anr")
        if "check_caused_crash_anr" in d3
        else d3.get("crash_anr_status")
    )
    assert crash_field is not None, "T063: runtime D3 must explicitly report crash/ANR caused by negative check"
    assert crash_field not in {True, "detected", "crash", "anr"}, (
        "T063: negative runtime check must not report check-caused crash/ANR as detected"
    )
    if data.get("overall") in {"pass", "pass-non-full"}:
        assert d3.get("status") == "pass"
        assert d3.get("negative_invocation_confirmed") is True or d3.get("hook_unreachable_confirmed") is True


@given(variant=st.sampled_from(["universalRelease", "arm64Release", "release"]))
@settings(max_examples=4, deadline=None)
def test_t063_red_full_mode_requires_d1_d4_mechanized_evidence_and_identity_binding(
    tmp_path_factory: pytest.TempPathFactory, variant: str
) -> None:
    """T063 RED: --mode full may pass only when D1-D4 pass with bound artifact/install identity.
    Representative inputs must drive mechanized static/runtime evidence, never fake pass.
    Fail-closed/blocked-not-evaluated invariants preserved for missing/insufficient proof.
    """
    tmp_path = tmp_path_factory.mktemp("t063_full")
    apk = _write_representative_apk(tmp_path / f"{variant}.apk")
    manifest = _write_contract_shaped_manifest(tmp_path / "AndroidManifest.merged.xml")
    merger = _write_contract_shaped_merger_report(tmp_path / f"manifest-merger-{variant}-report.txt")
    serial = "adb-53cf9aa7-T063"
    proc = run_script(
        [
            "--mode",
            "full",
            "--apk",
            str(apk),
            "--merged-manifest",
            str(manifest),
            "--merger-report",
            str(merger),
            "--variant",
            variant,
            "--serial",
            serial,
            "--package",
            "app.mdreader",
            "--component",
            "app.mdreader/.DebugIngestActivity",
            "--action",
            "app.mdreader.action.DEBUG_INGEST",
            "--json",
        ]
    )
    data = parse_json(proc)
    assert data["mode"] == "full"
    for dim in ("d1_scope", "d2_static_absence", "d3_runtime_negative", "d4_repeatable"):
        assert dim in data, f"T063: full mode missing {dim}"
    _assert_tool_provenance_bound(data)
    _assert_static_inspection_mechanized(data, expect_paths=[apk, manifest, merger])
    d3 = data["d3_runtime_negative"]
    assert isinstance(d3, dict)
    assert d3.get("invocation_attempted") is True, "T063: full mode must carry mechanized runtime invocation evidence"
    assert d3.get("adb_command") or d3.get("command") or d3.get("commands") or d3.get("invocation_command")
    assert "hook_success_detected" in d3
    crash_field = d3.get("crash_anr_detected", d3.get("check_caused_crash_anr", d3.get("crash_anr_status")))
    assert crash_field is not None
    d4 = data["d4_repeatable"]
    assert isinstance(d4, dict)
    assert d4.get("status") in {"pass", "fail", "blocked-not-evaluated"}
    if data.get("overall") == "pass":
        assert data.get("satisfies_fr010_sc004") is True
        assert data.get("artifact_identity"), "T063: full pass requires bound artifact identity"
        assert data.get("install_identity"), "T063: full pass requires bound install identity"
        for dim in ("d1_scope", "d2_static_absence", "d3_runtime_negative", "d4_repeatable"):
            assert data[dim].get("status") == "pass", f"T063: full pass requires {dim}=pass"
        assert (data.get("install_identity") or {}).get("verified_release_non_debug") is True
    else:
        assert data.get("satisfies_fr010_sc004") is False
        assert data.get("overall") in {"fail", "blocked-not-evaluated", "partial"}


@given(
    mode=st.sampled_from(["static", "runtime", "full"]),
    with_merged_manifest=st.booleans(),
    with_merger=st.booleans(),
)
@settings(max_examples=8, deadline=None)
def test_t063_red_never_claims_fr010_pass_without_mechanized_full_evidence(
    tmp_path_factory: pytest.TempPathFactory, mode: str, with_merged_manifest: bool, with_merger: bool
) -> None:
    """T063 RED invariant: no mode may claim satisfies_fr010_sc004 without mechanized full D1-D4
    evidence bound to artifact/install identity. Representative inputs must not enable fake pass.
    """
    tmp_path = tmp_path_factory.mktemp(f"t063_{mode}")
    args: list[str] = ["--mode", mode, "--json", "--package", "app.mdreader"]
    if mode in {"static", "full"}:
        apk = _write_representative_apk(tmp_path / "u.apk")
        args.extend(["--apk", str(apk), "--variant", "universalRelease"])
        if with_merged_manifest:
            mf = _write_contract_shaped_manifest(tmp_path / "merged.xml")
            args.extend(["--merged-manifest", str(mf)])
        if with_merger:
            mr = _write_contract_shaped_merger_report(tmp_path / "merger.txt")
            args.extend(["--merger-report", str(mr)])
    if mode in {"runtime", "full"}:
        args.extend(
            [
                "--serial",
                "adb-t063-invariant",
                "--component",
                "app.mdreader/.DebugIngestActivity",
                "--action",
                "app.mdreader.action.DEBUG_INGEST",
            ]
        )
    data = parse_json(run_script(args))
    if mode != "full":
        assert data.get("satisfies_fr010_sc004") is False
        return
    if data.get("satisfies_fr010_sc004") is True or data.get("overall") == "pass":
        assert all((data.get(dim) or {}).get("status") == "pass" for dim in ("d1_scope", "d2_static_absence", "d3_runtime_negative", "d4_repeatable"))
        assert data.get("artifact_identity")
        assert data.get("install_identity")
        assert (data.get("install_identity") or {}).get("verified_release_non_debug") is True
    else:
        assert data.get("satisfies_fr010_sc004") is False
