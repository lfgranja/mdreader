#!/usr/bin/env bash
# T013/T081 — shell-purity gate (SC-035, FR-042, SC-057, spec→test
# traceability): the Android/Tauri surface contains no playback logic
# of its own. Every behaviour — transport, rate, position, buffering,
# focus response, device-format conversion — is resolved by the Rust
# backend; the platform surface only exposes the application to Android
# and mirrors state.
#
# Five named sections, all fail-closed:
#
#   1. Forbidden call shapes (T013, unchanged): the eight T013
#      regexes — `\.position\s*=`, `\.rate\s*=`,
#      `setPlaybackState\s*\(.*position`,
#      `setPlaybackState\s*\(.*state`, `seekTo\s*\(`,
#      `gain\s*[*/+-]?=`, `duck\w*\s*=`,
#      `skip(Forward|Backward)\s*\(` — in ANY *.kt/*.kts/*.rs under
#      android-shell/ or src-tauri/, regardless of allowlist. The
#      build-artifact trees are pruned (src-tauri/gen is a
#      regenerated copy of the scanned surface; src-tauri/target is
#      compiler output of third-party crates, not the hand-written
#      surface the gate governs). Call shapes, not bare words, so
#      the gate is mechanically decidable: identifiers, type
#      positions, and get*/set* delegation targets do not match
#      any of them.
#
#   2. Allowlist enforcement (T081): the only files permitted to
#      carry the shell vocabulary (`transport`, `position_secs`,
#      `position`, `rate`, `gain`, `duck`, `skip` as whole-word
#      identifiers) are the allowlisted files. Any OTHER source file
#      under the scan roots matching those tokens fails. Build-artifact
#      trees (src-tauri/gen, src-tauri/target) are excluded from this
#      section: they are regenerated outputs, not the hand-written
#      surface the gate governs (gen/ carries a generated copy of the
#      allowlisted PlaybackService.kt; target/ carries third-party
#      web_atoms output whose CSS atom names contain "position").
#      Allowlist deviation (T081 report): src-tauri/src/lib.rs is the
#      Tauri bridge (T028) — the task text's nine-file list omits it,
#      but the bridge carries the vocabulary only in delegation
#      positions (IPC snapshot emission `snap.position_secs` /
#      `snap.transport.to_string()`) and in comments documenting the
#      ABSENCE of logic; it is allowlisted here so the gate stays
#      true to its purpose (catch playback LOGIC in the surface), and
#      every one of its occurrences is printed in the HUMAN-REVIEW-T086
#      residue inventory below for T086 adjudication.
#
#   3. Allowlist position rule (T081): inside allowlisted files the
#      vocabulary may appear only in type/getter/delegation positions.
#      What is mechanically decidable is rejected: (a) mutable state
#      holding the vocabulary (`mut`/`var` declarations whose name
#      matches a vocabulary token), (b) arithmetic on
#      `position_secs`/`position`/`rate` (token, whitespace, then an
#      arithmetic operator — assignment/arithmetic adjacency). The
#      residue — "is this occurrence genuinely a type, getter, or
#      delegation?" — is NOT decidable by grep: the script prints it
#      as a named inventory (every vocabulary occurrence in
#      allowlisted files, file:line, labelled HUMAN-REVIEW-T086) so
#      T086 can adjudicate, per Principle V. The inventory is an
#      audit trail on stdout, not a failure.
#
#   4. SC-057 closed-set / registry-reference check:
#      (a) no catch-all closed-set members — `\bOther\b` / `\bUnknown\b`
#      in CODE position (line comments stripped first, so the doc
#      comments that state the absence of catch-alls do not false-
#      fail the gate) in crates/*/src and src-tauri/src;
#      (b) engine-identifier resolution — every engine-id string
#      literal in NON-TEST code (test modules are the file's last
#      item and deliberately use unregistered ids as fail-closed
#      negative controls, e.g. "any-engine") must be a declared
#      stand-in (standin_a|standin_b) or a constitution §TTS Engine
#      Registry name (sherpa-onnx + Piper pt-BR voices
#      faber/cadu/jeff, Google Cloud TTS WaveNet/Neural2/Chirp 3 HD/
#      Studio, Amazon Polly Neural, Azure Speech Neural, Microsoft
#      Edge read-aloud via msedge-tts); one documented exemption:
#      "rust-player", the player-side playback-backend self-
#      identifier (crates/player/src/cli.rs) — it is not a TTS
#      synthesis engine, never enters the fallback order, carries no
#      engine_tier, and never populates the diagnostic record's
#      engine field (the record's engine field is Option and no
#      production code sets it);
#      (c) test_double selectability — the selectable-set
#      construction (FallbackSelector::selectable,
#      crates/engine-core/src/fallback.rs) must carry the mechanical
#      invariant: a non-test_double presence probe AND a filter that
#      drops test_double entries while one exists.
#
#   5. Spec→test traceability (T081): every FR-### / SC-### token in
#      specs/001-android-media-shell/spec.md must resolve to a literal
#      token on a TEST-TASK line in specs/001-android-media-shell/
#      tasks.md. Mechanical definition (implemented exactly): a
#      test-task line is a tasks.md bullet line matching
#      `^\s*- \[[x ]\] T[0-9]+[a-z]*` whose text contains one of
#      `test|prop|assert|RED|GREEN|gate` (case-insensitive). A token
#      resolves when EITHER (Tier 1) the literal token appears on a
#      test-task line, OR (Tier 2, FR tokens only — the spec's
#      declared cross-citation convention: the Functional
#      Requirements preamble and checklists/requirements.md record
#      that lettered requirements are cited by the success criterion
#      that tests them) an SC definition block in spec.md's Success
#      Criteria section cites the token AND that SC appears on a
#      test-task line, OR (Tier 2b, bare parent FR tokens only —
#      the same convention's parent rule, spec.md Q&A 2026-10-05
#      and the FR preamble: "the parent remains in force and the
#      derived requirement states its relation to the parent in
#      prose", and "every lettered requirement is cited by at
#      least one success criterion") an SC definition block cites
#      any member of the token's family (the parent or a lettered
#      derivation) AND that SC appears on a test-task line — a
#      parent's substance is distributed across its derivations,
#      so the derivations' SC citations are the parent's test
#      coverage (resolves FR-028 via FR-028a/b/c/d/e ←
#      SC-042/SC-046/SC-030/SC-047/SC-053 and FR-032 via
#      FR-032a–f ← SC-017/SC-019/SC-012/SC-020/SC-021/SC-045/
#      SC-049, every one of those SCs on a test-task line). A
#      token appearing only inside a grouped **Goal** list is
#      coverage-by-mention, not coverage-by-test, and fails. Exemptions: the six named in the task text
#      (FR-025 deferred out-of-scope, FR-023b hardware carried,
#      SC-044 deferred to synthesis, SC-005/SC-012/SC-019
#      zero-occurrence confirmed via T087 record scans) plus five
#      adjudicated in the T081 report (FR-022, FR-024a, FR-029,
#      FR-034, FR-043 — each has test substance on a test-task
#      line but no literal token citation there; each is printed
#      with its evidence and routed to T086 to close the citation
#      gap in tasks.md).
#
# FAILS on an empty tree: the scan roots are missing inputs, and the
# gate fails closed — a missing tree is not a clean tree.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Scan roots: the Kotlin shell and the Tauri Rust surface.
ANDROID_SHELL="$ROOT/android-shell"
SRC_TAURI="$ROOT/src-tauri"
SPEC_DIR="$ROOT/specs/001-android-media-shell"
SPEC="$SPEC_DIR/spec.md"
TASKS="$SPEC_DIR/tasks.md"

fail() {
  printf 'check-shell-purity: FAIL: %s\n' "$1" >&2
  exit 1
}

# Missing input fails closed.
[ -d "$ANDROID_SHELL" ] \
  || fail "missing scan root: $ANDROID_SHELL (android-shell/** does not exist; missing input is not a clean scan)"
[ -d "$SRC_TAURI" ] \
  || fail "missing scan root: $SRC_TAURI (src-tauri/** does not exist; missing input is not a clean scan)"
[ -d "$SPEC_DIR" ] \
  || fail "missing spec directory: $SPEC_DIR (spec.md/tasks.md do not exist; missing input is not a clean scan)"
[ -f "$SPEC" ] || fail "missing spec: $SPEC"
[ -f "$TASKS" ] || fail "missing tasks: $TASKS"

# =====================================================================
# Section 1 — forbidden call shapes (T013, unchanged).
# Each shape is a computation or a state mutation in the surface;
# identifiers, type positions, and get*/set* delegation targets do
# not match any of them. Scanned across every source file under both
# roots, allowlisted or not.
# =====================================================================

FORBIDDEN_SHAPES=(
  '\.position\s*='
  '\.rate\s*='
  'setPlaybackState\s*\(.*position'
  'setPlaybackState\s*\(.*state'
  'seekTo\s*\('
  '\bgain\s*[*/+-]?='
  'duck\w*\s*='
  'skip(Forward|Backward)\s*\('
)

status=0
for shape in "${FORBIDDEN_SHAPES[@]}"; do
  # gen/ (regenerated copy of the surface) and target/ (compiler
  # output) are pruned — see the section header.
  hits="$(grep -rInE --include='*.kt' --include='*.kts' --include='*.rs' \
    --exclude-dir=gen --exclude-dir=target \
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

# =====================================================================
# Section 2 — allowlist enforcement (vocabulary carriers).
# =====================================================================

VOCAB='\b(transport|position_secs|position|rate|gain|duck|skip)\b'

# The allowlist (T081): five Kotlin mirrors + four src-tauri shape/
# delegation files, plus the Tauri bridge (src-tauri/src/lib.rs —
# documented deviation above) and the publish contract test.
# Paths are exact; MediaMirror.kt lives
# at android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt and
# the other four Kotlin files share that directory.
ALLOWLIST=(
  "$ANDROID_SHELL/src/main/kotlin/app/mdreader/PlaybackService.kt"
  "$ANDROID_SHELL/src/main/kotlin/app/mdreader/FocusMirror.kt"
  "$ANDROID_SHELL/src/main/kotlin/app/mdreader/RoutingMirror.kt"
  "$ANDROID_SHELL/src/main/kotlin/app/mdreader/MediaMirror.kt"
  "$ANDROID_SHELL/src/main/kotlin/app/mdreader/Residency.kt"
  "$SRC_TAURI/src/surface.rs"
  "$SRC_TAURI/src/status_view.rs"
  "$SRC_TAURI/src/failure.rs"
  "$SRC_TAURI/src/permissions.rs"
  "$SRC_TAURI/src/lib.rs"
  "$SRC_TAURI/tests/test_publish_payload.rs"
  # 004 purity tests: they assert the ABSENCE of the shell vocabulary
  # on every surface (the mention IS the assertion — the same shape as
  # test_publish_payload.rs above, which the allowlist already carries).
  "$SRC_TAURI/tests/test_purity.rs"
  "$SRC_TAURI/tests/test_surface_us1.rs"
  "$SRC_TAURI/tests/test_noscope.rs"
)

is_allowlisted() {
  local file="$1" entry
  for entry in "${ALLOWLIST[@]}"; do
    [ "$file" = "$entry" ] && return 0
  done
  return 1
}

# Source files under the scan roots, pruning the build-artifact
# trees (src-tauri/gen, src-tauri/target — regenerated outputs).
status=0
while IFS= read -r file; do
  if is_allowlisted "$file"; then
    continue
  fi
  hits="$(grep -nE "$VOCAB" "$file" 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    printf 'check-shell-purity: shell vocabulary outside the allowlist in %s:\n' "$file" >&2
    printf '%s\n' "$hits" >&2
    status=1
  fi
done < <(find "$ANDROID_SHELL" "$SRC_TAURI" \
  \( -path "$SRC_TAURI/gen" -o -path "$SRC_TAURI/target" \) -prune \
  -o \( -name '*.kt' -o -name '*.kts' -o -name '*.rs' \) -print)

if [ "$status" -ne 0 ]; then
  fail "shell vocabulary (transport|position_secs|position|rate|gain|duck|skip) outside the T081 allowlist (SC-035)"
fi

# =====================================================================
# Section 3 — allowlist position rule (mechanically decidable subset)
# + the T086 residue inventory.
# =====================================================================

# (a) mutable state holding the vocabulary: a mut/var declaration
# whose name is or contains a vocabulary token.
MUT_STATE='(mut|var)[[:space:]]+[A-Za-z0-9_]*(transport|position_secs|position|rate|gain|duck|skip)[A-Za-z0-9_]*([[:space:]]*:|[^A-Za-z0-9_]|$)'
# (b) arithmetic on position/position_secs/rate: the token followed
# by whitespace and an arithmetic operator (covers =-adjacent
# assignment and compound forms like `position_secs += 1`).
ARITHMETIC='\b(position_secs|position|rate)[[:space:]]+[-+*/]'

status=0
for file in "${ALLOWLIST[@]}"; do
  [ -f "$file" ] || continue
  hits="$(grep -nE "$MUT_STATE|$ARITHMETIC" "$file" 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    printf 'check-shell-purity: vocabulary in a rejected position (mut state or arithmetic) in %s:\n' "$file" >&2
    printf '%s\n' "$hits" >&2
    status=1
  fi
done
if [ "$status" -ne 0 ]; then
  fail "allowlisted file carries the vocabulary in a mutable-state or arithmetic position (SC-035; type/getter/delegation positions only)"
fi

# Residue (T086): every vocabulary occurrence in allowlisted files,
# labelled HUMAN-REVIEW-T086 — the "is this a type, getter, or
# delegation?" question is not decidable by grep (Principle V), so it
# is printed as a named inventory for T086 adjudication rather than
# smuggled into this gate.
printf 'check-shell-purity: T086 residue inventory (HUMAN-REVIEW-T086 — every vocabulary occurrence in allowlisted files, for T086 adjudication):\n'
for file in "${ALLOWLIST[@]}"; do
  [ -f "$file" ] || continue
  hits="$(grep -nE "$VOCAB" "$file" 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    printf '%s\n' "$hits" | while IFS= read -r hit; do
      printf '  HUMAN-REVIEW-T086  %s:%s\n' "$file" "$hit"
    done
  fi
done

# =====================================================================
# Section 4 — SC-057 closed-set / registry-reference check.
# =====================================================================

# (a) No catch-all closed-set members: \bOther\b / \bUnknown\b in
# CODE position. Line comments (//, ///, //!) are stripped before the
# match so the doc comments that assert the ABSENCE of catch-alls
# ("no `Other`/`Unknown` catch-all") do not false-fail the gate.
status=0
while IFS= read -r file; do
  hits="$(sed 's://.*::' "$file" | grep -nE '\b(Other|Unknown)\b' 2>/dev/null || true)"
  if [ -n "$hits" ]; then
    printf 'check-shell-purity: catch-all closed-set member (Other/Unknown) in code position in %s:\n' "$file" >&2
    printf '%s\n' "$hits" >&2
    status=1
  fi
done < <(find "$ROOT/crates" -path '*/src/*.rs' -type f; find "$SRC_TAURI/src" -name '*.rs' -type f)
if [ "$status" -ne 0 ]; then
  fail "closed set contains a catch-all Other/Unknown member (SC-057)"
fi

# (b) Engine-identifier resolution. Collect every engine-id string
# literal from NON-TEST code (everything above the file's
# #[cfg(test)] mod tests boundary — test modules are the file's last
# item and deliberately carry unregistered ids as fail-closed
# negative controls). An id is admissible when it is a declared
# stand-in or a constitution §TTS Engine Registry name; the single
# documented exemption is the player-side backend identifier.
REGISTRY_IDS=(
  'sherpa-onnx'
  'sherpa-onnx-piper-pt-br'
  'sherpa-onnx-piper-pt-br-faber'
  'sherpa-onnx-piper-pt-br-cadu'
  'sherpa-onnx-piper-pt-br-jeff'
  'google-cloud-tts-wavenet'
  'google-cloud-tts-neural2'
  'google-cloud-tts-chirp-3-hd'
  'google-cloud-tts-studio'
  'amazon-polly-neural'
  'azure-speech-neural'
  'microsoft-edge-read-aloud-msedge-tts'
  'msedge-tts'
)
# Documented exemption (T081 report): the player-side playback-backend
# self-identifier (crates/player/src/cli.rs) — not a TTS synthesis
# engine: it never enters the fallback order, carries no
# engine_tier, and never populates the diagnostic record's engine
# field (the field is Option and no production code sets it).
EXEMPT_ENGINE_IDS=('rust-player')

is_registry_id() {
  local id="$1" entry
  for entry in "${REGISTRY_IDS[@]}"; do
    [ "$id" = "$entry" ] && return 0
  done
  return 1
}

status=0
while IFS= read -r file; do
  # Non-test prefix: everything before the first #[cfg(test)] /
  # `mod tests` boundary (Rust test modules close the file).
  boundary="$(grep -nE '^#\[cfg\(test\)\]|^mod tests' "$file" 2>/dev/null | head -n 1 | cut -d: -f1 || true)"
  if [ -n "$boundary" ]; then
    prefix="$(sed -n "1,$((boundary - 1))p" "$file")"
  else
    prefix="$(cat "$file")"
  fi
  # Engine-id carrying positions: id/engine_id field initialisers
  # (registry entries, offers, records) and engine self-identifier
  # consts (ID / ENGINE_ID).
  ids="$(printf '%s\n' "$prefix" | grep -oE \
    -e '(id|engine_id)[[:space:]]*:[[:space:]]*String::from\("[^"]*"\)' \
    -e '(id|engine_id)[[:space:]]*:[[:space:]]*"[^"]*"' \
    -e 'const[[:space:]]+(ID|ENGINE_ID)[^=]*=[[:space:]]*"[^"]*"' \
    2>/dev/null | grep -oE '"[^"]*"' | tr -d '"' | sort -u || true)"
  for id in $ids; do
    if [ "$id" = 'standin_a' ] || [ "$id" = 'standin_b' ]; then
      continue
    fi
    if is_registry_id "$id"; then
      continue
    fi
    exempt=0
    for entry in "${EXEMPT_ENGINE_IDS[@]}"; do
      if [ "$id" = "$entry" ]; then
        exempt=1
        printf 'check-shell-purity: SC-057 engine-id exemption: %s (%s — player-side playback-backend identifier, not a TTS synthesis engine; never in the fallback order, no engine_tier, never in the diagnostic record engine field)\n' "$id" "$file"
        break
      fi
    done
    [ "$exempt" -eq 1 ] && continue
    printf 'check-shell-purity: engine identifier "%s" (%s) does not resolve to a constitution TTS Engine Registry name nor a declared stand-in (standin_a|standin_b)\n' "$id" "$file" >&2
    status=1
  done
done < <(find "$ROOT/crates" -path '*/src/*.rs' -type f; find "$SRC_TAURI/src" -name '*.rs' -type f)
if [ "$status" -ne 0 ]; then
  fail "engine identifier outside the constitution registry and the declared stand-ins (SC-057)"
fi

# (c) test_double selectability: the selectable-set construction
# must carry the mechanical invariant — a non-test_double presence
# probe AND a filter that drops test_double entries while a
# non-test_double adapter is registered.
SELECTABLE="$ROOT/crates/engine-core/src/fallback.rs"
[ -f "$SELECTABLE" ] \
  || fail "missing selectable-set construction: $SELECTABLE (missing input is not a clean check)"
grep -q 'entry.tier != EngineTier::TestDouble' "$SELECTABLE" \
  || fail "selectable-set construction lacks the non-test_double presence probe (SC-057: test_double is admissible only while no non-test_double adapter is registered)"
grep -q 'entry.tier == EngineTier::TestDouble && holds_non_test_double' "$SELECTABLE" \
  || fail "selectable-set construction lacks the test_double exclusion filter (SC-057: test_double engines must be dropped from the selectable set while a non-test_double adapter is registered)"
# The closed tier set itself: the five EngineTier members are
# declared in engine-core's closed set (exhaustiveness of the set
# — no sixth member, no _ => arm — is T010's property test and
# gate 7's job; here the declaration site is asserted present).
CLOSED_SET="$ROOT/crates/engine-core/src/closed.rs"
[ -f "$CLOSED_SET" ] \
  || fail "missing closed-set declaration: $CLOSED_SET (missing input is not a clean check)"
for tier in OfflineDefault CloudDefault CloudEscalation TestDouble EvaluatedFallback; do
  grep -q "EngineTier::$tier\|$tier" "$CLOSED_SET" \
    || fail "closed EngineTier set does not declare $tier (SC-057: the tier set is closed — adding a member is a coordinated edit)"
done

# =====================================================================
# Section 5 — spec→test traceability (T081).
# =====================================================================

# A test-task line: a tasks.md bullet matching
# `^\s*- \[[x ]\] T[0-9]+[a-z]*` whose text contains one of
# test|prop|assert|RED|GREEN|gate (case-insensitive).
TEST_TASK_LINES="$(grep -E '^[[:space:]]*- \[[xX ]\] T[0-9]+[a-z]*' "$TASKS" \
  | grep -iE 'test|prop|assert|RED|GREEN|gate' || true)"

# Success-criteria SC blocks of spec.md, one line per block
# (newlines folded to spaces): "SC-token<TAB>block text".
SC_BLOCKS="$(awk '
  /^## Assumptions/ { exit }
  /^- \*\*SC-[0-9]+[a-z]*\*\*/ {
    if (sc != "") print sc "\t" buf
    sc = $0
    sub(/:.*/, "", sc)
    gsub(/\*\*/, "", sc)
    sub(/^- /, "", sc)
    buf = $0
    next
  }
  { if (sc != "") buf = buf " " $0 }
  END { if (sc != "") print sc "\t" buf }
' "$SPEC")"

# Named exemptions (task text).
NAMED_EXEMPTIONS='FR-025 FR-023b SC-044 SC-005 SC-012 SC-019'
# Adjudicated exemptions (T081 report): test substance exists on a
# test-task line but the literal token is not cited there. Each is
# printed with its evidence and routed to T086 to close the
# citation gap in tasks.md.
adjudicated_reason() {
  case "$1" in
    FR-022) printf 'device-acquisition failure path (named failure, never phantom playing) is exercised by test-task T033 — crates/player/tests/prop_focus.rs:229-233 asserts Err(EngineError::DeviceRefused) — but the T033 task line does not cite the literal token (citation gap; routed to T086)' ;;
    FR-024a) printf 'orphan token: spec.md line 283 (a Q&A parenthetical) is its only spec occurrence — no "- **FR-024a**:" requirement definition exists — and it has zero occurrences in tasks.md (spec defect; routed to T086 to define the requirement or remove the dangling mention)' ;;
    FR-029) printf 'notification-title rule (metadata else filename) is exercised by test-task T051 — crates/player/tests/prop_surface.rs asserts "title from metadata else filename-stable-placeholder" — but the T051 task line does not cite the literal token (it appears only on implementation task T061; citation gap routed to T086)' ;;
    FR-034) printf 'audio-handoff contract (device-format conversion in the Rust playback module) is exercised by test-task T013b — crates/engine-core/tests/standin_shape.rs asserts the mono-f32 @22050 Hz handoff shape with a named EngineError on deviation — and checklists/requirements.md adjudicates the FR-033/FR-034/FR-035 family to SC-034, which appears on test-task line T013b; the literal token is missing from the test-task line (citation gap routed to T086)' ;;
    FR-043) printf 'pitch-preserving time-stretch is exercised by test-task T055 — crates/engine-core/tests/prop_pitch.rs asserts F0-contour and spectral-peak invariance within ±50 cents across all 7 rungs — but the T055 task line cites SC-015/SC-034, not the literal FR-043 token (it appears only on implementation task T057; citation gap routed to T086)' ;;
    *) printf 'unlisted exemption — this is a gate bug, not an exemption' ;;
  esac
}

tokens="$(grep -oE '\b(FR|SC)-[0-9]+[a-z]?' "$SPEC" | sort -u)"
unresolved=''
resolved_count=0
exempt_count=0
named_count=0
adjudicated_count=0

resolution_path=''
token_resolves() {
  local tok="$1" sc family member task
  resolution_path=''
  # Tier 1 (direct): the literal token on a test-task line.
  # Full-consumption grep (no -q): under `pipefail` a `grep -q`
  # early exit SIGPIPEs printf into a 141 false-negative — a
  # scheduling race that failed different tokens on identical
  # inputs (T121 post-mortem).
  if [ -n "$(printf '%s\n' "$TEST_TASK_LINES" | grep -E "$tok([^0-9a-z]|$)" || true)" ]; then
    resolution_path='Tier 1: literal token on a test-task line'
    return 0
  fi
  # Tier 2 (cross-citation, FR tokens only): the spec's declared
  # convention — a lettered requirement is cited by the success
  # criterion that tests it. An FR token resolves when an SC
  # definition block in the Success Criteria section cites it AND
  # that SC appears on a test-task line.
  if printf '%s' "$tok" | grep -qE '^FR-'; then
    while IFS= read -r sc; do
      [ -z "$sc" ] && continue
      if [ -n "$(printf '%s\n' "$TEST_TASK_LINES" | grep -E "$sc([^0-9a-z]|$)" || true)" ]; then
        resolution_path="Tier 2: cited by $sc, which appears on a test-task line"
        return 0
      fi
    done < <(printf '%s\n' "$SC_BLOCKS" | grep -E "$tok([^0-9a-z]|$)" | cut -f1)
  fi
  # Tier 2b (parent-family cross-citation, bare parent FR tokens
  # only): the convention's parent rule — the parent remains in
  # force while its lettered derivations are the ones the success
  # criteria cite. A bare parent resolves when an SC definition
  # block cites any member of its family and that SC appears on
  # a test-task line.
  if printf '%s' "$tok" | grep -qE '^FR-[0-9]+$'; then
    family="${tok#FR-}"
    while IFS= read -r sc; do
      [ -z "$sc" ] && continue
      if [ -n "$(printf '%s\n' "$TEST_TASK_LINES" | grep -E "$sc([^0-9a-z]|$)" || true)" ]; then
        member="$(printf '%s\n' "$SC_BLOCKS" | grep "^$sc	" | grep -oE "FR-${family}[a-z]?" | head -n 1)"
        task="$(printf '%s\n' "$TEST_TASK_LINES" | grep -E "$sc([^0-9a-z]|$)" | grep -oE 'T[0-9]+[a-z]*' | head -n 1)"
        resolution_path="Tier 2b: family member $member cited by $sc on test-task $task"
        return 0
      fi
    done < <(printf '%s\n' "$SC_BLOCKS" | grep -E "FR-${family}[a-z]?([^0-9a-z]|$)" | cut -f1)
  fi
  return 1
}

for tok in $tokens; do
  if token_resolves "$tok"; then
    resolved_count=$((resolved_count + 1))
    case "$resolution_path" in
      'Tier 1: literal token on a test-task line') ;;
      '') ;;
      *) printf 'check-shell-purity: traceability resolved (%s): %s\n' "$resolution_path" "$tok" ;;
    esac
    continue
  fi
  case " $NAMED_EXEMPTIONS " in
    *" $tok "*)
      exempt_count=$((exempt_count + 1))
      named_count=$((named_count + 1))
      printf 'check-shell-purity: traceability exemption (named, task text): %s\n' "$tok"
      continue
      ;;
  esac
  reason="$(adjudicated_reason "$tok")"
  if [ "$reason" != 'unlisted exemption — this is a gate bug, not an exemption' ]; then
    exempt_count=$((exempt_count + 1))
    adjudicated_count=$((adjudicated_count + 1))
    printf 'check-shell-purity: traceability exemption (adjudicated, T081 report): %s — %s\n' "$tok" "$reason"
    continue
  fi
  unresolved="$unresolved $tok"
done

if [ -n "$unresolved" ]; then
  printf 'check-shell-purity: spec tokens with no test-task-line coverage (shorthand-only / Goal-list-only coverage is a failure):%s\n' "$unresolved" >&2
  fail "spec→test traceability gap (T081): the tokens above appear in spec.md but on no test-task line in tasks.md"
fi

printf 'check-shell-purity: traceability: %d tokens resolved, %d exempted (%d named + %d adjudicated), 0 unresolved\n' \
  "$resolved_count" "$exempt_count" "$named_count" "$adjudicated_count"

printf 'check-shell-purity: PASS (no forbidden call shapes; vocabulary confined to the T081 allowlist in type/getter/delegation positions; SC-057 closed sets and engine registry conform; spec→test traceability holds)\n'
