# Data Model: Carregamento de Documentos e Início de Reprodução pelo Título

- **Feature**: `009-open-file-from-title`
- **Date**: 2026-11-02
- **Status**: Complete

## Entidades e Ciclo de Vida

### 1. IngestSnapshot (Trânsito JNI / Rust)
Representa o arquivo lido e sanitizado pelo seletor nativo Android (SAF) e repassado para o Rust:

```rust
pub struct IngestSnapshot {
    pub path: String,           // Caminho absoluto do snapshot em cache local privado
    pub fingerprint_hex: String,// Hash BLAKE3-256 (64 hex chars)
    pub original_uri: String,   // URI SAF original (content://...)
    pub mime_type: String,      // "text/markdown" ou "text/plain"
}
```

- **Validações**:
  - `path`: Deve existir e ser legível no diretório seguro de snapshots da aplicação (`filesDir/snapshots/`).
  - `fingerprint_hex`: Exatamente 64 caracteres hexadecimais em caixa baixa.
  - `mime_type`: Estritamente `text/markdown` ou `text/plain`.

### 2. LoadedDocument & DocumentSection (Rust AppState)
O documento de texto em memória no Rust:

```rust
pub struct LoadedDocument {
    pub filename: String,
    pub sections: Vec<DocumentSection>,
    pub totals: DocumentTotals,
}

pub struct DocumentSection {
    pub title: String,
    pub text: String,
}
```

- **Regras de Negócio**:
  - Tamanho máximo em bytes do snapshot: `93.636 bytes` (`MAX_DOC_BYTES`). Arquivos que excedem esse tamanho são recusados com `FormatUndeliverable`.
  - Normalização: Codificação estritamente UTF-8 válida.

### 3. IngestOutcome (Kotlin Shell)
Enum fechado representando o resultado da interação do usuário com o seletor do sistema operacional:

```kotlin
sealed class IngestOutcome {
    data class Accepted(val result: IngestResult) : IngestOutcome()
    data class Refused(val reason: String) : IngestOutcome()
    object Unreadable : IngestOutcome()
    object Cancelled : IngestOutcome()
}
```

- **Transições de Estado**:
  - `User taps .meta-box` → `DocIngest.pickDocument` (SAF abre).
  - `SAF returns URI OK` → Leitura de bytes → `IngestOutcome.Accepted` → grava snapshot → dispara `onDocumentIngested` JNI → Rust `open_document_bytes_named` → Transição para `Playing` (se novo) ou exibe `offerDialog` (se lido anteriormente).
  - `SAF returns CANCELLED` → `IngestOutcome.Cancelled` → Nenhuma mudança de estado no Rust.
  - `SAF returns file > 93636 bytes` → `IngestOutcome.Refused(FormatUndeliverable)` → Notifica banner de erro, sem alterar a reprodução ativa.
