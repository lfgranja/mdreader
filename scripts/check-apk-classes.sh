#!/usr/bin/env bash
# T121 — device-artifact class-presence gate (the ff6da8b hollow-APK
# regression guard): the post-realignment universalDebug APK once
# shipped a dex carrying only R/BuildConfig — no MainActivity, no
# PlaybackService, none of the five mirrors — an exit-0 build whose
# artifact would crash on launch. Gate 9 alone cannot see that class
# of failure (it attests the build exits zero, not that the app's
# classes are inside the artifact), so this check asserts the full
# app class set is present in the produced universal APK's dex:
#   - Lapp/mdreader/MainActivity (the launcher activity)
#   - Lapp/mdreader/PlaybackService (the resident foreground service)
#   - Lapp/mdreader/FocusMirror, MediaMirror, Residency,
#     RoutingMirror (the five SC-035 allowlisted mirrors, wired by
#     T028b/T111 via the android-shell sourceSet)
# plus the generated Tauri plumbing (TauriActivity) as a canary that
# the dex dump itself parsed.
#
# FAILS on a missing APK as missing-input-fail-closed: a missing
# artifact is a failed check, never a passed one (Principle V).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APK="$ROOT/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk"

fail() {
  printf 'check-apk-classes: FAIL: %s\n' "$1" >&2
  exit 1
}

# Missing input fails closed.
[ -f "$APK" ] \
  || fail "device artifact not found: $APK (run 'cargo tauri android build --target aarch64 --debug' first; a missing APK is a failed check, not a pass)"

command -v unzip >/dev/null 2>&1 \
  || fail "unzip is required to inspect the APK dex (missing input fails closed)"

DEXLIST="$(mktemp)"
trap 'rm -f "$DEXLIST"' EXIT
if ! unzip -Z1 "$APK" | grep -E '^classes.*\.dex$' > "$DEXLIST"; then
  fail "no classes*.dex entries in $APK (not an installable artifact)"
fi

require_class() {
  local label="$1" class="$2" found=0
  while IFS= read -r dex; do
    # No `grep -q` here: under `pipefail` its early exit SIGPIPEs
    # the pipe into a 141 false-negative. Consume fully instead.
    local matches
    matches="$(unzip -p "$APK" "$dex" 2>/dev/null | strings | grep -F "$class;" || true)"
    if [ -n "$matches" ]; then
      found=1
      break
    fi
  done < "$DEXLIST"
  [ "$found" -eq 1 ] \
    || fail "dex is missing $label (expected Dalvik type $class; hollow-APK class of failure — see ff6da8b)"
  printf 'check-apk-classes: present %s\n' "$class"
}

require_class 'the launcher activity' 'Lapp/mdreader/MainActivity'
require_class 'the resident playback service' 'Lapp/mdreader/PlaybackService'
require_class 'the focus mirror (T040)' 'Lapp/mdreader/FocusMirror'
require_class 'the media/session mirror (T062)' 'Lapp/mdreader/MediaMirror'
require_class 'the resident-lifecycle mirror (T064)' 'Lapp/mdreader/Residency'
require_class 'the routing mirror (T047)' 'Lapp/mdreader/RoutingMirror'
require_class 'the generated Tauri activity (dex-parse canary)' 'Lapp/mdreader/TauriActivity'

printf 'check-apk-classes: PASS (%s)\n' "$APK"
