# Bug Fix: Documento não carrega ao selecionar arquivo via Caixa de Título (SAF)

- **Slug**: play-desabilitado
- **Fixed**: 2026-11-10T19:17:00-04:00
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Corrigido o descarte prematuro do resultado do seletor SAF (`PickActivity` / DocumentsUI) ao tocar na caixa de título: `ShellApplication.onActivityPaused` não anula mais `currentActivity` prematuramente durante a transição com o seletor do sistema, `MainActivity.onActivityResult` agora passa seu próprio contexto de Activity explicitamente para o `ShellHost`, e `ShellHost.onActivityResult` processa o resultado de forma determinística mesmo durante transições de ciclo de vida.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `android-shell/src/main/kotlin/app/mdreader/ShellApplication.kt` | modified | Removida anulação de `currentActivity` no `onActivityPaused`; a referência agora só é limpa em `onActivityDestroyed`, preservando o contexto para entrega de `onActivityResult`. |
| `android-shell/src/main/kotlin/app/mdreader/ShellHost.kt` | modified | Adicionado overload `onActivityResult(activityContext: Activity?, requestCode: Int, resultCode: Int, data: Intent?)` com resolução prioritária do contexto local e log de observabilidade. |
| `src-tauri/gen/android/app/src/main/java/app/mdreader/MainActivity.kt` | modified | `onActivityResult` passa `this` diretamente para `ShellHost.onActivityResult(this, requestCode, resultCode, data)`. |

## Diff Highlights

```kotlin
// android-shell/src/main/kotlin/app/mdreader/ShellApplication.kt
         override fun onActivityPaused(activity: Activity) {
-            if (ShellHost.currentActivity === activity) {
-                ShellHost.currentActivity = null
-            }
+            // Do NOT null out currentActivity on pause: onActivityResult is dispatched
+            // to the calling Activity before onResume runs when returning from SAF picker
+            // (DocumentsUI) or system dialogs. Annulling here drops the result silently.
         }

+        override fun onActivityDestroyed(activity: Activity) {
+            if (ShellHost.currentActivity === activity) {
+                ShellHost.currentActivity = null
+            }
+        }
```

```kotlin
// android-shell/src/main/kotlin/app/mdreader/ShellHost.kt
+    @JvmStatic
+    fun onActivityResult(activityContext: Activity?, requestCode: Int, resultCode: Int, data: Intent?) {
+        val activity = activityContext ?: currentActivity
+        if (activity == null) {
+            Log.w(TAG, "onActivityResult: dropped result (requestCode=$requestCode, resultCode=$resultCode) because no Activity is attached")
+            return
+        }
+        Log.i(TAG, "onActivityResult: processing requestCode=$requestCode resultCode=$resultCode for ${activity.componentName?.shortClassName ?: activity.javaClass.simpleName}")
+        if (DocIngest.handleDirectoryResult(activity, requestCode, resultCode, data)) return
+        DocIngest.handlePickResult(activity, requestCode, resultCode, data)
+    }
```

```kotlin
// src-tauri/gen/android/app/src/main/java/app/mdreader/MainActivity.kt
   @Deprecated("Deprecated in Java")
   override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
     super.onActivityResult(requestCode, resultCode, data)
-    ShellHost.onActivityResult(requestCode, resultCode, data)
+    ShellHost.onActivityResult(this, requestCode, resultCode, data)
   }
```

## Local & Device Verification

- **Build Local (Gradle & Tauri Android)**:
  - `cd src-tauri/gen/android && ./gradlew test` → `BUILD SUCCESSFUL in 4m`.
  - `cargo tauri android build --target aarch64 --debug` → `Finished 1 APK at app-universal-debug.apk`.
- **Instalação no Device Físico (`192.168.100.102:39179`, 2201117TY)**:
  - `adb install -r app-universal-debug.apk` → `Success`.
- **Validação E2E com Seleção de Arquivo Real via SAF**:
  - Clique disparado na Caixa de Título via CDP (`#titleCard.click()`) → DocumentsUI aberto.
  - Seleção do arquivo `01-introducao.md` via `adb shell input tap 300 740`.
  - Logcat capturado:
    ```text
    ShellHost: onActivityResult: processing requestCode=19791 resultCode=-1 for .MainActivity
    DocIngest: Persistable URI permission granted
    DocIngest: Snapshot written: /data/user/0/app.mdreader/files/snapshots/1c00014c999d7eac361046432c01815cf923a22b30afc0e8fa0121ff42b0289a.md
    ```
  - Inspeção do disco do app via `run-as app.mdreader`:
    Arquivo `1c00014c999d7eac361046432c01815cf923a22b30afc0e8fa0121ff42b0289a.md` criado com 267 bytes.
  - Consulta ao DOM do WebView via CDP:
    ```json
    {
      "statusView": {
        "title": "Introdução ao Leitor Soberano",
        "section_title": "Introdução ao Leitor Soberano",
        "rate": 1,
        "transport": "completed",
        "position_secs": 0,
        "fingerprint": "1c00014c999d7eac361046432c01815cf923a22b30afc0e8fa0121ff42b0289a"
      },
      "playBtnAria": null,
      "playBtnClass": "secondary"
    }
    ```
    `#documentTitle` agora exibe perfeitamente `"Introdução ao Leitor Soberano"`, a ingestão Rust é completa e os controles refletem o estado do documento.

## Deviations from Assessment

Nenhuma. A remediação seguiu exatamente a proposta preferencial documentada em `assessment.md`.

## Follow-ups

- Executar `/speckit-bug-test slug=play-desabilitado` para validação formal de fechamento do ciclo.
