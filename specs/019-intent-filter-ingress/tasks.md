# Tasks — 019-intent-filter-ingress

- [x] **T001**: Atualizar `src-tauri/gen/android/app/src/main/AndroidManifest.xml` adicionando os intent filters para `ACTION_VIEW` e `ACTION_SEND`
- [x] **T002**: Implementar `handleIncomingIntent` em `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` para resolver URIs de `ACTION_VIEW` e dados de `ACTION_SEND`
- [x] **T003**: Conectar o tratamento de intents em `ShellHost.kt` / `MainActivity.kt` no ciclo de vida (`onCreate` e `onNewIntent`)
- [x] **T004**: Executar os Gates 1 a 8 de `./scripts/verify.sh` para assegurar que a pureza do shell, lints e testes continuam passando
- [x] **T005**: Atualizar `.specify/memory/STATE.md` e convergir os artefatos da especificação
