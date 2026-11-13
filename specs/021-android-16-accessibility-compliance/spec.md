# Spec 021 — Android 16 (API 36) Accessibility Compliance

## 1. Contexto e Objetivo

Verificar a conformidade do shell Android do Ressoa com as exigências de acessibilidade do Android 16 (API 36), especificamente a migração de chamadas residuais de `announceForAccessibility`/`TYPE_ANNOUNCEMENT` para `setAccessibilityPaneTitle`/`setAccessibilityLiveRegion` (conforme ROADMAP.md, fase NEXT).

## 2. Estado Atual (Evidência)

### 2.1 Código Kotlin do shell android-shell/

Lidos e verificados todos os 10 arquivos Kotlin em `android-shell/src/main/kotlin/app/mdreader/`:
- `ShellHost.kt`
- `MediaMirror.kt`
- `Residency.kt`
- `FocusMirror.kt`
- `RoutingMirror.kt`
- `DocIngest.kt`
- `ShellApplication.kt`
- `PlaybackService.kt`
- `DebugIngestActivity.kt`
- `DebugIngestHook.kt`

**Resultado:** nenhuma chamada a `announceForAccessibility`, `TYPE_ANNOUNCEMENT`, `AccessibilityLiveRegion`, `AccessibilityPaneTitle`, `setAccessibilityPaneTitle` ou `setAccessibilityLiveRegion` existe em nenhum dos arquivos. O shell Kotlin atua como espelho estrito (`SC-035`) — toda lógica de playback/transporte/posição/taxa vive no Rust; o Kotlin apenas renderiza o JSON publicado. Não há lógica de anúncio de acessibilidade local a migrar.

### 2.2 Manifesto Android e build

`src-tauri/gen/android/app/src/main/AndroidManifest.xml`:
- Permissões declaradas: `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `POST_NOTIFICATIONS`, `WAKE_LOCK`, `VIBRATE`. Sem `INTERNET` (air-gapped, `T106`).
- `targetSdkVersion`: não declarado explicitamente no manifest gerado (usa valor padrão do build).

`src-tauri/gen/android/app/build.gradle.kts`:
- `compileSdk = 36` (linha 18)
- `minSdk = 29` (linha 23)
- `targetSdk = 36` (linha 24) — **já está no Android 16**

### 2.3 Frontend (status_view.html)

- Usa `aria-live="polite"` na `liveRegion` (linha 499)
- Usa `aria-label`, `aria-labelledby`, `role="button"`, `tabindex="0"`, `role="status"`, `aria-modal` — superfície acessível já implementada
- TalkBack 10/10 atestado (Spec 005 convergida)

### 2.4 Especificações e bugs existentes

- `.specify/assessments/`: 22 itens; nenhum trata de migração `announceForAccessibility`/`API 36`/`Android 16`. Itens relacionados: `android-accessibility-apps-study`, `android-accessibility-apps-study-2` (estudo competitivo de apps de acessibilidade, sem migração de API).
- `.specify/bugs/`: 24 itens; todos verificados/fechados; nenhum trata de migração de API de acessibilidade.
- `specs/`: 001–020; nenhuma spec trata de Android 16 compliance de acessibilidade.

## 3. Decisão

**Não há chamadas `announceForAccessibility`/`TYPE_ANNOUNCEMENT` para migrar.** O requisito do ROADMAP.md (fase NEXT, item 1) já está satisfeito:
- `targetSdk = 36` já configurado
- Não há lógica de anúncio de acessibilidade no shell Kotlin
- A superfície frontend já usa `aria-live` e atributos ARIA adequados
- TalkBack 10/10 atestado (Spec 005)

## 4. Critérios de Aceitação

- [ ] `targetSdk = 36` confirmado no `build.gradle.kts`
- [ ] Zero referências a `announceForAccessibility`/`TYPE_ANNOUNCEMENT` no shell Kotlin
- [ ] Frontend mantém `aria-live` e atributos ARIA na superfície
- [ ] Gate 9 (build Android debug) passa com `targetSdk 36`

## 5. Escopo Fora

- Não se cria novo código de migração (não há o que migrar)
- Não se altera o manifesto (já usa targetSdk 36 via build.gradle.kts)
- Não se cria novo bug (não há defeito); este documento serve de registro de verificação
