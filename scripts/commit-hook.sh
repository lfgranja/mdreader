#!/bin/sh
# Atomic conventional commit hook — determinism gate (user constraint #6, constitution V).
# Invoked after every editing file is finished; produces a single-commit delta with detailed summary.
# This script delegates to .specify/hooks/commit for the actual git operation.
set -euo pipefail
FILE="${1:-}"
SUMMARY="${2:-analysis/remediation edit: $(basename "$FILE" 2>/dev/null || echo unknown)}"
if [ -z "$FILE" ]; then
  echo "Usage: $0 <edited-file-path> [summary]" >&2
  exit 1
fi
exec "$(dirname "$0")/../.specify/hooks/commit" "$FILE" "$SUMMARY"
