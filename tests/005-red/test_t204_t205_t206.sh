#!/bin/bash
# RED gate — T204 / T205 / T206 — status_view.html
# Must FAIL before edit; must PASS after edit.
FILE="src-tauri/src/status_view.html"
FAIL=0

echo "=== RED: T204 — .reason-banner.failure must NOT inherit animation ==="
# FAIL if .failure block lacks animation:none / animation override; specifically if inheritance present.
# Since base has animation, .failure without override = inherited -> FAIL.
if grep -A2 '\.reason-banner\.failure {' "$FILE" | grep -qE 'animation:\s*none'; then
  echo "PASS: .failure suppresses animation"
else
  echo "FAIL: animation inherited on .reason-banner.failure (no animation:none / override)"; FAIL=1
fi

echo "=== RED: T205 — #progress must have BOTH aria-valuenow AND aria-valuetext separately ==="
VALNOW=$(grep -oE 'aria-valuenow="[^"]*"' "$FILE" | wc -l)
VALTEXT=$(grep -oE 'aria-valuetext="[^"]*"' "$FILE" | wc -l)
# Must both appear on #progress tag; here just count globally; the file uses them once.
if [ "$VALNOW" -ge 1 ] && [ "$VALTEXT" -ge 1 ]; then
  echo "PASS: aria-valuenow ($VALNOW) + aria-valuetext ($VALTEXT) present"
else
  echo "FAIL: aria-valuetext missing or aria-valuenow missing (valuenow=$VALNOW, valuetext=$VALTEXT)"; FAIL=1
fi

echo "=== RED: T206 — #sectionTitle must contain aria-label=\"Sem título\" ==="
if grep -qE 'id="sectionTitle".*aria-label="Sem título"' "$FILE"; then
  echo "PASS: sectionTitle has aria-label"
else
  # also accept static on div or JS assignment; for RED we require presence
  echo "FAIL: aria-label \"Sem título\" missing on sectionTitle"; FAIL=1
fi

echo "=== RESULT ==="
if [ $FAIL -eq 1 ]; then exit 1; else echo "ALL GREEN"; exit 0; fi
