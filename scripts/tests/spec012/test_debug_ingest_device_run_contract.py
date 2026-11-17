"""RED contract tests for Spec 012 T018/T024/T067 — normal device-run evidence
wiring contract.

Host-testable only. No adb/emulator/device required. Tests intentionally fail
until `scripts/automate-t184-t226-hook.sh` adds deterministic host contract modes
for normal device-run evidence emission/wiring (e.g. `--print-device-run-contract`
or `--classify-device-run` with `--result-file`, `--crash-anr-status`,
`--correlation-key`, etc.) without weakening existing host contract modes.

Required semantics (FR-006/FR-008/FR-012/SC-001/SC-002):
- full_parent_completion_claimed always false (programmatic non-UI scope only)
- scope_class/evidence_scope explicitly programmatic-non-ui
- residual_scope_note explicit
- FR-008 minimum attributable fields present
- crash_anr_status=unknown or detected never counts as explicit pass/success
- blocked:fixture-inaccessible/inconclusive/busy/cold-start explicit and non-pass
- SC-001/SC-002 counters/classification present
- preferred/effective channel/location, write_status, corroboration,
  reviewer_accessible_location, retention_scope, correlation/build/device/diagnostic identities
  (or explicit unavailable)
"""
from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
HOOK = REPO / "scripts" / "automate-t184-t226-hook.sh"

RAW_OUTCOMES = st.sampled_from(
    [
        "ok:document:/tmp/f.md",
        "ok:directory:/tmp/d:chapters=1",
        "blocked:fixture-inaccessible",
        "blocked:busy",
        "blocked:cold-start",
        "inconclusive",
        "error:io:read",
        "refused:not-debug",
    ]
)
CACHE = st.sampled_from(["written", "unavailable", "failed:permission", "failed:io"])
CORR = st.sampled_from(
    ["authoritative-only", "corroborated", "none", "conflicting", "blocked", "inconclusive"]
)
OPS = st.sampled_from(["document", "directory", "uri"])
CRASH = st.sampled_from(["not-detected", "detected", "unknown"])


def run_hook(args):
    return subprocess.run(
        [str(HOOK), *args], cwd=REPO, text=True, capture_output=True, check=False
    )


def parse_json(proc):
    raw = (proc.stdout or proc.stderr).strip()
    assert raw, f"empty output rc={proc.returncode} stderr={proc.stderr[:300]}"
    try:
        return json.loads(raw)
    except Exception as e:
        raise AssertionError(f"not json rc={proc.returncode}: {raw[:200]} err={e}")


def is_pass_like(v):
    s = str(v or "").lower()
    if s in {"pass", "passed", "success", "ok", "explicit-pass", "true", "explicit_ok"}:
        return True
    if s.startswith("ok:"):
        return True
    return False


def _base_device_run_args(raw_outcome, cache, corr, op, crash, corr_key="k1"):
    # Try common flag shapes for normal device-run evidence wiring
    return [
        "--classify-device-run",
        "--json",
        "--result-file",
        "-",  # may be overridden by writing temp file below in specific tests
        "--raw-outcome",
        raw_outcome,
        "--cache-write-status",
        cache,
        "--corroboration",
        corr,
        "--operation",
        op,
        "--fixture-reference",
        "fixture://spec012/app-accessible",
        "--build-identity",
        "debug:test",
        "--device-identity",
        "serial:test",
        "--diagnostic-capture",
        "logcat://test",
        "--crash-anr-status",
        crash,
        "--correlation-key",
        corr_key,
        "--reviewer-accessible-location",
        "artifact://t018-t024-t067/test",
        "--retention-scope",
        "spec012-t018-t024-t067",
    ]


@given(raw_outcome=RAW_OUTCOMES, cache=CACHE, corr=CORR, op=OPS, crash=CRASH)
@settings(max_examples=40, deadline=None)
def test_print_device_run_contract_exists_and_fr008_minimum(raw_outcome, cache, corr, op, crash):
    # Contract print mode for normal device-run evidence wiring
    p = run_hook(["--print-device-run-contract", "--json"])
    data = parse_json(p)
    assert isinstance(data, dict)
    s = json.dumps(data, sort_keys=True).lower()
    assert "fr-008" in s or "evidence" in s
    assert "full_parent_completion_claimed" in s
    assert "programmatic-non-ui" in s or "programmatic" in s


@given(raw_outcome=RAW_OUTCOMES, cache=CACHE, corr=CORR, op=OPS, crash=CRASH)
@settings(max_examples=40, deadline=None)
def test_classify_device_run_emits_fr008_minimum_and_non_ui_scope(raw_outcome, cache, corr, op, crash):
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".json") as f:
        json.dump({"outcome": raw_outcome}, f)
        tmp = f.name
    try:
        args = [
            "--classify-device-run",
            "--json",
            "--result-file",
            tmp,
            "--raw-outcome",
            raw_outcome,
            "--cache-write-status",
            cache,
            "--corroboration",
            corr,
            "--operation",
            op,
            "--fixture-reference",
            "fixture://spec012/app-accessible",
            "--build-identity",
            "debug:test",
            "--device-identity",
            "serial:test",
            "--diagnostic-capture",
            "logcat://test",
            "--crash-anr-status",
            crash,
            "--correlation-key",
            "corr-123",
            "--reviewer-accessible-location",
            "artifact://spec012/run1",
            "--retention-scope",
            "spec012",
        ]
        data = parse_json(run_hook(args))
        assert data.get("full_parent_completion_claimed") is False
        scope = str(data.get("scope_class") or data.get("evidence_scope") or "")
        assert "programmatic-non-ui" in scope
        assert data.get("residual_scope_note")
        for k in [
            "fixture_reference",
            "operation",
            "outcome",
            "build_identity",
            "device_identity",
            "diagnostic_capture",
            "crash_anr_status",
            "preferred_location_or_channel",
            "effective_location_or_channel",
            "write_status",
            "reviewer_accessible_location",
            "retention_scope",
            "correlation_key",
        ]:
            assert k in data and str(data.get(k)) != "", f"missing/empty FR-008 field {k}"
        blob = json.dumps(data, sort_keys=True).lower()
        assert "sc001" in blob or "attempted" in blob
        assert "sc002" in blob or "explicit" in blob or "denominator" in blob
        assert "classification" in blob or "status" in blob
    finally:
        try:
            os.unlink(tmp)
        except Exception:
            pass


@given(raw_outcome=RAW_OUTCOMES, cache=CACHE, corr=CORR, op=OPS)
@settings(max_examples=35, deadline=None)
def test_unknown_crash_anr_never_pass_in_device_run(raw_outcome, cache, corr, op):
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".json") as f:
        json.dump({"outcome": raw_outcome}, f)
        tmp = f.name
    try:
        args = [
            "--classify-device-run",
            "--json",
            "--result-file",
            tmp,
            "--raw-outcome",
            raw_outcome,
            "--cache-write-status",
            cache,
            "--corroboration",
            corr,
            "--operation",
            op,
            "--fixture-reference",
            "fixture://spec012",
            "--build-identity",
            "b",
            "--device-identity",
            "d",
            "--diagnostic-capture",
            "diag",
            "--crash-anr-status",
            "unknown",
            "--correlation-key",
            "k",
            "--reviewer-accessible-location",
            "loc",
            "--retention-scope",
            "r",
        ]
        data = parse_json(run_hook(args))
        cls = str(
            data.get("classification")
            or data.get("status")
            or data.get("normalized_outcome")
            or data.get("outcome_status")
            or ""
        )
        assert not is_pass_like(cls)
        for k in ["counts_as_pass", "counts_as_full_pass", "passed", "is_pass"]:
            if k in data:
                assert data[k] is not True
    finally:
        try:
            os.unlink(tmp)
        except Exception:
            pass


def test_blocked_fixture_inaccessible_explicit_non_pass_device_run():
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".txt") as f:
        f.write("blocked:fixture-inaccessible\n")
        tmp = f.name
    try:
        args = [
            "--classify-device-run",
            "--json",
            "--result-file",
            tmp,
            "--crash-anr-status",
            "unknown",
            "--correlation-key",
            "k",
            "--reviewer-accessible-location",
            "loc",
            "--retention-scope",
            "r",
        ]
        data = parse_json(run_hook(args))
        s = json.dumps(data, sort_keys=True).lower()
        assert "blocked" in s and "fixture-inaccessible" in s
        cls = str(
            data.get("classification")
            or data.get("status")
            or data.get("outcome_status")
            or ""
        )
        assert not is_pass_like(cls)
    finally:
        try:
            os.unlink(tmp)
        except Exception:
            pass


def test_blocked_busy_cold_start_explicit_non_pass_device_run():
    for val in ["blocked:busy", "blocked:cold-start"]:
        with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".txt") as f:
            f.write(val + "\n")
            tmp = f.name
        try:
            args = [
                "--classify-device-run",
                "--json",
                "--result-file",
                tmp,
                "--crash-anr-status",
                "unknown",
                "--correlation-key",
                "k",
                "--reviewer-accessible-location",
                "loc",
                "--retention-scope",
                "r",
            ]
            data = parse_json(run_hook(args))
            s = json.dumps(data, sort_keys=True).lower()
            assert "blocked" in s
            cls = str(
                data.get("classification")
                or data.get("status")
                or data.get("outcome_status")
                or ""
            )
            assert not is_pass_like(cls)
        finally:
            try:
                os.unlink(tmp)
            except Exception:
                pass


def test_inconclusive_explicit_and_not_pass_device_run():
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".txt") as f:
        f.write("inconclusive\n")
        tmp = f.name
    try:
        args = [
            "--classify-device-run",
            "--json",
            "--result-file",
            tmp,
            "--crash-anr-status",
            "unknown",
            "--correlation-key",
            "k",
            "--reviewer-accessible-location",
            "loc",
            "--retention-scope",
            "r",
        ]
        data = parse_json(run_hook(args))
        s = json.dumps(data, sort_keys=True).lower()
        assert "inconclusive" in s
        cls = str(
            data.get("classification")
            or data.get("status")
            or data.get("outcome_status")
            or ""
        )
        assert not is_pass_like(cls)
    finally:
        try:
            os.unlink(tmp)
        except Exception:
            pass


def test_detected_crash_anr_never_counts_as_pass_device_run():
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".json") as f:
        json.dump({"outcome": "ok:document:/tmp/f.md"}, f)
        tmp = f.name
    try:
        args = [
            "--classify-device-run",
            "--json",
            "--result-file",
            tmp,
            "--cache-write-status",
            "written",
            "--corroboration",
            "authoritative-only",
            "--operation",
            "document",
            "--fixture-reference",
            "fixture://spec012",
            "--build-identity",
            "b",
            "--device-identity",
            "d",
            "--diagnostic-capture",
            "diag",
            "--crash-anr-status",
            "detected",
            "--correlation-key",
            "k",
            "--reviewer-accessible-location",
            "loc",
            "--retention-scope",
            "r",
        ]
        data = parse_json(run_hook(args))
        cls = str(
            data.get("classification")
            or data.get("status")
            or data.get("outcome_status")
            or ""
        )
        assert not is_pass_like(cls)
    finally:
        try:
            os.unlink(tmp)
        except Exception:
            pass


def test_full_parent_completion_claim_always_false_device_run():
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".json") as f:
        json.dump({"outcome": "ok:directory:/tmp/d:chapters=2"}, f)
        tmp = f.name
    try:
        args = [
            "--classify-device-run",
            "--json",
            "--result-file",
            tmp,
            "--crash-anr-status",
            "not-detected",
            "--correlation-key",
            "k",
            "--reviewer-accessible-location",
            "loc",
            "--retention-scope",
            "r",
        ]
        data = parse_json(run_hook(args))
        assert data.get("full_parent_completion_claimed") is False
        assert '"full_parent_completion_claimed": true' not in json.dumps(data).lower()
    finally:
        try:
            os.unlink(tmp)
        except Exception:
            pass


def test_existing_host_contract_modes_unchanged_device_run_context():
    p = run_hook(["--print-residual-rules", "--json"])
    data = parse_json(p)
    assert isinstance(data, dict)
    p2 = run_hook(["--classify-outcome", "ok:document:/tmp/f.md", "--json"])
    d2 = parse_json(p2)
    assert d2.get("status") in {"pass", "ok", "explicit"} or d2.get(
        "explicit_within_budget"
    ) is True
