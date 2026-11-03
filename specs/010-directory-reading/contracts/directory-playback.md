# Interface Contracts: Leitura Contínua de Diretórios de Markdown

- **Feature**: `010-directory-reading`
- **Date**: 2026-11-03
- **Status**: Complete

## 1. Android Shell SAF Picker & JNI Signatures

### 1.1 Invocação do Seletor de Diretório (Rust → Kotlin)

Rust dispara a abertura da árvore nativa no Android invocando o método estático `ShellHost.pickDirectory()`:

```rust
#[cfg(target_os = "android")]
pub fn pick_directory_from_shell() -> Result<(), String> {
    let vm = JVM
        .get()
        .ok_or_else(|| "shell JNI: JVM not initialized; cannot pick directory".to_string())?;

    let mut env = vm
        .attach_current_thread()
        .map_err(|error| format!("shell JNI: attach failed: {error}"))?;

    let class = get_shell_host_class()?;

    let _ = env
        .call_static_method(class, "pickDirectory", "()Z", &[])
        .map_err(|error| format!("shell JNI: ShellHost.pickDirectory failed: {error}"))?;
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub const fn pick_directory_from_shell() -> Result<(), String> {
    Ok(())
}
```

### 1.2 Implementação Kotlin (`ShellHost.kt` / `DocIngest.kt`)

```kotlin
@JvmStatic
fun pickDirectory(): Boolean {
    val activity = currentActivity ?: return false
    val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
        addFlags(
            Intent.FLAG_GRANT_READ_URI_PERMISSION or
            Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
        )
    }
    activity.startActivityForResult(intent, REQUEST_CODE_PICK_DIRECTORY)
    return true
}
```

No retorno de `onActivityResult`:
1. `contentResolver.takePersistableUriPermission(treeUri, Intent.FLAG_GRANT_READ_URI_PERMISSION)`
2. Disparo de coroutine em `Dispatchers.IO` para varrer a árvore usando `DocumentFile.fromTreeUri(context, treeUri)`.
3. Coleta dos arquivos com extensão `.md` e `.txt` na raiz e subpastas de nível 1.
4. Aplicação de ordenação natural alfanumérica.
5. Emissão imediata do primeiro arquivo para Rust via `onDirectorySelected` para que o TTFA seja < 3.0s.

### 1.3 Callback JNI de Retorno do Diretório (Kotlin → Rust)

```rust
#[unsafe(no_mangle)]
pub extern "C" fn Java_app_mdreader_DocIngest_onDirectorySelected(
    mut env: jni::JNIEnv,
    _this: jni::objects::JObject,
    tree_uri: jni::objects::JString,
    root_name: jni::objects::JString,
    manifest_json: jni::objects::JString,
) {
    // Processamento estritamente offline:
    // Converte manifest_json (lista de capítulos ordenados com caminhos de snapshot locais)
    // Instancia ChapterQueue no AppState
    // Despacha para open_directory_session()
}
```

---

## 2. Tauri IPC Commands & Channel Events

### 2.1 Comandos Invocados pelo Frontend (`status_view.html` / `lib.rs`)

| Comando | Assinatura Rust | Descrição |
|---|---|---|
| `pick_directory` | `run_mobile_plugin(action: "pick_directory")` | Aciona o picker SAF no Android |
| `next_chapter` | `run_mobile_plugin(action: "next_chapter")` | Salta explicitamente para o próximo capítulo da fila |
| `previous_chapter` | `run_mobile_plugin(action: "previous_chapter")` | Salta para o capítulo anterior da fila |

Mapeamento no parser de ações fechadas de `crates/player`:
```rust
// Extensão fechada dos verbos IPC (Princípio VII)
"pick_directory" => Ok(BridgeAction::Verb(ShellVerb::PickDirectory)),
"next_chapter" => Ok(BridgeAction::Verb(ShellVerb::NextChapter)),
"previous_chapter" => Ok(BridgeAction::Verb(ShellVerb::PreviousChapter)),
```

### 2.2 Evento Emitido via Channel: `chapter_change`

Quando ocorre a transição de capítulo (seja automática ou por comando), Rust emite via `tauri::ipc::Channel` o evento de sincronização estruturado:

```json
{
  "type": "chapter_change",
  "active_index": 2,
  "total_chapters": 14,
  "filename": "03-conceitos-avancados.md",
  "section_title": "1. Ownership e Lifetimes",
  "has_previous": true,
  "has_next": true
}
```

---

## 3. Wire Format JSON Schemas

### 3.1 Directory Manifest Schema (`directory-manifest.json`)
Formato entregue pelo Kotlin via JNI ao inicializar a sessão do diretório:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DirectoryManifest",
  "type": "object",
  "required": ["tree_uri", "root_name", "chapters"],
  "additionalProperties": false,
  "properties": {
    "tree_uri": { "type": "string" },
    "root_name": { "type": "string" },
    "chapters": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["filename", "snapshot_path", "relative_path", "byte_size", "fingerprint_hex"],
        "additionalProperties": false,
        "properties": {
          "filename": { "type": "string" },
          "snapshot_path": { "type": "string" },
          "relative_path": { "type": "string" },
          "byte_size": { "type": "integer", "minimum": 0 },
          "fingerprint_hex": { "type": "string", "pattern": "^[0-9a-f]{64}$" }
        }
      }
    }
  }
}
```

### 3.2 Chapter Change Event Schema (`chapter-change.json`)
Formato do evento emitido no canal Tauri IPC para atualização da interface da Lente Sinestésica:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ChapterChangeEvent",
  "type": "object",
  "required": [
    "type",
    "active_index",
    "total_chapters",
    "filename",
    "section_title",
    "has_previous",
    "has_next"
  ],
  "additionalProperties": false,
  "properties": {
    "type": { "type": "string", "enum": ["chapter_change"] },
    "active_index": { "type": "integer", "minimum": 0 },
    "total_chapters": { "type": "integer", "minimum": 1 },
    "filename": { "type": "string" },
    "section_title": { "type": ["string", "null"] },
    "has_previous": { "type": "boolean" },
    "has_next": { "type": "boolean" }
  }
}
```

### 3.3 Media Surface Snapshot Mirror Payload (Atualizado)
Atualização do payload emitido por Rust para renderização no `status_view.html` e no `MediaSession`:

```json
{
  "title": "03-conceitos-avancados.md",
  "section_title": "Capítulo 3 de 14 • 1. Ownership e Lifetimes",
  "progress": {
    "kind": "determinate",
    "consumed_units": 120,
    "total_units": 540,
    "k": 3,
    "n": 14
  },
  "transport": "playing",
  "reason": null,
  "reason_ttl_ms": null,
  "dismissal": "non_dismissable_while_playing"
}
```

**Regras Invariantes de Exibição**:
- A string de `section_title` segue rigorosamente o formato de numerais tabulares:
  `"Capítulo {k} de {n} • {heading}"` se houver cabeçalho ativo, ou `"Capítulo {k} de {n}"` se for o início do documento sem cabeçalhos.
- Leitores de tela anunciam as mudanças de capítulo através da região acessível `#liveRegion` com `aria-live="polite"` sem deslocamento visual do cursor.
