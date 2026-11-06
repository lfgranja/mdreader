"""RED contract tests for Spec 012 T060/T067 — deterministic FR-008/SC-001/SC-002
device-execution evidence helper.

Host-testable only. No adb/emulator/device required. Tests intentionally fail
until `scripts/automate-t184-t226-hook.sh` exposes deterministic evidence helper
mode(s) (e.g. --build-device-evidence --json) without weakening existing host
contract modes.

Required semantics (FR-006/FR-008/FR-012/SC-001/SC-002):
- full_parent_completion_claimed always false (programmatic non-UI scope only)
- scope_class/evidence_scope explicitly programmatic-non-ui
- residual_scope_note explicit
- crash_anr_status=unknown never counts as explicit pass/success
- blocked:fixture-inaccessible/inconclusive explicit and non-pass
- SC-001/SC-002 classification/counter hooks present
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
HOOK = REPO / "scripts" / "automate-t184-t226-hook.sh"

RAW_OUTCOMES = st.sampled_from([
    "ok:document:/tmp/f.md", "ok:directory:/tmp/d:chapters=1",
    "blocked:fixture-inaccessible", "inconclusive", "error:io:read", "refused:not-debug",
])
CACHE = st.sampled_from(["written", "unavailable", "failed:permission", "failed:io"])
CORR = st.sampled_from(["authoritative-only", "corroborated", "none", "conflicting", "blocked", "inconclusive"])
OPS = st.sampled_from(["document", "directory", "uri"])
CRASH = st.sampled_from(["not-detected", "detected", "unknown"])


def run_hook(args):
    return subprocess.run([str(HOOK), *args], cwd=REPO, text=True, capture_output=True, check=False)


def parse_json(proc):
    raw = (proc.stdout or proc.stderr).strip()
    assert raw, f"empty output rc={proc.returncode} stderr={proc.stderr[:200]}"
    return json.loads(raw)


def is_pass_like(v):
    s = str(v or "").lower()
    return s in {"pass", "passed", "success", "ok", "explicit-pass", "true"} or s.startswith("ok:")


def build_args(raw_outcome, cache, corr, op, crash, corr_key="k1"):
    return [
        "--build-device-evidence", "--json",
        "--raw-outcome", raw_outcome,
        "--cache-write-status", cache,
        "--corroboration", corr,
        "--operation", op,
        "--fixture-reference", "fixture://spec012",
        "--build-identity", "debug:test",
        "--device-identity", "serial:test",
        "--diagnostic-capture", "logcat://test",
        "--crash-anr-status", crash,
        "--correlation-key", corr_key,
        "--reviewer-accessible-location", "artifact://test",
        "--retention-scope", "spec012-test",
    ]


@given(raw_outcome=RAW_OUTCOMES, cache=CACHE, corr=CORR, op=OPS, crash=CRASH)
@settings(max_examples=45, deadline=None)
def test_build_device_evidence_emits_fr008_minimum_and_non_ui_scope(raw_outcome, cache, corr, op, crash):
    data = parse_json(run_hook(build_args(raw_outcome, cache, corr, op, crash)))
    assert data.get("full_parent_completion_claimed") is False
    scope = str(data.get("scope_class") or data.get("evidence_scope") or "")
    assert "programmatic-non-ui" in scope or ("programmatic" in scope and "ui" in scope.lower())
    assert data.get("residual_scope_note")
    for k in ["fixture_reference","operation","outcome","build_identity","device_identity","diagnostic_capture","crash_anr_status","preferred_location_or_channel","effective_location_or_channel","write_status","cache_write_status","reviewer_accessible_location","retention_scope","correlation_key"]:
        assert k in data and str(data.get(k) or data.get("corroboration") if k == "corroboration" else data.get(k)) != "", f"missing/empty FR-008 field {k}: {data.keys()}"
    # SC hooks/counters
    blob = json.dumps(data, sort_keys=True).lower()
    assert "sc001" in blob or "sc-001" in blob or "attempted" in blob or "counter" in blob
    assert "sc002" in blob or "sc-002" in blob or "explicit" in blob or "denominator" in blob
    assert "classification" in blob or "status" in blob


@given(raw_outcome=RAW_OUTCOMES, cache=CACHE, corr=CORR, op=OPS)
@settings(max_examples=35, deadline=None)
def test_unknown_crash_anr_never_counts_as_explicit_pass(raw_outcome, cache, corr, op):
    data = parse_json(run_hook(build_args(raw_outcome, cache, corr, op, "unknown")))
    cls = str(data.get("classification") or data.get("status") or data.get("normalized_outcome") or data.get("outcome_status") or "")
    assert not is_pass_like(cls)
    for k in ["counts_as_pass","counts_as_full_pass","passed","is_pass","explicit_pass","explicit_within_budget_counts_as_pass"]:
        if k in data:
            assert data[k] is not True, f"unknown crash/ANR must not count as pass: {k}={data[k]}"


@given(raw_outcome=RAW_OUTCOMES, cache=CACHE, corr=CORR, op=OPS, crash=CRASH)
@settings(max_examples=35, deadline=None)
def test_full_parent_completion_claim_always_false(raw_outcome, cache, corr, op, crash):
    data = parse_json(run_hook(build_args(raw_outcome, cache, corr, op, crash)))
    assert data.get("full_parent_completion_claimed") is False
    assert json.dumps(data).lower().find('"full_parent_completion_claimed": true') == -1


def test_blocked_fixture_inaccessible_explicit_and_non_pass():
    data = parse_json(run_hook(build_args("blocked:fixture-inaccessible", "unavailable", "none", "document", "not-detected")))
    s = json.dumps(data, sort_keys=True).lower()
    assert "blocked" in s
    assert "fixture-inaccessible" in s or "fixture_inaccessible" in s or "fixture" in s
    cls = str(data.get("classification") or data.get("status") or data.get("outcome_status") or "")
    assert not is_pass_like(cls)


def test_inconclusive_explicit_and_unknown_crash_still_not_pass():
    data = parse_json(run_hook(build_args("inconclusive", "unavailable", "none", "directory", "unknown")))
    s = json.dumps(data, sort_keys=True).lower()
    assert "inconclusive" in s
    cls = str(data.get("classification") or data.get("status") or data.get("outcome_status") or "")
    assert not is_pass_like(cls)


def test_detected_crash_anr_never_counts_as_pass():
    data = parse_json(run_hook(build_args("ok:document:/tmp/f.md", "written", "authoritative-only", "document", "detected")))
    cls = str(data.get("classification") or data.get("status") or data.get("outcome_status") or "")
    assert not is_pass_like(cls)


def test_existing_host_contract_modes_not_weakened():
    # deterministic existing modes must still behave
    p = run_hook(["--classify-residual", "--status", "partial", "--json"]) if False else run_hook(["--print-residual-rules", "--json"])
    data = parse_json(p)
    assert isinstance(data, dict)
    # classify-outcome still works
    p2 = run_hook(["--classify-outcome", "ok:document:/tmp/f.md", "--json"])
    d2 = parse_json(p2)
    assert d2.get("status") in {"pass","ok","explicit"} or d2.get("explicit_within_budget") is True
