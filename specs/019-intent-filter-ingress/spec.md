# Specification — 019-intent-filter-ingress: Android Intent Filter Ingress

**Status:** Ready for Implementation  
**Prioridade:** Alta (80/20 de Usabilidade e Adoção)  
**Dependências:** Spec 001 (Shell), Spec 004 (Dogfood), Spec 009 (SAF Ingest)

---

## 1. Visão Geral

Permitir que o aplicativo **Ressoa** intercepte intents do sistema Android para abrir e compartilhar arquivos e notas Markdown (`.md`, `text/markdown`, `text/plain`) diretamente a partir de gerenciadores de arquivos (Google Files, MiXplorer) e editores de notas (Obsidian, Logseq, Markor), executando a ingestão instantânea e iniciando a reprodução offline via motor Rust.

---

## 2. Requisitos Funcionais

- **FR-INT-001 (Manifest Declaration):** O `AndroidManifest.xml` deve declarar filtros de intent para `ACTION_VIEW` e `ACTION_SEND` na `MainActivity`.
- **FR-INT-002 (MIME & Scheme Support):** Suporte aos esquemas `content://` e `file://`, tipos MIME `text/markdown`, `text/plain`, `text/x-markdown` e padrão de arquivo `*.md`.
- **FR-INT-003 (Lifecycle Handoff):** Tratamento consistente em inicialização fria (`onCreate`) e quando o app já estiver ativo em segundo plano (`onNewIntent`).
- **FR-INT-004 (Content Resolution & Titling):** Resolução do nome do arquivo através do `OpenableColumns.DISPLAY_NAME` do `ContentResolver` (com fallback seguro para o último segmento da URI ou `"documento.md"`).
- **FR-INT-005 (Safe Byte Ingestion):** Leitura dos bytes brutos em stream via `ContentResolver.openInputStream(uri)`, validação UTF-8 segura e repasse via JNI para o comando de ingestão de documento.
- **FR-INT-006 (Zero Playback Logic in Kotlin):** Nenhuma decisão de áudio, buffer, timer ou posição deve residir no Kotlin. O Kotlin é mero transportador dos bytes para o Rust.
- **FR-INT-007 (Auto-Play & Bookmark State):** O arquivo recebido deve disparar a oferta de retomar/iniciar reprodução conforme o comportamento padrão do motor Rust para arquivos ingeridos.

---

## 3. Critérios de Aceite

1. Ao selecionar um arquivo `.md` no aplicativo Files e escolher "Abrir com Ressoa", o app abre, carrega o título do arquivo no `#documentTitle`, sintetiza o áudio via `sherpa-onnx` e inicia o som.
2. Ao compartilhar uma nota dentro do Obsidian via Share Sheet e selecionar "Ressoa", a nota é carregada e reproduzida.
3. Se o Ressoa já estiver aberto e reproduzindo outro arquivo, abrir uma nova nota substitui o contexto de reprodução limpamente sem travar ou crashar o aplicativo.
4. Gates 1 a 8 de `./scripts/verify.sh` continuam rigorosamente verdes.
