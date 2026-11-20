# Concept & Shaping: Ingress via Intent Filters

- **Slug**: intent-filter-ingress
- **Date**: 2026-11-11

## 1. Opções de Solução

### Opção 1: Intent Filters na `MainActivity` delegando para `DocIngest` (Recomendada)
- Configurar os filtros de intenção em `MainActivity` no `AndroidManifest.xml`.
- No `MainActivity.kt` (ou extensão acoplada a `ShellHost`), interceptar Intents de ação `ACTION_VIEW` e `ACTION_SEND` em `onCreate` e `onNewIntent`.
- Ler os bytes do stream via `contentResolver.openInputStream(uri)`, inferir o nome do documento via `DocumentFile` ou query em `OpenableColumns.DISPLAY_NAME`.
- Encaminhar os bytes e o título para a função existente em `DocIngest.kt` (`handleDocumentBytesSelected` / `ingestDocument`), que já se comunica via JNI com o bridge Tauri.
- **Apetite:** Pequeno (~1 dia, ~60 linhas de código no total).
- **Risco:** Mínimo. Não afeta nenhuma invariante de áudio Rust nem altera o modelo de dados de sessão.

### Opção 2: Criar uma Activity Trampoline Dedicada (`IngestActivity`)
- Uma Activity sem interface (`Theme.Translucent.NoTitleBar`) apenas para receber a Intent e despachar para o `PlaybackService` ou `MainActivity`.
- **Desvantagem:** Complexidade desnecessária de ciclo de vida e concorrência no Android 13+.

## 2. Decisão de Escopo
- Adotar a **Opção 1**. É a mais simples, direta, robusta e compatível com a arquitetura `singleTask` do Tauri v2.
