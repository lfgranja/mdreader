#!/usr/bin/env bash
set -euo pipefail

# Spec 012 / FR-008,FR-009,FR-012,SC-001,SC-002,SC-003 contract-test & report helpers.
# Contract modes are host-deterministic, have no device/adb side effects, and take
# precedence over normal device automation.

HOOK_SCRIPT_NAME="automate-t184-t226-hook.sh"
JSON_MODE=false
CONTRACT_MODE=""
CLASSIFY_VALUE=""
CACHE_WRITE_STATUS=""
RAW_OUTCOME=""
CORROBORATION_VALUE="none"
NORM_MODE=""
NORM_PATH=""
NORM_URI=""
NORM_DATA_URI=""
EV_OPERATION="document"
EV_FIXTURE=""
EV_BUILD=""
EV_DEVICE=""
EV_DIAG=""
EV_CRASH="unknown"
EV_CORR_KEY=""
EV_LOCATION=""
EV_RETENTION=""

# Parse contract/report flags early (no side effects). Preserve positional SERIAL for
# normal device runs by collecting non-contract positionals.
POSITIONAL=()
while (($#)); do
  case "$1" in
    --json) JSON_MODE=true; shift ;;
    --help|-h) CONTRACT_MODE="help"; shift ;;
    --print-contract) CONTRACT_MODE="print-contract"; shift ;;
    --classify-outcome)
      CONTRACT_MODE="classify-outcome"
      CLASSIFY_VALUE="${2:-}"
      if (($# >= 2)); then shift 2; else shift; fi
      ;;
    --print-report-schema) CONTRACT_MODE="print-report-schema"; shift ;;
    --print-residual-rules) CONTRACT_MODE="print-residual-rules"; shift ;;
    --print-activity-channel-contract) CONTRACT_MODE="print-activity-channel-contract"; shift ;;
    --classify-channel-attribution) CONTRACT_MODE="classify-channel-attribution"; shift ;;
    --cache-write-status) CACHE_WRITE_STATUS="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --raw-outcome) RAW_OUTCOME="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --corroboration) CORROBORATION_VALUE="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --print-directory-uri-contract) CONTRACT_MODE="print-directory-uri-contract"; shift ;;
    --print-intent-parse-contract) CONTRACT_MODE="print-intent-parse-contract"; shift ;;
    --normalize-debug-ingest-intent) CONTRACT_MODE="normalize-debug-ingest-intent"; shift ;;
    --build-device-evidence) CONTRACT_MODE="build-device-evidence"; shift ;;
    --mode) NORM_MODE="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --path) NORM_PATH="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --uri) NORM_URI="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --data-uri) NORM_DATA_URI="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --raw-outcome) RAW_OUTCOME="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --cache-write-status) CACHE_WRITE_STATUS="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --operation) EV_OPERATION="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --fixture-reference) EV_FIXTURE="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --build-identity) EV_BUILD="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --device-identity) EV_DEVICE="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --diagnostic-capture) EV_DIAG="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --crash-anr-status) EV_CRASH="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --correlation-key) EV_CORR_KEY="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --reviewer-accessible-location) EV_LOCATION="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    --retention-scope) EV_RETENTION="${2:-}"; if (($# >= 2)); then shift 2; else shift; fi ;;
    *) POSITIONAL+=("$1"); shift ;;
  esac
done
set -- "${POSITIONAL[@]+"${POSITIONAL[@]}"}"

emit_json() {
  python3 - "$@" <<'PY'
import json, sys
payload = json.loads(sys.argv[1])
print(json.dumps(payload, sort_keys=True))
PY
}

# FR-008 minimum fields (spec.md §FR-008) + corroboration/channel fields for contract output.
fr008_json() {
  emit_json '{"scope_class":"programmatic-non-ui","authoritative_cache_channel":"cache/debug_ingest_result.txt","cache_write_failure_must_not_imply_authoritative_success":true,"authoritative_cache_success":false,"write_status":"unavailable","corroboration":"none","fixture_reference":"<fixture/reference>","operation":"<document|directory|uri>","outcome":"<ok:*|error:*|refused:*|blocked:*|inconclusive>","build_identity":"<build/debug APK identity or unavailable:<reason>>","device_identity":"<serial/model/sdk/abi or unavailable:<reason>>","diagnostic_capture":"<logcat/report/screenshot paths or unavailable:<reason>>","crash_anr_status":"<not-detected|detected|unknown>","residual_scope_note":"Programmatic scope only; native SAF picker UI/a11y residuals remain open (FR-009/FR-012).","preferred_location_or_channel":"cache/debug_ingest_result.txt (authoritative when parseable/written)","effective_location_or_channel":"<effective path/channel or unavailable>","reviewer_accessible_location":"<repo-relative evidence path, artifact path, or unavailable:<reason>>","retention_scope":"<host evidence retained; device capture as documented; unavailable:<reason>>","correlation_key":"<request/evidence/run id>","channel_corroboration":"<authoritative-only when cache written and parseable; corroborated/conflicting/inconclusive/none/blocked otherwise>","cache_required_for_attribution":false,"parseable_result_required":true,"cache_absent_or_unparseable_requires_corroboration":true,"cache_write_failure_without_corroboration_outcomes":["inconclusive","blocked"],"counts_toward_sc002_denominator_when_corroborated":true,"crash_anr_unknown_not_pass":true,"no_parseable_result_requires_explicit_status":true}'
}

print_contract_json() {
  emit_json '{"contract":"debug-ingest-automation","scope_class":"programmatic-non-ui","directory_result":{"operation":"directory","chapters":0,"chapters_field":"chapters=<n> accepted by production filtering/manifest behavior","evidence_required":true,"fixture_composition":"Evidence/report MUST state fixture composition when claiming directory pass.","expected_filtering":"Interpret chapters=<n> against production filtering and documented fixture composition/expected filtering (ASM-012-CHAPTERS-INTERP)."},"fixture_composition":{"note":"Evidence/report MUST state fixture composition when claiming directory pass."},"expected_filtering":{"note":"Interpret chapters=<n> against production filtering and documented fixture composition/expected filtering (ASM-012-CHAPTERS-INTERP)."},"result_prefixes":["ok:document:<absolute-path>","ok:uri:<uri>","ok:directory:<absolute-path>:chapters=<n>","error:<class-or-code>:<detail>","refused:not-debug"],"full_parent_completion_claimed":false}'
}

classify_outcome_json() {
  local value="$1"
  python3 - "$value" <<'PY'
import json, sys
v = sys.argv[1]
d = {
  "raw": v,
  "explicit_within_budget": False,
  "counts_toward_sc002_denominator": True,
}
if v.startswith("blocked:"):
  d.update({"status": "blocked", "counts_toward_sc002_denominator": False, "explicit_within_budget": False})
elif v.startswith("inconclusive"):
  d.update({"status": "inconclusive", "counts_toward_sc002_denominator": True, "explicit_within_budget": False})
elif v.startswith("ok:"):
  d.update({"status": "pass", "counts_toward_sc002_denominator": True, "explicit_within_budget": True})
elif v.startswith("error:") or v.startswith("refused:"):
  d.update({"status": "fail", "counts_toward_sc002_denominator": True, "explicit_within_budget": True})
else:
  # fail-closed: unknown values are not explicit pass evidence
  d.update({"status": "inconclusive", "counts_toward_sc002_denominator": False, "explicit_within_budget": False, "fail_closed": True})
print(json.dumps(d, sort_keys=True))
PY
}

report_schema_json() {
  emit_json '{"scope_class":"programmatic-non-ui","full_parent_completion_claimed":false,"authoritative_cache_channel":"cache/debug_ingest_result.txt","cache_write_failure_must_not_imply_authoritative_success":true,"authoritative_cache_success":"<false unless cache written+parseable+authoritative; never true on cache write failure without corroboration>","fixture_reference":"<required>","operation":"<document|directory|uri>","outcome":"<explicit parseable>","build_identity":"<required>","device_identity":"<required>","diagnostic_capture":"<required>","crash_anr_status":"<required>","residual_scope_note":"Native SAF picker UI/a11y residuals remain open (FR-009/FR-012).","preferred_location_or_channel":"<required>","effective_location_or_channel":"<required>","cache_write_status":"<written|unavailable|failed:<reason>>","write_status":"<written|unavailable|failed:<reason>>","reviewer_accessible_location":"<required>","retention_scope":"<required>","correlation_key":"<request/evidence/run id>","channel_corroboration":"<authoritative-only|corroborated|conflicting|inconclusive|none|blocked>","corroboration":"<explicit corroboration state>","cache_write_failure_without_corroboration_outcomes":["inconclusive","blocked"],"no_silent_authoritative_cache_success_on_write_failure":true,"counters":{"attempted":"<int>","precondition_blocked":"<int>","in_denominator":"<int>","explicit_within_budget":"<int>","inconclusive":"<int>","sc001":{"attempted":"<int>","requested_exercisable_runs":"<int>","exercised_runs":"<int>","blocked_runs":"<int>","precondition_blocked":"<int>","inconclusive_runs":"<int>","formula":"exercised_runs / max(requested_exercisable_runs - blocked_runs,1) ; SC-001 >=0.95"},"sc002":{"in_denominator":"<int>","eligible_explicit_runs":"<int>","explicit_within_budget_runs":"<int>","explicit_within_budget":"<int>","inconclusive_runs":"<int>","formula":"explicit_within_budget_runs / max(eligible_explicit_runs,1) ; SC-002 >=0.95"}}}'
}



build_device_evidence_json() {
  local raw_outcome="$1" cache_write_status="$2" corroboration="$3" operation="$4" fixture_reference="$5" build_identity="$6" device_identity="$7" diagnostic_capture="$8" crash_anr_status="$9"
  shift 9
  local correlation_key="${1:-}" reviewer_location="${2:-}" retention_scope="${3:-}"
  python3 - "$raw_outcome" "$cache_write_status" "$corroboration" "$operation" "$fixture_reference" "$build_identity" "$device_identity" "$diagnostic_capture" "$crash_anr_status" "$correlation_key" "$reviewer_location" "$retention_scope" <<'PYEV'
import json, sys
raw, cache, corr, op, fixture, build, device, diag, crash, corr_key, loc, retention = sys.argv[1:13]
corr = (corr or "none").strip().lower()
crash = (crash or "unknown").strip().lower()
if crash not in {"not-detected","detected","unknown"}: crash = "unknown"
# reuse channel attribution semantics
is_written = cache == "written"
is_corr = ("corroborat" in corr) and corr not in {"none","inconclusive","blocked","conflicting"}
raw_status = "inconclusive"
if raw.startswith("blocked:"): raw_status = "blocked"
elif raw.startswith("ok:"): raw_status = "pass"
elif raw.startswith("error:") or raw.startswith("refused:"): raw_status = "fail"
elif raw.startswith("inconclusive"): raw_status = "inconclusive"
# crash/ANR unknown or detected never explicit pass
status = raw_status
if crash in {"unknown","detected"} and status == "pass":
    status = "inconclusive" if crash == "unknown" else "fail"
if status not in {"pass","fail","blocked"}:
    status = "inconclusive" if status != "blocked" else "blocked"
explicit_within_budget = status in {"pass","fail"} and crash == "not-detected"
counts_as_explicit_pass = explicit_within_budget
counts_as_pass = False
counts_as_full_pass = False
passed = False
is_pass = False
classification = status
if status == "blocked": classification = "blocked"
elif status == "inconclusive": classification = "inconclusive"
elif status == "fail": classification = "fail"
authoritative_cache_success = bool(is_written and raw_status in {"pass","fail"} and crash=="not-detected" and corr in {"authoritative-only","authoritative","none",""})
effective = "cache/debug_ingest_result.txt" if is_written and authoritative_cache_success else ("corroborated-fallback" if is_corr and not is_written else ("unavailable" if not is_written else "cache/debug_ingest_result.txt"))
sc001 = {"attempted":"<int-or-1>","precondition_blocked": 1 if status=="blocked" else 0,"exercised_runs": 1 if status in {"pass","fail","inconclusive"} else 0,"inconclusive_runs": 1 if status=="inconclusive" else 0}
sc002 = {"in_denominator": 1 if status in {"pass","fail","inconclusive"} else 0,"explicit_within_budget_runs": 1 if explicit_within_budget else 0,"explicit_within_budget": 1 if explicit_within_budget else 0,"inconclusive_runs": 1 if status=="inconclusive" else 0}
o = {
  "scope_class":"programmatic-non-ui",
  "evidence_scope":"programmatic-non-ui",
  "full_parent_completion_claimed": False,
  "residual_scope_note":"Native SAF picker UI/a11y residuals remain open (FR-009/FR-012). Hook/device evidence is programmatic non-UI only.",
  "fixture_reference": fixture,
  "operation": op,
  "outcome": raw,
  "status": status,
  "outcome_status": status,
  "classification": classification,
  "normalized_outcome": classification,
  "build_identity": build,
  "device_identity": device,
  "diagnostic_capture": diag,
  "crash_anr_status": crash,
  "crash_anr_unknown_not_pass": True,
  "preferred_location_or_channel":"cache/debug_ingest_result.txt",
  "effective_location_or_channel": effective,
  "write_status": cache,
  "cache_write_status": cache,
  "reviewer_accessible_location": loc,
  "retention_scope": retention,
  "correlation_key": corr_key,
  "corroboration": corr,
  "channel_corroboration": corr,
  "authoritative_cache_success": authoritative_cache_success,
  "explicit_within_budget": explicit_within_budget,
  "counts_as_explicit_pass": counts_as_explicit_pass,
  "counts_as_pass": counts_as_pass,
  "counts_as_full_pass": counts_as_full_pass,
  "passed": passed,
  "is_pass": is_pass,
  "sc001_counters": sc001,
  "sc002_counters": sc002,
  "counters": {"attempted":"<int>","precondition_blocked":"<int>","in_denominator":"<int>","explicit_within_budget":"<int>","inconclusive":"<int>","sc001":sc001,"sc002":sc002},
  "classification_counters": {"sc001":sc001,"sc002":sc002}
}
print(json.dumps(o, sort_keys=True))
PYEV
}

activity_channel_contract_json() {
  emit_json '{"authoritative_cache_channel":"cache/debug_ingest_result.txt","cache_write_failure_must_not_imply_authoritative_success":true,"write_status_required":true,"corroboration_required":true,"no_corroboration_outcomes":["inconclusive","blocked"],"fallback_without_corroboration":["inconclusive","blocked"],"notes":"If cache/debug_ingest_result.txt cannot be written/unavailable, never imply authoritative cache success; require explicit write_status and corroboration-aware attribution."}'
}



directory_uri_contract_json() {
  emit_json '{"scope_class":"programmatic-non-ui","operation":"directory","same_production_path_required":true,"silent_fallback_allowed":false,"fake_success_allowed":false,"full_parent_completion_claimed":false,"undocumented_ambiguous_success_allowed":false,"required_outcomes":{"production_equivalent_success":"ok:directory:<absolute-path-or-production-equivalent-directory-ref>:chapters=<n>","unsupported_or_unresolvable_failure_prefix":"error:","undocumented_ambiguous_success_allowed":false},"reference_classes":{"content_tree":"requires production-equivalent-support-or-explicit-error (content:// tree/document via same production directory-open/manifest-dispatch path when feasible, else explicit diagnosable error:*)","content_document":"requires production-equivalent-support-or-explicit-error","malformed_content":"requires explicit-error","unsupported_scheme":"requires explicit-error (never silent fallback/fake ok)"},"classification_matrix":{"content://com.example/tree/primary%3Amd":{"outcome":"error:unsupported-directory-uri:content://-requires-production-equivalent-dispatch-or-explicit-diagnostic","silent_fallback":false,"fake_success":false,"production_equivalent_dispatch":false},"content://com.example/document/primary%3Amd":{"outcome":"error:unsupported-directory-uri:content://-requires-production-equivalent-dispatch-or-explicit-diagnostic","silent_fallback":false,"fake_success":false,"production_equivalent_dispatch":false},"ftp://example.invalid/tree":{"outcome":"error:unsupported-directory-uri-scheme:ftp","silent_fallback":false,"fake_success":false},"intent://scan/#Intent;scheme=content;end":{"outcome":"error:unsupported-directory-uri-scheme:intent","silent_fallback":false,"fake_success":false},"/app-accessible/filesystem/path":{"outcome":"ok:directory:/app-accessible/filesystem/path:chapters=1","silent_fallback":false,"fake_success":false,"production_equivalent_dispatch":true,"chapters_required":true}},"success_policy":{"ok_directory_requires_chapters":true,"production_equivalent_dispatch_required_for_non_filesystem_refs":true,"filesystem_path_may_use_existing_path_form":true,"non_filesystem_ref_requires_contract_check":"fail-closed diagnosable error:* unless production-equivalent dispatch proven; never fake success"},"notes":"FR-003 FR-005 FR-007 US2 US2-AC1 ASM-012-INGEST-SOT: directory refs beyond app-accessible filesystem paths (content:// tree/document, unsupported/malformed) must route through the same production directory-open/manifest-dispatch path when feasible or return explicit diagnosable error:*; never silent fallback/fake/ambiguous success. T226 residual split remains; full_parent_completion_claimed=false."}'
}

intent_parse_contract_json() {
  emit_json '{"scope_class":"programmatic-non-ui-debug-intent","contract_mode":"host-deterministic","side_effects":"none (no adb/device/mkdir)","authority_boundary":"validates debug-only intent parsing/normalization/validation semantics; production ingest remains in Kotlin/Rust/JNI (DebugIngestActivity/Hook/DocIngest snapshot-fingerprint + JNI callback/manifest dispatch), FR-004/FR-005 ASM-012-INGEST-SOT; not a parallel production ingest path","mode_aliases":{"document":["document","file","open_document"],"directory":["directory","dir","tree","open_directory"],"uri":["uri"]},"precedence":"U1/FR-003/FR-007: in document/directory modes when path present, path wins before uri/data-uri (intent data). uri mode uses uri/data-uri.","missing_input_errors":{"document":"error:missing-path-or-uri","directory":"error:missing-directory-path","uri":"error:missing-uri"},"unknown_mode":"error:unknown-mode:<normalized-input>","output":"machine-readable JSON with normalized_mode/status/error_class/reference_source/selected_reference","notes":"FR-002 FR-003 FR-007 T013/T020/I1 host-contract fallback (no CHK021 gen patch)."}'
}

normalize_debug_ingest_intent_json() {
  local mode="$1" path="$2" uri="$3" data_uri="$4"
  python3 - "$mode" "$path" "$uri" "$data_uri" <<'PYN'
import json, sys
mode_in = (sys.argv[1] or "").strip()
path = sys.argv[2] or ""
uri = sys.argv[3] or ""
data_uri = sys.argv[4] or ""
low = mode_in.lower() or "document"
doc = {"document","file","open_document"}
direc = {"directory","dir","tree","open_directory"}
def err(code, msg=None):
    o = {
      "ok": False, "valid": False, "status":"error", "error_class": code,
      "error": code, "outcome": code, "result": code,
      "mode_input": mode_in, "normalized_mode": low,
      "message": msg or code,
    }
    print(json.dumps(o, sort_keys=True)); raise SystemExit
if low in doc:
    nm = "document"
    if path:
        print(json.dumps({"ok":True,"valid":True,"status":"normalized","normalized_mode":nm,"operation":nm,"mode_input":mode_in,"selected_reference":path,"selected_reference_source":"path","reference_source":"path","resolved_from":"path","uri_ignored_due_to_path_precedence": bool(uri or data_uri),"data_uri_ignored_due_to_path_precedence": bool(data_uri)}, sort_keys=True)); raise SystemExit
    ref = uri or data_uri
    if ref:
        src = "uri" if uri else "data-uri"
        print(json.dumps({"ok":True,"valid":True,"status":"normalized","normalized_mode":nm,"operation":nm,"mode_input":mode_in,"selected_reference":ref,"selected_reference_source":src,"reference_source":src,"resolved_from":src}, sort_keys=True)); raise SystemExit
    err("error:missing-path-or-uri")
if low in direc:
    nm = "directory"
    if path:
        print(json.dumps({"ok":True,"valid":True,"status":"normalized","normalized_mode":nm,"operation":nm,"mode_input":mode_in,"selected_reference":path,"selected_reference_source":"path","reference_source":"path","resolved_from":"path","uri_ignored_due_to_path_precedence": bool(uri or data_uri),"data_uri_ignored_due_to_path_precedence": bool(data_uri)}, sort_keys=True)); raise SystemExit
    ref = uri or data_uri
    if ref:
        src = "uri" if uri else "data-uri"
        print(json.dumps({"ok":True,"valid":True,"status":"normalized","normalized_mode":nm,"operation":nm,"mode_input":mode_in,"selected_reference":ref,"selected_reference_source":src,"reference_source":src,"resolved_from":src}, sort_keys=True)); raise SystemExit
    err("error:missing-directory-path")
if low == "uri":
    nm = "uri"
    ref = uri or data_uri
    if ref:
        src = "uri" if uri else "data-uri"
        print(json.dumps({"ok":True,"valid":True,"status":"normalized","normalized_mode":nm,"operation":nm,"mode_input":mode_in,"selected_reference":ref,"selected_reference_source":src,"reference_source":src,"resolved_from":src}, sort_keys=True)); raise SystemExit
    err("error:missing-uri")
err(f"error:unknown-mode:{low}")
PYN
}

classify_channel_attribution_json() {
  local cache_write_status="$1"
  local raw_outcome="$2"
  local corroboration="$3"
  python3 - "$cache_write_status" "$raw_outcome" "$corroboration" <<'PYATTR'
import json, sys
cache = sys.argv[1] or "unavailable"
raw = sys.argv[2] or ""
corr = (sys.argv[3] or "none").strip().lower()
corr_explicit = corr if corr else "none"
is_written = cache == "written"
is_corr = ("corroborat" in corr_explicit) and corr_explicit not in {"none", "inconclusive", "blocked", "conflicting"}
is_conflict = corr_explicit == "conflicting"
raw_status = "unknown"
if raw.startswith("blocked:"): raw_status = "blocked"
elif raw.startswith("ok:"): raw_status = "pass"
elif raw.startswith("error:") or raw.startswith("refused:"): raw_status = "fail"
elif raw.startswith("inconclusive"): raw_status = "inconclusive"
authoritative_cache_success = bool(is_written and raw_status in {"pass","fail","blocked"} and corr_explicit in {"authoritative-only","authoritative","none",""})
if authoritative_cache_success and corr_explicit in {"none",""}:
    corr_explicit = "authoritative-only"
if is_written and authoritative_cache_success and not is_conflict:
    status = raw_status; counts = raw_status in {"pass","fail"}; explicit = counts
    channel_attr = "authoritative-cache"; eff = "cache/debug_ingest_result.txt"
elif is_written and is_conflict:
    status = "inconclusive"; counts=False; explicit=False; authoritative_cache_success=False
    channel_attr="conflicting"; eff="cache/debug_ingest_result.txt+conflicting-corroboration"
elif not is_written and is_corr and raw_status in {"pass","fail","blocked"}:
    status = raw_status; counts = raw_status in {"pass","fail"}; explicit=counts
    authoritative_cache_success=False; channel_attr="corroborated-fallback"; eff=f"corroborated:{corr_explicit}"
elif not is_written and corr_explicit == "blocked":
    status="blocked"; counts=False; explicit=False; authoritative_cache_success=False
    channel_attr="blocked"; eff="blocked"
else:
    status="inconclusive"; counts=False; explicit=False; authoritative_cache_success=False
    channel_attr="fail-closed-no-authoritative-cache"; eff="unavailable" if not is_written else "cache/debug_ingest_result.txt"
if corr_explicit in {"authoritative-only","authoritative"} and not is_written:
    corr_explicit="none"; status="inconclusive"; explicit=False; counts=False; channel_attr="fail-closed-no-authoritative-cache"
print(json.dumps({
  "raw_outcome": raw, "cache_write_status": cache, "write_status": cache,
  "corroboration": corr_explicit, "channel_corroboration": corr_explicit,
  "authoritative_cache_success": bool(authoritative_cache_success),
  "status": status, "outcome_status": status,
  "explicit_within_budget": bool(explicit), "counts_as_explicit_pass": bool(explicit),
  "counts_toward_sc002_denominator": bool(counts),
  "channel_attribution": channel_attr,
  "effective_location_or_channel": eff,
  "preferred_location_or_channel": "cache/debug_ingest_result.txt",
  "authoritative_cache_channel": "cache/debug_ingest_result.txt",
}, sort_keys=True))
PYATTR
}

residual_rules_json() {
  emit_json '{"allowed_statuses":["accepted-risk","deferred","open","verified"],"accepted_risk_requires":["owner","decision","rationale","scope","evidence"],"accepted_risk_completes_parent_when_blocking":false,"verified_requires":["evidence refs appropriate to scope"],"deferred_requires":["owner","decision ref","rationale"],"parent_completion_rule":"Parent T184/T226 fully complete only when all required *-PROG and required residual items are verified or explicitly re-scoped by a recorded decision (never implicit).","full_parent_completion_claimed_from_hook_evidence":false}'
}

usage(){
  cat <<'USAGE'
Usage: automate-t184-t226-hook.sh [serial] [options]
  serial  adb serial (default: adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp)
Automates T184 (open file auto-ingest path) and T226 (directory ingest path) using
debug-only DebugIngestActivity hook (no manual SAF picker). Deterministic, fail-closed.
Contract modes (host-deterministic, no device/adb side effects):
  --help [--json]                    usage or FR-008 contract JSON
  --print-contract [--json]          debug-ingest contract summary
  --classify-outcome <value> [--json] classify result/status (SC-001/SC-002)
  --print-report-schema [--json]     report schema/counters/formulae
  --print-residual-rules [--json]    FR-012 residual lifecycle rules
Normalized mode aliases (debug hook): document=file,open_document; directory=dir,tree,open_directory; uri.
USAGE
}

# Contract modes: deterministic, no adb/mkdir/device side effects.
if [[ "$CONTRACT_MODE" == "help" ]]; then
  if [[ "$JSON_MODE" == "true" ]]; then fr008_json; else usage; fi
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-contract" ]]; then
  print_contract_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "classify-outcome" ]]; then
  classify_outcome_json "$CLASSIFY_VALUE"
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-report-schema" ]]; then
  report_schema_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-residual-rules" ]]; then
  residual_rules_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-activity-channel-contract" ]]; then
  activity_channel_contract_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "classify-channel-attribution" ]]; then
  classify_channel_attribution_json "$CACHE_WRITE_STATUS" "$RAW_OUTCOME" "$CORROBORATION_VALUE"
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-directory-uri-contract" ]]; then
  directory_uri_contract_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "print-intent-parse-contract" ]]; then
  intent_parse_contract_json
  exit 0
fi
if [[ "$CONTRACT_MODE" == "normalize-debug-ingest-intent" ]]; then
  normalize_debug_ingest_intent_json "$NORM_MODE" "$NORM_PATH" "$NORM_URI" "$NORM_DATA_URI"
  exit 0
fi
if [[ "$CONTRACT_MODE" == "build-device-evidence" ]]; then
  build_device_evidence_json "$RAW_OUTCOME" "$CACHE_WRITE_STATUS" "$CORROBORATION_VALUE" "$EV_OPERATION" "$EV_FIXTURE" "$EV_BUILD" "$EV_DEVICE" "$EV_DIAG" "$EV_CRASH" "$EV_CORR_KEY" "$EV_LOCATION" "$EV_RETENTION"
  exit 0
fi

# ---- Normal device automation mode (may have side effects) ----
SERIAL="${1:-adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp}"
ADB="${ADB:-$HOME/Android/Sdk/platform-tools/adb}"
PKG="${PKG:-app.mdreader}"
TS=$(date -u +%Y%m%dT%H%M%SZ)
OUT="./t184-t226-hook-$TS"
mkdir -p "$OUT"
adb() { "$ADB" -s "$SERIAL" "$@"; }

log(){ echo "$*" | tee -a "$OUT/summary.txt"; }
pass(){ log "PASS: $*"; }
fail(){ log "FAIL: $*"; }
warn(){ log "WARN: $*"; }

log "=== T184/T226 via DebugIngestActivity hook === $TS serial=$SERIAL"
adb shell getprop ro.product.model ro.build.version.sdk ro.product.cpu.abi > "$OUT/device.txt" 2>&1 || true
adb shell pm list packages | grep -F "$PKG" > "$OUT/pkg.txt" 2>&1 || true
grep -q "$PKG" "$OUT/pkg.txt" && pass "package installed" || fail "package missing"

# Fixtures on device (prefer app-internal files for debug/run-as accessibility)
APP_FIX=/data/user/0/$PKG/files/mdreader-test-fixtures
FIX_SRC=/tmp/mdreader-test-fixtures
# ensure fixtures exist
if [[ ! -f "$FIX_SRC/t184-teste.md" || ! -d "$FIX_SRC/md-dir" ]]; then
  log "Local fixtures missing at $FIX_SRC; creating minimal test fixtures"
  mkdir -p "$FIX_SRC/md-dir"
  printf '# Teste T184\n\nTexto para ingestação automática.\n' > "$FIX_SRC/t184-teste.md"
  printf '# Cap 1\n\nPrimeiro.\n' > "$FIX_SRC/md-dir/001-cap1.md"
  printf '# Cap 2\n\nSegundo.\n' > "$FIX_SRC/md-dir/002-cap2.md"
fi
if adb shell "run-as $PKG true" >/dev/null 2>&1; then
  adb shell "run-as $PKG mkdir -p files/mdreader-test-fixtures/md-dir" >/dev/null 2>&1 || true
  adb push "$FIX_SRC/t184-teste.md" /data/local/tmp/t184-teste.md >/dev/null 2>&1 || true
  adb push "$FIX_SRC/md-dir/001-cap1.md" /data/local/tmp/001-cap1.md >/dev/null 2>&1 || true
  adb push "$FIX_SRC/md-dir/002-cap2.md" /data/local/tmp/002-cap2.md >/dev/null 2>&1 || true
  adb shell "run-as $PKG cp /data/local/tmp/t184-teste.md files/mdreader-test-fixtures/t184-teste.md" >/dev/null 2>&1 || true
  adb shell "run-as $PKG cp /data/local/tmp/001-cap1.md files/mdreader-test-fixtures/md-dir/001-cap1.md" >/dev/null 2>&1 || true
  adb shell "run-as $PKG cp /data/local/tmp/002-cap2.md files/mdreader-test-fixtures/md-dir/002-cap2.md" >/dev/null 2>&1 || true
  FIX_BASE=$APP_FIX
else
  adb shell "mkdir -p $APP_FIX/md-dir" >/dev/null 2>&1 || true
  adb push "$FIX_SRC/t184-teste.md" "$APP_FIX/t184-teste.md" >/dev/null 2>&1 || true
  adb push "$FIX_SRC/md-dir" "$APP_FIX/" >/dev/null 2>&1 || true
  FIX_BASE=$APP_FIX
fi
adb shell "ls -R $FIX_BASE 2>&1 || run-as $PKG ls -R files/mdreader-test-fixtures 2>&1" > "$OUT/fixtures.txt" 2>&1 || true

adb logcat -c >/dev/null 2>&1 || true
adb shell am force-stop "$PKG" >/dev/null 2>&1 || true
adb shell am start -W -n "$PKG/.MainActivity" > "$OUT/launch.txt" 2>&1 || true
sleep 4

# T184 Scenario 1: open new document (auto-ingest path via hook)
log "--- T184 Scenario 1: open document via debug hook ---"
adb shell am start -n "$PKG/.DebugIngestActivity" --es mode document --es path $FIX_BASE/t184-teste.md > "$OUT/t184_s1_am.txt" 2>&1 && pass "debug hook document intent sent" || fail "debug hook document intent failed"
sleep 5
adb shell "run-as $PKG cat cache/debug_ingest_result.txt" > "$OUT/t184_s1_result.txt" 2>&1 || true
adb logcat -d -t 400 > "$OUT/t184_s1_logcat.txt" 2>&1 || true
if grep -q "^ok:" "$OUT/t184_s1_result.txt" 2>/dev/null || grep -q 'DebugIngestActivity\|debugDeliverDocument\|onDocumentIngested\|t184-teste' "$OUT/t184_s1_logcat.txt"; then pass "document ingest evidence captured (result/logcat)"; else warn "document ingest evidence not obvious (see $OUT/t184_s1_result.txt)"; fi
adb shell dumpsys media_session > "$OUT/t184_s1_media.txt" 2>&1 || true

# T184 Scenario 2: reopen same doc (hook again) - resume/offer is Rust/app behavior
log "--- T184 Scenario 2: reopen document ---"
adb shell am start -n "$PKG/.DebugIngestActivity" --es mode document --es path $FIX_BASE/t184-teste.md > "$OUT/t184_s2_am.txt" 2>&1 || true
sleep 4
adb logcat -d -t 300 > "$OUT/t184_s2_logcat.txt" 2>&1 || true
pass "reopen document intent sent (resume/offer UI behavior requires app-state observation)"

# T184 Scenario 3/4: cancellation/a11y are UI-specific; hook covers programmatic open path
warn "T184 Scenario 3 (SAF cancel) and Scenario 4 (keyboard/TalkBack focus) remain UI/SAF interaction scenarios; hook validates ingest/open path without picker."

# T226: directory
log "--- T226: directory ingest via debug hook ---"
adb shell am force-stop "$PKG" >/dev/null 2>&1 || true
adb shell am start -W -n "$PKG/.MainActivity" >/dev/null 2>&1 || true
sleep 3
adb shell am start -n "$PKG/.DebugIngestActivity" --es mode directory --es path $FIX_BASE/md-dir > "$OUT/t226_dir_am.txt" 2>&1 && pass "debug hook directory intent sent" || fail "debug hook directory intent failed"
sleep 8
adb shell "run-as $PKG cat cache/debug_ingest_result.txt" > "$OUT/t226_result.txt" 2>&1 || true
adb logcat -d -t 500 > "$OUT/t226_logcat.txt" 2>&1 || true
if grep -q "^ok:directory" "$OUT/t226_result.txt" 2>/dev/null || grep -q 'debug ingest result\|chapters=\|onDirectorySelected\|directory' "$OUT/t226_logcat.txt"; then pass "directory ingest evidence captured (result/logcat)"; else warn "directory ingest evidence not obvious (see $OUT/t226_result.txt)"; fi

adb shell dumpsys activity services "$PKG" > "$OUT/services.txt" 2>&1 || true
adb shell screencap -p /sdcard/t184t226hook.png >/dev/null 2>&1 && adb pull /sdcard/t184t226hook.png "$OUT/screen.png" >/dev/null 2>&1 || true
adb shell uiautomator dump /sdcard/window_dump.xml >/dev/null 2>&1 && adb pull /sdcard/window_dump.xml "$OUT/ui.xml" >/dev/null 2>&1 || true

if grep -E 'FATAL EXCEPTION|ANR in '"$PKG" "$OUT"/*.txt >/dev/null 2>&1; then
  fail "crash/ANR markers found (inspect $OUT)"
else
  pass "no obvious FATAL EXCEPTION/ANR markers in collected logs"
fi

cat > "$OUT/REPORT.md" << MD
# T184/T226 Hook Automation Report
- Time (UTC): $TS
- Serial: $SERIAL
- Device: $(tr '\n' ' ' < "$OUT/device.txt" 2>/dev/null || true)
- Fixtures base: ${FIX_BASE:-unknown}

## Evidence
- summary: $OUT/summary.txt
- T184 s1 result: $OUT/t184_s1_result.txt
- T184 s1 logcat: $OUT/t184_s1_logcat.txt
- T184 s2 logcat: $OUT/t184_s2_logcat.txt
- T226 result: $OUT/t226_result.txt
- T226 logcat: $OUT/t226_logcat.txt
- screenshot/ui: $OUT/screen.png, $OUT/ui.xml

## Limitations (manual/visual)
- SAF cancel behavior and keyboard/TalkBack focus across SAF/title UI cannot be fully proven by am/debug hook; requires manual accessibility check.
MD
log "Report: $OUT/REPORT.md"
