# Roteiro de validações humanas — Ressoa / mdreader

Roteiro complementar a `docs/roteiro-testes.md` (que cobre o que é automatizável). Aqui: apenas julgamento humano e confirmação de estado vivo.

**Regra de ouro**: ADB prepara fixtures; **o julgamento é humano**. Toda afirmação não executada aqui permanece `[NV]`. Não alterar `secure settings` (inclusive TalkBack) por ADB — a ativação é manual pelo usuário na tela. Não editar `src-tauri/gen/**` diretamente.

**Dispositivo**: `2201117TY` (Android 13), endpoint `192.168.100.102:5555`, `PKG=app.mdreader`. Volume médio, ambiente silencioso, carregador conectado.

---

## 1. Escuta — vozes, prosódia, símbolos, transição (V1)

**Tempo**: 12–15 min. **Pré**: app instalado, `adb logcat -c`.

Fixture `prosodia.md` (≥3 parágrafos por estrutura): `# Título`; `## Subtítulo`; `**negrito**`, `_ênfase_`, `__strong__`, `==grifo==`; `1. item`, `- item`, `> citação`; tabela `| *a* | **b** |`; link `[rótulo](https://x/a*b)`; `https://x.com/a*b`; `2*3`; fence `~~~` com `**x**`; fence ``` com `int *p`; `---` e `***`.

```sh
adb -s $S shell mkdir -p /sdcard/Download/mdreader-test-fixtures
adb -s $S push prosodia.md /sdcard/Download/mdreader-test-fixtures/prosodia.md
```

**Passos**
1. Ingerir via ACTION_SEND e reproduzir até ouvir todas as estruturas. Síntese é mais lenta que tempo real — observar até `session_end`, não janela fixa.
2. Alternar vozes pelos botões da **notificação** (“Voz seguinte”/“Voz anterior” — não há botão de voz na tela): faber (padrão) → cadu → jeff → faber.
3. Ouvir heading (≈0,9×), citação/lista (≈0,95×), código (≈0,85×) vs. texto corrido.
4. Ouvir asteriscos fora de zonas protegidas.
5. Confirmar `---`/`***`/`___` → “ponto”, links → rótulo, fences verbatim.
6. Repetir com diretório de 3 capítulos e cronometrar troca capítulo→capítulo.

**Esperado**: timbres distinguíveis e estáveis no wrap; variação de velocidade perceptível; nenhum “asterisco, asterisco” em texto/tabela comum; `*` **pode** ser falado dentro de código/URL/matemática (contrato 014 — não é falha); sem pausa morta >~1 s na troca.

**Critério explícito**: escala 1–5 por voz (1=irritante/ininteligível, 3=aceitável, 5=natural). Aprovar se ≥3 em cada voz, nenhum asterisco em texto comum, transição sem soluço.

**Evidência**: áudio gravado com carimbo de tempo, anotação por estrutura, `run-as app.mdreader cat /data/data/app.mdreader/diagnostic-record.jsonl`.

**Se falhar**: manter `[NV]` nos bugs `tts-asterisk-read-aloud-regression` e `slow-reading-transition`; registrar trecho exato, voz e capítulo; abrir bug. Não alegar melhora perceptiva sem gravação.

---

## 2. TalkBack — anúncios e ordem de foco (V2)

**Tempo**: 8–10 min. **Pré**: abrir Configurações e **ativar TalkBack manualmente**; app com documento e depois com diretório.

```sh
adb -s $S shell am start -a android.settings.ACCESSIBILITY_SETTINGS
```

**Passos**: 1) Deslizar para a direita percorrendo o foco. 2) Ativar com duplo toque em cada controle. 3) Observar `#liveRegion`. 4) Repetir com diálogo de oferta aberto.

**Ordem esperada** (`src-tauri/src/status_view.html:469-504`):
`#titleCard` (“Selecionar documento para leitura”) → “Repetir seção” → “Retroceder” → “Pausar”↔“Reproduzir” → **“Parar e limpar posição”** (botão oculto por CSS, provavelmente ainda focalizável — verificar) → “Avançar” → “Diminuir velocidade” → “Ajustar velocidade” → “Aumentar velocidade” → “Selecionar diretório” → (modo diretório) “Capítulo anterior”/“Próximo capítulo”.

Canvas `#waveform` e ondas decorativas `aria-hidden` **não** devem ser anunciados. `progressbar` anuncia “carregando”/“progresso N por cento”. Mudança de capítulo anuncia “Capítulo X de Y”. Recusa anuncia “Arquivo ignorado: <nome> não pode ser lido”. Anel de foco visível; sem armadilha; alvos ≥48dp (≤420px pode cair para 44dp — observar).

**Atenção**: `#rateValue` traz `1.25×` sancionado pela spec 007 — adjudicar a tensão contrato/código em vez de assumir falha.

**Evidência**: vídeo com áudio do TalkBack, `uiautomator dump`, screenshot do anel de foco, transcrição da ordem real.

**Se falhar**: anotar rótulo ausente e ordem real; comparar com `status_view.html:469-504`; registrar bug de a11y sem editar a superfície.

---

## 3. Overflow de diretório com muitos capítulos (V3)

**Tempo**: 6–8 min. **Fixture**: 31 capítulos em raiz + subpasta.

```sh
adb -s $S shell mkdir -p /sdcard/Download/mdreader-test-fixtures/md-dir/vol2
for i in $(seq -w 1 30); do adb -s $S shell "printf '# Capitulo $i\n\nTexto.\n' > /sdcard/Download/mdreader-test-fixtures/md-dir/cap$i.md"; done
adb -s $S shell "printf '# Capitulo 31\n\nTexto.\n' > /sdcard/Download/mdreader-test-fixtures/md-dir/vol2/cap31.md"
```

**Passos**: 1) Selecionar diretório pelo SAF (§4). 2) Confirmar `#chapterNav` visível e 31 capítulos. 3) Repetir em 360/390/412dp (`adb shell wm size 360x800`; **restaurar ao final**). 4) Tocar “Capítulo anterior”/“Próximo capítulo” e navegar com TalkBack. 5) Abrir arquivo único e confirmar que os botões desaparecem sem regressão.

**Esperado**: botões dentro do card e da viewport, sem corte/scroll horizontal, ≥44dp, foco visível, rótulos corretos.

**Evidência**: screenshots por largura, `uiautomator dump`, saída de `wm size`.

**Se falhar**: manter `partial` no bug `directory-nav-buttons-overflow`; anexar largura exata e screenshot. Não promover a `verified`.

---

## 4. SAF picker manual — arquivo e diretório (V4)

**Tempo**: 6–8 min. **Pré**: fixtures presentes; `adb logcat -c`; acompanhar `logcat -s DocIngest ShellHost`.

**Passos**: 1) Tocar `#titleCard` → `ACTION_OPEN_DOCUMENT` abre no primeiro toque (debounce 500 ms). 2) Cancelar → transporte/título inalterados. 3) Selecionar `prosodia.md` → snapshot `<hex>.md`, título e leitura iniciam. 4) Tocar `#btnPickDirectory` → `ACTION_OPEN_DOCUMENT_TREE`; selecionar `md-dir` → log `directory-selected`, 31 capítulos. 5) Selecionar outro diretório (grant anterior liberado) e cancelar sem mudança. 6) Opcional: escolher `.png` e arquivo >1 048 576 B → recusa com toast de limite, sem snapshot.

**Esperado**: sem `SecurityException`; cancelamento é no-op; logs `DocIngest` nomeados; exatamente um snapshot de documento avulso.

**Evidência**: logcat recortado, `run-as app.mdreader ls files/snapshots`, screenshots do picker e resultado, `diagnostic-record.jsonl`.

**Se falhar**: verificar `ShellHost.currentActivity` no logcat; registrar sob `title-doc-selected-no-play` mantendo `[NV]`.

---

## 5. C-H1..C-H9 — execução manual deferida (V5)

**Tempo**: ~65 min total. Contract: `specs/005-adopt-design-ressoa/contracts/surface-audit.md:20-29`. `verify.sh` **não** executa esta etapa — não reivindicar PASS automatizado.

| ID | Passo humano | Esperado | min |
|---|---|---|---|
| C-H1 hex census | Inspecionar `:root` em `status_view.html:463`; conferir cada hex pintado vs ledger | 11 hex fechados, nenhum órfão/legado | 8 |
| C-H2 papéis | Tab pelos controles; screenshot foco e repouso | `#8FD8E8` só em `:focus-visible`; `#FFD400` só banner/ponto; `#FF4D63` só falha | 6 |
| C-H3 accent-only | Iniciar leitura; `dumpsys notification --noredact`; shade/lock | só `#F2F0E6` pintado; sem logo/onda/voz/velocidade/corpo | 5 |
| C-H4 inventário | Cold start sem doc e com doc | 9 elementos, `#waveform` visível mesmo vazio, `#headerMark` concha-r | 5 |
| C-H5 aria | TalkBack ON; percorrer 6 controles; pedir velocidade | rótulos 10/10, `progressbar`+valuetext, rate nunca pintado fora `#rateValue` (adjudicar 007) | 12 |
| C-H6 movimento | Screenshots a 6 s; ligar “Remover animações” | só pill loopa; reduce congela **visível** (não `opacity:0`); amarelo ≤3 s | 10 |
| C-H7 contraste | Amostrar pares texto/fundo e bordas | texto ≥4,5:1; não-texto ≥3:1; legível à vista | 6 |
| C-H8 identidade servida | `cmp -s src-tauri/src/status_view.html <cada bundle>`; abrir app | zero DRIFT; UI mostra superfície restilizada | 5 |
| C-H9 short-mix ≤100 ms | Captura de mic **etiquetada por ação**; cronometrar ação→1º sample; rajada 2–3 ações | cada ≤100 ms; “latest wins”; sem cauda FIFO | 15 |

**Evidência**: screenshot por check + tabela de resultados + notas de adjudicação. **Se falhar**: abrir bug de deriva com arquivo:linha; não alterar gate.

**Nota C-H9**: capturas anteriores via `parec` tiveram peak 32767 (clip) e ZCR 0,93 — **não** provam voz. Gravador externo com ganho reduzido e nome de arquivo por ação.

---

## 6. Revisão visual dos diagramas Archify (V6)

**Tempo**: 15 min. Artefato real: `.archify/architecture-ressoa-20261116-190400/ressoa.html` (browser-check PASS, `visualReview: "not-requested"`).

**Passos**: 1) Abrir `file://…/ressoa.html`. 2) Inspecionar 1440×900, 1600×1000, 1920×1080, 2048×1320 em light + dark (1440/2048). 3) Verificar zoom, dock/legenda sem sobreposição, texto ≥8 px projetado, sem scroll/overflow. 4) **Traçar a rota `tauri_bridge → diag_log`** (“append JSONL”) — `finalize-summary.json` sinaliza 3 bends/stretch 1.086. 5) Conferir 9 componentes e 2 cards vs `candidate.json`.

**Esperado**: legível, sem cruzamentos confusos, rota compreensível.

**Evidência**: screenshot por viewport/tema + anotação da rota + sha256 do HTML.

**Se falhar**: abrir ticket de layout com screenshot; reflow no candidato + `finalize` novo (fora desta rodada).

**Armadilha**: `docs/architecture.md:9-10` aponta para `.archify/architecture-ressoa-system-20261110-160117/ressoa-system.html`, que **não existe** no tree — link stale. Corrigir referência ou localizar artefato.

---

## 7. [NV] remotos — confirmação na UI do GitHub (V7)

**Tempo**: ~18 min. Todos abaixo permanecem `[NV]` até inspeção ao vivo. **Não confie em IDs, números de PR, hashes ou resultados registrados em `STATE.md` ou neste documento — STATE é snapshot.** Obtenha o estado real via UI do GitHub ou `gh` no momento da validação.

| Item | Passo ( estado vivo ) | Política esperada | min |
|---|---|---|---|
| CI checks | `gh run list --branch dev --limit 5` e `gh pr checks <PR aberto/recente>`; conferir o HEAD **atual** de `dev` | Host gates + Android debug success no HEAD atual; sem aggregate `pending` divergente | 3 |
| Ruleset | Settings → Branches/Rulesets: **identifique o ruleset de release atual** — não procure um ID fixo | contexts Host+Android exigidos em tags `v*`; `dev` sem aprovação; `main` 1 aprovador + CODEOWNER | 4 |
| Aprovação distinta | `gh pr list --state all --limit 10` e abrir cada PR recente na UI; comparar aprovadores com autores | em `main` aprovador ≠ autor; em `dev` zero aprovações permitido após checks | 5 |
| Merge/review + release | `git log --oneline -10` + `gh run list` + aba Actions na UI; checar merges recentes e um `workflow_dispatch` | merges intencionais via merge button, ordem respeitada; **conteúdo/hash dos APKs permanece [NV]** | 6 |

**Evidência**: screenshots da aba Checks, regras, reviewers, merge commit e artefatos.

**Se falhar**: registrar check vermelho/pending e parar a premissa de verde; abrir bug de governança; não afirmar proteção efetiva sem UI.

---

## 8. Pré-requisitos / harness-gaps (decisão do SE) — não são validações humanas



### 5.2 — TTFA via ACTION_VIEW
- **Falta**: ID MediaStore vivo. `content://media/external/file/1000848919` está stale.
- **Gerar**:
  ```sh
  adb -s $S shell 'printf "# Hamlet trecho\n\nTexto de validacao.\n" > /sdcard/Download/hamlet-ttfa.md'
  adb -s $S shell content call --uri content://media --method scan_file --arg /sdcard/Download/hamlet-ttfa.md
  adb -s $S shell content query --uri content://media/external/file --projection _id:_display_name --where "_display_name='hamlet-ttfa.md'"
  ```
- **Executar**: trocar `VIEW_URI` no harness para o `_id` retornado; rodar com cap 90. `-f 0x1` é obrigatório.
- **Aceite**: `session_start` + `play_to_audio_ms` + `session_end` no diag; TTFA pareado vs 7506/183089 ms. Classe permanece **PARCIAL** (escuta [NV]).
- **Risco**: baixo; se `am_start_rc≠0`, reconconsultar `_id`.

### 5.11 — Offline
- **Falta**: marker de readiness provando que Wi-Fi/dados caíram **antes** do trigger.
- **Gerar (device-side, uma única chamada)**:
  ```sh
  adb -s $S shell 'nohup sh -c "svc wifi disable; svc data disable; echo OFFLINE_\$(date +%s)_wifi=\$(settings get global wifi_on) > /data/local/tmp/mdreader-offline.ready; sleep 330; svc wifi enable; svc data enable; echo RESTORED_\$(date +%s) >> /data/local/tmp/mdreader-offline.ready" >/dev/null 2>&1 &'
  ```
- **Aceite**: marker `wifi_on=0` cobrindo `session_start→session_end`; `INTERNET_STATUS=absent`; logcat sem `UnknownHost|SocketException`; `wifi_on=1` ao final. Continuidade de leitura/síntese continua **HUMANO/[NV]**.
- **Risco alto**: ADB é TCP/Wi-Fi. Manter USB como endpoint reserva, janela curta, acesso físico para religar Wi-Fi. **Nunca rodar sem supervisão.**

### 5.9 — Longa duração
- **Falta**: documento com **elapsed ≥300 s real**; `--long` só eleva o teto de espera.
- **Gerar**: arquivo ~12–15 KB de texto repetido (≤1 048 576 B), scan + query `_id`, ingerir via VIEW.
- **Executar**: `harness-device.sh --long --cap-secs 1200`; observar até `session_end`.
- **Aceite**: `ELAPSED_MS ≥ 300000` **e** `session_end_reason=completed` **e** `preroll_cap_secs=60` **e** `dropped_samples=0` **e** meminfo não vazio. Síntese lenta: 210 s relógio → ~57 s de posição; reservar 15–25 min.
- **Risco**: MIUI/thermal/bateria podem causar `reason=stopped`; manter carregador e tela ligada.

---

## 9. Ordem resumida (checklist)

- [ ] V1 — escuta (12–15 min)
- [ ] V2 — TalkBack (8–10 min)
- [ ] V3 — overflow diretório (6–8 min)
- [ ] V4 — SAF picker (6–8 min)
- [ ] V5 — C-H1..C-H9 (~65 min)
- [ ] V6 — revisão visual Archify (15 min)
- [ ] V7 — [NV] remoto GitHub (~18 min)
- [ ] Pré-requisitos §8 (decisão do SE, **não** validação humana): 5.2 ID MediaStore vivo → 5.11 marker offline supervisionado → 5.9 documento >5 min
- [ ] Registrar resultados em `docs/PROGRESSO.md` (append-only) com `[NV]` explícito
