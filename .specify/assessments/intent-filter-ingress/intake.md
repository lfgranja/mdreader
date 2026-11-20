# Idea Intake: Android Intent Filter Ingress (ACTION_VIEW & ACTION_SEND)

- **Slug**: intent-filter-ingress
- **Created**: 2026-11-11
- **Source**: codebase analysis and GTM research
- **Type**: new-capability

## Idea (as captured)

> "Suporte a Android Intent Filter ACTION_VIEW e ACTION_SEND para abrir arquivos Markdown e notas compartilhadas do Obsidian/Files diretamente no Ressoa"

## Restated

Permitir que o aplicativo Ressoa apareça nativamente no menu "Abrir com..." e na folha de compartilhamento (Share Sheet) do Android ao tocar ou compartilhar arquivos Markdown (`.md`, `text/markdown`, `text/plain`) a partir de gerenciadores de arquivos ou aplicativos como Obsidian e Logseq, iniciando imediatamente a ingestão e reprodução no motor Rust.

## Origin & Context

Identificado durante a análise de alavancagem 80/20 do projeto. Atualmente, o Ressoa requer navegação manual profunda via SAF (`ACTION_OPEN_DOCUMENT`) dentro do app. Os usuários de PKMS e anotações técnicas mantêm seus arquivos no Obsidian/Logseq e esperam abri-los diretamente a partir de seus editores ou do gerenciador de arquivos do sistema. O `AndroidManifest.xml` atual contém apenas `MAIN`/`LAUNCHER`.

## First-Glance Unknowns

- Como diferenciar `ACTION_VIEW` (abertura direta de arquivo via URI) de `ACTION_SEND` (compartilhamento de texto puro ou stream via `EXTRA_STREAM`) no ciclo de vida da `MainActivity`?
- Como repassar a URI recebida na Intent para o motor Rust respeitando as permissões de leitura temporárias (`FLAG_GRANT_READ_URI_PERMISSION`) do Android?
- Qual o comportamento esperado se o Ressoa já estiver tocando outro arquivo quando uma nova Intent chegar (`onNewIntent` com `singleTask`)?
