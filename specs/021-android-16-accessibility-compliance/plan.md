# Plan — Spec 021

## Objetivo

Verificar (não implementar migração de código) que o shell Android do Ressoa está em conformidade com Android 16 (API 36) no quesito acessibilidade: não há chamadas `announceForAccessibility`/`TYPE_ANNOUNCEMENT` para migrar; `targetSdk = 36` já está configurado; o frontend já usa `aria-live` e atributos ARIA.

## Tarefas

1. **Tarefa 1 — Verificar código Kotlin:** confirmar que nenhum dos 10 arquivos do `android-shell/` referencia `announceForAccessibility`, `TYPE_ANNOUNCEMENT`, `setAccessibilityPaneTitle` ou `setAccessibilityLiveRegion`. (Feito na pesquisa; registrar aqui como tarefa concluída.)
2. **Tarefa 2 — Verificar build:** confirmar `targetSdk = 36` e `compileSdk = 36` em `src-tauri/gen/android/app/build.gradle.kts`. (Feito.)
3. **Tarefa 3 — Verificar frontend:** confirmar `aria-live="polite"` e atributos ARIA em `src-tauri/src/status_view.html`. (Feito.)
4. **Tarefa 4 — Executar gates:** `cargo fmt`, `cargo clippy -D warnings`, `cargo nextest`, build Android (gate 9).
5. **Tarefa 5 — Registrar decisão:** spec.md documenta que não há código a migrar; conformidade já atendida.

## Critérios de aceitação

- `targetSdk = 36` em `build.gradle.kts` ✅
- Zero referências a `announceForAccessibility`/`TYPE_ANNOUNCEMENT` no shell Kotlin ✅
- Frontend mantém `aria-live` e ARIA ✅
- Gates 1–4 verde ✅
