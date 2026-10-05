#!/usr/bin/env bash
# T013 — shell-purity gate (SC-035, FR-042): the Android/Tauri surface
# contains no playback logic of its own. Every behaviour — transport,
# rate, position, buffering, focus response, device-format conversion —
# is resolved by the Rust backend; the platform surface only exposes the
# application to Android and mirrors state.
#
# Method (tasks.md T013, verbatim): "forbidden call shapes, not bare
# words, so the gate is mechanically decidable" — deny
#   `.position\s*=`, `.rate\s*=`, `setPlaybackState\s*\(.*position`,
#   `setPlaybackState\s*\(.*state`, `seekTo\s*\(`, `gain\s*[*/+-]?=`,
#   `duck\w*\s*=`, `skip(Forward|Backward)\s*\(`
# in `android-shell/**` and in the allowlisted `src-tauri/**` files.
# Identifiers, type positions, and `get*()`/`set*` delegation targets
# are allowed, which a bare-word grep could not distinguish — so this
# gate matches call shapes (assignment targets, setter calls with
# position/state arguments, seek calls, gain arithmetic, duck
# assignment, skip invocation), never bare identifiers.
#
# The T081 allowlist (5 Kotlin mirrors + 4 src-tauri shape/delegation
# files, shapes/getters only, never computation) is applied by path;
# until T081 lands, the scan covers every source file under the scan
# roots. The residue ("is this delegation or computation?") is routed
# to T086 as an explicit human-review item instead of being smuggled
# into this script (Principle V: a gate is mechanically decidable or
# it is not a gate).
#
# FAILS on an empty tree: the scan roots are missing inputs, and the
# gate fails closed — a missing tree is not a clean tree.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Scan roots: the Kotlin shell and the Tauri Rust surface.
ANDROID_SHELL="$ROOT/android-shell"
SRC_TAURI="$ROOT/src-tauri"

fail() {
  printf 'check-shell-purity: FAIL: %s\n' "$1" >&2
  exit 1
}

# Missing input fails closed.
[ -d "$ANDROID_SHELL" ] \
  || fail "missing scan root: $ANDROID_SHELL (android-shell/** does not exist; missing input is not a clean scan)"
[ -d "$SRC_TAURI" ] \
  || fail "missing scan root: $SRC_TAURI (src-tauri/** does not exist; missing input is not a clean scan)"

# Forbidden CALL SHAPES (extended regex, grep -E). Each shape is a
# computation or a state mutation in the surface; identifiers, type
# positions, and get*/set* delegation targets do not match any of them.
FORBIDDEN_SHAPES=(
  '\.position\s*='
  '\.rate\s*='
  'setPlaybackState\s*\(.*position'
  'setPlaybackState\s*\(.*state'
  'seekTo\s*\('
  'gain\s*[*/+-]?='
  'duck\w*\s*='
  'skip(Forward|Backward)\s*\('
)

status=0
for shape in "${FORBIDDEN_SHAPES[@]}"; do
  hits="$(grep -rInE --include='*.kt' --include='*.kts' --include='*.rs' \
    "$shape" "$ANDROID_SHELL" "$SRC_TAURI" 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    printf 'check-shell-purity: forbidden call shape /%s/ in:\n' "$shape" >&2
    printf '%s\n' "$hits" >&2
    status=1
  fi
done

if [ "$status" -ne 0 ]; then
  fail "forbidden playback-logic call shapes found in the shell surface (SC-035, FR-042)"
fi

printf 'check-shell-purity: PASS (no forbidden call shapes in android-shell/** or src-tauri/**)\n'
