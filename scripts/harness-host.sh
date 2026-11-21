#!/usr/bin/env bash
# harness-host.sh — executa automaticamente as seções 1-3 de docs/roteiro-testes.md.
#
# Uso:
#   bash scripts/harness-host.sh
#   OUT=/tmp/out-custom FAIL_FAST=1 bash scripts/harness-host.sh
#
# Cada item gera $OUT/<id>.out, .err, .rc e .cmd. O relatório é $OUT/summary.txt.
# Itens device-only (sinais no diagnostic-record.jsonl, escuta) são [NV]: nunca PASS.
# Nunca usar `set -e`: cada item captura o próprio $?.

set -uo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="${OUT:-/tmp/mdreader-harness-$TS}"
FAIL_FAST="${FAIL_FAST:-0}"
SUMMARY="$OUT/summary.txt"

PASS=0
FAIL=0
NV=0

mkdir -p "$OUT"
cd "$ROOT" || exit 2

# ---------------------------------------------------------------- helpers ---

finish() {
  {
    printf '\n=== RESUMO (roteiro secoes 1-3) ===\n'
    printf 'PASS=%d FAIL=%d NV=%d\n' "$PASS" "$FAIL" "$NV"
    printf 'OUT=%s\n' "$OUT"
    if [[ $FAIL -eq 0 ]]; then
      printf 'Resultado: OK\n'
    else
      printf 'Resultado: FALHOU\n'
    fi
  } >> "$SUMMARY"
  tail -n 6 "$SUMMARY"
  exit $((FAIL > 0))
}

section() {
  printf '\n== %s ==\n' "$1" | tee -a "$SUMMARY"
}

nv() {
  NV=$((NV + 1))
  printf '[NV] %s\n' "$1" | tee -a "$SUMMARY"
}

record_pass() {
  PASS=$((PASS + 1))
  printf 'PASS %s\n' "$1" | tee -a "$SUMMARY"
}

record_fail() {
  FAIL=$((FAIL + 1))
  printf 'FAIL %s :: %s\n' "$1" "$2" | tee -a "$SUMMARY"
  if [[ $FAIL_FAST == 1 ]]; then
    finish
  fi
}

# pyjson <mode> <file> <arg>
#   keys  : arg = lista de caminhos separados por virgula; todos devem existir
#   value : arg = "caminho|valor-esperado"
#   expr  : arg = expressao python sobre `d` (payload JSON carregado)
# Caminhos usam '.' para objetos e [n]/[] para arrays (ex.: voices[0].present).
pyjson() {
  python3 - "$1" "$2" "$3" <<'PY'
import json
import sys

mode, filename, arg = sys.argv[1], sys.argv[2], sys.argv[3]


def resolve(obj, path):
    cur = obj
    for token in path.split("."):
        name, idx = token, None
        if "[" in token:
            name, rest = token.split("[", 1)
            idx = rest.split("]", 1)[0]
        if name:
            if not isinstance(cur, dict) or name not in cur:
                raise KeyError(name)
            cur = cur[name]
        if idx is not None:
            if not isinstance(cur, list) or not cur:
                raise IndexError(idx)
            cur = cur[0] if idx == "" else cur[int(idx)]
    return cur


if mode == "keys":
    with open(filename, encoding="utf-8") as handle:
        data = json.load(handle)
    for key in arg.split(","):
        resolve(data, key.strip())
elif mode == "value":
    path, expected = arg.split("|", 1)
    with open(filename, encoding="utf-8") as handle:
        value = resolve(json.load(handle), path)
    if value is None:
        rendered = "null"
    elif isinstance(value, bool):
        rendered = "true" if value else "false"
    elif isinstance(value, float) and value.is_integer():
        rendered = str(int(value))
    else:
        rendered = str(value)
    if rendered != expected:
        sys.exit(3)
elif mode == "expr":
    with open(filename, encoding="utf-8") as handle:
        data = json.load(handle)
    if not eval(arg, {"d": data, "json": json}):
        sys.exit(4)
else:
    sys.exit(2)
PY
}

# run <id> <expect 0|1> <keys|-|...> <cmd...>
# Variaveis de ambiente por chamada:
#   ERR_TOKEN='re'  -> exige que stderr case com a regex (casos expect=1)
#   ALLOW_STDERR=1  -> nao exige stderr vazio (verify.sh, cargo test)
run() {
  local id="$1" expect="$2" keys="$3"
  shift 3
  local out="$OUT/$id.out" err="$OUT/$id.err" rcf="$OUT/$id.rc"
  local token="${ERR_TOKEN:-}" allow_stderr="${ALLOW_STDERR:-0}"
  local cmd_str
  cmd_str="$(printf '%q ' "$@")"
  printf '%s\n' "$cmd_str" > "$OUT/$id.cmd"

  "$@" > "$out" 2> "$err"
  local rc=$?
  printf '%d\n' "$rc" > "$rcf"

  local why=""
  if [[ $rc -ne $expect ]]; then
    why="rc=$rc (esperado $expect)"
  elif [[ $expect -eq 0 && $allow_stderr -ne 1 && -s "$err" ]]; then
    why="stderr deveria ser vazio no sucesso"
  elif [[ $expect -eq 1 && -s "$out" ]]; then
    why="stdout deveria ser vazio na falha"
  fi

  if [[ -z $why && $expect -eq 1 && -n $token ]]; then
    if ! grep -Eiq -- "$token" "$err"; then
      why="token ausente no stderr: $token"
    fi
  fi

  if [[ -z $why && $expect -eq 0 && $keys != '-' ]]; then
    local kerr
    if ! kerr="$(pyjson keys "$out" "$keys" 2>&1)"; then
      why="chaves JSON ausentes [$keys]: $kerr"
    fi
  fi

  if [[ -z $why ]]; then
    record_pass "$id"
  else
    printf '  cmd: %s\n' "$cmd_str" >> "$SUMMARY"
    printf '  err: %s\n' "$(head -c 500 "$err" | tr '\n' ' ')" >> "$SUMMARY"
    record_fail "$id" "$why"
  fi
}

# assert_json <id> <caminho> <valor>  — usa $OUT/<id>.out
assert_json() {
  local id="$1" path="$2" expected="$3"
  local detail
  if detail="$(pyjson value "$OUT/$id.out" "$path|$expected" 2>&1)"; then
    record_pass "$id@$path"
  else
    record_fail "$id@$path" "valor '$path' != '$expected' ($detail)"
  fi
}

# json_check <id> <rotulo> <expr> — expressao python sobre o payload JSON
json_check() {
  local id="$1" label="$2" expr="$3"
  local detail
  if detail="$(pyjson expr "$OUT/$id.out" "$expr" 2>&1)"; then
    record_pass "$id@$label"
  else
    record_fail "$id@$label" "invariante falsa: $expr ($detail)"
  fi
}

assert_text() {
  local id="$1" file="$2" needle="$3"
  if grep -Fq -- "$needle" "$file"; then
    record_pass "$id"
  else
    record_fail "$id" "texto ausente: $needle"
  fi
}

assert_no_text() {
  local id="$1" file="$2" needle="$3"
  if grep -Fq -- "$needle" "$file"; then
    record_fail "$id" "texto proibido presente: $needle"
  else
    record_pass "$id"
  fi
}

assert_tests_ran() {
  local id="$1" combined="$OUT/$id.tests"
  cat "$OUT/$id.out" "$OUT/$id.err" > "$combined"
  if grep -Eq 'test result: ok\. [1-9][0-9]* passed' "$combined"; then
    record_pass "$id@ran"
  else
    record_fail "$id@ran" "nenhum teste executado (filtro vazio?)"
  fi
}

# -------------------------------------------------------------- fixture ---

FIX="$OUT/doc.md"
DB="$OUT/bookmarks.redb"
rm -f "$DB" "$DB.lock"

cat > "$FIX" <<'EOF'
# Título

Parágrafo com **ênfase** e [link](https://example.invalid).

---

Último parágrafo.
EOF

{
  printf '# Harness host — docs/roteiro-testes.md secoes 1-3\n'
  printf 'TS=%s ROOT=%s OUT=%s\n' "$TS" "$ROOT" "$OUT"
  printf 'FIX=%s DB=%s\n' "$FIX" "$DB"
} | tee -a "$SUMMARY"

# ------------------------------------------- 1. gates deterministicos ---

section '1. Host — gates determinísticos'

ALLOW_STDERR=1 run '1.verify' 0 - bash "$ROOT/scripts/verify.sh"
VERIFY_EXIT="$(< "$OUT/1.verify.rc")"
printf 'VERIFY_EXIT=%s\n' "$VERIFY_EXIT" >> "$SUMMARY"
assert_text '1.verify.gate8-deferred' "$OUT/1.verify.out" 'gate-8-full: DEFERRED'
assert_no_text '1.verify.no-bare-gate8-pass' "$OUT/1.verify.out" 'gate-8: PASS'

ALLOW_STDERR=1 run '1.fmt' 0 - cargo fmt --all -- --check
ALLOW_STDERR=1 run '1.clippy' 0 - cargo clippy --workspace --all-targets -- -D warnings
ALLOW_STDERR=1 run '1.check-manifest' 0 - bash "$ROOT/scripts/check-manifest.sh"
ALLOW_STDERR=1 run '1.check-shell-purity' 0 - bash "$ROOT/scripts/check-shell-purity.sh"
ALLOW_STDERR=1 run '1.ci-sdd' 0 - python3 "$ROOT/scripts/ci/check_sdd_completeness.py"
ALLOW_STDERR=1 run '1.ci-dag' 0 - python3 "$ROOT/scripts/ci/check_work_order_dag.py"
ALLOW_STDERR=1 run '1.ci-task-ids' 0 - python3 "$ROOT/scripts/ci/check_task_id_uniqueness.py"
ALLOW_STDERR=1 run '1.ci-append-only' 0 - python3 "$ROOT/scripts/ci/check_converge_append_only.py"

# ---------------------------------------------- 2. engine / sintese ---

section '2. Engine / sintese / vozes (CLI)'

run '2.1.adapter-capabilities' 0 'capability,ssml_level,alignment_granularity' \
  cargo run -q -p adapter-synthesis -- capabilities
assert_json '2.1.adapter-capabilities' ssml_level none
assert_json '2.1.adapter-capabilities' alignment_granularity none

run '2.1.engine-capabilities' 0 'capability,ssml_levels,alignment_granularities' \
  cargo run -q -p engine-core -- capabilities
json_check '2.1.engine-capabilities' none-closed-set \
  "'none' in d['ssml_levels'] and 'none' in d['alignment_granularities']"

run '2.2.format-check' 0 'capability,valid' \
  cargo run -q -p adapter-synthesis -- format-check --rate-hz 22050 --channels 1 --width-bits 32
assert_json '2.2.format-check' valid true

run '2.3.delivery-check' 0 'capability,valid' \
  cargo run -q -p adapter-synthesis -- delivery-check --rate-hz 22050 --channels 1 \
    --width-bits 32 --pcm-samples 22050 --duration-ms 1000
assert_json '2.3.delivery-check' valid true

for rate in 0.75 1 1.25 1.5 1.75 2 4; do
  id="2.4.set-rate-$rate"
  run "$id" 0 'capability,rate,pitch_preserving,delivered_rate_hz' \
    cargo run -q -p engine-core -- set-rate --rate "$rate"
  assert_json "$id" rate "$rate"
  assert_json "$id" pitch_preserving true
  assert_json "$id" delivered_rate_hz 22050
done

run '2.5.stream' 0 'capability,position_secs,budget_secs,frame_count,sample_rate_hz,channels' \
  cargo run -q -p engine-core -- stream --at 0 --budget-secs 60
assert_json '2.5.stream' budget_secs 60
json_check '2.5.stream' preroll-cap 'd["budget_secs"] <= 60 and d["frame_count"] > 0'

run '2.6.chunk' 0 'capability,segment_count,segments[0].segment_index,segments[0].line_start,segments[0].line_end,segments[0].boundary_kind,segments[0].byte_length' \
  cargo run -q -p adapter-synthesis -- chunk --input "$FIX"
json_check '2.6.chunk' byte-budget 'all(s["byte_length"] <= 5000 for s in d["segments"])'
json_check '2.6.chunk' line-order 'all(s["line_start"] <= s["line_end"] for s in d["segments"])'
json_check '2.6.chunk' resume-point 'any(s.get("resume_point") is not None for s in d["segments"])'
json_check '2.6.chunk' no-document-text \
  'not any(w in json.dumps(d, ensure_ascii=False) for w in ("Título", "ênfase", "example.invalid"))'

run '2.7.voices' 0 'capability,default_voice,active_voice,voices[0].id,voices[0].present' \
  cargo run -q -p adapter-synthesis -- voices
assert_json '2.7.voices' default_voice pt_BR-faber-medium
json_check '2.7.voices' catalog-of-3 'len(d["voices"]) == 3'
json_check '2.7.voices' all-present 'all(v["present"] for v in d["voices"])'
json_check '2.7.voices' catalog-ids \
  '{v["id"] for v in d["voices"]} == {"pt_BR-faber-medium", "pt_BR-cadu-medium", "pt_BR-jeff-medium"}'

for voice in pt_BR-faber-medium pt_BR-cadu-medium pt_BR-jeff-medium; do
  id="2.8.synth-$voice"
  run "$id" 0 'capability,voice,segment_count,pcm_samples' \
    cargo run -q -p adapter-synthesis -- synthesize --voice "$voice" --input "$FIX"
  assert_json "$id" voice "$voice"
  json_check "$id" pcm-positive 'd["pcm_samples"] > 0 and d["segment_count"] > 0'
done
nv 'escuta de faber/cadu/jeff e prosodia (secao 6) — device-only, nao executada no host'

ALLOW_STDERR=1 run '2.9.normalize-unit' 0 - \
  cargo test -q -p adapter-synthesis --lib normalize
assert_tests_ran '2.9.normalize-unit'
ALLOW_STDERR=1 run '2.9.normalize-invariants' 0 - \
  cargo test -q -p adapter-synthesis --test normalize_invariants
assert_tests_ran '2.9.normalize-invariants'
ALLOW_STDERR=1 run '2.10.prosody-unit' 0 - \
  cargo test -q -p adapter-synthesis --test prosody_unit
assert_tests_ran '2.10.prosody-unit'
ALLOW_STDERR=1 run '2.10.prosody-invariants' 0 - \
  cargo test -q -p adapter-synthesis --test prosody_invariants
assert_tests_ran '2.10.prosody-invariants'

ERR_TOKEN='RateMismatch' run '2.E1.rate-mismatch' 1 - \
  cargo run -q -p adapter-synthesis -- format-check --rate-hz 44100 --channels 1 --width-bits 32
ERR_TOKEN='ChannelMismatch' run '2.E2.channel-mismatch' 1 - \
  cargo run -q -p adapter-synthesis -- format-check --rate-hz 22050 --channels 2 --width-bits 32
ERR_TOKEN='rate must be one of' run '2.E3.off-ladder-rate' 1 - \
  cargo run -q -p engine-core -- set-rate --rate 1.1
ERR_TOKEN='unknown voice id' run '2.E4.unknown-voice' 1 - \
  cargo run -q -p adapter-synthesis -- synthesize --voice pt_BR-edresson-low --input "$FIX"
ERR_TOKEN='I/O failure' run '2.E5.io-missing-file' 1 - \
  cargo run -q -p adapter-synthesis -- chunk --input "$OUT/inexistente.md"

# -------------------------------- 3. transporte / bookmark / offer ---

section '3. Transporte / posicao / bookmark / offer (CLI)'

nv '3.1 sinais em diagnostic-record.jsonl (play/pause/stop/skip/rate/voz/resume/document_end) — device-only'
nv '3.2 PositionClock em dispositivo (pausa 10s, avanco so apos audio, preroll/dropped_samples) — device-only'

ALLOW_STDERR=1 run '3.2.position-tests' 0 - \
  cargo test -q -p player position::tests
assert_tests_ran '3.2.position-tests'

run '3.3.ingest-new' 0 'fingerprint,resume,offset' \
  cargo run -q -p session -- --db "$DB" ingest "$FIX"
assert_json '3.3.ingest-new' resume false
assert_json '3.3.ingest-new' offset 0
FINGERPRINT="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1],encoding="utf-8"))["fingerprint"])' "$OUT/3.3.ingest-new.out")"
printf 'fingerprint=%s\n' "$FINGERPRINT" >> "$SUMMARY"

run '3.3.offer-before-put' 0 'capability,file,fingerprint,offer' \
  cargo run -q -p session -- --db "$DB" offer show "$FIX"
assert_json '3.3.offer-before-put' offer null

run '3.3.put-paused' 0 'capability,file,fingerprint,stored' \
  cargo run -q -p session -- --db "$DB" put "$FIX" --offset-secs 30.25 \
    --first-yield-ms 500 --end-kind paused
assert_json '3.3.put-paused' stored.offset_secs 30.25
assert_json '3.3.put-paused' stored.first_yield_ms 500
assert_json '3.3.put-paused' stored.end_kind paused

run '3.3.get' 0 'capability,file,fingerprint,stored' \
  cargo run -q -p session -- --db "$DB" get "$FIX"
assert_json '3.3.get' stored.offset_secs 30.25
assert_json '3.3.get' stored.end_kind paused

run '3.3.offer-after-put' 0 'capability,file,fingerprint,offer' \
  cargo run -q -p session -- --db "$DB" offer show "$FIX"
assert_json '3.3.offer-after-put' offer.kind continue
assert_json '3.3.offer-after-put' offer.offset_secs 30.25
assert_json '3.3.offer-after-put' offer.first_yield_ms 500

run '3.3.expire-foreground' 0 'capability,file,fingerprint,foregrounded,outcome' \
  cargo run -q -p session -- --db "$DB" offer expire "$FIX" --foreground
assert_json '3.3.expire-foreground' foregrounded true
assert_json '3.3.expire-foreground' outcome.kind auto_start
assert_json '3.3.expire-foreground' outcome.offset_secs 30.25

run '3.3.expire-background' 0 'capability,file,fingerprint,foregrounded,outcome' \
  cargo run -q -p session -- --db "$DB" offer expire "$FIX"
assert_json '3.3.expire-background' foregrounded false
assert_json '3.3.expire-background' outcome.kind explicit_resume

# Chave = BLAKE3 dos bytes: editar 1 byte invalida o bookmark.
EDITED="$OUT/doc-edited.md"
cp "$FIX" "$EDITED"
printf 'X' >> "$EDITED"
run '3.3.ingest-after-1byte-edit' 0 'fingerprint,resume,offset' \
  cargo run -q -p session -- --db "$DB" ingest "$EDITED"
assert_json '3.3.ingest-after-1byte-edit' resume false
assert_json '3.3.ingest-after-1byte-edit' offset 0
if python3 -c 'import json,sys; a=json.load(open(sys.argv[1],encoding="utf-8"))["fingerprint"]; b=json.load(open(sys.argv[2],encoding="utf-8"))["fingerprint"]; sys.exit(0 if a!=b else 1)' "$OUT/3.3.ingest-new.out" "$OUT/3.3.ingest-after-1byte-edit.out"; then
  record_pass '3.3.ingest-after-1byte-edit@fingerprint-changed'
else
  record_fail '3.3.ingest-after-1byte-edit@fingerprint-changed' 'fingerprint nao mudou apos editar 1 byte'
fi

ERR_TOKEN='terminal session end stopped' run '3.3.put-stopped-rejected' 1 - \
  cargo run -q -p session -- --db "$DB" put "$FIX" --offset-secs 30.25 \
    --first-yield-ms 500 --end-kind stopped

ALLOW_STDERR=1 run '3.3.offer3s-test' 0 - \
  cargo test -q -p session --test test_offer3s
assert_tests_ran '3.3.offer3s-test'

nv '3.3 prazo de 3s e auto-start em foreground no dispositivo — device-only (CLI cobre o outcome)'

finish
