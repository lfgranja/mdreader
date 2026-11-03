# Phase 0 Research: Carregamento de Documentos e Início de Reprodução pelo Título

- **Feature**: `009-open-file-from-title`
- **Date**: 2026-11-02
- **Status**: Complete (All unknowns resolved)

## Technical Decisions & Findings

### Decision 1: Delimitação da Área de Toque e Semântica de Acessibilidade
- **Chosen**: O container completo `.meta-box` em `src-tauri/src/status_view.html` atua como controle interativo (`role="button"`, `tabindex="0"`, `aria-label="Selecionar documento para leitura"`, `cursor: pointer;`, animação suave no `:active` com scale `0.985`, e `:focus-visible` com anel de foco `var(--sona-etwahl)`).
- **Rationale**: 
  - O alvo de toque mede ~450px de largura por ~120-160px de altura, superando com folga o mínimo recomendado pelas diretrizes WCAG 2.5.8 (AA, 24px) e Google Material Design 3 / Android Accessibility Scanner (48×48dp).
  - Resolve o problema do estado ocioso/vazio: quando não há texto carregado, `#documentTitle.textContent = ''`, o que colapsaria a área de toque se ela ficasse restrita apenas ao `<h1>`.
  - Evita áreas "mortas" dentro do card de vidro caso o usuário toque na região do subtítulo `#sectionTitle`.
- **Alternatives Considered**: 
  - Restringir o toque apenas a `#documentTitle`: Rejeitado por deixar o subtítulo inerte e colapsar o hit target no estado vazio.
  - Botão de ícone extra na barra de status: Rejeitado por quebrar o minimalismo e a pureza estética do design SONA da Lente Sinestésica.

### Decision 2: Disparo do Picker Android (SAF) e Roteamento JNI/Tauri
- **Chosen**:
  1. `ShellApplication` registra o `ActivityLifecycleCallbacks` capturando a instância ativa da `MainActivity`.
  2. Um método estático `ShellHost.pickDocument()` é adicionado no Kotlin que chama `DocIngest.pickDocument(activity)`.
  3. `MainActivity.onActivityResult` encaminha o resultado para `DocIngest.handlePickResult(this, requestCode, resultCode, data)`.
  4. Um comando Tauri `commands::pick_document` (ou ação via `sendAction('pick_document')`) é invocado pelo frontend webview ao clicar na `.meta-box`, chamando `ShellHost.pickDocument` via JNI.
- **Rationale**:
  - `DocIngest.kt` já possui toda a lógica de intent (`Intent.ACTION_OPEN_DOCUMENT`), extração de stream segura, snapshot atômico em `filesDir/snapshots/`, hashing Blake3-256 e invocação JNI de `onDocumentIngested`.
  - `MainActivity` no Android precisa receber `onActivityResult` para fechar o ciclo do SAF sem introduzir bibliotecas pesadas de terceiros ou plugins instáveis.
- **Alternatives Considered**:
  - Usar plugin de terceiros (`tauri-plugin-dialog`): Rejeitado porque introduziria dependência externa desnecessária e incompatível com o isolamento do Ressoa e regras de supply-chain / Pure-Offline.

### Decision 3: Auto-Play e Respeito aos Bookmarks Salvos (FR-009)
- **Chosen**: Manter a autoridade de reprodução integralmente no Rust através da função já existente `open_document_bytes_named` em `src-tauri/src/lib.rs`.
- **Rationale**:
  - `open_document_bytes_named` já implementa com perfeição a regra:
    - Se o documento é novo ou nunca foi pausado deliberadamente (`StartNow`): Chama `play_document`, iniciando síntese e áudio audível imediatamente (**auto-play total**).
    - Se o documento foi pausado deliberadamente anteriormente (`AwaitOffer`): Parkeia a abertura e emite `"offer_route": "in_app_dialog"`, abrindo o diálogo de oferta (`offerDialog`) na Lente Sinestésica por 3 segundos para que o usuário escolha entre retomar de onde parou ou começar do início.
  - Isso cumpre 100% dos requisitos do contrato FR-009 e as diretrizes de acessibilidade WCAG 1.4.2 (Audio Control).

### Decision 4: Tratamento de Erros e Recusas (> 93.636 bytes ou não-texto)
- **Chosen**: 
  - Se o usuário selecionar um arquivo inválido ou cancela a seleção, o estado de reprodução atual permanece intacto.
  - Se `DocIngest.handlePickResult` recusar o arquivo (`IngestOutcome.Refused`), ele notifica a interface emitindo a razão de recusa padronizada (`FormatUndeliverable`) para que o `reasonBanner` e o `liveRegion` exibam feedback acessível e discreto sem travar o aplicativo.
- **Rationale**:
  - Alinhado com WCAG 3.3.1 (Error Identification) e `contracts/surface.md` (onde `FormatUndeliverable` mapeia para `"Formato do documento não pode ser lido"`).
