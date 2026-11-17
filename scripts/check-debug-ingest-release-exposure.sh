#!/usr/bin/env bash
# FR-010/SC-004 release/non-debug debug-ingest exposure evidence (Spec 012).
# Non-mutating, fail-closed. static/runtime NEVER satisfy FR-010/SC-004; only
# mode=full can claim satisfies_fr010_sc004=true when D1-D4 pass with identity binding.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 - "$SCRIPT_DIR" "$@" <<'PYTHON'
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

IN_SCOPE_IDENTIFIERS = [
    "DebugIngestActivity",
    "DebugIngestHook",
    "app.mdreader.action.DEBUG_INGEST",
]
CLASS_PRESENCE_NOTE = (
    "Class/dex scanning is supplemental only and provides insufficient proof of presence or absence of external exposure: "
    "debug hook Kotlin sources may be compiled via the shared android-shell main source set, so class presence/absence "
    "must not be treated as proof. Authoritative signals are manifest/component/export/action resolution plus runtime invocation behavior."
)


def sha256_file(path: Path) -> str | None:
    try:
        h = hashlib.sha256()
        with path.open("rb") as f:
            for chunk in iter(lambda: f.read(65536), b""):
                h.update(chunk)
        return h.hexdigest()
    except Exception:
        return None


def now_utc() -> str:
    try:
        return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    except Exception as e:  # pragma: no cover
        return f"unavailable:{e}"


def usage_text() -> str:
    return (
        "Usage: scripts/check-debug-ingest-release-exposure.sh [OPTIONS]\n"
        "FR-010/SC-004 release/non-debug debug-ingest exposure evidence (Spec 012).\n"
        "Fail-closed/blocked: missing required inputs, tools, or identity binding =>\n"
        "blocked/not-evaluated or fail; never satisfies FR-010/SC-004 without full D1-D4 evidence.\n\n"
        "Options:\n"
        "  --mode static|runtime|full   Evidence mode (default: full when omitted? explicit recommended).\n"
        "                               static/runtime NEVER satisfy FR-010/SC-004 (satisfies_fr010_sc004=false).\n"
        "                               mode=full requires complete D1-D4 + artifact/install identity binding.\n"
        "  --json                       Emit machine-readable JSON.\n"
        "  --print-scope                Report in-scope identifiers/evidence scope.\n"
        "  --apk <path>                 Release/non-debug APK/AAB path (static/full artifact input).\n"
        "  --variant <name>             Build variant (e.g. universalRelease).\n"
        "  --merger-report <path>       Manifest merger report (static/full when available).\n"
        "  --merged-manifest <path>     Merged/packaged manifest (static/full when available).\n"
        "  --serial <adb-serial>        Device serial (runtime/full).\n"
        "  --package <id>               Package id (default app.mdreader).\n"
        "  --component <component>      Component for negative invocation.\n"
        "  --action <action>            Intent action for negative invocation.\n"
        "  --adb-runner <cmd>           Injectable ADB/runner command for contract-testable mechanization\n"
        "                               (also DEBUG_INGEST_ADB_RUNNER env var).\n"
        "  --evidence-dir <dir>         Optional evidence output directory.\n"
        "  -h, --help                   Show help.\n"
    )


def d(status: str, summary: str, **extra) -> dict:
    row = {"status": status, "summary": summary}
    row.update(extra)
    return row


def tool_record(name: str, required: bool = False, *, runner: str | None = None) -> dict:
    if runner:
        return {
            "name": name,
            "path": runner,
            "available": True,
            "required": required,
            "runner": runner,
            "kind": "injectable-runner",
        }
    path = shutil.which(name)
    return {"name": name, "path": path, "available": bool(path), "required": required}


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(add_help=False)
    ap.add_argument("--mode", choices=["static", "runtime", "full"], default=None)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--print-scope", action="store_true")
    ap.add_argument("--apk", default=None)
    ap.add_argument("--variant", default=None)
    ap.add_argument("--merger-report", default=None)
    ap.add_argument("--merged-manifest", default=None)
    ap.add_argument("--serial", default=None)
    ap.add_argument("--package", default="app.mdreader")
    ap.add_argument("--component", default=None)
    ap.add_argument("--action", default=None)
    ap.add_argument("--adb-runner", default=None)
    ap.add_argument("--evidence-dir", default=None)
    ap.add_argument("--help", "-h", action="store_true")
    return ap


def scope_payload() -> dict:
    return {
        "tool_name": "check-debug-ingest-release-exposure",
        "in_scope_identifiers": list(IN_SCOPE_IDENTIFIERS),
        "in_scope_artifacts": [
            "DebugIngestActivity component/manifest materialization",
            "DebugIngestHook guard/inert behavior outside debug",
            "app.mdreader.action.DEBUG_INGEST intent action",
            "debug manifest/intent-filter identifiers",
            "materialized debug/androidTest overlay targets",
        ],
        "evidence_dimensions": ["d1_scope", "d2_static_absence", "d3_runtime_negative", "d4_repeatable"],
        "modes": {
            "static": "host/artifact checks only; never satisfies FR-010/SC-004",
            "runtime": "installed-build negative invocation only; never satisfies FR-010/SC-004",
            "full": "requires D1-D4 + artifact/install identity; only mode that may satisfy FR-010/SC-004 when all pass",
        },
        "fail_closed": True,
        "blocked_behavior": "missing required inputs/tools/identity => blocked-not-evaluated or fail; never stale identity reuse",
        "satisfies_fr010_sc004": False,
        "captured_at_utc": now_utc(),
    }


def emit(data: dict, as_json: bool) -> None:
    text = json.dumps(data, indent=2, sort_keys=True)
    if as_json:
        sys.stdout.write(text + "\n")
    else:
        overall = data.get("overall", "unknown")
        sys.stdout.write(f"overall={overall} satisfies_fr010_sc004={data.get('satisfies_fr010_sc004')}\n")
        sys.stdout.write(text + "\n")


def _read_text(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        return None


def _scan_text_for_identifiers(label: str, path: Path, text: str | None, tool: dict) -> dict:
    findings = {}
    present_any = False
    errors = []
    if text is None:
        for ident in IN_SCOPE_IDENTIFIERS:
            findings[ident] = {"found": None, "count": 0, "status": "unreadable"}
        errors.append(f"unreadable:{path}")
    else:
        for ident in IN_SCOPE_IDENTIFIERS:
            count = text.count(ident)
            found = count > 0
            present_any = present_any or found
            findings[ident] = {"found": found, "count": count, "status": "scanned"}
    return {
        "artifact_kind": label,
        "path": str(path),
        "exists": path.exists(),
        "sha256": sha256_file(path),
        "tool": tool,
        "identifier_findings": findings,
        "identifier_scan": findings,
        "any_in_scope_identifier_present": present_any,
        "scan_performed": text is not None,
        "errors": errors,
    }


def _apk_inspection(apk: Path, tool_provenance: dict) -> dict:
    errors = []
    scans = []
    any_present = False
    scan_performed = False
    candidates = []
    if tool_provenance.get("apkanalyzer", {}).get("available"):
        candidates.append(("apkanalyzer", ["apkanalyzer", "manifest", "print", str(apk)]))
    if tool_provenance.get("aapt2", {}).get("available"):
        candidates.append(("aapt2-xmltree", ["aapt2", "dump", "xmltree", str(apk), "AndroidManifest.xml"]))
        candidates.append(("aapt2-badging", ["aapt2", "dump", "badging", str(apk)]))
    if tool_provenance.get("unzip", {}).get("available"):
        candidates.append(("unzip-list", ["unzip", "-l", str(apk)]))
    if not candidates:
        errors.append("no APK inspection tool available (apkanalyzer/aapt2/unzip); fail-closed for authoritative APK content proof")
    for name, cmd in candidates:
        try:
            p = subprocess.run(cmd, capture_output=True, text=True, check=False)
            out = (p.stdout or "") + "\n" + (p.stderr or "")
            rec = _scan_text_for_identifiers(
                f"apk:{name}",
                apk,
                out if (out.strip() or p.returncode == 0) else None,
                {"name": cmd[0], "command": cmd, "returncode": p.returncode, "path": shutil.which(cmd[0])},
            )
            scans.append(rec)
            scan_performed = scan_performed or rec.get("scan_performed") is True
            any_present = any_present or rec.get("any_in_scope_identifier_present") is True
            if p.returncode != 0 and not out.strip():
                errors.append(f"{name}:returncode={p.returncode}")
        except Exception as e:
            errors.append(f"{name}:{e}")
    findings = {
        ident: {"found": False, "count": 0, "status": "not-detected-by-attempted-tools", "sources": []}
        for ident in IN_SCOPE_IDENTIFIERS
    }
    for rec in scans:
        for ident, row in (rec.get("identifier_findings") or {}).items():
            if ident not in findings:
                continue
            if row.get("found"):
                findings[ident]["found"] = True
                findings[ident]["status"] = "scanned"
            findings[ident]["count"] += int(row.get("count") or 0)
            findings[ident]["sources"].append(
                {"artifact_kind": rec.get("artifact_kind"), "found": row.get("found"), "count": row.get("count")}
            )
    if not scan_performed:
        for ident in findings:
            findings[ident]["status"] = "blocked-no-apk-tool"
    return {
        "artifact_kind": "apk_or_aab",
        "path": str(apk),
        "exists": apk.exists(),
        "sha256": sha256_file(apk),
        "scan_performed": scan_performed,
        "tools_attempted": [c[0] for c in candidates],
        "sub_inspections": scans,
        "identifier_findings": findings,
        "identifier_scan": findings,
        "any_in_scope_identifier_present": any_present,
        "errors": errors,
        "fail_closed": bool(errors),
        "tool": {"name": "apk-host-inspection", "tools_attempted": [c[0] for c in candidates], "provenance": tool_provenance},
    }


def mechanize_static(args, mode: str) -> tuple[dict, dict, dict, dict, bool]:
    apk = Path(args.apk).resolve() if args.apk else None
    merger = Path(args.merger_report).resolve() if args.merger_report else None
    merged_manifest = Path(args.merged_manifest).resolve() if args.merged_manifest else None
    missing = []
    if apk is None:
        missing.append("--apk (release/non-debug artifact)")
    elif not apk.exists():
        missing.append(f"--apk exists: {apk}")
    tool_provenance = {
        "aapt2": tool_record("aapt2"),
        "apkanalyzer": tool_record("apkanalyzer"),
        "unzip": tool_record("unzip"),
        "bundletool": tool_record("bundletool"),
    }
    artifact_identity = {
        "variant": args.variant,
        "apk_path": str(apk) if apk else None,
        "apk_sha256": sha256_file(apk) if apk and apk.exists() else None,
        "merger_report_path": str(merger) if merger else None,
        "merger_report_sha256": sha256_file(merger) if merger and merger.exists() else None,
        "merged_manifest_path": str(merged_manifest) if merged_manifest else None,
        "merged_manifest_sha256": sha256_file(merged_manifest) if merged_manifest and merged_manifest.exists() else None,
        "package_id": args.package,
    } if (apk or merger or merged_manifest or args.variant) else None
    inspections = []
    inspected_artifacts = []
    inspection_performed = False
    inspection_tools = []
    if merged_manifest and merged_manifest.exists():
        rec = _scan_text_for_identifiers("merged_manifest", merged_manifest, _read_text(merged_manifest), {"name": "text-scan", "path": "builtin"})
        inspections.append(rec)
        inspected_artifacts.append({"kind": rec["artifact_kind"], "path": rec["path"], "sha256": rec["sha256"], "tool": rec["tool"]})
        inspection_performed = True
        inspection_tools.append(rec["tool"])
    elif merged_manifest:
        missing.append(f"--merged-manifest exists: {merged_manifest}")
    if merger and merger.exists():
        rec = _scan_text_for_identifiers("merger_report", merger, _read_text(merger), {"name": "text-scan", "path": "builtin"})
        inspections.append(rec)
        inspected_artifacts.append({"kind": rec["artifact_kind"], "path": rec["path"], "sha256": rec["sha256"], "tool": rec["tool"]})
        inspection_performed = True
        inspection_tools.append(rec["tool"])
    elif merger:
        missing.append(f"--merger-report exists: {merger}")
    class_scan_ran = bool(apk and apk.exists())
    if apk and apk.exists():
        apk_scan = _apk_inspection(apk, tool_provenance)
        inspections.append(apk_scan)
        inspected_artifacts.append({"kind": apk_scan["artifact_kind"], "path": apk_scan["path"], "sha256": apk_scan["sha256"], "tool": apk_scan.get("tool")})
        inspection_performed = True
        inspection_tools.extend([{"name": t, "path": shutil.which(t)} for t in apk_scan.get("tools_attempted", [])])
    aggregate = {ident: {"found": False, "count": 0, "sources": [], "status": "scanned" if inspection_performed else "not-scanned"} for ident in IN_SCOPE_IDENTIFIERS}
    any_present = False
    for rec in inspections:
        for ident, row in (rec.get("identifier_findings") or {}).items():
            if row.get("found"):
                aggregate[ident]["found"] = True
                any_present = True
            aggregate[ident]["count"] += int(row.get("count") or 0)
            aggregate[ident]["sources"].append({"artifact_kind": rec.get("artifact_kind"), "path": rec.get("path"), "found": row.get("found"), "count": row.get("count")})
    blocked = bool(missing)
    if inspection_performed and not blocked:
        status = "fail" if any_present else "pass"
        summary = (
            "Static inspection found in-scope debug-ingest identifier(s) in release/non-debug artifact inputs."
            if any_present
            else "Static inspection of provided merged manifest/merger/APK-AAB inputs found no in-scope debug-ingest identifiers."
        )
    elif blocked:
        status = "blocked-not-evaluated"
        summary = "Missing required static artifact inputs; fail-closed."
    else:
        status = "fail"
        summary = "Static artifact inspection incomplete; fail-closed."
    inspection_tool_provenance = {
        "tools": inspection_tools,
        "tool_provenance": tool_provenance,
        "tool_path": "builtin/text-scan;apk-host-inspection-attempted",
        "tool": "static-inspection",
    }
    d2 = d(
        status,
        summary,
        inspection_performed=inspection_performed,
        inspected_artifacts=inspected_artifacts,
        artifact_inspections=inspections,
        inspections=inspections,
        identifier_findings=aggregate,
        identifier_scan=aggregate,
        inspection_tools=inspection_tools,
        tools_used=inspection_tools,
        inspection_tool_provenance=inspection_tool_provenance,
        tool_provenance=tool_provenance,
        tool_path=inspection_tool_provenance["tool_path"],
        tool=inspection_tool_provenance["tool"],
        any_in_scope_identifier_present=any_present,
        missing_inputs=missing,
        mode=mode,
    )
    top = {
        "inspection_performed": inspection_performed,
        "inspected_artifacts": inspected_artifacts,
        "artifact_inspections": inspections,
        "identifier_findings": aggregate,
        "inspection_tools": inspection_tools,
        "inspection_tool_provenance": inspection_tool_provenance,
    }
    return d2, tool_provenance, artifact_identity, top, class_scan_ran


def _run_adb_runner(runner: str, serial: str, package: str, component: str | None, action: str | None) -> dict:
    env = os.environ.copy()
    env.update({
        "SERIAL": serial or "",
        "PACKAGE": package or "",
        "COMPONENT": component or "",
        "ACTION": action or "",
        "DEBUG_INGEST_SERIAL": serial or "",
        "DEBUG_INGEST_PACKAGE": package or "",
        "DEBUG_INGEST_COMPONENT": component or "",
        "DEBUG_INGEST_ACTION": action or "",
    })
    try:
        p = subprocess.run(runner, shell=True, capture_output=True, text=True, check=False, env=env)
        return {"executed": True, "runner": runner, "returncode": p.returncode, "stdout": (p.stdout or "")[-4000:], "stderr": (p.stderr or "")[-4000:]}
    except Exception as e:
        return {"executed": False, "runner": runner, "error": str(e)}


def mechanize_runtime(args, mode: str) -> tuple[dict, dict, dict]:
    missing = []
    if not args.serial:
        missing.append("--serial (device with verified release/non-debug install identity)")
    if not args.component and not args.action:
        missing.append("--component/--action (negative invocation target)")
    runner = args.adb_runner or os.environ.get("DEBUG_INGEST_ADB_RUNNER")
    adb = tool_record("adb", required=True, runner=runner)
    tool_provenance = {"adb": adb}
    if runner:
        tool_provenance["adb-runner"] = tool_record("adb-runner", required=False, runner=runner)
    install_identity = {
        "serial": args.serial,
        "package_id": args.package,
        "component": args.component,
        "action": args.action,
        "verified_release_non_debug": False,
        "verification_note": "Not verified from provided inputs; fail-closed. No stale identity reuse.",
    }
    component = args.component or (f"{args.package}/.DebugIngestActivity" if args.package else None)
    action = args.action or "app.mdreader.action.DEBUG_INGEST"
    am_start = ["adb"]
    if args.serial:
        am_start += ["-s", args.serial]
    am_start += ["shell", "am", "start"]
    if action:
        am_start += ["-a", action]
    if component:
        am_start += ["-n", component]
    adb_command = {
        "argv": am_start,
        "shell": " ".join(shlex.quote(x) for x in am_start),
        "serial": args.serial,
        "package": args.package,
        "component": component,
        "action": action,
    }
    invocation_attempted = bool(args.serial and (args.component or args.action))
    run_result = None
    hook_success_detected = False
    hook_unreachable_confirmed = False
    negative_invocation_confirmed = False
    crash_anr_detected = False
    check_caused_crash_anr = False
    crash_anr_status = "not-detected"
    invocation_reason = "required runtime inputs provided; explicit negative ADB invocation evidence constructed"
    if not invocation_attempted:
        invocation_reason = "missing required runtime inputs (--serial and --component/--action); invocation not attempted"
        status = "blocked-not-evaluated"
        summary = "Required runtime inputs missing; fail-closed."
    else:
        if not adb.get("available") and not runner:
            missing.append("adb tool available for runtime checks (or --adb-runner/DEBUG_INGEST_ADB_RUNNER)")
        if runner or adb.get("available"):
            run_result = (
                _run_adb_runner(runner, args.serial or "", args.package, component, action)
                if runner
                else {"executed": False, "note": "adb present but direct execution disabled in checker; provide --adb-runner for mechanized execution"}
            )
            stdout = (run_result or {}).get("stdout", "") if isinstance(run_result, dict) else ""
            stderr = (run_result or {}).get("stderr", "") if isinstance(run_result, dict) else ""
            out = f"{stdout}\n{stderr}".lower()
            hook_success_detected = any(x in out for x in ["hook_success", "status=ok", "debug ingest accepted"]) and "error" not in out
            hook_unreachable_confirmed = (
                any(x in out for x in ["error", "securityexception", "permission denial", "activity not started", "unable to resolve intent", "does not exist", "not exported", "inaccessible", "java.lang.securityexception"])
                and not hook_success_detected
            )
            negative_invocation_confirmed = hook_unreachable_confirmed
            crash_anr_detected = any(x in out for x in ["anr in", "fatal exception", "has crashed", " crash", "application not responding"])
            check_caused_crash_anr = crash_anr_detected
            crash_anr_status = "detected-check-caused" if check_caused_crash_anr else "not-detected"
            if hook_success_detected or crash_anr_detected:
                status = "fail"
                summary = "Negative runtime invocation indicated hook success or check-caused crash/ANR; fail-closed."
            elif negative_invocation_confirmed:
                status = "fail"
                summary = "Negative invocation observed unreachable/error, but release/non-debug install identity not verified; fail-closed (no FR-010 pass)."
                install_identity["negative_invocation_observed"] = True
            elif run_result and run_result.get("executed"):
                status = "fail"
                summary = "ADB invocation executed but negative no-hook/no-crash outcome not sufficiently proven; fail-closed."
            else:
                status = "blocked-not-evaluated"
                summary = "Invocation command evidence bound but mechanized execution unavailable/unproven; fail-closed."
                invocation_reason = "explicit invocation attempted/bound; mechanized execution unavailable"
                if not runner:
                    missing.append("mechanized adb execution (--adb-runner/DEBUG_INGEST_ADB_RUNNER) or verified runtime harness")
        else:
            status = "blocked-not-evaluated"
            summary = "ADB tool unavailable; explicit invocation command evidence recorded, execution not performed. Fail-closed."
            invocation_reason = "explicit invocation attempted/bound; adb tool unavailable and no runner provided"
    d3 = d(
        status,
        summary,
        invocation_attempted=invocation_attempted,
        invocation_reason=invocation_reason,
        adb_command=adb_command,
        command=adb_command,
        commands=[adb_command],
        invocation_command=adb_command,
        runner_result=run_result,
        hook_success_detected=hook_success_detected,
        hook_unreachable_confirmed=hook_unreachable_confirmed,
        negative_invocation_confirmed=negative_invocation_confirmed,
        crash_anr_detected=crash_anr_detected,
        check_caused_crash_anr=check_caused_crash_anr,
        crash_anr_status=crash_anr_status,
        missing_inputs=missing,
        adb_available=adb["available"],
        adb_runner=runner,
        install_identity=install_identity,
        tool_provenance=tool_provenance,
        mode=mode,
    )
    return d3, tool_provenance, install_identity


def static_record(args) -> dict:
    apk = Path(args.apk).resolve() if args.apk else None
    d2, tool_provenance, artifact_identity, static_top, class_scan_ran = mechanize_static(args, "static")
    data = {
        "tool_name": "check-debug-ingest-release-exposure",
        "mode": "static",
        "overall": d2["status"],
        "satisfies_fr010_sc004": False,
        "d1_scope": d("not-evaluated", "D1 debug-only scope requires guard/variant+artifact context; static-only partial evidence.", mode="static"),
        "d2_static_absence": d2,
        "d3_runtime_negative": d("not-evaluated", "Runtime negative invocation not performed in --mode static.", mode="static"),
        "d4_repeatable": d(
            "partial" if d2["status"] in {"pass", "fail"} else "blocked-not-evaluated",
            "Machine-readable output emitted; static-only cannot satisfy FR-010/SC-004.",
            mode="static",
            mechanized_static=d2.get("inspection_performed"),
        ),
        "artifact_identity": artifact_identity,
        "install_identity": None,
        "tool_provenance": tool_provenance,
        "class_scan_ran": class_scan_ran,
        "captured_at_utc": now_utc(),
        **static_top,
    }
    if class_scan_ran:
        data["class_presence_note"] = CLASS_PRESENCE_NOTE
    if d2["status"] == "blocked-not-evaluated":
        data["blocked_not_evaluated"] = {
            "missing_inputs": d2.get("missing_inputs", []),
            "attempted_modes": ["static"],
            "reason": "Required static artifact inputs unavailable or inspection blocked; fail-closed, no FR-010/SC-004 satisfaction.",
        }
    return data


def runtime_record(args) -> dict:
    d3, tool_provenance, install_identity = mechanize_runtime(args, "runtime")
    data = {
        "tool_name": "check-debug-ingest-release-exposure",
        "mode": "runtime",
        "overall": d3["status"],
        "satisfies_fr010_sc004": False,
        "d1_scope": d("not-evaluated", "D1 debug-only scope not fully evaluated in --mode runtime.", mode="runtime"),
        "d2_static_absence": d("not-evaluated", "Static artifact absence not evaluated in --mode runtime.", mode="runtime"),
        "d3_runtime_negative": d3,
        "d4_repeatable": d(
            "partial" if d3["status"] in {"pass", "fail"} else "blocked-not-evaluated",
            "Machine-readable output emitted; runtime-only cannot satisfy FR-010/SC-004.",
            mode="runtime",
            mechanized_runtime=d3.get("invocation_attempted"),
        ),
        "artifact_identity": None,
        "install_identity": install_identity,
        "tool_provenance": tool_provenance,
        "captured_at_utc": now_utc(),
    }
    if d3["status"] == "blocked-not-evaluated":
        data["blocked_not_evaluated"] = {
            "missing_inputs": d3.get("missing_inputs", []),
            "attempted_modes": ["runtime"],
            "reason": "Required runtime inputs/tools unavailable or insufficient; fail-closed, no FR-010/SC-004 satisfaction.",
        }
    return data


def full_record(args) -> dict:
    attempted = ["full"]
    apk = Path(args.apk).resolve() if args.apk else None
    d2, static_tp, artifact_identity, static_top, class_scan_ran = mechanize_static(args, "full")
    d3, runtime_tp, install_identity = mechanize_runtime(args, "full")
    tool_provenance = {}
    tool_provenance.update(static_tp)
    tool_provenance.update(runtime_tp)
    d1 = d(
        "fail",
        "D1 debug-only scope cannot be fully proven from provided inputs; fail-closed (requires guard/variant/source+runtime scope evidence).",
        mode="full",
        guard_evidence=None,
    )
    identity_bound = bool(
        artifact_identity and artifact_identity.get("apk_path") and artifact_identity.get("apk_sha256")
        and install_identity and install_identity.get("serial") and install_identity.get("package_id")
    )
    d4_status = "fail" if identity_bound else "blocked-not-evaluated"
    d4 = d(
        d4_status,
        "Repeatable regression evidence composed with bound identities and mechanized D2/D3 results." if identity_bound else "D4 repeatable evidence incomplete or not identity-bound; fail-closed.",
        artifact_identity_bound=bool(artifact_identity and artifact_identity.get("apk_sha256")),
        install_identity_bound=bool(install_identity and install_identity.get("serial")),
        mechanized_static=d2.get("inspection_performed"),
        mechanized_runtime=d3.get("invocation_attempted"),
        mode="full",
    )
    dims = {"d1_scope": d1, "d2_static_absence": d2, "d3_runtime_negative": d3, "d4_repeatable": d4}
    all_pass = all(v.get("status") == "pass" for v in dims.values())
    verified_release = bool(install_identity and install_identity.get("verified_release_non_debug") is True)
    satisfies = bool(all_pass and verified_release and artifact_identity and artifact_identity.get("apk_sha256") and install_identity and install_identity.get("serial"))
    overall = "pass" if satisfies else ("blocked-not-evaluated" if any(v.get("status") == "blocked-not-evaluated" for v in dims.values()) else "fail")
    data = {
        "tool_name": "check-debug-ingest-release-exposure",
        "mode": "full",
        "overall": overall,
        "satisfies_fr010_sc004": satisfies,
        **dims,
        "artifact_identity": artifact_identity,
        "install_identity": install_identity,
        "tool_provenance": tool_provenance,
        "class_scan_ran": class_scan_ran,
        "attempted_modes": attempted,
        "captured_at_utc": now_utc(),
        **static_top,
    }
    if class_scan_ran:
        data["class_presence_note"] = CLASS_PRESENCE_NOTE
    if overall != "pass":
        missing = []
        for k, v in dims.items():
            if v.get("status") != "pass":
                missing.extend([f"{k}:{x}" for x in (v.get("missing_inputs") or [v.get("summary") or v.get("status")])])
        if not verified_release:
            missing.append("install_identity.verified_release_non_debug true (external verified release/non-debug identity required)")
        data["blocked_not_evaluated"] = {
            "missing_inputs": missing,
            "attempted_modes": attempted,
            "reason": "Required full-mode mechanized D1-D4 evidence, tools, or verified artifact/install identity unavailable or insufficient; fail-closed. FR-010/SC-004 not satisfied.",
        }
    return data


def main(argv: list[str]) -> int:
    ap = build_parser()
    args = ap.parse_args(argv[2:])
    if args.help:
        if args.json:
            emit({"tool_name": "check-debug-ingest-release-exposure", "help": usage_text(), "modes": ["static", "runtime", "full"], "blocked": True, "fail_closed": True, "captured_at_utc": now_utc()}, True)
        else:
            sys.stdout.write(usage_text())
        return 0

    if args.print_scope:
        data = scope_payload()
        emit(data, args.json)
        return 0

    mode = args.mode or "full"
    if mode == "static":
        data = static_record(args)
    elif mode == "runtime":
        data = runtime_record(args)
    else:
        data = full_record(args)

    if mode in ("static", "runtime"):
        data["satisfies_fr010_sc004"] = False
        if data.get("overall") == "pass":
            data["overall"] = "pass-non-full"
    else:
        if data.get("overall") != "pass":
            data["satisfies_fr010_sc004"] = False
        if data.get("satisfies_fr010_sc004") is True and data.get("overall") == "pass":
            inst = data.get("install_identity") or {}
            if inst.get("verified_release_non_debug") is not True:
                data["satisfies_fr010_sc004"] = False
                data["overall"] = "fail"

    if args.evidence_dir:
        try:
            ed = Path(args.evidence_dir)
            ed.mkdir(parents=True, exist_ok=True)
            out = ed / "release-exposure-evidence.json"
            out.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            data["evidence_dir"] = str(ed)
            data["evidence_file"] = str(out)
            data["write_status"] = "written"
        except Exception as e:
            data["evidence_dir"] = str(args.evidence_dir)
            data["write_status"] = f"failed:{e}"

    emit(data, args.json)
    overall = data.get("overall")
    if overall in ("pass", "pass-non-full"):
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
PYTHON
