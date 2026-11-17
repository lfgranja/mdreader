#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
html="${1:-$root/dist/index.html}"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
python3 - "$html" "$tmp" <<'PY'
import sys
s=open(sys.argv[1],encoding='utf-8').read()
needle='<script type="module">'
if needle not in s: needle='<script>'
i=s.index(needle)+len(needle)
j=s.index('</script>',i)
open(sys.argv[2],'w',encoding='utf-8').write(s[i:j])
PY
node -e "require('fs')" >/dev/null 2>&1 || { echo "node required"; exit 1; }
# acorn may not be global; use node --check in module mode if supported, else npx acorn
if node --input-type=module --check < "$tmp" >/dev/null 2>&1; then
  echo "OK: module syntax $html"
  exit 0
fi
npx --yes acorn --ecma2022 --module "$tmp" >/dev/null
echo "OK: module syntax $html (acorn)"
