#!/usr/bin/env bash
# T013 — Android manifest gate (FR-006, FR-007, SC-006): audio
# playback runs in a background service declared with the media
# playback foreground-service type, and the manifest declares
# that foreground service explicitly — its presence verifiable
# by an automated check rather than by inspection.
#
# Asserts in the BUILT manifest
# (src-tauri/gen/android/app/src/main/AndroidManifest.xml):
#   - `<service android:foregroundServiceType="mediaPlayback">`
#   - `android.permission.FOREGROUND_SERVICE`
#   - `android.permission.FOREGROUND_SERVICE_MEDIA_PLAYBACK`
#
# The manifest is hand-edited (the plugin API does not exist in
# Tauri 2: `mobile::update_manifest` and `registerService` are
# not real methods) and must be re-verified on every Tauri
# upgrade — which is why this check runs against the built
# artifact, not a template.
#
# FAILS on an empty tree as missing-input-fail-closed: a
# missing manifest is a failed check, never a passed one.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/src-tauri/gen/android/app/src/main/AndroidManifest.xml"

fail() {
  printf 'check-manifest: FAIL: %s\n' "$1" >&2
  exit 1
}

# Missing input fails closed.
[ -f "$MANIFEST" ] \
  || fail "built manifest not found: $MANIFEST (run the Android build first; a missing manifest is a failed check, not a pass)"

# Collapse newlines so a multi-line <service> element still
# matches as one tag.
manifest="$(tr '\n' ' ' < "$MANIFEST")"

require_pattern() {
  local label="$1" pattern="$2"
  if ! printf '%s' "$manifest" | grep -qE "$pattern"; then
    fail "manifest is missing $label (expected pattern: $pattern)"
  fi
}

require_pattern 'the mediaPlayback foreground service declaration' \
  '<service[^>]*android:foregroundServiceType="mediaPlayback"'
require_pattern 'the FOREGROUND_SERVICE permission' \
  'android\.permission\.FOREGROUND_SERVICE"'
require_pattern 'the FOREGROUND_SERVICE_MEDIA_PLAYBACK permission' \
  'android\.permission\.FOREGROUND_SERVICE_MEDIA_PLAYBACK"'

printf 'check-manifest: PASS (%s)\n' "$MANIFEST"
