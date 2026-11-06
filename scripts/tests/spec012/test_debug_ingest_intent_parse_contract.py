"""Spec 012 debug ingest intent parse/normalization contract tests (T059/T013/T020 RED).

TDD RED: expects host-deterministic hook contract modes that are NOT implemented yet.
These tests encode FR-002/FR-003/FR-007 invariants for debug-only intent parsing
semantics (normalization, deterministic precedence, explicit diagnosable errors).

Authority boundary (I1, i1-harness-decision.md): validates debug-only intent parsing
semantics via hook contract modes; production ingest remains in Kotlin/Rust/JNI.
This is not a parallel production ingest path and has no device/adb side effects.
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import assume, given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
HOOK = REPO / "scripts" / "automate-t184-t226-hook.sh"

# FR-002/FR-003 normalization matrix (T013/T020 traceability)
DOCUMENT_ALIASES = ("document", "file", "open_document")
DIRECTORY_ALIASES = ("directory", "dir", "tree", "open_directory")
URI_MODES = ("uri",)
UNKNOWN_PREFIX = "unknown-mode"


def run_hook(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(HOOK), *args], cwd=REPO, text=True, capture_output=True, check=False)


def parse_json(proc: subprocess.CompletedProcess[str]) -> dict:
    raw = (proc.stdout or proc.stderr).strip()
    assert raw, f"expected JSON output, got empty stdout/stderr; rc={proc.returncode}"
    return json.loads(raw)


def classify(obj: dict) -> str:
    """Stable error/pass class string for assertions (contract-defined field names)."""
    for key in ("error_class", "error", "status_class", "outcome", "result", "status", "normalized_outcome"):
        if key in obj and obj[key] is not None:
            return str(obj[key])
    return json.dumps(obj, sort_keys=True)


def normalized_mode(obj: dict) -> str:
    for key in ("normalized_mode", "mode", "operation"):
        if key in obj and obj[key] is not None:
            return str(obj[key]).lower()
    return classify(obj).lower()


def is_error(obj: dict) -> bool:
    cls = classify(obj).lower()
    status = str(obj.get("status", "")).lower()
    return (
        status in {"error", "fail", "invalid"}
        or cls.startswith("error")
        or "error:" in cls
        or obj.get("ok") is False
        or obj.get("valid") is False
    )


def is_ok(obj: dict) -> bool:
    cls = classify(obj).lower()
    status = str(obj.get("status", "")).lower()
    return status in {"ok", "pass", "valid", "normalized"} or cls.startswith("ok") or obj.get("ok") is True


# -- small strategies (safe tokens; do not require filesystem access) --
safe_token = st.text(alphabet="abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/._-", min_size=1, max_size=48)
maybe_token = st.one_of(st.none(), safe_token)
case_variant = lambda s: st.sampled_from([s, s.upper(), s.title(), s.swapcase() if s.swapcase() != s else s.capitalize()])
doc_alias = st.sampled_from(DOCUMENT_ALIASES).flatmap(case_variant)
dir_alias = st.sampled_from(DIRECTORY_ALIASES).flatmap(case_variant)
any_known_alias = st.one_of(doc_alias, dir_alias, st.sampled_from(URI_MODES).flatmap(case_variant))


def run_normalize(mode: str, path: str | None = None, uri: str | None = None, data_uri: str | None = None) -> dict:
    args = ["--normalize-debug-ingest-intent", "--mode", mode, "--json"]
    if path is not None:
        args += ["--path", path]
    if uri is not None:
        args += ["--uri", uri]
    if data_uri is not None:
        args += ["--data-uri", data_uri]
    return parse_json(run_hook(args))


# -----------------------------------------------------------------------------
# RED contract presence
# -----------------------------------------------------------------------------

def test_print_intent_parse_contract_mode_exists_and_is_json() -> None:
    proc = run_hook(["--print-intent-parse-contract", "--json"])
    data = parse_json(proc)
    # must describe FR-002/FR-003/FR-007 parsing semantics without device side effects
    blob = json.dumps(data, sort_keys=True).lower()
    for token in ("document", "directory", "uri", "path", "error", "unknown"):
        assert token in blob, f"intent parse contract missing token {token!r}: {blob[:240]}"


def test_normalize_debug_ingest_intent_mode_exists() -> None:
    data = run_normalize("document", path="/data/local/tmp/t.md")
    assert isinstance(data, dict)
    # RED until implemented: must not be a device-mode artifact or opaque help text
    blob = json.dumps(data, sort_keys=True).lower()
    assert "normalize" in blob or "mode" in blob or "document" in blob


# -----------------------------------------------------------------------------
# FR-003/FR-002 mode normalization invariants (PBT)
# -----------------------------------------------------------------------------

@given(alias=doc_alias)
@settings(max_examples=40, deadline=None)
def test_document_aliases_normalize_deterministically(alias: str) -> None:
    data = run_normalize(alias, path="/sdcard/Download/fixture.md")
    mode = normalized_mode(data)
    assert "document" in mode or mode in {"file", "open_document", "document"}, data
    assert not is_error(data) or "unknown" not in classify(data).lower(), data


@given(alias=dir_alias)
@settings(max_examples=40, deadline=None)
def test_directory_aliases_normalize_deterministically(alias: str) -> None:
    data = run_normalize(alias, path="/sdcard/Download/md-dir")
    mode = normalized_mode(data)
    assert "directory" in mode or mode in {"dir", "tree", "open_directory", "directory"}, data
    assert not is_error(data) or "unknown" not in classify(data).lower(), data


@given(case_variant("uri"))
@settings(max_examples=12, deadline=None)
def test_uri_mode_normalizes_with_explicit_data_uri(alias: str) -> None:
    data = run_normalize(alias, data_uri="content://com.example/tree/primary%3Amd")
    mode = normalized_mode(data)
    assert "uri" in mode, data


# -----------------------------------------------------------------------------
# U1/FR-003/FR-007 deterministic precedence: path before data/uri in doc/dir modes
# -----------------------------------------------------------------------------

@given(alias=doc_alias, path=safe_token, uri=safe_token, data_uri=safe_token)
@settings(max_examples=60, deadline=None)
def test_document_mode_prefers_path_over_uri_and_data_uri(alias: str, path: str, uri: str, data_uri: str) -> None:
    data = run_normalize(alias, path=path, uri=f"content://other/{uri}", data_uri=f"file:///{data_uri}")
    # contract must make precedence observable and deterministic
    blob = json.dumps(data, sort_keys=True)
    selected = str(data.get("selected_reference_source", data.get("reference_source", data.get("resolved_from", "")))).lower()
    assert "path" in selected or path in blob, f"expected path precedence evidence: {data}"
    assert is_ok(data) or ("missing" not in classify(data).lower()), data


@given(alias=dir_alias, path=safe_token, uri=safe_token, data_uri=safe_token)
@settings(max_examples=60, deadline=None)
def test_directory_mode_prefers_path_over_uri_and_data_uri(alias: str, path: str, uri: str, data_uri: str) -> None:
    data = run_normalize(alias, path=path, uri=f"file:///{uri}", data_uri=f"file:///{data_uri}")
    blob = json.dumps(data, sort_keys=True)
    selected = str(data.get("selected_reference_source", data.get("reference_source", data.get("resolved_from", "")))).lower()
    assert "path" in selected or path in blob, f"expected path precedence evidence: {data}"
    assert is_ok(data) or ("missing" not in classify(data).lower()), data


@given(alias=doc_alias, path=safe_token)
@settings(max_examples=30, deadline=None)
def test_precedence_is_deterministic_for_same_inputs(alias: str, path: str) -> None:
    a = run_normalize(alias, path=path, uri="content://x/1", data_uri="content://x/2")
    b = run_normalize(alias, path=path, uri="content://x/1", data_uri="content://x/2")
    assert a == b, "normalization must be deterministic for identical inputs"


# -----------------------------------------------------------------------------
# FR-007 explicit diagnosable missing-input errors (never ok/silent pass)
# -----------------------------------------------------------------------------

@given(alias=doc_alias)
@settings(max_examples=30, deadline=None)
def test_document_missing_all_references_is_explicit_error(alias: str) -> None:
    data = run_normalize(alias)
    assert is_error(data), data
    cls = classify(data).lower()
    assert "missing" in cls or "error" in cls
    assert not is_ok(data)
    assert "ok:" not in cls


@given(alias=dir_alias)
@settings(max_examples=30, deadline=None)
def test_directory_missing_path_and_file_data_is_explicit_error(alias: str) -> None:
    # Directory contract: path/reference required; contract mode must diagnose missing resolvable ref.
    data = run_normalize(alias)
    assert is_error(data), data
    cls = classify(data).lower()
    assert "missing" in cls or "directory" in cls or "error" in cls
    assert not is_ok(data)


@given(case_variant("uri"))
@settings(max_examples=20, deadline=None)
def test_uri_mode_missing_data_is_explicit_error(alias: str) -> None:
    data = run_normalize(alias)
    assert is_error(data), data
    cls = classify(data).lower()
    assert "missing" in cls or "uri" in cls or "error" in cls
    assert not is_ok(data)


# -----------------------------------------------------------------------------
# FR-007 unknown mode => explicit diagnosable unknown-mode error
# -----------------------------------------------------------------------------

@given(
    mode=st.text(alphabet="abcdefghijklmnopqrstuvwxyz_-", min_size=1, max_size=24).filter(
        lambda m: m.lower() not in set(DOCUMENT_ALIASES + DIRECTORY_ALIASES + URI_MODES)
    )
)
@settings(max_examples=60, deadline=None)
def test_unknown_mode_is_explicit_diagnosable_error(mode: str) -> None:
    data = run_normalize(mode, path="/tmp/fixture.md")
    assert is_error(data), data
    cls = classify(data).lower()
    blob = json.dumps(data, sort_keys=True).lower()
    assert UNKNOWN_PREFIX in cls or UNKNOWN_PREFIX in blob or "unknown" in cls or "unknown" in blob
    assert not is_ok(data)


# -----------------------------------------------------------------------------
# Invariants over mixed inputs (safety: no silent pass, parseable JSON)
# -----------------------------------------------------------------------------

@given(mode=any_known_alias, path=maybe_token, uri=maybe_token, data_uri=maybe_token)
@settings(max_examples=80, deadline=None)
def test_normalization_never_silent_pass_without_resolvable_reference(mode: str, path: str | None, uri: str | None, data_uri: str | None) -> None:
    assume(not (path or uri or data_uri))
    data = run_normalize(mode, path=path, uri=uri, data_uri=data_uri)
    assert isinstance(data, dict)
    assert is_error(data), f"missing all refs must not pass: {data}"
    assert not is_ok(data)


@given(mode=any_known_alias, path=maybe_token, uri=maybe_token, data_uri=maybe_token)
@settings(max_examples=80, deadline=None)
def test_normalization_always_returns_parseable_object(mode: str, path: str | None, uri: str | None, data_uri: str | None) -> None:
    data = run_normalize(mode, path=path, uri=uri, data_uri=data_uri)
    assert isinstance(data, dict)
    # must carry machine-readable classification/outcome fields
    assert classify(data)
    assert json.dumps(data, sort_keys=True)
