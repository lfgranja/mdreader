#!/usr/bin/env bash
# FR-010 release/non-debug exposure evidence (Spec 012). SKELETON/TDD stub.
set -euo pipefail
usage() {
  cat <<'USAGE'
Usage: scripts/check-debug-ingest-release-exposure.sh --mode static|runtime|full [--apk <apk>] [--serial <serial>] [--evidence-dir <dir>] [--help]
Produces D1-D4 combined evidence for mode=full only; static/runtime never satisfy FR-010/SC-004.
Missing required inputs => blocked/not-evaluated (satisfies_fr010_sc004=false), fail-closed.
USAGE
}
for a in "$@"; do case "$a" in --help|-h) usage; exit 0;; esac; done
echo "blocked:not-implemented: FR-010 release-exposure skeleton (Spec 012 TDD RED phase)" >&2
exit 2
