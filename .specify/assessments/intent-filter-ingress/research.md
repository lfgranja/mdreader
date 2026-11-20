# Research: Android Intent Filter Ingress (ACTION_VIEW & ACTION_SEND)

- **Slug**: intent-filter-ingress
- **Date**: 2026-11-11
- **Status**: complete

## 1. Contexto e Landscape Técnico no Android (minSdk 29+)

### 1.1 Intent Filters para Arquivos Markdown
No Android, diferentes aplicativos emitem intents com MIME types distintos para Markdown:
- `text/markdown` (MIME formal IANA)
- `text/plain` (Obsidian e editores simples frequentemente compartilham como texto puro ou `.txt`/`.md`)
- `text/x-markdown`
- URI scheme `content://` e `file://` com `pathPattern=".*\\.md"` ou extensões compatíveis.

Para cobrir 100% dos emissores (Obsidian, Markor, Google Files, MiXplorer, Solid Explorer, Zettel Notes, navegadores web):
1. **`ACTION_VIEW`**:
   - `scheme="content"`, `mimeType="text/markdown"`
   - `scheme="content"`, `mimeType="text/plain"` com filtro de caminho para extensões markdown quando disponível, ou tratamento de fallback.
   - `scheme="file"`, `pathPattern=".*\\.md"`
2. **`ACTION_SEND`**:
   - `mimeType="text/markdown"` ou `mimeType="text/plain"` (recebendo `Intent.EXTRA_STREAM` para arquivos ou `Intent.EXTRA_TEXT` para notas inline).

### 1.2 Segurança e Permissões de Acesso (Scored Storage & URI Permissions)
- As URIs recebidas via `ACTION_VIEW` trazem permissão temporária concedida pelo chamador (`Intent.FLAG_GRANT_READ_URI_PERMISSION`).
- O `ContentResolver` do Android pode abrir a stream diretamente via:
  ```kotlin
  context.contentResolver.openInputStream(uri)?.use { stream ->
      val bytes = stream.readBytes()
      // repassar bytes para o motor Rust via JNI
  }
  ```
- O Ressoa já possui o pipeline de snapshot (`DocIngest.kt`): ele lê bytes de documentos SAF, calcula o hash criptográfico Blake3 e injeta diretamente no motor Rust (`crates/session`). A mesma infraestrutura de ingestão é reutilizada integralmente.

### 1.3 Ciclo de Vida da `MainActivity` (`singleTask`)
- Como a `MainActivity` é configurada como `android:launchMode="singleTask"`:
  - Se o app não estiver aberto: o Intent chega em `onCreate(savedInstanceState)`.
  - Se o app já estiver em execução: o Intent chega em `onNewIntent(intent: Intent)`.
- Ambas as rotas devem direcionar para um manipulador comum (`handleIncomingIntent(intent)`), garantindo que se o usuário clicar em uma nota enquanto o Ressoa está aberto em segundo plano, a nova nota é carregada e substitui a leitura atual sem reiniciar a WebView ou perder o estado do motor.

## 2. Invariantes do mdreader
- **Zero Lógica de Playback em Kotlin:** O Kotlin apenas abre o `InputStream`, extrai os bytes e o nome do arquivo, e despacha a ação de ingestão via JNI (`open_document_from_shell`).
- **Zero Permissão de Rede:** O app continua 100% air-gapped.
- **Audible Progress:** O `PositionClock` só avança quando o novo arquivo começar a tocar efetivamente.
