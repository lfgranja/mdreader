#!/usr/bin/env bash
# FR-011 durable overlay check (Spec 012). SKELETON/TDD stub — not contract-satisfying.
set -euo pipefail
usage() {
  cat <<'USAGE'
Usage: scripts/check-debug-ingest-overlay.sh [--root <repo-root>] [--gen-dir <src-tauri/gen/android/app>] [--json] [--evidence-dir <dir>] [--help]
Non-mutating fail-closed check of canonical vs generated FR-011 overlay targets.
Reports pass|fail|blocked|not-applicable per target with path/expected-vs-actual/remediation.
USAGE
}
for a in "$@"; do case "$a" in --help|-h) usage; exit 0;; esac; done
echo "blocked:not-implemented: FR-011 check skeleton (Spec 012 TDD RED phase)" >&2
exit 2
