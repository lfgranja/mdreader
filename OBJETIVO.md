# Objetivo do Projeto — mdreader / Audio Media Shell

**Referência:** `AGENTS.md`, `.specify/memory/constitution.md` v1.12.0 (ratificada 2026-10-01), `specs/001-android-media-shell/`.

## O que é

Um leitor de documentos de texto/Markdown (`mdreader`) voltado a áudio/síntese de voz, construído como workspace modular Rust com shell Android (Kotlin `Service` / `MediaSession`), engine de síntese (`adapter-synthesis`), e design-system (`DESIGN-SONA`). A autoridade de posição, transporte, buffering e rate pertence ao Rust (`crates/player`, `crates/session`, `crates/diag`, `crates/engine-core`); Kotlin é apenas espelho estrutural (`SC-035`).

## Por que existe (evidência)

- `AGENTS.md`: greenfield com `specs/`, `constitution.md`; arquitetura intencional (mono f32 @ 22050 Hz, `blake3` fingerprint, `redb` bookmarks, fingerprint de documento; zero playback logic em Kotlin).
- Constituição: governança de gates (1–8), supply-chain (`deny` + `audit`), closed-set (rate ladder `[0.75,1.0,1.25,1.5,1.75,2.0,4.0]`, 7 `EventKind`, 9 `EventKind` no diag, 7 confirmações de `EarconKind`), design-dir (`DESIGN-SONA`) como fonte de verdade de tokens/paleta.
- `specs/`: 5 features rastreadas (001 shell, 002 synthesis adapter, 003 reference device, 004 dogfood build, 005 design-sona adoption); 4 assessments (`directory-reading`, `design-son-adoption`, `fr-024-synthesis-adapter`, `ressoa-mobile-app`).

## Resultado esperado (convergência)

- `001` + `002` + `003`: dispositivo de referência verificável (gate-8-shell PASS, gate-8-full DEFERRED até síntese fechar `SC-044`), com `verify.sh` verde e attestation V3 para adaptador.
- `004`: build de dogfood com `verify.sh` GREEN (já atingido em `9501bba`, 2026-10-22, Phase 17 completa).
- `005`: adoção completa do `DESIGN-SONA` (tokens, contraste, animações congeladas sob `prefers-reduced-motion`, `SC-007` TalkBack 10/10), com plano pronto e tarefas em execução (`STATE.md` 2026-10-24: “NEXT: `/speckit.tasks`”).

## Restrições não negociáveis (constituição)

- Nenhuma variante `Other`/`Unknown` em `crates/*/src`; nenhum `_ =>` (gate 7).
- Nenhum `unwrap/expect/panic/todo` no código (lints `deny`); erro via `thiserror` + `tracing`.
- Nenhum segredo no repo (`.env`, `*.pem`, `*.key` gitignored); consentimento revogável por provider.
- `Tauri` fixo **2.12.1**; manifest editado diretamente (não via `mobile::update_manifest` que não existe).

---
*Criado: 2026-10-26 · Autor: processo speckit (assessment + specify + plan + tasks) · Fonte: `.specify/memory/constitution.md`, `AGENTS.md`, `specs/*/spec.md`*
