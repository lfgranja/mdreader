# Interface Contracts: Feature 009 (open-file-from-title)

- **Feature**: `009-open-file-from-title`
- **Date**: 2026-11-02
- **Status**: Complete

## 1. Frontend Webview → Rust Tauri IPC Contract

### A. Ação de Selecionar Documento
- **Disparo**: Usuário clica ou aciona via teclado (`Enter` / `Space`) no container `.meta-box`.
- **Método**: Invocação via canal existente de ações ou comando Tauri:
  ```javascript
  // No status_view.html:
  els.metaBox.addEventListener('click', async () => {
    await invoke('run_mobile_plugin', { action: 'pick_document' });
  });
  ```
- **Assinatura no Rust**:
  ```rust
  // Em commands::parse_action:
  "pick_document" => Ok(BridgeAction::PickDocument)
  ```

## 2. Rust Tauri → Kotlin Shell Contract (JNI)

### A. Solicitação de Abertura do SAF Picker
- **JNI Call**: Rust invoca o método estático `ShellHost.pickDocument()`:
  ```rust
  pub fn pick_document_from_shell() -> Result<(), String> {
      // Anexa thread, localiza app/mdreader/ShellHost, invoca void pickDocument()
  }
  ```
- **Implementação Kotlin (`ShellHost.kt`)**:
  ```kotlin
  @JvmStatic
  fun pickDocument(): Boolean {
      val activity = currentActivity ?: return false
      return DocIngest.pickDocument(activity)
  }
  ```

## 3. Kotlin Shell → Rust Contract (JNI Callback)

### A. Documento Ingerido com Sucesso
- **Callback JNI já existente**:
  ```rust
  pub extern "C" fn Java_app_mdreader_DocIngest_onDocumentIngested(
      env: *mut jni::sys::JNIEnv,
      _this: jni::sys::jobject,
      snapshot_path: jni::sys::jstring,
      fingerprint_hex: jni::sys::jstring,
      original_uri: jni::sys::jstring,
      mime_type: jni::sys::jstring,
  );
  ```
- **Comportamento em Rust**:
  - Lê os bytes do snapshot.
  - Executa `open_document_bytes_named(state, bytes, filename)`.
  - Se for arquivo novo: executa `play_document`, iniciando síntese e playback imediatamente.
  - Se possuir bookmark de pausa deliberada: parkeia e publica `"offer_route": "in_app_dialog"`, abrindo o diálogo de oferta por 3s.

## 4. Acessibilidade do Container na Lente Sinestésica

- **Markup (`status_view.html`)**:
  ```html
  <div class="meta-box" role="button" tabindex="0" aria-label="Selecionar documento para leitura">
    <h1 id="documentTitle"></h1>
    <div class="subtitle" id="sectionTitle"><span aria-hidden="true">—</span><span class="sr-only">Sem título</span></div>
  </div>
  ```
- **CSS**:
  ```css
  .meta-box {
    cursor: pointer;
    user-select: none;
    transition: transform 120ms ease, box-shadow 120ms ease;
  }
  .meta-box:active {
    transform: scale(0.985);
  }
  .meta-box:focus-visible {
    outline: 2px solid var(--sona-etwahl);
    outline-offset: 4px;
  }
  ```
