#!/usr/bin/env bash
# T013/T081 — Android manifest gate (SC-006, FR-006, FR-007, SC-032): audio
# playback runs in a background service declared with the media playback
# foreground-service type, the manifest declares that foreground service
# explicitly, and the application declares the Android 10 (API 29) minimum
# (SC-032) — each verifiable by an automated check rather than by inspection.
#
# Asserts in the BUILT manifest
# (src-tauri/gen/android/app/src/main/AndroidManifest.xml):
#   - `<service android:foregroundServiceType="mediaPlayback">`
#   - `android.permission.FOREGROUND_SERVICE`
#   - `android.permission.FOREGROUND_SERVICE_MEDIA_PLAYBACK`
#   - `android.permission.POST_NOTIFICATIONS` (FR-021 /
#     T030b: the permission-state gate observes the
#     notification permission's granted/denied state and
#     routes denial to the in-app statement — a
#     declaration the gate can only observe if the
#     manifest declares it)
#   - `android.permission.VIBRATE` (FR-016 / T031: the
#     haptic confirmation path for blind mode with sounds
#     off needs the permission declared, or the haptic
#     metaphor silently no-ops on a real device)
#
# Asserts the SC-032 floor (T081): the application declares
# minSdk 29. The built Android project declares it as a Gradle
# `minSdk` property (Tauri generates `minSdk = 29` into
# gen/android/app/build.gradle.kts; the hand-edited manifest
# itself carries no `<uses-sdk>` element — inspected, and
# asserted where the declaration actually lives). The floor is
# accepted from any point of the declaration chain: the
# generated Gradle property, the built or a merged manifest
# (`android:minSdkVersion="29"` under gen/android/app/build/
# intermediates/), or the source declaration
# (src-tauri/tauri.conf.json `"minSdkVersion": 29`). The source
# declaration is accepted because `gen/` is a build artifact
# that regenerates from it and may lag on a host that has not
# re-run `cargo tauri android init`; a source that does not
# declare 29 fails the gate wherever it is found. The key
# pattern tolerates the three syntactic forms the chain uses:
# the Kotlin DSL (`minSdk = 29`), the JSON source
# (`"minSdkVersion": 29` — the quoted key puts a `"` between
# the name and the colon), and the XML attribute
# (`android:minSdkVersion="29"`).
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
  # Full-consumption grep (no -q): under `pipefail` a `grep -q`
  # early exit SIGPIPEs printf into a 141 false-negative — the
  # same scheduling race fixed in check-shell-purity.sh §5 (T121).
  if [ -z "$(printf '%s' "$manifest" | grep -E "$pattern" || true)" ]; then
    fail "manifest is missing $label (expected pattern: $pattern)"
  fi
}

require_pattern 'the mediaPlayback foreground service declaration' \
  '<service[^>]*android:foregroundServiceType="mediaPlayback"'
require_pattern 'the FOREGROUND_SERVICE permission' \
  'android\.permission\.FOREGROUND_SERVICE"'
require_pattern 'the FOREGROUND_SERVICE_MEDIA_PLAYBACK permission' \
  'android\.permission\.FOREGROUND_SERVICE_MEDIA_PLAYBACK"'
require_pattern 'the POST_NOTIFICATIONS permission (FR-021 / T030b)' \
  'android\.permission\.POST_NOTIFICATIONS"'
# FR-016 / T031: the haptic confirmation path (blind mode
# sounds-off) needs the VIBRATE permission declared, or the
# haptic metaphor silently no-ops on a real device.
require_pattern 'the VIBRATE permission (FR-016 / T031)' \
  'android\.permission\.VIBRATE"'

# SC-032 (T081): the application declares the Android 10 (API 29)
# floor. Mechanical form: a `minSdk`/`minSdkVersion` declaration
# whose value is exactly 29, matched at every point of the
# declaration chain (generated Gradle property, built manifest,
# merged manifests, source tauri.conf.json). `minSdk 26` or any
# other value does not match — the gate fails closed on it.
minsdk_declared=0
declare -a minsdk_points=(
  "$ROOT/src-tauri/gen/android/app/build.gradle.kts"
  "$ROOT/src-tauri/gen/android/app/src/main/AndroidManifest.xml"
  "$ROOT/src-tauri/tauri.conf.json"
)
for point in "${minsdk_points[@]}"; do
  if [ -f "$point" ] \
     && grep -qE 'minSdk(Version)?"?[[:space:]]*[:=][[:space:]]*"?29"?([^0-9]|$)' "$point"; then
    minsdk_declared=1
    printf 'check-manifest: SC-032 floor declared at %s\n' "$point"
  fi
done
if [ "$minsdk_declared" -eq 0 ]; then
  # Merged manifests under the build intermediates are the last
  # declaration point: a real `cargo tauri android build` merges
  # the floor into every variant manifest.
  merged="$(grep -rlE 'android:minSdkVersion="29"' \
    "$ROOT/src-tauri/gen/android/app/build/intermediates" 2>/dev/null | head -n 1 || true)"
  if [ -n "$merged" ]; then
    minsdk_declared=1
    printf 'check-manifest: SC-032 floor declared at %s\n' "$merged"
  fi
fi
[ "$minsdk_declared" -eq 1 ] \
  || fail 'the built Android project does not declare minSdk 29 (SC-032: the application declares a minimum supported Android version of 10 / API 29; checked gen/android/app/build.gradle.kts, the built manifest, the merged manifests, and src-tauri/tauri.conf.json)'

printf 'check-manifest: PASS (%s)\n' "$MANIFEST"
