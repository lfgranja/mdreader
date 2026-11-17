# Architecture & Implementation Plan — 019-intent-filter-ingress

## 1. Arquitetura e Fluxo de Dados

```
App Externo (Obsidian / Files)
      │
      ▼ Intent (ACTION_VIEW / ACTION_SEND com URI ou EXTRA_TEXT)
MainActivity.kt (onCreate / onNewIntent)
      │
      ▼ handleIncomingIntent(intent)
DocIngest.kt (processIntentUri / processIntentText)
      │  - Resolve displayName (ContentResolver)
      │  - Lê InputStream -> ByteArray
      │
      ▼ JNI: open_document_from_shell(title, bytes)
Tauri Bridge (src-tauri/src/lib.rs)
      │
      ▼ Armazena snapshot, calcula Blake3, atualiza AppState
Motor Rust (crates/session & crates/player)
      │
      ▼ Carrega redb bookmark, sintetiza via sherpa-onnx e inicia áudio
Hardware Audio (CPAL / AAudio)
```

## 2. Componentes e Arquivos Alterados

1. **`src-tauri/gen/android/app/src/main/AndroidManifest.xml`**:
   - Adicionar `<intent-filter>` para `ACTION_VIEW` com esquemas `content` e `file` e tipos MIME `text/markdown`, `text/plain`, `text/x-markdown`.
   - Adicionar `<intent-filter>` para `ACTION_SEND` com MIME `text/markdown`, `text/plain`.

2. **`android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`**:
   - Implementar método público `handleIncomingIntent(activity: Activity, intent: Intent)`:
     - Trata `ACTION_VIEW` extraindo `intent.data`.
     - Trata `ACTION_SEND` extraindo `EXTRA_STREAM` (URI de arquivo) ou `EXTRA_TEXT` (string compartilhada).
     - Extrai bytes e despacha via JNI para `ingestDocumentFromBytes`.

3. **`android-shell/src/main/kotlin/app/mdreader/ShellHost.kt` / `MainActivity.kt`**:
   - Conectar chamadas em `onCreate` e `onNewIntent` chamando `DocIngest.handleIncomingIntent`.

4. **Testes e Validação:**
   - Adicionar teste de integração/contrato garantindo que chamadas de abertura direta via JNI continuam válidas e determinísticas.
   - Executar `./scripts/verify.sh` (Gates 1 a 8).
