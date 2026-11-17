"""Spec 012 debug ingest Activity/cache channel attribution contract tests (T055 RED).

Covers FR-006/FR-008/EC-012: preferred authoritative cache channel
(`cache/debug_ingest_result.txt`) write unavailable/failure MUST NOT silently imply
authoritative cache success. Contract/reporting must be explicit and diagnosable with
`write_status`, `channel`/effective channel, `corroboration`, and fail-closed
`inconclusive`/`blocked` semantics when no authorized corroboration exists.

These tests target the contract shape T056 is expected to satisfy (hook contract mode
and/or classify/report schema). They intentionally fail against the current script
until T056 implements the contract.
"""
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import assume, given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
HOOK_SCRIPT = REPO / "scripts" / "automate-t184-t226-hook.sh"
CACHE_CHANNEL = "cache/debug_ingest_result.txt"
ALLOWED_WRITE_STATUS_RE = re.compile(r"^(written|unavailable|failed:.+)$")
FAIL_CLOSED_NO_CORROBORATION = {"inconclusive", "blocked"}
CORROBORATION_AWARE = {
    "authoritative-only",
    "corroborated",
    "conflicting",
    "inconclusive",
    "none",
    "blocked",
}


def run_hook(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(HOOK_SCRIPT), *args],
        cwd=REPO,
        text=True,
        capture_output=True,
        check=False,
    )


def parse_json_stdout(proc: subprocess.CompletedProcess[str]) -> dict:
    assert proc.returncode == 0, proc.stdout + proc.stderr
    return json.loads(proc.stdout)


@st.composite
def cache_write_failures(draw: st.DrawFn) -> str:
    reason = draw(
        st.text(
            alphabet=st.characters(blacklist_categories=("Cs",), blacklist_characters="\x00\n\r"),
            min_size=1,
            max_size=48,
        ).filter(lambda s: s.strip() != "")
    )
    return draw(st.sampled_from(["unavailable", f"failed:{reason.strip()}"]))


@given(cache_write_failures())
@settings(max_examples=40, deadline=None)
def test_cache_write_failure_status_is_explicit_and_diagnosable(write_status: str) -> None:
    # Invariant: cache write failure/unavailability is never an empty/ambiguous status.
    assert ALLOWED_WRITE_STATUS_RE.match(write_status), write_status
    assert write_status != "written"


def test_hook_exposes_activity_channel_contract_mode() -> None:
    # RED: T056 should expose a deterministic host-side contract for Activity/cache
    # channel attribution failures (no adb/device side effects).
    proc = run_hook(["--print-activity-channel-contract", "--json"])
    data = parse_json_stdout(proc)
    assert data.get("authoritative_cache_channel") == CACHE_CHANNEL
    assert data.get("cache_write_failure_must_not_imply_authoritative_success") is True
    assert data.get("write_status_required") is True
    assert data.get("corroboration_required") is True
    fallback = set(data.get("no_corroboration_outcomes") or data.get("fallback_without_corroboration") or [])
    assert fallback, "contract must define fail-closed outcomes when cache is unwritten and uncorroborated"
    assert fallback.issubset(FAIL_CLOSED_NO_CORROBORATION)


@given(cache_write_failures())
@settings(max_examples=35, deadline=None)
def test_cache_write_failure_without_corroboration_never_implies_authoritative_success(
    write_status: str,
) -> None:
    # RED: classify/reporting must treat unwritten authoritative cache without
    # corroboration as fail-closed (inconclusive/blocked), never explicit success.
    proc = run_hook(
        [
            "--classify-channel-attribution",
            "--cache-write-status",
            write_status,
            "--corroboration",
            "none",
            "--json",
        ]
    )
    data = parse_json_stdout(proc)
    assert data.get("cache_write_status") == write_status or data.get("write_status") == write_status
    assert data.get("authoritative_cache_success") is False
    assert data.get("explicit_within_budget") is False
    assert data.get("counts_as_explicit_pass") is False
    status = str(data.get("status") or data.get("outcome_status") or "")
    assert status in FAIL_CLOSED_NO_CORROBORATION, data
    corroboration = str(data.get("corroboration") or "")
    assert corroboration, "corroboration must be explicit"
    assert corroboration not in {"authoritative-only", "authoritative"}, data


@given(cache_write_failures(), st.sampled_from(["ok:document:/data/local/tmp/t.md", "ok:directory:/data/local/tmp/d:chapters=1", "error:io:read"]))
@settings(max_examples=35, deadline=None)
def test_cache_failure_cannot_upgrade_raw_outcome_to_authoritative_cache_attribution(
    write_status: str, raw_outcome: str
) -> None:
    # Invariant: even if an in-memory/raw outcome exists, failing to write the
    # authoritative cache channel cannot be reported as authoritative cache evidence.
    assume(raw_outcome.startswith(("ok:", "error:", "refused:")))
    proc = run_hook(
        [
            "--classify-channel-attribution",
            "--raw-outcome",
            raw_outcome,
            "--cache-write-status",
            write_status,
            "--corroboration",
            "none",
            "--json",
        ]
    )
    data = parse_json_stdout(proc)
    assert data.get("authoritative_cache_success") is False
    assert data.get("channel_attribution") != "authoritative-cache"
    assert data.get("effective_location_or_channel") != CACHE_CHANNEL or data.get("status") in FAIL_CLOSED_NO_CORROBORATION
    # Fail-closed: no corroboration means the run is not explicit authoritative success.
    assert data.get("counts_as_explicit_pass") is False


def test_report_schema_requires_cache_attribution_fields_for_fr008() -> None:
    proc = run_hook(["--print-report-schema", "--json"])
    data = parse_json_stdout(proc)
    # Existing FR-008 disposition fields remain required.
    for field in (
        "preferred_location_or_channel",
        "effective_location_or_channel",
        "write_status",
        "reviewer_accessible_location",
        "retention_scope",
    ):
        assert field in data, field
    # T056 contract: schema must make cache-authoritative attribution/corroboration explicit.
    serialized = json.dumps(data, sort_keys=True)
    assert "authoritative_cache_channel" in data or CACHE_CHANNEL in serialized
    assert "cache_write_failure" in serialized or "authoritative_cache_success" in serialized
    assert "corroboration" in serialized or "channel_corroboration" in serialized
    assert "inconclusive" in serialized and "blocked" in serialized


@given(st.sampled_from(["corroborated", "logcat-corroborated", "script-corroborated"]))
@settings(max_examples=20, deadline=None)
def test_correlated_fallback_must_be_explicit_not_silent_cache_success(corroboration: str) -> None:
    # If cache write fails but an authorized corroborated channel exists, reporting may be
    # attributable only as corroborated fallback; never silently authoritative cache.
    proc = run_hook(
        [
            "--classify-channel-attribution",
            "--raw-outcome",
            "ok:document:/data/local/tmp/t.md",
            "--cache-write-status",
            "failed:permission",
            "--corroboration",
            corroboration,
            "--json",
        ]
    )
    data = parse_json_stdout(proc)
    assert data.get("authoritative_cache_success") is False
    reported = str(data.get("corroboration") or data.get("channel_corroboration") or "")
    assert "corroborat" in reported.lower(), data
    assert reported not in {"authoritative-only", "authoritative"}, data
    assert ALLOWED_WRITE_STATUS_RE.match(str(data.get("cache_write_status") or data.get("write_status") or ""))


def test_fr008_json_contract_exposes_non_silent_cache_attribution() -> None:
    proc = run_hook(["--help", "--json"])
    data = parse_json_stdout(proc)
    # Current script emits placeholder values; T056 must make cache attribution explicit.
    assert data.get("authoritative_cache_channel") == CACHE_CHANNEL
    assert data.get("cache_write_failure_must_not_imply_authoritative_success") is True
    # A static FR-008 contract blob must not claim authoritative-only success while write is unavailable.
    if str(data.get("write_status")) != "written":
        assert str(data.get("corroboration")) not in {"authoritative-only", "authoritative"}, data
        assert data.get("authoritative_cache_success") is False
