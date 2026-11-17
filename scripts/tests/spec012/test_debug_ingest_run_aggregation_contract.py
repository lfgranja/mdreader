"""RED contract tests for Spec 012 T067/T049 — host-deterministic FR-008/SC
aggregation/packaging from per-op JSON inputs (no adb).

Host-testable only. No adb/emulator/device required. Tests intentionally fail
until `scripts/automate-t184-t226-hook.sh` exposes deterministic aggregation
contract mode(s), e.g.:
  --print-run-aggregation-contract --json
  --aggregate-device-evidence --json --inputs <per-op-fr008.json>...
or an equivalent host-deterministic shape (no side effects).

Required semantics (FR-006/FR-008/FR-012/SC-001/SC-002, CHK030/ASM/DEP-012-*):
- Computes/reports SC-001/SC-002 counters from per-op FR-008 JSON inputs
  (`attempted`,`precondition_blocked`,`in_denominator`,`explicit_within_budget`,
  `inconclusive`, formula-consistent).
- Preserves `full_parent_completion_claimed=false` (programmatic non-UI scope).
- Classifies HW/SW matrix as `not-evaluated` explicitly when unsupported.
- Includes correlation/build/device identity aggregation or explicit unavailable.
- Includes retention scope and reviewer-accessible location (or explicit unavailable).
- Never counts `crash_anr_status unknown|detected`, blocked, or inconclusive as
  explicit pass/success/full completion.
"""
from __future__ import annotations

import json
import math
import subprocess
from pathlib import Path
from typing import Any

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import HealthCheck, given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
HOOK = REPO / "scripts" / "automate-t184-t226-hook.sh"

CRASH = st.sampled_from(["not-detected", "detected", "unknown"])
OPS = st.sampled_from(["document", "directory", "uri"])
STATUS = st.sampled_from(["pass", "fail", "blocked", "inconclusive", "error"])
RAW_OUTCOMES = st.sampled_from(
    [
        "ok:document:/tmp/f.md",
        "ok:uri:file:///tmp/f.md",
        "ok:directory:/tmp/d:chapters=2",
        "blocked:fixture-inaccessible",
        "blocked:busy",
        "blocked:cold-start",
        "inconclusive",
        "error:io:read",
        "error:no-chapters:/tmp/d",
        "refused:not-debug",
    ]
)
TEXT = st.text(min_size=1, max_size=24).filter(lambda s: "\x00" not in s)


def run_hook(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(HOOK), *args], cwd=REPO, text=True, capture_output=True, check=False
    )


def parse_json(proc: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    raw = (proc.stdout or proc.stderr).strip()
    assert raw, f"empty output rc={proc.returncode} stderr={proc.stderr[:500]}"
    try:
        data = json.loads(raw)
    except Exception as e:  # pragma: no cover - RED assertion clarity
        raise AssertionError(f"not json rc={proc.returncode} raw={raw[:500]} err={e}") from e
    assert isinstance(data, dict), f"expected object, got {type(data)}: {data!r}"
    return data


def is_pass_like(value: Any) -> bool:
    s = str(value or "").strip().lower()
    if s in {
        "pass",
        "passed",
        "success",
        "ok",
        "explicit-pass",
        "true",
        "1",
        "yes",
        "explicit_ok",
        "explicit-pass-within-budget",
        "full-pass",
    }:
        return True
    return s.startswith("ok:")


def assert_no_full_pass_claim(data: dict[str, Any]) -> None:
    blob = json.dumps(data, sort_keys=True).lower()
    assert '"full_parent_completion_claimed": true' not in blob
    assert '"full_parent_completion_claimed":true' not in blob
    # Never claim HW/SW matrix evaluated/passed when aggregation contract requires explicit not-evaluated.
    # (Positive evaluation is out of scope for this contract; explicit not-evaluated is required.)
    for key in [
        "counts_as_pass",
        "counts_as_full_pass",
        "passed",
        "is_pass",
        "explicit_pass",
        "satisfies_fr010_sc004",
    ]:
        if key in data:
            assert data[key] is not True, f"{key} must not be true in aggregate non-pass-safe context: {data[key]!r}"


def walk_values(obj: Any):
    if isinstance(obj, dict):
        for v in obj.values():
            yield from walk_values(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from walk_values(v)
    else:
        yield obj


def find_numeric(data: dict[str, Any], names: set[str]) -> int | None:
    def rec(obj: Any) -> int | None:
        if isinstance(obj, dict):
            for k, v in obj.items():
                kl = str(k).lower()
                if kl in names and isinstance(v, (int, float)) and not isinstance(v, bool):
                    return int(v)
                found = rec(v)
                if found is not None:
                    return found
        elif isinstance(obj, list):
            for v in obj:
                found = rec(v)
                if found is not None:
                    return found
        return None

    return rec(data)


def per_op_evidence(
    *,
    raw_outcome: str,
    status: str,
    crash_anr_status: str,
    attempted: int = 1,
    precondition_blocked: int = 0,
    in_denominator: int = 1,
    explicit_within_budget: int = 0,
    inconclusive: int = 0,
    operation: str = "document",
    correlation_key: str = "run-1",
    build_identity: str = "debug:app.mdreader/0.1.0",
    device_identity: str = "adb-serial:model:sdk33:arm64",
) -> dict[str, Any]:
    sc001 = {
        "attempted": attempted,
        "precondition_blocked": precondition_blocked,
        "in_denominator": in_denominator,
        "explicit_within_budget": explicit_within_budget,
        "inconclusive": inconclusive,
    }
    sc002 = dict(sc001)
    return {
        "fixture_reference": "fixture://spec012/t184-or-t226",
        "operation": operation,
        "outcome": raw_outcome,
        "status": status,
        "classification": status,
        "normalized_outcome": raw_outcome,
        "scope_class": "programmatic-non-ui",
        "full_parent_completion_claimed": False,
        "residual_scope_note": "Native SAF picker UI/a11y residuals remain open (FR-009/FR-012).",
        "build_identity": build_identity,
        "device_identity": device_identity,
        "diagnostic_capture": "logcat://spec012/run",
        "crash_anr_status": crash_anr_status,
        "preferred_location_or_channel": "cache/debug_ingest_result.txt",
        "effective_location_or_channel": "cache/debug_ingest_result.txt",
        "write_status": "written",
        "cache_write_status": "written",
        "corroboration": "authoritative-only",
        "reviewer_accessible_location": "artifact://spec012/evidence",
        "retention_scope": "spec012-until-covered-ids-decided-or-closed",
        "correlation_key": correlation_key,
        "counters": {
            "attempted": attempted,
            "precondition_blocked": precondition_blocked,
            "in_denominator": in_denominator,
            "explicit_within_budget": explicit_within_budget,
            "inconclusive": inconclusive,
            "sc001": sc001,
            "sc002": sc002,
        },
        "sc001_counters": list(sc001.keys()),
        "sc002_counters": list(sc002.keys()),
    }


@st.composite
def per_op_case(draw: st.DrawFn) -> dict[str, Any]:
    crash = draw(CRASH)
    raw = draw(RAW_OUTCOMES)
    op = draw(OPS)
    status: str
    pre = 0
    in_den = 1
    explicit = 0
    incon = 0
    if raw.startswith("blocked:"):
        status = "blocked"
        pre = 1
        in_den = 0
        explicit = 0
    elif raw == "inconclusive":
        status = "inconclusive"
        incon = 1
        explicit = 0
    elif raw.startswith("ok:") and crash == "not-detected":
        status = "pass"
        explicit = 1
    elif raw.startswith("ok:"):
        # unknown/detected crash must never become explicit pass
        status = "fail" if crash == "detected" else "inconclusive"
        explicit = 0
        incon = 1 if crash == "unknown" and not raw.startswith("blocked:") else 0
        if status == "inconclusive":
            in_den = 1
    else:
        status = "fail"
        explicit = 0
    # formula consistency
    if status == "blocked":
        pre = 1
        in_den = 0
    attempted = 1
    if incon and status == "inconclusive":
        in_den = 1
    return per_op_evidence(
        raw_outcome=raw,
        status=status,
        crash_anr_status=crash,
        attempted=attempted,
        precondition_blocked=pre,
        in_denominator=in_den,
        explicit_within_budget=explicit,
        inconclusive=incon,
        operation=op,
        correlation_key=draw(st.text(min_size=1, max_size=16, alphabet="abc123-_")),
        build_identity=draw(TEXT),
        device_identity=draw(TEXT),
    )


def write_inputs(tmp_path: Path, cases: list[dict[str, Any]]) -> list[str]:
    paths: list[str] = []
    for i, case in enumerate(cases):
        p = tmp_path / f"fr008_op_{i}.json"
        p.write_text(json.dumps(case, indent=2) + "\n", encoding="utf-8")
        paths.append(str(p))
    return paths


def aggregate(args_inputs: list[str], extra: list[str] | None = None) -> dict[str, Any]:
    cmd = ["--aggregate-device-evidence", "--json", "--inputs", *args_inputs, *(extra or [])]
    return parse_json(run_hook(cmd))


def expected_totals(cases: list[dict[str, Any]]) -> dict[str, int]:
    totals = {
        "attempted": 0,
        "precondition_blocked": 0,
        "in_denominator": 0,
        "explicit_within_budget": 0,
        "inconclusive": 0,
    }
    for c in cases:
        ctr = (c.get("counters") or {}) if isinstance(c.get("counters"), dict) else {}
        for k in totals:
            v = ctr.get(k)
            if isinstance(v, bool):
                continue
            if isinstance(v, (int, float)):
                totals[k] += int(v)
            elif isinstance(c.get(k), (int, float)) and not isinstance(c.get(k), bool):
                totals[k] += int(c[k])
    return totals


def test_print_run_aggregation_contract_mentions_required_invariants():
    data = parse_json(run_hook(["--print-run-aggregation-contract", "--json"]))
    blob = json.dumps(data, sort_keys=True).lower()
    assert "full_parent_completion_claimed" in blob
    assert "not-evaluated" in blob or "not_evaluated" in blob
    assert "hw" in blob and "sw" in blob
    assert "sc-001" in blob or "sc001" in blob or "attempted" in blob
    assert "sc-002" in blob or "sc002" in blob or "in_denominator" in blob or "explicit_within_budget" in blob
    assert "correlation" in blob
    assert "build" in blob and "device" in blob
    assert "retention" in blob
    assert "reviewer" in blob
    assert "crash_anr" in blob or "crash-anr" in blob or "crash" in blob
    assert_no_full_pass_claim(data)


@given(cases=st.lists(per_op_case(), min_size=1, max_size=6))
@settings(max_examples=30, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_aggregate_reports_sc_counters_formula_consistent(tmp_path: Path, cases: list[dict[str, Any]]):
    inputs = write_inputs(tmp_path, cases)
    data = aggregate(inputs)
    assert_no_full_pass_claim(data)
    totals = expected_totals(cases)
    # Accept either nested sc001/sc002 totals or top-level totals; at least the expected totals
    # must be discoverable for the computed aggregate.
    for name, expected in totals.items():
        found = find_numeric(data, {name})
        assert found == expected, f"aggregate {name} expected {expected}, found {found} in {json.dumps(data)[:1200]}"
    blob = json.dumps(data, sort_keys=True).lower()
    assert "sc-001" in blob or "sc001" in blob
    assert "sc-002" in blob or "sc002" in blob


@given(cases=st.lists(per_op_case(), min_size=1, max_size=5))
@settings(max_examples=25, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_aggregate_full_parent_completion_always_false(tmp_path: Path, cases: list[dict[str, Any]]):
    data = aggregate(write_inputs(tmp_path, cases))
    assert data.get("full_parent_completion_claimed") is False
    assert_no_full_pass_claim(data)


@given(cases=st.lists(per_op_case(), min_size=1, max_size=5))
@settings(max_examples=25, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_aggregate_hw_sw_matrix_explicit_not_evaluated_when_unsupported(
    tmp_path: Path, cases: list[dict[str, Any]]
):
    data = aggregate(write_inputs(tmp_path, cases))
    blob = json.dumps(data, sort_keys=True).lower()
    assert "not-evaluated" in blob or "not_evaluated" in blob
    assert "hw" in blob and "sw" in blob
    # Must not imply matrix pass/evaluated-as-pass.
    assert "hw_sw_matrix_pass" not in blob or "false" in blob
    assert_no_full_pass_claim(data)


@given(cases=st.lists(per_op_case(), min_size=1, max_size=5))
@settings(max_examples=25, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_aggregate_includes_identity_retention_reviewer_or_explicit_unavailable(
    tmp_path: Path, cases: list[dict[str, Any]]
):
    data = aggregate(write_inputs(tmp_path, cases))
    blob = json.dumps(data, sort_keys=True).lower()
    assert "correlation" in blob
    assert "build" in blob
    assert "device" in blob
    assert "retention" in blob
    assert "reviewer" in blob
    # Aggregation may include concrete values or explicit unavailable markers.
    assert "unavailable" in blob or len(blob) > 40


@given(op=OPS)
@settings(max_examples=12, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_unknown_crash_never_counts_as_aggregate_pass(tmp_path: Path, op: str):
    case = per_op_evidence(
        raw_outcome=f"ok:{op}:/tmp/x" if op != "directory" else "ok:directory:/tmp/x:chapters=1",
        status="inconclusive",
        crash_anr_status="unknown",
        in_denominator=1,
        explicit_within_budget=0,
        inconclusive=1,
        operation=op,
    )
    data = aggregate(write_inputs(tmp_path, [case]))
    assert_no_full_pass_claim(data)
    explicit = find_numeric(data, {"explicit_within_budget", "explicit_pass_count", "passed_count"})
    if explicit is not None:
        assert explicit == 0
    cls = str(data.get("classification") or data.get("aggregate_status") or data.get("status") or "")
    assert not is_pass_like(cls)


@given(op=OPS)
@settings(max_examples=12, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_detected_crash_never_counts_as_aggregate_pass(tmp_path: Path, op: str):
    case = per_op_evidence(
        raw_outcome=f"ok:{op}:/tmp/x" if op != "directory" else "ok:directory:/tmp/x:chapters=1",
        status="fail",
        crash_anr_status="detected",
        in_denominator=1,
        explicit_within_budget=0,
        inconclusive=0,
        operation=op,
    )
    data = aggregate(write_inputs(tmp_path, [case]))
    assert_no_full_pass_claim(data)
    explicit = find_numeric(data, {"explicit_within_budget", "explicit_pass_count", "passed_count"})
    if explicit is not None:
        assert explicit == 0
    cls = str(data.get("classification") or data.get("aggregate_status") or data.get("status") or "")
    assert not is_pass_like(cls)


@given(raw=st.sampled_from(["blocked:fixture-inaccessible", "blocked:busy", "blocked:cold-start", "inconclusive"]))
@settings(max_examples=12, deadline=None, suppress_health_check=[HealthCheck.function_scoped_fixture])
def test_blocked_and_inconclusive_inputs_never_count_as_pass(tmp_path: Path, raw: str):
    status = "blocked" if raw.startswith("blocked:") else "inconclusive"
    case = per_op_evidence(
        raw_outcome=raw,
        status=status,
        crash_anr_status="unknown" if status == "inconclusive" else "not-detected",
        precondition_blocked=1 if status == "blocked" else 0,
        in_denominator=0 if status == "blocked" else 1,
        explicit_within_budget=0,
        inconclusive=1 if status == "inconclusive" else 0,
    )
    data = aggregate(write_inputs(tmp_path, [case]))
    assert_no_full_pass_claim(data)
    explicit = find_numeric(data, {"explicit_within_budget", "explicit_pass_count", "passed_count"})
    if explicit is not None:
        assert explicit == 0
    cls = str(data.get("classification") or data.get("aggregate_status") or data.get("status") or "")
    assert not is_pass_like(cls)


def test_mixed_run_with_non_pass_crash_and_blocked_cannot_claim_completion(tmp_path: Path):
    cases = [
        per_op_evidence(
            raw_outcome="ok:document:/tmp/a.md",
            status="pass",
            crash_anr_status="not-detected",
            in_denominator=1,
            explicit_within_budget=1,
        ),
        per_op_evidence(
            raw_outcome="ok:directory:/tmp/d:chapters=2",
            status="inconclusive",
            crash_anr_status="unknown",
            operation="directory",
            in_denominator=1,
            explicit_within_budget=0,
            inconclusive=1,
            correlation_key="run-2",
        ),
        per_op_evidence(
            raw_outcome="blocked:fixture-inaccessible",
            status="blocked",
            crash_anr_status="not-detected",
            precondition_blocked=1,
            in_denominator=0,
            explicit_within_budget=0,
            correlation_key="run-3",
        ),
    ]
    data = aggregate(write_inputs(tmp_path, cases))
    assert_no_full_pass_claim(data)
    totals = expected_totals(cases)
    for name, expected in totals.items():
        found = find_numeric(data, {name})
        assert found == expected, f"{name} expected {expected}, found {found}"
    cls = str(data.get("classification") or data.get("aggregate_status") or data.get("status") or "").lower()
    assert cls not in {"pass", "passed", "success", "full-pass", "complete", "completed", "ok"}
    # At least one explicit pass input must not become aggregate full completion given unknown/blocked.
    blob = json.dumps(data, sort_keys=True).lower()
    assert "not-evaluated" in blob or "not_evaluated" in blob


def test_existing_host_contract_modes_still_parse_after_aggregation_contract_surface():
    # Preserve deterministic host contract modes (no weakening).
    p = run_hook(["--print-residual-rules", "--json"])
    data = parse_json(p)
    assert isinstance(data, dict)
    p2 = run_hook(["--classify-outcome", "ok:document:/tmp/f.md", "--json"])
    d2 = parse_json(p2)
    assert d2.get("status") in {"pass", "ok", "explicit"} or d2.get("explicit_within_budget") is True
