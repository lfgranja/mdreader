"""T057 RED: Spec 012 directory/tree URI contract tests (US2, FR-003/FR-005/FR-007).

Host-contract level invariants (no git, no pytest execution required by assignment):
directory fixture references beyond app-accessible filesystem paths (content://
tree/document URIs, unsupported schemes, malformed forms) MUST either be handled
through the same production directory-open/manifest-dispatch path with
production-equivalent semantics, OR return an explicit diagnosable `error:*`
result. They MUST NEVER silently fall back, return fake success, or emit an
undocumented/ambiguous `ok:directory:*` form.

This file is intentionally RED/blocked until the hook exposes a deterministic
contract mode describing required outcomes (for example
`--print-directory-uri-contract --json`). Implementation is out of scope here.
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
EXPLICIT_ERROR_RE = re.compile(r"^error:[A-Za-z0-9_.-]+:[^\n]+$")


def run_hook(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(HOOK_SCRIPT), *args],
        cwd=REPO,
        text=True,
        capture_output=True,
        check=False,
    )


def contract_json() -> dict:
    proc = run_hook(["--print-directory-uri-contract", "--json"])
    assert proc.returncode == 0, (
        "T057 RED: hook must expose deterministic directory/tree URI contract mode "
        f"`--print-directory-uri-contract --json` (host-contract, no device side effects). "
        f"stdout={proc.stdout!r} stderr={proc.stderr!r}"
    )
    return json.loads((proc.stdout or proc.stderr).strip())


@st.composite
def non_filesystem_directory_refs(draw: st.DrawFn) -> str:
    authority = draw(
        st.text(
            alphabet=st.characters(
                whitelist_categories=("Ll", "Lu", "Nd"),
                whitelist_characters="._-:@",
            ),
            min_size=1,
            max_size=40,
        )
    )
    doc_id = draw(
        st.text(
            alphabet=st.characters(
                whitelist_categories=("Ll", "Lu", "Nd"),
                whitelist_characters="%:/._-",
            ),
            min_size=1,
            max_size=80,
        )
    )
    content_tree = f"content://{authority}/tree/{doc_id}"
    content_doc = f"content://{authority}/document/{doc_id}"
    malformed_content = draw(
        st.sampled_from(
            [
                "content://",
                "content:/missing-slash/tree/abc",
                "content://authority/tree",
                "content://authority/document/",
                "content:// authority/tree/abc",
                "content://%zz/tree/abc",
            ]
        )
    )
    unsupported = draw(
        st.sampled_from(
            [
                "ftp://example.invalid/tree",
                "http://example.invalid/tree",
                "https://example.invalid/tree",
                "intent://scan/#Intent;scheme=content;end",
                "android.resource://app.mdreader/123",
                "asset://fixtures/md-dir",
                "data:text/plain,not-a-directory",
            ]
        )
    )
    return draw(
        st.sampled_from([content_tree, content_doc, malformed_content, unsupported])
    )


def test_directory_uri_contract_mode_is_explicit_and_fail_closed() -> None:
    data = contract_json()
    assert data.get("scope_class") == "programmatic-non-ui"
    assert data.get("operation") == "directory"
    assert data.get("same_production_path_required") is True
    assert data.get("silent_fallback_allowed") is False
    assert data.get("fake_success_allowed") is False
    assert data.get("full_parent_completion_claimed") is False

    outcomes = data.get("required_outcomes") or {}
    # Contract must explicitly allow only production-equivalent success or diagnosable error.
    assert outcomes.get("production_equivalent_success") == "ok:directory:<absolute-path-or-production-equivalent-directory-ref>:chapters=<n>"
    assert outcomes.get("unsupported_or_unresolvable_failure_prefix") == "error:"
    assert outcomes.get("undocumented_ambiguous_success_allowed") is False

    notes = json.dumps(data, sort_keys=True)
    assert "content://" in notes
    assert "FR-003" in notes and "FR-005" in notes and "FR-007" in notes
    assert "US2-AC1" in notes or "US2" in notes
    assert "ASM-012-INGEST-SOT" in notes


@given(non_filesystem_directory_refs())
@settings(max_examples=80, deadline=None)
def test_non_filesystem_directory_refs_require_explicit_contract(ref: str) -> None:
    data = contract_json()
    classes = data.get("reference_classes") or {}
    # Generated refs must fall into a named class with an explicit required outcome.
    matched = []
    for name, spec in classes.items():
        if isinstance(spec, str):
            spec = {"requires": spec}
        requires = str(spec.get("requires") or spec.get("requirement") or spec)
        if "content://" in ref and "content" in name:
            matched.append((name, requires))
        elif "content://" not in ref and "unsupported" in name:
            matched.append((name, requires))
        elif "malformed" in name and "content:" in ref:
            matched.append((name, requires))
        if ref in (spec.get("examples") or []):
            matched.append((name, requires))
    # fallback: any non-fs ref must be covered by at least one class string mentioning explicit error/support
    if not matched:
        blob = json.dumps(classes).lower()
        assert "error" in blob and ("production" in blob or "explicit" in blob), (
            f"T057 RED: contract must classify representative non-filesystem directory ref {ref!r}; "
            f"reference_classes={sorted(classes)}"
        )
        return
    for name, requires in matched:
        low = requires.lower()
        assert ("production-equivalent" in low and "explicit" in low and "error" in low) or ("explicit" in low and "error" in low), f"class {name} must fail closed; requires={requires!r}"


@given(non_filesystem_directory_refs())
@settings(max_examples=80, deadline=None)
def test_non_filesystem_directory_outcomes_never_silent_or_fake(ref: str) -> None:
    """Invariant: any reported outcome for these refs is explicit diagnosable/ok form.

    The hook/script is not implemented in this T057 RED change; this test constrains
    the exposed contract classification matrix for representative outcomes.
    """
    data = contract_json()
    matrix = data.get("classification_matrix") or []
    assert matrix, "contract must provide classification_matrix for representative outcomes"

    # Every representative outcome must be parseable and never silent/ambiguous.
    if isinstance(matrix, dict):
        rows = []
        for k,v in matrix.items():
            rows.append({"reference": k, **v} if isinstance(v, dict) else {"reference": k, "outcome": v})
    else:
        rows = list(matrix)
    for row in rows:
        if isinstance(row, str):
            row = {"outcome": row}
        outcome = row.get("outcome", "")
        assert RESULT_RE.match(outcome) or outcome.startswith("blocked:"), (
            f"representative outcome must be explicit: {row!r}"
        )
        assert row.get("silent_fallback") is False
        assert row.get("fake_success") is False
        if outcome.startswith("ok:directory:"):
            assert row.get("production_equivalent_dispatch") is True, (
                "ok:directory for non-filesystem/tree refs is allowed only with explicit "
                f"production-equivalent dispatch evidence: {row!r}"
            )
            assert ":chapters=" in outcome
        if "content://" in (row.get("reference") or "") and outcome.startswith("ok:"):
            assert row.get("production_equivalent_dispatch") is True
        if row.get("reference") in {"ftp://example.invalid/tree", "intent://scan/#Intent;scheme=content;end"}:
            assert outcome.startswith("error:"), f"unsupported scheme must be explicit error: {row!r}"

    # Sanity: generated ref appears in examples/matrix intent (contract must cover class).
    refs = " ".join(str(r.get("reference", "")) for r in rows)
    assert "content://" in refs
    assert any(x in refs for x in ("ftp://", "intent://", "content:/", "example"))


@given(
    st.sampled_from(
        [
            "error:unsupported-directory-uri:content://authority/tree/doc%3A1",
            "error:invalid-directory-uri:content:/missing-slash/tree/abc",
            "error:unresolvable-directory-uri:content://authority/document/doc%3A1",
            "error:unsupported-directory-uri:ftp://example.invalid/tree",
        ]
    )
)
@settings(max_examples=20, deadline=None)
def test_explicit_directory_uri_errors_are_classified_as_fail_not_silent(outcome: str) -> None:
    assert EXPLICIT_ERROR_RE.match(outcome)
    proc = run_hook(["--classify-outcome", outcome, "--json"])
    assert proc.returncode == 0, proc.stderr
    data = json.loads((proc.stdout or proc.stderr).strip())
    assert data["status"] in {"fail", "error"}
    assert data.get("explicit_within_budget") is True
    assert data.get("counts_toward_sc002_denominator") is True
    assert data.get("counts_toward_sc002_denominator") is not None


@given(st.sampled_from(["ok:directory:/tmp/fake:chapters=1", "ok:directory::chapters=0"]))
@settings(max_examples=10, deadline=None)
def test_directory_success_form_cannot_be_claimed_without_contract_evidence(outcome: str) -> None:
    data = contract_json()
    policy = data.get("success_policy") or {}
    assert (policy.get("requires_chapters_field") is True or policy.get("ok_directory_requires_chapters") is True)
    assert (policy.get("requires_production_equivalent_dispatch_for_non_filesystem_refs") is True or policy.get("production_equivalent_dispatch_required_for_non_filesystem_refs") is True)
    assert policy.get("filesystem_path_may_use_existing_path_form") is True
    # Existing classify-outcome parses ok form generically; T057 requires the directory
    # URI contract to additionally constrain non-filesystem refs (fail closed/diagnosable).
    proc = run_hook(["--classify-outcome", outcome, "--json"])
    assert proc.returncode == 0
    classified = json.loads((proc.stdout or proc.stderr).strip())
    assert classified["status"] == "pass"
    assert "non_filesystem_ref_requires_contract_check" in json.dumps(policy)
