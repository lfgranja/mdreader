#!/usr/bin/env bash
# FR-011 durable overlay check (Spec 012). Non-mutating, fail-closed.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 - "$SCRIPT_DIR" "$@" <<'PYTHON'
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

HOOK_IDS = ("DebugIngestActivity", "app.mdreader.action.DEBUG_INGEST")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def now_utc() -> str:
    try:
        return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    except Exception as e:  # pragma: no cover
        return f"unavailable:{e}"


def usage_text() -> str:
    return (
        "Usage: scripts/check-debug-ingest-overlay.sh [--root <repo-root>] [--gen-dir <src-tauri/gen/android/app>] "
        "[--json] [--evidence-dir <dir>] [--help]\n"
        "Non-mutating fail-closed check of canonical vs generated FR-011 overlay targets.\n"
        "Reports pass|fail|blocked|not-applicable per target with path/expected-vs-actual/remediation.\n"
    )


def detect_format(root: Path) -> tuple[str | None, str, str | None]:
    marker = root / "specs/012-debug-ingest-hook/android-overlays/overlay-format.json"
    if not marker.exists():
        return None, "blocked", "missing overlay-format.json marker"
    try:
        obj = json.loads(marker.read_text(encoding="utf-8"))
    except Exception as e:
        return None, "fail", f"unreadable overlay-format.json: {e}"
    ver = obj.get("format_version")
    if ver is None:
        return "v1-implicit", "blocked", "missing format_version; v1-implicit not declared by script"
    sver = str(ver)
    if sver == "1":
        return sver, "pass", None
    return sver, "blocked", f"unknown/unsupported overlay format_version {sver!r}"


def safe_join(base: Path, rel: Path) -> Path | None:
    try:
        base_r = base.resolve()
        cand = (base / rel).resolve()
        cand.relative_to(base_r)
        return cand
    except Exception:
        return None


def target_row(tid: str, target: Path, canonical: Path | None, status: str, action: str, expected: str, actual: str, remediation: str, canon_bytes: bytes | None = None, target_bytes: bytes | None = None) -> dict:
    row = {
        "id": tid,
        "status": status,
        "action": action,
        "target_path": str(target),
        "path": str(target),
        "canonical_source": str(canonical) if canonical else "n/a",
        "expected": expected,
        "actual": actual,
        "remediation": remediation,
    }
    if canon_bytes is not None:
        row["canonical_sha256"] = sha256_bytes(canon_bytes)
    if target_bytes is not None:
        row["actual_sha256"] = sha256_bytes(target_bytes)
    return row


def scan_forbidden_exposure(gen_dir: Path) -> list[dict]:
    rows: list[dict] = []
    main_manifest = gen_dir / "src" / "main" / "AndroidManifest.xml"
    candidates: list[Path] = []
    if main_manifest.exists():
        candidates.append(main_manifest)
    src = gen_dir / "src"
    if src.exists():
        for p in src.rglob("AndroidManifest.xml"):
            try:
                rel = p.relative_to(src)
            except Exception:
                rel = p
            if rel.parts and rel.parts[0] == "debug":
                continue
            if p not in candidates:
                candidates.append(p)
    for p in candidates:
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except Exception as e:
            rows.append(target_row(
                "forbidden-hook-exposure", p, None, "blocked", "scanned",
                "no debug hook identifiers in non-debug managed/forbidden manifest context",
                f"unreadable: {e}",
                "Fix manifest readability and remove any debug hook exposure from non-debug targets.",
            ))
            continue
        hits = [h for h in HOOK_IDS if h in text]
        if hits:
            rows.append(target_row(
                "forbidden-hook-exposure", p, None, "fail", "scanned",
                "no debug hook identifiers in non-debug managed/forbidden manifest context",
                f"forbidden identifiers present: {', '.join(hits)}",
                "Remove debug hook identifiers from non-debug/main manifests; keep hook materialization debug-only (FR-010/FR-011).",
            ))
    return rows


def main(argv: list[str]) -> int:
    script_dir = Path(argv[1]).resolve()
    default_root = script_dir.parent
    ap = argparse.ArgumentParser(add_help=False)
    ap.add_argument("--root", default=str(default_root))
    ap.add_argument("--gen-dir", default=None)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--evidence-dir", default=None)
    ap.add_argument("--help", "-h", action="store_true")
    args = ap.parse_args(argv[2:])
    if args.help:
        sys.stdout.write(usage_text())
        return 0

    root = Path(args.root).resolve()
    gen_dir = Path(args.gen_dir).resolve() if args.gen_dir else (root / "src-tauri" / "gen" / "android" / "app").resolve()

    fmt_ver, fmt_status, fmt_msg = detect_format(root)
    overall_candidates: list[str] = []
    managed: list[dict] = []

    if fmt_status != "pass":
        overall_candidates.append(fmt_status)

    canonical_root = root / "specs" / "012-debug-ingest-hook" / "android-overlays" / "app" / "src"
    debug_canon = canonical_root / "debug" / "AndroidManifest.xml"
    debug_target = gen_dir / "src" / "debug" / "AndroidManifest.xml"
    at_canon = canonical_root / "androidTest" / "java" / "app" / "mdreader" / "DebugIngestSeamSmokeTest.kt"
    at_target = gen_dir / "src" / "androidTest" / "java" / "app" / "mdreader" / "DebugIngestSeamSmokeTest.kt"

    # Path safety for configured managed target locations (no writes performed).
    for rel in [Path("src/debug/AndroidManifest.xml"), Path("src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt")]:
        if safe_join(gen_dir, rel) is None:
            managed.append(target_row(f"path-safety:{rel}", gen_dir / rel, None, "fail", "checked",
                                      "managed target resolves inside --gen-dir", "path escapes or is unsafe",
                                      "Provide a safe --gen-dir and do not use traversal segments."))
            overall_candidates.append("fail")

    # Debug manifest (required)
    if not debug_canon.exists():
        managed.append(target_row("MAT-012-DEBUG-MANIFEST", debug_target, debug_canon, "fail", "checked",
                                  "canonical debug manifest exists and target matches it",
                                  "missing canonical debug manifest source",
                                  f"Restore canonical source at {debug_canon}; never use src-tauri/gen/** as canonical."))
        overall_candidates.append("fail")
    else:
        canon_bytes = debug_canon.read_bytes()
        if not debug_target.exists():
            managed.append(target_row("MAT-012-DEBUG-MANIFEST", debug_target, debug_canon, "fail", "checked",
                                      "debug manifest target exists and matches canonical", "missing target",
                                      "Run scripts/apply-debug-ingest-overlay.sh to materialize from canonical overlay.", canon_bytes))
            overall_candidates.append("fail")
        else:
            try:
                target_bytes = debug_target.read_bytes()
            except Exception as e:
                managed.append(target_row("MAT-012-DEBUG-MANIFEST", debug_target, debug_canon, "blocked", "checked",
                                          "debug manifest target readable and matches canonical", f"unreadable: {e}",
                                          "Fix filesystem/path permissions and rerun check.", canon_bytes))
                overall_candidates.append("blocked")
                target_bytes = None
            if target_bytes is not None:
                if target_bytes == canon_bytes:
                    managed.append(target_row("MAT-012-DEBUG-MANIFEST", debug_target, debug_canon, "pass", "checked",
                                              "debug manifest target matches canonical", "aligned",
                                              "No action needed.", canon_bytes, target_bytes))
                else:
                    managed.append(target_row("MAT-012-DEBUG-MANIFEST", debug_target, debug_canon, "fail", "checked",
                                              "debug manifest target matches canonical", "stale/mismatched",
                                              "Run scripts/apply-debug-ingest-overlay.sh; do not edit src-tauri/gen/** manually.",
                                              canon_bytes, target_bytes))
                    overall_candidates.append("fail")

    # androidTest (conditional FR-013/SC-006): always report a row
    if not at_canon.exists():
        managed.append(target_row("MAT-012-ANDROIDTEST-SMOKE", at_target, at_canon, "not-applicable", "checked",
                                  "androidTest scaffold required only when canonical source exists",
                                  "canonical androidTest source absent",
                                  "No action needed."))
    else:
        canon_bytes = at_canon.read_bytes()
        if not at_target.exists():
            managed.append(target_row("MAT-012-ANDROIDTEST-SMOKE", at_target, at_canon, "fail", "checked",
                                      "androidTest scaffold target exists and matches canonical", "missing target",
                                      "Run scripts/apply-debug-ingest-overlay.sh to materialize canonical androidTest scaffold.",
                                      canon_bytes))
            overall_candidates.append("fail")
        else:
            try:
                target_bytes = at_target.read_bytes()
            except Exception as e:
                managed.append(target_row("MAT-012-ANDROIDTEST-SMOKE", at_target, at_canon, "blocked", "checked",
                                          "androidTest scaffold target readable and matches canonical", f"unreadable: {e}",
                                          "Fix filesystem/path permissions and rerun check.", canon_bytes))
                overall_candidates.append("blocked")
                target_bytes = None
            if target_bytes is not None:
                if target_bytes == canon_bytes:
                    managed.append(target_row("MAT-012-ANDROIDTEST-SMOKE", at_target, at_canon, "pass", "checked",
                                              "androidTest scaffold target matches canonical", "aligned",
                                              "No action needed.", canon_bytes, target_bytes))
                else:
                    managed.append(target_row("MAT-012-ANDROIDTEST-SMOKE", at_target, at_canon, "fail", "checked",
                                              "androidTest scaffold target matches canonical", "stale/mismatched",
                                              "Run scripts/apply-debug-ingest-overlay.sh; do not edit src-tauri/gen/** manually.",
                                              canon_bytes, target_bytes))
                    overall_candidates.append("fail")

    # Non-mutating scan for forbidden exposure in managed/non-debug context under gen dir.
    for row in scan_forbidden_exposure(gen_dir):
        managed.append(row)
        overall_candidates.append(row["status"] if row["status"] in {"fail", "blocked"} else "fail")

    # overall: pass only when every required managed target is pass or explicit not-applicable and no blocking/format error
    if "fail" in overall_candidates:
        overall = "fail"
    elif "blocked" in overall_candidates:
        overall = "blocked"
    else:
        bad = [r for r in managed if r.get("status") not in {"pass", "not-applicable"}]
        if not bad:
            overall = "pass"
        else:
            overall = "fail" if any(r.get("status") == "fail" for r in bad) else "blocked"

    report = {
        "tool_name": "check-debug-ingest-overlay",
        "operation": "check",
        "overall_status": overall,
        "detected_overlay_format_version": fmt_ver,
        "format_status": fmt_status,
        "format_message": fmt_msg,
        "repo_root": str(root),
        "gen_dir": str(gen_dir),
        "canonical_overlay_root": str(canonical_root),
        "managed_targets": managed,
        "targets": managed,
        "captured_at_utc": now_utc(),
        "supports_ids": ["FR-011", "FR-013", "SC-006", "CHK018", "CHK024", "NFR-012-MAINT-001", "NFR-012-MAINT-002"],
    }
    if args.evidence_dir:
        try:
            ed = Path(args.evidence_dir)
            ed.mkdir(parents=True, exist_ok=True)
            (ed / "check-debug-ingest-overlay.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            report["evidence_dir"] = str(ed)
            report["write_status"] = "written"
        except Exception as e:
            report["evidence_dir"] = str(args.evidence_dir)
            report["write_status"] = f"failed:{e}"

    out = json.dumps(report, indent=2, sort_keys=True)
    if args.json:
        sys.stdout.write(out + "\n")
    else:
        sys.stdout.write(f"overall_status={overall}\n")
        for r in managed:
            sys.stdout.write(f"- {r.get('id')} {r.get('status')} {r.get('action')} {r.get('target_path')}\n  expected={r.get('expected')}\n  actual={r.get('actual')}\n  remediation={r.get('remediation')}\n")
        sys.stdout.write(out + "\n")
    return 0 if overall == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
PYTHON
