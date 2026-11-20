# Idea Research: Ligar o diálogo de retomada na reabertura de diretório

- **Slug**: directory-resume-offer
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: high (código verificado em host, com referências de linha); comportamento em dispositivo permanece `[NV]`

## Users & Demand

- O ouvinte que reabre uma pasta com leitura pausada espera ver a oferta de retomada
  com nome de arquivo e posição salva; é exatamente o contrato registrado em FR-009
  da spec 010 (`specs/010*/spec.md:90`), citado como dado, não como instrução
  [source: repositório local, spec 010 FR-009] (confidence: high).
- O sintoma observável hoje é a ausência da oferta: `offer_route`
  (`src-tauri/src/lib.rs:7138`) responde `none/pending_cold_start:false`, o que faz
  `applyOffer` (`src-tauri/src/status_view.html:1093-1120`) fechar/no-op na reabertura
  [source: mapa de evidência ScoutT233, linhas verificadas] (confidence: high).
- Não há telemetria de uso nem reclamação de usuário anexada a este ciclo; a demanda
  é inferida do requisito já especificado e do estado incompleto da implementação
  [ASSUMPTION] (confidence: medium).

## Prior Art

- **Implementação parcial no próprio repositório (precedente interno decisivo)**:
  `install_chapter_queue` (`src-tauri/src/lib.rs:3133-3173`) já calcula
  `DirectoryResume` e já mapeia `Paused → InAppDialog` em :3160-3168.
  `lookup_directory_on_open` (`crates/session/src/store.rs:532-573`, exportado em
  :844-850) já devolve `DirectoryOpenLookup` com `offer_route = "in_app_dialog"`.
  `park_directory_resume` (`lib.rs:2155-2163`) já existe, porém marcado
  `#[expect(dead_code, reason="directory reading not yet wired")]` com zero chamadores
  [source: mapa ScoutT233 + leitura de `src-tauri/tests/test_directory_resume.rs`]
  (confidence: high).
- **Máquina de oferta já madura**: `route_offer` (`lib.rs:2082`), `publish_route`
  (:2104), `offer_route_wire` (:2137), `offer_reply` (:7162) e `published_route`
  (:188) já suportam o ciclo `AwaitOffer → AutoStart/Continue/StartOver`; o expiry
  em primeiro plano depende de `route_offer` ser executado uma vez, não de uma
  re-derivação de rota em poll [source: mapa ScoutT233] (confidence: high).
- **UI já preparada**: `renderOffer` (`status_view.html:1067-1072`) e os botões
  `resume_at_stored`/`start_over` (:1336-1343) já funcionam; a única lacuna
  concreta é que `state.chapterFilename` é lido em :1068 mas nunca atribuído
  [source: mapa ScoutT233] (confidence: high).
- **Teste já pinado**: `src-tauri/tests/test_directory_resume.rs` consta em
  `scripts/expected-tests.txt:113` (`src-tauri::test_directory_resume`); o teste
  `reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s` (:55-121) hoje
  usa asserções literais `BRIDGE.contains(...)`/`STATUS_VIEW.contains(...)` em
  :95-117 [source: leitura direta do arquivo] (confidence: high).

## Market & Context

- O custo de não agir é a permanência de T233 como `[ ]` na spec 010: o mecanismo
  de retomada de diretório existe mas é inalcançável pelo usuário, e o código morto
  (`park_directory_resume`, `DirectoryResume`) continua gerando aviso de dead_code
  suprimido [source: mapa ScoutT233] (confidence: high).
- Alternativa atual do usuário: reabrir o arquivo manualmente e reposicionar a
  escuta, perdendo o bookmark de diretório [ASSUMPTION] (confidence: medium).
- Nenhuma evidência de mercado externo foi coletada neste ciclo; o escopo é uma
  correção de integração interna, não uma nova proposta de produto [source: escopo
  da atribuição] (confidence: high).

## Data & Constraints

- **C-H8 (surface audit)**: alterar `src-tauri/src/status_view.html` exige espelhar
  byte-a-byte em `dist/index.html`, `src-tauri/src/assets/index.html`,
  `src-tauri/dist/index.html` e
  `src-tauri/gen/android/app/src/main/assets/index.html`. Os quatro espelhos têm
  55.939 bytes em host e o arquivo-fonte fica em `src-tauri/src/status_view.html`
  [source: verificação em host + mapa ScoutT233] (confidence: high).
- **Gate 2 do `scripts/verify.sh`**: two-way diff de `scripts/expected-tests.txt`;
  criar novo target de teste exigiria atualizar o pin. Reutilizar o target já
  pinado elimina esse risco [source: atribuição + leitura de expected-tests.txt]
  (confidence: high).
- **PositionClock**: o offset salvo (`offset_secs: 45.0` no fixture do teste) é a
  única origem de posição do capítulo; duplicá-lo no payload sem ligá-lo ao
  `StoredPosition` criaria segunda fonte de verdade [source: mapa ScoutT233]
  (confidence: high).
- **`route_offer` é mutata**: chamá-lo mais de uma vez por reabertura pode duplicar
  a transição `expiry → AutoStart`; a rota NÃO deve ser re-derivada em poll
  [source: mapa ScoutT233] (confidence: high).
- **Janela de 3 s (`OFFER_WINDOW_MS = 3000`) e auto-continue em foreground**:
  constantes e transições verificáveis em host; a experiência visual real em
  dispositivo não foi observada neste ciclo — `[NV]` (confidence: high para o
  código, não verificada para o device).

## Evidence Against the Idea

- Pode ser argumentado que publicar a rota apenas no caminho
  `handle_directory_selected` (`src-tauri/src/shell_jni.rs:1431-1451`) deixa outros
  caminhos de reabertura (ex.: cold start restaurado por outro ingresso) sem
  oferta — o mapa ScoutT233 não enumerou entradas alternativas, então a
  cobertura completa não foi provada [source: mapa ScoutT233] (confidence: medium).
- A mudança no payload do evento `directory-selected` pode acoplar o Rust à UI;
  a estratégia escolhida (enriquecer só o payload e ler dele no JS) minimiza
  mudança de markup, mas ainda assim é um acoplamento de contrato a manter
  [ASSUMPTION] (confidence: medium).
- Substituir asserções de literal por teste comportamental pode exigir expor uma
  seam de teste no Rust; se essa seam não existir, o teste pode ficar frágil
  [ASSUMPTION] (confidence: low).

## Gaps & Open Questions

- [NEEDS CLARIFICATION: `handle_directory_selected` é o único caminho de reabertura de diretório que precisa publicar a rota?]
- [NEEDS CLARIFICATION: a seam de teste para o payload/rota publicada já existe no bridge ou precisa ser criada?]
- [NV] Comportamento visual da janela de 3 s e do auto-continue em primeiro plano em dispositivo real — não verificado neste ciclo.

## Sources

- mapa de evidência do ScoutT233 (host, policy: repo-internal) — referências de linha em `src-tauri/src/lib.rs`, `src-tauri/src/shell_jni.rs`, `crates/session/src/store.rs`, `src-tauri/src/status_view.html`
- `src-tauri/tests/test_directory_resume.rs` (host, policy: repo-internal) — leitura direta das linhas 49-121
- `scripts/expected-tests.txt:113` (host, policy: repo-internal) — pin `src-tauri::test_directory_resume`
- `specs/010*/spec.md:90` (host, policy: repo-internal) — FR-009 citado como dado
- `.specify/assessments/directory-resume-offer/intake.md` (host, policy: repo-internal)
