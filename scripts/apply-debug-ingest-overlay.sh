#!/usr/bin/env bash
# FR-011 durable overlay apply (Spec 012). SKELETON/TDD stub — not contract-satisfying.
set -euo pipefail
usage() {
  cat <<'USAGE'
Usage: scripts/apply-debug-ingest-overlay.sh [--root <repo-root>] [--gen-dir <src-tauri/gen/android/app>] [--dry-run] [--json] [--evidence-dir <dir>] [--help]
Idempotently materializes canonical debug-only overlays from
specs/012-debug-ingest-hook/android-overlays/app/src/ into generated targets.
Fail-closed: missing/stale/forbidden/unknown-format/path-safety issues => non-pass.
USAGE
}
for a in "$@"; do case "$a" in --help|-h) usage; exit 0;; esac; done
echo "blocked:not-implemented: FR-011 apply skeleton (Spec 012 TDD RED phase)" >&2
exit 2
