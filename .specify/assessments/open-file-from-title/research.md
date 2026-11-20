# Idea Research: Abrir arquivo pelo botão de título e iniciar reprodução

- **Slug**: open-file-from-title
- **Created**: 2026-11-02
- **Evidence confidence (overall)**: high

## Users & Demand

- **Redução de atrito no carregamento de novos documentos**: Em aplicativos utilitários e de áudio-leitura (como Voice Dream Reader, @Voice Aloud Reader e Moon+ Reader), os usuários frequentemente abrem um arquivo com o objetivo imediato de iniciar a escuta, tornando o fluxo de dois toques ("Abrir Arquivo" -> "Aguardar carregamento" -> "Pressionar Play") redundante quando o usuário acabou de escolher intencionalmente um documento [source: ASSUMPTION, baseado em padrões de uso de audiolivros/TTS em Voice Dream e @Voice Aloud]. (confidence: high)
- **Acessibilidade e affordance no cabeçalho**: Em leitores móveis minimalistas ou de tela cheia, o cabeçalho/título frequentemente atua como gatilho de biblioteca ou seletor de arquivos. Contudo, quando o título aparenta ser texto estático (`h1`), usuários de leitores de tela (TalkBack) e novos usuários não identificam a área como interativa a menos que possua semântica acessível (`role="button"`, `tabindex="0"`, `aria-label`) e affordance visual (como ícone sutil, chevron ou estilo tátil) [source: WCAG 2.1 SC 4.1.2 Name, Role, Value]. (confidence: high)
- **Retomada de documentos existentes vs novos documentos**: Usuários que reabrem um documento previamente interrompido esperam manter o progresso gravado; iniciar automaticamente a reprodução do início destruiria o contexto da pausa anterior ou violaria a expectativa de confirmação de retomada estipulada no app (FR-009) [source: `.specify/memory/constitution.md` §VIII, `contracts/resume.md`]. (confidence: high)

## Prior Art

- **Módulo `DocIngest.kt` existente**: A infraestrutura de seleção de arquivo via Android Storage Access Framework (`ACTION_OPEN_DOCUMENT`) já está 90% implementada no repositório. O método `DocIngest.pickDocument(activity)` configura intent explícito com `FLAG_GRANT_READ_URI_PERMISSION`, filtro de tipos MIME (`text/markdown`, `text/plain`), leitura de stream única para isolamento de I/O em background, cálculo de impressão digital Blake3-256 e gravação de snapshot atômico em `filesDir/snapshots/<fingerprint>.<ext>` [source: `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`]. (confidence: high)
- **Ponte JNI e comando Rust `open_document_bytes_named`**: O arquivo `src-tauri/src/shell_jni.rs` já exporta a função nativa `Java_app_mdreader_DocIngest_onDocumentIngested` que recebe o caminho do snapshot e o fingerprint, repassando os bytes diretamente para o motor Tauri através de `commands::open_document_bytes_named(state, bytes, &filename)`. Falta apenas a chamada do frontend/Tauri command para disparar `DocIngest.pickDocument` [source: `src-tauri/src/shell_jni.rs`, `src-tauri/src/lib.rs`]. (confidence: high)
- **Atores de Mercado (Voice Dream, Moon+ Reader, @Voice Aloud)**:
  - *Voice Dream Reader*: O toque no título do documento abre um menu com detalhes do texto, índice de capítulos e botão para trocar de arquivo. Ao importar um novo arquivo, oferece a opção nas configurações de "Auto-play on document open" (desativado por padrão para acessibilidade).
  - *Podcast Players (Apple Podcasts, Pocket Casts, Spotify)*: Tocar no título na tela de reprodução expande os detalhes do episódio ou exibe um menu de capítulos. O seletor de novos itens fica concentrado em botões de ação dedicados ou na tela de biblioteca. (confidence: high)

## Market & Context

- **Alternativas atuais no Ressoa**: No momento, a abertura de documentos em tempo de execução só ocorre se um intent externo enviar o arquivo ou em testes/CLI via `open_document`. A tela da Lente Sinestésica (`status_view.html`) exibe títulos fixos/padrão ("Capítulo 1", "Soneto da Fidelidade") sem permitir que o usuário troque de arquivo diretamente pela interface gráfica. O custo de não implementar é a impossibilidade de o usuário trocar de audiolivro/documento diretamente pelo app sem invocar comandos externos. (confidence: high)
- **Contratos e Governança do Projeto**:
  - O projeto Ressoa é estritamente **offline-only**, sem rede e com I/O rigorosamente validado (`check-shell-purity.sh`, `verify.sh`). O pipeline de ingestão atual já respeita essas premissas ao sanitizar e gravar snapshots locais [source: `contracts/ingest.md`]. (confidence: high)
  - Autonomia do Rust: Rust é a autoridade exclusiva de playback (`crates/player/src/position.rs`). O Android shell é mero espelho (`android-shell/`). A ação de abrir arquivo deve solicitar ao host Android a abertura do picker SAF, mas a decisão de sintetizar e reproduzir pertence ao Rust [source: `.specify/memory/constitution.md` §I]. (confidence: high)

## Data & Constraints

- **Limites de formato e tamanho de arquivo (DocIngest)**:
  - Formatos aceitos: `text/markdown`, `text/plain` (extensões `.md`, `.txt`).
  - Limite rígido de tamanho: `MAX_DOC_BYTES = 93636` (~91.4 KiB, limite orçado de memória). Arquivos maiores recebem recusa controlada (`IngestOutcome.Refused(FormatUndeliverable)`) [source: `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`]. (confidence: high)
- **Ciclo de vida do Android (SAF & Permissions)**:
  - `startActivityForResult(Intent.ACTION_OPEN_DOCUMENT)` requer uma instância de `Activity` em primeiro plano.
  - A permissão do URI requer flags persistíveis (`FLAG_GRANT_PERSISTABLE_URI_PERMISSION`) para permitir reaberturas sem re-seleção [source: `DocIngest.kt#L111`]. (confidence: high)
- **Diretrizes de Acessibilidade (WCAG 2.1 SC 1.4.2 Audio Control & SC 4.1.2)**:
  - *WCAG 1.4.2*: Qualquer áudio que inicialize automaticamente por mais de 3 segundos deve possuir um mecanismo acessível e imediato de pausa/parada. No Ressoa, o botão Play/Pause central é garantido e persistente.
  - *Leitores de tela (TalkBack)*: Áudio que toca imediatamente pode sobrepor a voz do sintetizador de tela, dificultando a navegação inicial para deficientes visuais se não houver um pequeno delay ou se o botão de pausa não receber foco prioritário [source: Section 508 / Deque University Accessibility Guidelines]. (confidence: high)

## Evidence Against the Idea

- **Toque acidental durante manuseio**: Usuários que tocam na tela para inspecionar o título ou tentar rolar a página podem acidentalmente abrir o seletor de arquivos do sistema, interrompendo a experiência de leitura atual [source: ASSUMPTION, padrão comum de feedback em leitores móveis]. (confidence: medium)
- **Conflito com a Política de Retomada (FR-009 / Constitution §VIII)**:
  - Se um arquivo já lido anteriormente pelo usuário for selecionado novamente, inicializar imediatamente a reprodução do início violaria o bookmark existente gravado no banco `redb`.
  - Se inicializar reproduzindo da posição salva sem confirmação, pode desorientar o usuário que esperava começar do topo. O app já define a exibição do diálogo de oferta (`offerDialog`: "Continuar de onde parou ou começar do início?") quando há bookmark prévio com encerramento por pausa deliberada [source: `contracts/resume.md`, `specs/001-android-media-shell/spec.md`]. (confidence: high)
- **Expectativa do Usuário vs Semântica de Título**: Títulos em visualizadores de mídia são predominantemente estáticos ou acionam a visualização de metadados/capítulos, e não o explorador de arquivos do sistema operacional. Um botão ou ícone dedicado ("Trocar documento" / "Abrir pasta") na barra superior ou na própria caixa de título oferece affordance mais clara e reduz surpresas [source: ASSUMPTION]. (confidence: medium)

## Gaps & Open Questions

- [NEEDS CLARIFICATION: O clique deve ser restrito ao elemento `#documentTitle`, ou deve abranger toda a `.meta-box` (caixa de vidro com título e subtítulo)?]
- [NEEDS CLARIFICATION: Se o arquivo escolhido já possuir bookmark salvo (ex: lido até 45%), deve ser exibido o diálogo de oferta (`offerDialog`) ou deve iniciar a reprodução imediatamente de onde parou?]
- [NEEDS CLARIFICATION: Se o usuário cancelar a seleção no seletor do sistema (botão Voltar/Cancelar no SAF), o estado da reprodução atual deve permanecer inalterado?]
- [NEEDS CLARIFICATION: Deve haver uma indicação visual na caixa de título (ex: cursor pointer, ícone de pasta/documento ou tooltip "Toque para abrir documento") para sinalizar que o título é interativo?]

## Sources

- `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (código-fonte interno, analisado localmente)
- `src-tauri/src/shell_jni.rs` e `src-tauri/src/lib.rs` (código-fonte interno, analisado localmente)
- `.specify/memory/constitution.md` e `contracts/resume.md` (especificações normativas do repositório)
- `https://www.w3.org/WAI/WCAG21/Understanding/audio-control.html` (host: w3.org, policy: confirmed-research)
- `https://developer.android.com/guide/topics/providers/document-provider` (host: developer.android.com, policy: confirmed-research)
