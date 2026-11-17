"""Spec 012 FR-011 durable overlay dependency/layout contract tests (T064 RED).

Covers durable-overlay-materialization.md §10/CHK026/CHK027, ASM-012-SOURCESET-WIRING,
ASM-012-REPO-LAYOUT, structured `dependency_layout_results`, FR-011, NFR-012-MAINT-004.

TDD RED gate (blocks T066): expected to FAIL until check/apply report structured
`dependency_layout_results` and fail closed on unverifiable source-set wiring.
No git, no pytest-cargo/long suites required.
"""
from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path
from typing import Any, Iterable

import pytest

hypothesis = pytest.importorskip("hypothesis")
from hypothesis import given, settings, strategies as st  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
APPLY = REPO / "scripts" / "apply-debug-ingest-overlay.sh"
CHECK = REPO / "scripts" / "check-debug-ingest-overlay.sh"
CANONICAL_DEBUG = REPO / "specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml"
FORMAT_MARKER = REPO / "specs/012-debug-ingest-hook/android-overlays/overlay-format.json"
REQUIRED_SOURCE_SET = "android-shell/src/main/kotlin"
MIN_DEP_LAYOUT_IDS = {"ASM-012-REPO-LAYOUT", "ASM-012-SOURCESET-WIRING"}
RELEVANT_DEP_PREFIXES = ("DEP-012-", "ASM-012-")


def run_script(script: Path, args: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(script), *args],
        cwd=cwd or REPO,
        text=True,
        capture_output=True,
        check=False,
        timeout=60,
    )


def parse_json(proc: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    out = (proc.stdout or "").strip() or (proc.stderr or "").strip()
    try:
        return json.loads(out)
    except json.JSONDecodeError as e:  # pragma: no cover - RED path
        raise AssertionError(
            f"expected JSON output, rc={proc.returncode}\nstdout={proc.stdout}\nstderr={proc.stderr}"
        ) from e


def iter_dep_rows(report: dict[str, Any]) -> list[dict[str, Any]]:
    raw = report.get("dependency_layout_results")
    assert raw is not None, f"missing structured dependency_layout_results; keys={sorted(report)}"
    if isinstance(raw, dict):
        rows: list[dict[str, Any]] = []
        for key, value in raw.items():
            if isinstance(value, dict):
                row = {"id": key, **value}
            else:
                row = {"id": key, "value": value}
            rows.append(row)
        return rows
    assert isinstance(raw, list), f"dependency_layout_results must be structured list/dict, got {type(raw)}"
    out: list[dict[str, Any]] = []
    for item in raw:
        assert isinstance(item, dict), f"dependency_layout row must be object, got {type(item)}: {item!r}"
        out.append(item)
    return out


def dep_row_ids(rows: Iterable[dict[str, Any]]) -> set[str]:
    ids: set[str] = set()
    for row in rows:
        for key in ("id", "dep_id", "asm_id", "name"):
            value = row.get(key)
            if isinstance(value, str) and value:
                ids.add(value)
                break
    return ids


def rows_for_id(rows: Iterable[dict[str, Any]], wanted: str) -> list[dict[str, Any]]:
    return [r for r in rows if wanted in {r.get("id"), r.get("dep_id"), r.get("asm_id"), r.get("name")}]


def assert_dep_ids_covered(report: dict[str, Any]) -> list[dict[str, Any]]:
    rows = iter_dep_rows(report)
    ids = dep_row_ids(rows)
    missing = sorted(MIN_DEP_LAYOUT_IDS - ids)
    assert not missing, f"dependency_layout_results missing required ASM IDs: {missing}; present={sorted(ids)}"
    dep_asm_rows = [r for r in rows if any(str(r.get(k, "")).startswith(RELEVANT_DEP_PREFIXES) for k in ("id", "dep_id", "asm_id", "name"))]
    assert dep_asm_rows, f"expected relevant DEP-012-* / ASM-012-* rows in invoked scope; rows={rows}"
    for row in rows:
        status = row.get("status") or row.get("result") or row.get("state")
        assert status in {"pass", "fail", "blocked", "not-applicable", "required", "ok", "validated"}, (
            f"dependency row missing explicit status/state: {row}"
        )
    return rows


def make_tmp_repo(tmp_path: Path, *, wiring_includes: bool | None, remove_build_gradle: bool = False) -> tuple[Path, Path]:
    """Create minimal repo-shaped root for FR-011 dep/layout checks.

    wiring_includes=None means do not create build.gradle.kts (unverifiable).
    """
    root = tmp_path / "repo"
    overlay_src = root / "specs/012-debug-ingest-hook/android-overlays/app/src/debug"
    overlay_src.mkdir(parents=True)
    marker = root / "specs/012-debug-ingest-hook/android-overlays/overlay-format.json"
    marker.write_text(FORMAT_MARKER.read_text(encoding="utf-8"), encoding="utf-8")
    (overlay_src / "AndroidManifest.xml").write_text(CANONICAL_DEBUG.read_text(encoding="utf-8"), encoding="utf-8")

    gen = root / "src-tauri/gen/android/app"
    (gen / "src/debug").mkdir(parents=True)
    build_gradle = gen / "build.gradle.kts"
    if remove_build_gradle:
        if build_gradle.exists():
            build_gradle.unlink()
    elif wiring_includes is True:
        build_gradle.write_text(
            'plugins { id("com.android.application") }\n'
            f'android {{ sourceSets["main"].java.srcDir("{REQUIRED_SOURCE_SET}") }}\n',
            encoding="utf-8",
        )
    elif wiring_includes is False:
        build_gradle.write_text(
            'plugins { id("com.android.application") }\nandroid { sourceSets["main"].java.srcDir("src/main/java") }\n',
            encoding="utf-8",
        )
    # wiring_includes is None => unverifiable/missing file
    return root, gen


def assert_wiring_fail_closed(report: dict[str, Any], proc: subprocess.CompletedProcess[str]) -> None:
    assert report.get("overall_status") in {"fail", "blocked"}, report
    assert proc.returncode != 0, f"fail-closed dep/layout claim must use non-zero exit; report={report}"
    rows = assert_dep_ids_covered(report)
    wiring_rows = rows_for_id(rows, "ASM-012-SOURCESET-WIRING")
    assert wiring_rows, f"missing ASM-012-SOURCESET-WIRING row: {rows}"
    assert any((r.get("status") or r.get("result") or r.get("state")) in {"fail", "blocked"} for r in wiring_rows), (
        f"ASM-012-SOURCESET-WIRING must be explicit fail/blocked when unverifiable/invalid: {wiring_rows}"
    )
    details = " ".join(json.dumps(r, sort_keys=True) for r in wiring_rows).lower()
    assert "android-shell/src/main/kotlin" in details or "source-set" in details or "sourceset" in details or "build.gradle" in details, (
        f"ASM-012-SOURCESET-WIRING row must mention required wiring context: {wiring_rows}"
    )


@pytest.mark.parametrize("script", [CHECK, APPLY])
def test_reports_structured_dependency_layout_results_for_aligned_fixture(tmp_path: Path, script: Path) -> None:
    root, gen = make_tmp_repo(tmp_path, wiring_includes=True)
    # materialize first for check; apply is idempotent/non-regressive.
    run_script(APPLY, ["--root", str(root), "--gen-dir", str(gen), "--json"])
    proc = run_script(script, ["--root", str(root), "--gen-dir", str(gen), "--json"])
    report = parse_json(proc)
    rows = assert_dep_ids_covered(report)
    wiring_rows = rows_for_id(rows, "ASM-012-SOURCESET-WIRING")
    assert wiring_rows, rows
    assert any((r.get("status") or r.get("result") or r.get("state")) in {"pass", "validated", "ok", "required"} for r in wiring_rows) or report.get("overall_status") in {"pass", "fail", "blocked"}, wiring_rows
    repo_rows = rows_for_id(rows, "ASM-012-REPO-LAYOUT")
    assert repo_rows, rows
    for row in repo_rows + wiring_rows:
        text = json.dumps(row, sort_keys=True)
        assert "ASM-012" in text


def test_check_dep_layout_validation_is_non_mutating(tmp_path: Path) -> None:
    root, gen = make_tmp_repo(tmp_path, wiring_includes=True)
    build_gradle = gen / "build.gradle.kts"
    debug_target = gen / "src/debug/AndroidManifest.xml"
    before = {
        "build_gradle": build_gradle.read_bytes(),
        "debug_exists": debug_target.exists(),
        "gen_tree": sorted(str(p.relative_to(gen)) for p in gen.rglob("*")),
    }
    evidence_dir = tmp_path / "evidence"
    proc = run_script(CHECK, ["--root", str(root), "--gen-dir", str(gen), "--json", "--evidence-dir", str(evidence_dir)])
    report = parse_json(proc)
    assert_dep_ids_covered(report)
    assert build_gradle.read_bytes() == before["build_gradle"], "check must not mutate build.gradle.kts"
    assert debug_target.exists() == before["debug_exists"], "check must not create/mutate managed debug target"
    assert sorted(str(p.relative_to(gen)) for p in gen.rglob("*")) == before["gen_tree"], "check must remain non-mutating under gen dir"
    assert (evidence_dir / "check-debug-ingest-overlay.json").exists(), "explicit evidence-dir output is allowed"


def test_check_fails_closed_when_sourceset_wiring_cannot_be_verified(tmp_path: Path) -> None:
    root, gen = make_tmp_repo(tmp_path, wiring_includes=None, remove_build_gradle=True)
    run_script(APPLY, ["--root", str(root), "--gen-dir", str(gen), "--json"])
    proc = run_script(CHECK, ["--root", str(root), "--gen-dir", str(gen), "--json"])
    report = parse_json(proc)
    assert_wiring_fail_closed(report, proc)


def test_apply_fails_closed_when_sourceset_wiring_missing_required_include(tmp_path: Path) -> None:
    root, gen = make_tmp_repo(tmp_path, wiring_includes=False)
    proc = run_script(APPLY, ["--root", str(root), "--gen-dir", str(gen), "--json"])
    report = parse_json(proc)
    assert_wiring_fail_closed(report, proc)


@given(
    prefix=st.text(alphabet=st.characters(blacklist_categories=("Cs",), blacklist_characters="\x00"), max_size=80),
    suffix=st.text(alphabet=st.characters(blacklist_categories=("Cs",), blacklist_characters="\x00"), max_size=80),
    include_required=st.booleans(),
)
@settings(max_examples=30, deadline=None)
def test_sourceset_wiring_invariant_is_explicit_and_fail_closed(prefix: str, suffix: str, include_required: bool) -> None:
    tmp_path = Path(REPO) / "target" / "spec012-t064-hypothesis"
    case = tmp_path / f"case-{abs(hash((prefix, suffix, include_required)))}"
    if case.exists():
        shutil.rmtree(case)
    case.mkdir(parents=True)
    try:
        root, gen = make_tmp_repo(case, wiring_includes=None)
        content = f"{prefix}\n"
        if include_required:
            content += f'android {{ sourceSets["main"].java.srcDir("{REQUIRED_SOURCE_SET}") }}\n'
        content += f"{suffix}\n"
        (gen / "build.gradle.kts").write_text(content, encoding="utf-8")
        # Keep managed targets aligned so dependency/layout behavior is the distinguishing claim.
        run_script(APPLY, ["--root", str(root), "--gen-dir", str(gen), "--json"])
        proc = run_script(CHECK, ["--root", str(root), "--gen-dir", str(gen), "--json"])
        report = parse_json(proc)
        rows = assert_dep_ids_covered(report)
        wiring_rows = rows_for_id(rows, "ASM-012-SOURCESET-WIRING")
        assert wiring_rows, rows
        if include_required:
            assert report.get("overall_status") in {"pass", "fail", "blocked"}
            assert not any((r.get("status") or r.get("result") or r.get("state")) in {"fail", "blocked"} and "unverifiable" in json.dumps(r).lower() for r in wiring_rows), wiring_rows
        else:
            assert_wiring_fail_closed(report, proc)
    finally:
        shutil.rmtree(case, ignore_errors=True)


@given(
    extra_dep=st.sampled_from([
        "DEP-012-TAURI-GEN",
        "DEP-012-ANDROID-BUILD",
        "DEP-012-REPO-SCRIPTS",
        "DEP-012-GIT-CHECKOUT",
        "DEP-012-GEN-COMPAT",
    ])
)
@settings(max_examples=5, deadline=None)
def test_dependency_layout_results_cover_relevant_invoked_scope_ids(extra_dep: str) -> None:
    """Structured dependency_layout_results must validate required items for invoked scope.

    Contract minimum: ASM-012-REPO-LAYOUT, ASM-012-SOURCESET-WIRING (where claims depend),
    and relevant DEP-012-* for invoked scope. This invariant checks coverage is explicit,
    not a silent pass/omission.
    """
    tmp_path = Path(REPO) / "target" / "spec012-t064-scope"
    case = tmp_path / extra_dep
    if case.exists():
        shutil.rmtree(case)
    case.mkdir(parents=True)
    try:
        root, gen = make_tmp_repo(case, wiring_includes=True)
        run_script(APPLY, ["--root", str(root), "--gen-dir", str(gen), "--json"])
        proc = run_script(CHECK, ["--root", str(root), "--gen-dir", str(gen), "--json"])
        report = parse_json(proc)
        rows = assert_dep_ids_covered(report)
        ids = dep_row_ids(rows)
        # RED: implementations must include relevant DEP-012-* scope validation rows; at minimum
        # the common script/checkout/layout-scoped deps and the sampled relevant dep when in scope.
        expected_scope = {"DEP-012-REPO-SCRIPTS", "DEP-012-GIT-CHECKOUT", "ASM-012-REPO-LAYOUT", "ASM-012-SOURCESET-WIRING"}
        missing = sorted(expected_scope - ids)
        assert not missing, f"dependency_layout_results missing relevant invoked-scope IDs {missing}; present={sorted(ids)}"
        # sampled dep must be represented or explicitly justified as not-applicable in scope; absence is silent.
        assert extra_dep in ids or any(extra_dep in json.dumps(r) for r in rows), (
            f"relevant DEP row {extra_dep} not explicitly represented in dependency_layout_results: {rows}"
        )
    finally:
        shutil.rmtree(case, ignore_errors=True)
