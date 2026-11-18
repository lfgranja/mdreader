# Checklist de Requisitos — 019-intent-filter-ingress

- [x] Declaração de `ACTION_VIEW` com MIME `text/markdown`, `text/plain`, `text/x-markdown` no AndroidManifest
- [x] Declaração de `ACTION_SEND` com suporte a texto puro e anexos no AndroidManifest
- [x] Captura de Intent em `onCreate` e `onNewIntent` na `MainActivity`
- [x] Resolução segura de `InputStream` via `ContentResolver` com try-with-resources / `.use`
- [x] Fallback resiliente de nome de documento quando `DISPLAY_NAME` não estiver disponível
- [x] Encaminhamento de bytes para JNI sem processamento de playback em Kotlin
- [x] Verificação de integridade dos Gates 1 a 8 (`./scripts/verify.sh`)
