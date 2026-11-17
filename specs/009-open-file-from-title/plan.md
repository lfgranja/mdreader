# Implementation Plan: Abrir Arquivo Pelo Título e Iniciar Reprodução

**Branch**: `009-open-file-from-title` | **Date**: 2026-11-02 | **Spec**: [specs/009-open-file-from-title/spec.md](spec.md)

**Input**: Feature specification from `/specs/009-open-file-from-title/spec.md`

## Summary

Implementar a seleção interativa de documentos de texto compatíveis (`.md`, `.txt`) ao tocar na caixa de título (`.meta-box`) da Lente Sinestésica no Android, integrando a interface ao seletor nativo do sistema operacional (Storage Access Framework `ACTION_OPEN_DOCUMENT`) através da infraestrutura existente em `DocIngest.kt`, e disparando automaticamente a reprodução do áudio (TTS) para novos documentos enquanto preserva o diálogo de oferta de retomada (`offerDialog` / FR-009) para documentos com histórico prévio de pausa deliberada.

## Technical Context

**Language/Version**: Rust 2024 edition (stable toolchain) / Kotlin (Android minSdk 29, compileSdk 34) / HTML5 + Vanilla JS
**Primary Dependencies**: Tauri 2.12.1, cpal 0.18.2, redb 2.4.0, sherpa-onnx (pt-BR Piper)
**Storage**: redb local (`bookmarks.redb`), snapshots privados em `filesDir/snapshots/`
**Testing**: `cargo nextest run --workspace`, testes de contrato JNI e testes de interface no Android
**Target Platform**: Android minSdk 29+ (AArch64 / ARM64)
**Project Type**: Android media-shell TTS audiobook reader (Pure Offline)
**Performance Goals**: Início da síntese/reprodução em < 1s após seleção no SAF
**Constraints**: Pure-Offline (sem permissão de INTERNET no manifest), limite de arquivo `93.636 bytes` (`MAX_DOC_BYTES`), autoridade exclusiva de playback no Rust
**Scale/Scope**: 1 tela principal (Lente Sinestésica), 1 fluxo nativo SAF, 0 dependências externas adicionadas

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Princípio I (Rust = Sole Playback Authority)**: PASS. O Android SAF e a JNI apenas obtêm os bytes do arquivo e passam para o Rust; a decisão de sintetizar, bufferizar e tocar pertence 100% ao motor Rust em `open_document_bytes_named`.
- **Princípio II (Library-First & Same Code Path)**: PASS. A ingestão utiliza exatamente a mesma função Rust (`open_document_bytes_named`) que o IPC e a CLI utilizam.
- **Princípio VII (Closed Sets Everywhere)**: PASS. Os códigos de ação, tipos MIME aceitos e razões de recusa (`FormatUndeliverable`) pertencem aos conjuntos fechados estritos sem catch-alls (`_ =>`).
- **Princípio IX (Pure Offline)**: PASS. Nenhuma permissão de rede solicitada; processamento puramente local via SAF e modelos locais.

## Project Structure

### Documentation (this feature)

```text
specs/009-open-file-from-title/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (decisões técnicas consolidadas)
├── data-model.md        # Phase 1 output (entidades de dados e ciclo de vida)
├── quickstart.md        # Phase 1 output (cenários de validação ponta a ponta)
├── contracts/           # Phase 1 output (especificações de IPC e JNI)
│   └── ipc-and-jni.md
└── checklists/
    └── requirements.md  # Validação de qualidade de requisitos
```

### Source Code (repository root)

```text
android-shell/src/main/kotlin/app/mdreader/
├── DocIngest.kt         # Picker SAF, limites de tamanho e leitura de stream
├── ShellHost.kt         # Ponto de despacho estático JNI para pickDocument
├── ShellApplication.kt  # Rastreamento de Activity ativa para lançamento da intent
└── MainActivity.kt      # Encaminhamento de onActivityResult para DocIngest

src-tauri/src/
├── status_view.html     # Adição de listeners, acessibilidade e microinterações na .meta-box
├── lib.rs               # Inclusão da ação 'pick_document' em parse_action / dispatch_verb
└── shell_jni.rs         # Chamada JNI Rust -> ShellHost.pickDocument
```
