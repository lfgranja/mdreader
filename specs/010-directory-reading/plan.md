# Implementation Plan: Leitura Contínua de Diretórios de Markdown (010-directory-reading)

**Branch**: `010-directory-reading` | **Date**: 2026-11-03 | **Spec**: [specs/010-directory-reading/spec.md](spec.md)

**Input**: Feature specification from `/specs/010-directory-reading/spec.md`

## Summary

Implementar a leitura contínua e hands-free de diretórios e vaults inteiros de notas Markdown/texto estruturados (`01-`, `02-`) ou não organizados no Android. A funcionalidade utiliza o seletor nativo `ACTION_OPEN_DOCUMENT_TREE` (SAF) para conceder acesso à pasta, indexa os arquivos elegíveis em background coroutine sem bloquear a UI, e inicia a reprodução do primeiro capítulo em menos de 3,0 segundos (TTFA ≤ 3s). A autoridade de transporte do Rust gerencia uma fila linear de capítulos (`ChapterQueue`), orquestrando transições contínuas (*gapless*) entre arquivos consecutivos, armazenando marcadores compostos em tabela dedicada no `redb` (`DIRECTORY_BOOKMARKS`), e acomodando a contagem de capítulos na Lente Sinestésica sem romper o teto de 100dvh nem causar Cumulative Layout Shift (CLS).

## Technical Context

**Language/Version**: Rust 2024 edition (stable toolchain) / Kotlin (Android minSdk 29, compileSdk 34) / HTML5 + Vanilla JS
**Primary Dependencies**: Tauri 2.12.1, cpal 0.18.2, redb 2.4.0, sherpa-onnx (pt-BR Piper offline)
**Storage**: `redb` local (`bookmarks.redb` com tabela `DIRECTORY_BOOKMARKS`), snapshots privados em `filesDir/snapshots/<fingerprint>.<ext>`
**Testing**: `cargo nextest run --workspace`, testes de contrato JNI, testes de propriedade (`proptest`), testes de superfície e UI
**Target Platform**: Android minSdk 29+ (AArch64 / ARM64) com suporte a host Linux/x86_64 para testes
**Project Type**: Android media-shell TTS audiobook reader (Pure Offline)
**Performance Goals**: TTFA < 3,0 segundos a partir da seleção da pasta; transição audível entre capítulos em < 1,0 segundo; 60 fps na Lente Sinestésica
**Constraints**: Pure-Offline (zero permissão `INTERNET`), teto de tamanho individual de 93.636 bytes (`MAX_DOC_BYTES`) por arquivo, autoridade exclusiva de playback no Rust, closed sets sem `_ =>`
**Scale/Scope**: Suporte a diretórios com até 100 arquivos Markdown, 1 seletor de pasta SAF, 0 bibliotecas externas adicionadas

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Princípio I (Rust = Sole Playback Authority)**: PASS. O Android SAF e o Kotlin apenas enumeram URIs e fornecem streams de bytes sob demanda; a ordenação canônica, fila de capítulos (`ChapterQueue`), decisão de sintetizar, relógio de posição e transição entre arquivos pertencem 100% ao motor Rust.
- **Princípio II (Library-First & Same Code Path)**: PASS. A ingestão de diretórios e capítulos utiliza os mesmos tipos e despachos no Rust (`ChapterQueue`, `open_document_bytes_named`), acessíveis por API e IPC.
- **Princípio VII (Closed Sets Everywhere)**: PASS. Os status de capítulo (`pending`, `playing`, `completed`, `skipped`), verbos de navegação e códigos de recusa pertencem a enums estritos sem variantes genéricas (`_ =>` / `Other`).
- **Princípio VIII (Driving-Time Safety)**: PASS. A transição automática hands-free elimina a necessidade de toques na tela entre arquivos, e os botões de fone de ouvido suportam avanço contextual seguro.
- **Princípio IX (Pure Offline)**: PASS. Processamento estritamente local; nenhuma chamada de rede ou telemetria externa.

## Project Structure

### Documentation (this feature)

```text
specs/010-directory-reading/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (decisões técnicas consolidadas)
├── data-model.md        # Phase 1 output (entidades de dados e ciclo de vida)
├── quickstart.md        # Phase 1 output (cenários de validação ponta a ponta)
├── contracts/           # Phase 1 output (especificações de IPC e JNI)
│   └── directory-playback.md
├── checklists/
│   └── requirements.md  # Validação de qualidade de requisitos
└── tasks.md             # Phase 2 output (/speckit-tasks command)
```

### Source Code (repository root)

```text
android-shell/src/main/kotlin/app/mdreader/
├── DocIngest.kt         # Adição de pickDirectory (ACTION_OPEN_DOCUMENT_TREE), varredura assíncrona
├── ShellHost.kt         # Ponto de despacho estático JNI para pickDirectory
└── MediaMirror.kt       # Encaminhamento de ações de mídia contextual (skipToNext / previous)

crates/session/src/
├── store.rs             # Tabela DIRECTORY_BOOKMARKS, gravação e lookup de progresso de diretório
└── fingerprint.rs       # Hashing composto da coleção ordenada (blake3(fp_1 || ... || fp_n))

crates/player/src/
├── lib.rs               # Integração da fila de capítulos (ChapterQueue) no Player
└── skip.rs              # Lógica de salto contextual (próxima seção vs próximo capítulo na borda)

src-tauri/src/
├── lib.rs               # Comandos Tauri: open_directory, next_chapter, previous_chapter, canal de capítulos
├── shell_jni.rs         # Despacho JNI Rust -> ShellHost.pickDirectory e retorno de árvore
└── status_view.html     # Atualização do subtítulo (#sectionTitle) com contagem de capítulos tabular
```
