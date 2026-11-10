# Bug Assessment: Play desabilitado / Documento não carrega ao selecionar arquivo via Caixa de Título (SAF)

- **Slug**: play-desabilitado
- **Updated**: 2026-11-10T19:05:00-04:00 (device live test 2201117TY via ADB Wi-Fi 192.168.100.102:39179)
- **Source**: relato do usuário ("Continua não abrindo arquivo ao clicar na Caixa de Título") + triagem em device físico com automação via Chrome DevTools Protocol (CDP) e ADB.
- **Verdict**: valid (reproduzido em device e causa raiz exata identificada na borda Android/Kotlin/SAF)
- **Severity**: critical

## Report (verbatim or summarized)

> **User report**: Continua não abrindo arquivo ao clicar na "Caixa de Título".
>
> **Sintoma histórico**: Play desabilitado (`btnPlayPause` desabilitado, console repetindo `Invoke get_status_view failed: no document loaded`, `#documentTitle` vazio).
>
> **Sintoma atual investigado**: O usuário clica na caixa de título ("Selecionar documento para leitura"). A intenção nativa SAF é disparada e a interface do DocumentsUI (`PickActivity`) é exibida na tela. No entanto, ao tocar em um arquivo válido (`01-introducao.md`), a janela do seletor fecha, a `MainActivity` volta ao foco, mas **o arquivo selecionado nunca é ingerido pelo app**: `#documentTitle` continua vazio, nenhum áudio é carregado, nenhum snapshot é gravado em `files/snapshots/` e os controles de reprodução permanecem desabilitados com o WebView registrando `no document loaded`.

---

## Symptom

1. **Abertura do Seletor**: O clique no `#titleCard` dispara com sucesso o evento `pick_document` via Tauri bridge, chamando `ShellHost.pickDocument()` e abrindo o Android Storage Access Framework (`ACTION_OPEN_DOCUMENT`).
2. **Seleção de Arquivo Inócua**: Quando o usuário seleciona um arquivo Markdown ou Texto (`01-introducao.md`) no DocumentsUI:
   - O DocumentsUI fecha normalmente com `RESULT_OK` e o app `app.mdreader` retorna ao primeiro plano (`mCurrentFocus=MainActivity`).
   - **Observado**: Absolutamente nada acontece. O título `#documentTitle` permanece em branco, o estado de transporte permanece `none`, nenhum snapshot é criado em `/data/data/app.mdreader/files/snapshots/`, `DocIngest.handlePickResult` **não executa**, e o Rust nunca recebe `onDocumentIngested`.
   - **Esperado**: `onActivityResult` deve interceptar o URI do arquivo escolhido, gravar o snapshot em cache/disco, acionar o JNI `onDocumentIngested`, e o motor Rust deve carregar o documento, calculando fingerprint e iniciando a reprodução ou oferecendo retomar a partir de bookmark existente.

---

## Reproduction & Live Device Evidence

Testado ao vivo no dispositivo de referência **Xiaomi Redmi Note 11 (2201117TY, Android 13, API 33)** conectado via ADB Wi-Fi (`192.168.100.102:39179`):

1. **Disparo do Seletor via CDP**:
   - Conexão estabelecida via WebSocket CDP no socket Unix `@webview_devtools_remote_5675`.
   - Executado `document.getElementById('titleCard').click()`.
   - Logcat e `dumpsys window` confirmaram a abertura imediata do DocumentsUI:
     `mCurrentFocus=Window{53936e9 u0 com.google.android.documentsui/com.android.documentsui.picker.PickActivity}`.
2. **Seleção do Arquivo de Teste**:
   - Disparado toque nas coordenadas de `01-introducao.md` (`bounds [198,715][484,764]`).
   - DocumentsUI processou a seleção e finalizou, devolvendo o foco para `app.mdreader.MainActivity`.
3. **Verificação pós-seleção**:
   - Dump da árvore DOM via CDP revelou:
     `{"titleText":"","currentTransport":"none"}`.
   - Consulta ao diretório interno do app (`run-as app.mdreader ls -la files/snapshots`):
     `files/snapshots: No such file or directory`.
   - Análise do Logcat completo:
     - Nenhuma mensagem de `DocIngest` (`handlePickResult`, `Snapshot written`, ou erro).
     - Nenhuma mensagem de JNI `onDocumentIngested`.
     - WebView em polling emitindo continuamente:
       `Invoke get_status_view failed: no document loaded`.

---

## Root Cause Analysis

### Causa Raiz Primária: Interceptação no Ciclo de Vida da `Activity` (`currentActivity == null` no retorno de SAF)

No arquivo `android-shell/src/main/kotlin/app/mdreader/ShellHost.kt`:
```kotlin
@JvmStatic
fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
    val activity = currentActivity ?: return
    if (DocIngest.handleDirectoryResult(activity, requestCode, resultCode, data)) return
    DocIngest.handlePickResult(activity, requestCode, resultCode, data)
}
```
E no arquivo `android-shell/src/main/kotlin/app/mdreader/ShellApplication.kt`:
```kotlin
override fun onActivityPaused(activity: Activity) {
    if (ShellHost.currentActivity === activity) {
        ShellHost.currentActivity = null
    }
}
```

### O Mecanismo da Falha:
1. Quando a `MainActivity` dispara `startActivityForResult(intent, PICK_DOCUMENT_REQUEST)`, o sistema Android coloca o `DocumentsUI` em primeiro plano.
2. A `MainActivity` sofre o callback de ciclo de vida **`onActivityPaused(activity)`**.
3. O `ShellApplication.onActivityPaused` executa:
   `ShellHost.currentActivity = null`.
4. Quando o usuário toca no arquivo no `DocumentsUI`, o Android finaliza o picker e chama **`MainActivity.onActivityResult(requestCode, resultCode, data)`**.
5. No ciclo de vida do Android, **`onActivityResult` é chamado ANTES de `onResume`**!
6. Quando `MainActivity.onActivityResult` encaminha para `ShellHost.onActivityResult(...)`:
   `val activity = currentActivity ?: return`
   Como a Activity ainda está pausada e `onResume()` ainda não foi disparado, **`ShellHost.currentActivity` é `null`**!
7. O método simplesmente dá um **`return` silencioso**, descartando completamente o resultado do SAF picker (`data.data` contendo a URI do arquivo).
8. Segundos depois, `MainActivity.onResume()` roda e restaura `currentActivity`, mas o resultado da seleção já foi irremediavelmente perdido no vácuo.

---

## Proposed Remediation

### 1. Correção Definitiva no Ciclo de Vida (`ShellApplication.kt` e `MainActivity.kt`)
- **Não anular `currentActivity` em `onActivityPaused`**: A Activity ainda está viva e é o contexto receptor válido de resultados pendentes (`onActivityResult`). O reset para `null` deve ocorrer em `onActivityDestroyed` ou `onActivityStopped` apenas se não houver instância ativa.
- **Passar a instância receptora explicitamente**:
  Em `MainActivity.kt`:
  ```kotlin
  @Deprecated("Deprecated in Java")
  override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
    super.onActivityResult(requestCode, resultCode, data)
    ShellHost.handleActivityResult(this, requestCode, resultCode, data)
  }
  ```
  Permitindo que a própria Activity que recebeu o callback seja utilizada para obter o `contentResolver`, sem depender do ponteiro estático volátil `currentActivity` que pode estar em transição de lifecycle.

### 2. Observabilidade e Logs
- Adicionar logs explícitos em `ShellHost.onActivityResult` / `ShellHost.handleActivityResult` com o `requestCode`, `resultCode` e `uri`.
- Se o `requestCode` for rejeitado ou a URI for nula, logar em nível `WARN`.

---

## Action Plan

1. **Fix (`/speckit.bug.fix`)**:
   - Ajustar `MainActivity.kt` para passar `this` como Activity contextual para o `ShellHost`.
   - Ajustar `ShellHost.kt` para aceitar `activity: Activity` ou usar fallback confiável.
   - Ajustar `ShellApplication.kt` para não anular a referência da activity enquanto ela estiver viva (limpar somente em `onActivityDestroyed`).
2. **Build & Deploy**:
   - Compilar `cargo tauri android build --target aarch64 --debug`.
   - Instalar no dispositivo de referência (`adb install -r`).
3. **Teste de Verificação (`/speckit.bug.test`)**:
   - Disparar clique no `#titleCard`.
   - Tocar no arquivo `01-introducao.md`.
   - Validar logcat: `DocIngest: Snapshot written`, `JNI onDocumentIngested returned 0`.
   - Validar UI: `#documentTitle` exibindo o nome do documento e áudio iniciando.
