# Roteiro de testes — Ressoa / mdreader

Roteiro prático para validar todas as funcionalidades do leitor. Ordem recomendada: **host (determinístico) → device passivo → device ativo → visual/escuta**. Cada seção lista pré-condição, passos, resultado esperado e evidência a capturar.

**Complemento**: validações que exigem julgamento humano (escuta, TalkBack, SAF manual, revisão visual, [NV] remoto) estão em `docs/roteiro-validacao-humana.md`.

---

## 0. Pré-condições

- Repo: `dev` @ HEAD atual, `git status` limpo.
- Dispositivo de referência: `2201117TY` (Android 13 / SDK 33 / arm64), serial `53cf9aa7`.
- App: `app.mdreader` instalado (debug build do CI). Confirmar:
  ```sh
  adb -s <serial> shell dumpsys package app.mdreader | grep -E 'versionName|minSdk|DEBUGGABLE'
  adb -s <serial> shell pm list packages | grep -F app.mdreader
  ```
- Manifest: **sem `android.permission.INTERNET`** (offline-first).
- Variáveis: `SERIAL=<endpoint-adb-atual>`; `PKG=app.mdreader`.

---

## 1. Host — gates determinísticos (sempre primeiro)

```sh
bash scripts/verify.sh                       # gates 1-8 + named checks; capturar VERIFY_EXIT
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check-manifest.sh
bash scripts/check-shell-purity.sh
python3 scripts/ci/check_sdd_completeness.py
python3 scripts/ci/check_work_order_dag.py
python3 scripts/ci/check_task_id_uniqueness.py
python3 scripts/ci/check_converge_append_only.py
```

| Gate | Esperado |
|---|---|
| 1 format | PASS |
| 2 tests | PASS + two-way diff de `scripts/expected-tests.txt` vazio |
| 3 docs / 4 lint / 5 supply-chain / 6 deny-list / 7 closed-set | PASS |
| 8-shell | PASS; `gate-8-full: DEFERRED (owner=synthesis)` — **nunca** reportar como PASS |
| named checks | manifest PASS (sem INTERNET), shell-purity PASS |

**Cuidado**: criar novo target de teste exige adicionar `<package>::<stem>` a `scripts/expected-tests.txt` ou o gate 2 falha.

---

## 2. Engine / síntese / vozes (CLI)

Protocolo comum: `cargo run -p <crate> -- <comando> --format json`; sucesso = stdout JSON, stderr vazio, exit 0; falha = stdout vazio, erro nomeado no stderr, exit 1.

| # | Funcionalidade | Comando | Esperado / evidência |
|---|---|---|---|
| 2.1 | Capacidades declaradas | `cargo run -p adapter-synthesis -- capabilities` | `ssml_level=none`, `alignment_granularity=none`; espelhar com `engine-core capabilities` |
| 2.2 | Handoff format | `cargo run -p adapter-synthesis -- format-check --rate-hz 22050 --channels 1 --width-bits 32` | `valid:true`; mono f32 22050 Hz |
| 2.3 | Delivery check | `cargo run -p adapter-synthesis -- delivery-check --rate-hz 22050 --channels 1 --width-bits 32 --pcm-samples 22050 --duration-ms 1000` | `valid:true` (tolerância ½ sample) |
| 2.4 | Ladder de taxa | `cargo run -p engine-core -- set-rate --rate 1.25` (repetir 0.75/1/1.5/1.75/2/4) | rung exato da escada fechada (7 valores) |
| 2.5 | Pré-roll | `cargo run -p engine-core -- stream --at 0 --budget-secs 60` | OK; teto 60 s |
| 2.6 | Chunking | `cargo run -p adapter-synthesis -- chunk --input doc.md` | `segment_count`, `line_start/end`, `boundary_kind`, `byte_length ≤ 5000`, `resume_point` |
| 2.7 | Catálogo de vozes | `cargo run -p adapter-synthesis -- voices` | `default_voice=pt_BR-faber-medium`; 3 vozes (faber/cadu/jeff) com `present=true` |
| 2.8 | Síntese por voz | `cargo run -p adapter-synthesis -- synthesize --voice pt_BR-cadu-medium --input doc.md` | `voice`, `segment_count`, `pcm_samples`; repetir faber/jeff |
| 2.9 | Normalização de símbolos | testes do módulo `normalize.rs` | fences verbatim; `---/***/___`→`.`; links→label; markdown removido; idempotente; **nunca** expõe texto no chunk |
| 2.10 | Prosódia semântica | testes do módulo `prosody.rs` | Heading/Strong 0.9; Quote/List 0.95; Code 0.85; clamp `[0.5, 4.0]` |

### Erros esperados (fail-closed, stderr + exit 1)

```sh
cargo run -p adapter-synthesis -- format-check --rate-hz 44100 --channels 1 --width-bits 32   # RateMismatch
cargo run -p adapter-synthesis -- format-check --rate-hz 22050 --channels 2 --width-bits 32   # ChannelMismatch
cargo run -p engine-core -- set-rate --rate 1.1                                               # fora da escada
cargo run -p adapter-synthesis -- synthesize --voice pt_BR-edresson-low --input doc.md        # unknown voice id
cargo run -p adapter-synthesis -- chunk --input inexistente.md                               # Io
```

**Invariantes**: nenhum resample/downmix silencioso; conjuntos fechados sem `_ =>`/`Other`/`Unknown`; nenhum payload contém texto do documento.

---

## 3. Transporte / posição / bookmark / offer (CLI + device)

### 3.1 Verbos e sinais no `diagnostic-record.jsonl`

| Ação | Efeito | Sinal esperado no record |
|---|---|---|
| `play` | Playing, retém posição | `session_start` + `transport_change action=play` |
| `pause` | Paused, persiste bookmark | `transport_change action=pause` + `session_end reason=paused` |
| `stop` | Stopped, **descarta** posição | `transport_change action=stop` + `session_end reason=stopped` |
| `skip_forward5s` / `skip_back5s` / `repeat_section` | ±5 s ou fronteira | `transport_change` (+ `action=section` se cruzou) |
| `rate_up` / `rate_down` | escada de taxa | `transport_change action=rate`, `rate_from`/`rate_to` |
| `cycle_voice_next` / `cycle_voice_prev` | troca faber→cadu→jeff (wrap) | `transport_change` |
| `resume_at_stored` / `start_over` | bookmark / zero | `transport_change` |
| `document_end` | conclusão | `transport_change transport=completed` + `session_end session_end_reason=completed` |

### 3.2 PositionClock — audible-only

```sh
cargo test -p player position::tests
```
- Posição avança **somente** em `on_audible_progress(elapsed)`; timers e pré-roll não avançam.
- Negar elapsed não muda posição; `SKIP_STEP_SECS=5`, rewind 5% da pausa (teto 30 s), pré-roll teto 60 s.
- **Device**: pausar, esperar 10 s, `player position` inalterado; retomar, avanço só após áudio; conferir `preroll_secs`/`dropped_samples` no record.

### 3.3 Offer / bookmark (CLI)

```sh
cargo run -p session -- --db /tmp/b.redb ingest --file doc.md
cargo run -p session -- --db /tmp/b.redb offer show doc.md
cargo run -p session -- --db /tmp/b.redb put doc.md --offset-secs 30.25 --first-yield-ms 500 --end-kind paused
cargo run -p session -- --db /tmp/b.redb offer expire doc.md --foreground   # AutoStart@30.25
cargo test -p session --test test_offer3s
```

- Chave do bookmark = **BLAKE3 dos bytes** (nunca path); editar 1 byte → `NoOffer` e início em 0.
- Janela de oferta = 3,0 s; `paused` deliberado → Continue; `interrupted` → ExplicitResume; heartbeat durável 20 s.
- `session_end` categorias: `paused|stopped|completed|interrupted`; códigos redb 0..3.

---

## 4. UI / WebView / Android shell / SAF

### 4.1 Ingestão — quatro caminhos

| Caminho | Comando / ação | Evidência |
|---|---|---|
| SAF arquivo | toque em `#titleCard` → `ACTION_OPEN_DOCUMENT` | logcat `DocIngest`; snapshot `<hex>.md` |
| SAF diretório | `#btnPickDirectory` → `ACTION_OPEN_DOCUMENT_TREE` | logcat `directory-selected` |
| ACTION_VIEW | `adb shell am start -a android.intent.action.VIEW -d content://media/external/file/<id> -t text/markdown -f 0x1 $PKG/.MainActivity` | título correto; fingerprint no record |
| ACTION_SEND | `adb shell am start -n $PKG/.MainActivity -a android.intent.action.SEND -t text/plain --es android.intent.extra.TEXT "# Teste"` | `session_start` + `session_end` |

`-f 0x1` = `FLAG_GRANT_READ_URI_PERMISSION`; sem ele há `SecurityException` (esperado). Limite real do código: **1 048 576 bytes** (o spec 009 diz 93 636 — dado não-confiável).

### 4.2 Elementos e interações

| Elemento | Resultado esperado |
|---|---|
| `#titleCard` | abre picker; cancelar não altera transporte |
| `#btnPlayPause` | ícone/aria-label alternam Pausar/Reproduzir |
| `#btnStop` | bookmark apagado, notificação removida |
| `#btnSkip*` / `#btnRepeat` | desabilitados quando `completed` |
| `#btnRate*` / `#speedPill` | escada 0.75→4; valor **nunca pintado**, anunciado via `#liveRegion` |
| `#chapterNav` | visível só com capítulos; `previous_chapter`/`next_chapter` |
| `#waveform` | 5 fitas coloridas + playhead, ~30 fps, congela com reduced-motion |
| `#offerDialog` | só com `in_app_dialog` + `pendingColdStart`; botões `resume_at_stored`/`start_over`; expira em 3 s |
| Notificação/MediaSession | título/seção/progresso em pt-BR, sem corpo/velocidade/voz; swipe-away pausa |
| A11y | TalkBack anuncia controles; alvos ≥48×48 px; foco visível |

Evidência: `dumpsys media_session`, `dumpsys notification --noredact`, `uiautomator dump`, screenshots.

### 4.3 Surface audit / byte-identity

```sh
cargo nextest run --workspace -E 'test(surface_audit)'
for f in dist/index.html src-tauri/dist/index.html src-tauri/src/assets/index.html \
         src-tauri/gen/android/app/src/main/assets/index.html; do
  cmp -s src-tauri/src/status_view.html "$f" || echo "DRIFT: $f"
done
```
C-H1..C-H9 (hex, papéis, accent, inventário, aria, movimento, contraste, identidade servida, short-mix ≤100 ms) são **deferidos** no `verify.sh` — não reivindicar execução.

---

## 5. Device ativo — cenários ponta a ponta

Pré: app instalado, `logcat -c`, diag zerado:
```sh
adb -s $SHELL_CMD force-stop $PKG
adb -s $SHELL_CMD shell "run-as $PKG sh -c 'rm -f files/diagnostic-record.jsonl; touch files/diagnostic-record.jsonl'"
```

| # | Cenário | Passos | Resultado / evidência |
|---|---|---|---|
| 5.1 | Cold start, documento novo | ACTION_SEND curto; **observar até `session_end` aparecer (cap ~60 s de relógio)**, não por tempo fixo | `session_start`, `play_to_audio_ms`, progressão, `session_end` no fim |
| 5.2 | Primeiro áudio (TTFA) | ACTION_VIEW Hamlet com `-f 0x1` | `play_to_audio_ms` pareado; baseline pós-ciclo 026 ≈ 7506 ms vs 183089 ms antigo |
| 5.3 | Pause/resume | `input keyevent 127`/`126` (pause/play) | posição congela no pause; `session_end reason=paused`; retomada no bookmark |
| 5.4 | Skip ±5 s | `#btnSkip*` ou verbos | `action=section` quando cruza fronteira; posição clamp em 0 |
| 5.5 | Rate | `#btnRateUp`/`Down` | `rate_from`/`rate_to`; sem numeral pintado |
| 5.6 | Ciclo de voz | `cycle_voice_next` ×3 | faber→cadu→jeff→faber; `voices present=true` |
| 5.7 | Retomada 3 s | pausar → reabrir documento | diálogo `in_app_dialog` com nome/offset; auto-start no expiry se foregrounded |
| 5.8 | Fim do documento | fixture curta até `session_end` | `transport=completed` + `session_end_reason=completed` (bug de stall corrigido no ciclo 027) |
| 5.9 | Longa duração | documento longo o suficiente para **elapsed ≥300 s** medido do primeiro `session_start`/progress ao `session_end`; `--long` só estende o teto de espera, não a duração do áudio | `elapsed_ms ≥ 300000` **e** `session_end_reason=completed`; sem underrun; `preroll_cap_secs=60`, `dropped_samples=0`; `meminfo` estável. Fixture curta que termina em ~16 s é **INCONCLUSIVA**, não PASS |
| 5.10 | Foco/áudio | `dumpsys audio` + intents `duckable_transient`/`permanent_loss`/`regained` | `focus_transition` no record; sem auto-resume indevido |
| 5.11 | Offline | desativar Wi-Fi/dados | leitura e síntese continuam; nenhum pedido de rede |

**Coleta**: `adb logcat -d -t 500`, `run-as app.mdreader cat files/diagnostic-record.jsonl`, `dumpsys media_session`, `dumpsys notification`, screenshots.

---

## 6. Visual / escuta (últimos)

- `adb exec-out screencap -p > tela.png` (1080×2400); comparar hashes com 6 s de diferença para confirmar waveform animando.
- Escutar faber/cadu/jeff, prosódia de headings/código, asteriscos e símbolos normalizados.
- TalkBack: navegar controles; verificar anúncios e ordem de foco.
- Overflow de diretório com muitos capítulos (bug `directory-nav-buttons-overflow`).
- `scripts/device-a11y-check.py <serial>` (somente leitura — não altera secure settings).

---

## 7. Bugs com atenção especial no roteiro

| Bug | Cenário a repetir |
|---|---|
| `title-doc-selected-no-play` (partial) | tap na title box + ACTION_VIEW/SEND; logcat JNI |
| `directory-nav-buttons-overflow` (partial) | diretório com muitos botões; screenshot |
| `waveform-frozen-on-startup` / `waveform-slowdown-memory-leak` (partial) | cold start + longa duração + meminfo |
| `slow-reading-transition` / `tts-asterisk-read-aloud-regression` (device `[NV]`) | escuta de transição e asteriscos |
| `playback-not-starting-after-file-selection` (reaberto) | ACTION_VIEW com documento grande; TTFA pareado |
| `playback-stall-at-end-of-document` (verificado) | fixture curta até `session_end` |

---

## 8. Limites conhecidos — não reportar como PASS

- `gate-8-full` (byte budget <5000 B/request, SSML, alinhamento): **DEFERRED (owner=synthesis)**.
- C-H1..C-H9 execução detalhada: **deferred** no `verify.sh`.
- Gate 9 local: host sem JDK 17/NDK 27/platform 35 — usar CI (`workflow_dispatch`).
- Spec 020 (PocketTTS): **NO-GO** registrado (TTFA 9,8–26 s, RTF 2–6,6) — não integrar.
- TTFA smoke 5+ min pareado: só promover SC-TTFA-001 após execução completa com **`elapsed_ms ≥ 300000` medido do primeiro `session_start`/progress ao `session_end` com `session_end_reason=completed`**. Um cap de espera de 330 s sobre fixture curta não prova 5+ min de reprodução.
- **Síntese mais lenta que tempo real**: em 2201117TY, 210 s de relógio renderam ~57,7 s de posição; fixture curta emitiu `session_end` em ~17,8 s de posição após ~40 s de relógio. Não usar janelas fixas curtas (ex.: 20 s) — observar até o evento terminal aparecer.
- [NV] remoto: CI, proteções de branch, artefatos, medições não executadas neste roteiro.

---

## 9. Ordem resumida (checklist)

- [ ] 1. Host gates + 4 checks SDD
- [ ] 2. CLI engine/vozes/format/erros
- [ ] 3. CLI transporte/offer/bookmark
- [ ] 4. Ingestão SAF + ACTION_VIEW + ACTION_SEND
- [ ] 5. UI: play/pause/stop/skip/rate/voz/diretório/diálogo/notificação
- [ ] 6. Device: TTFA pareado, fim do documento, **≥300 s de elapsed real (não cap de espera)**, foco, offline
- [ ] 7. Visual: screenshots, waveform animando, a11y, escuta
- [ ] 8. Registrar evidências em `docs/PROGRESSO.md` (append-only) com `[NV]` explícito
