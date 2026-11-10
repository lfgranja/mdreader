# Bug Verification: Documento não carrega ao selecionar arquivo via Caixa de Título (SAF)

- **Slug**: play-desabilitado
- **Tested**: 2026-11-10T19:22:00-04:00
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

O bug foi testado e validado de ponta a ponta no dispositivo de referência físico Xiaomi Redmi Note 11 (2201117TY, Android 13). A seleção de arquivos Markdown via SAF (`PickActivity` / DocumentsUI) ao tocar na Caixa de Título (`#titleCard`) agora processa `onActivityResult` de forma determinística, gravando snapshots, ingerindo o documento no motor Rust, atualizando `#documentTitle`, habilitando controles e eliminando os erros repetidos de `Invoke get_status_view failed: no document loaded`.

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reprodução pós-fix (SAF Picker Real) | Disparo via `#titleCard.click()` e seleção de `01-introducao.md` e `02-arquitetura.md` no DocumentsUI | pass | `ShellHost.onActivityResult` processou `requestCode=19791 resultCode=-1`, snapshot gravado e ingerido no Rust. |
| Integridade dos Snapshots em Disco | `run-as app.mdreader ls -la files/snapshots` | pass | Snapshot gravado com trim automático do anterior (`f26ea757...md` com 204 bytes). |
| Atualização do DOM e Estado Rust | Consulta via Chrome DevTools Protocol (`get_status_view`) | pass | `#documentTitle` reflete `"Arquitetura e Engenharia de Áudio"`; `get_status_view` retorna metadados completos sem erros de "no document loaded". |
| Compilação e Testes Android | `cd src-tauri/gen/android && ./gradlew test` | pass | 141 tarefas executadas com sucesso (`BUILD SUCCESSFUL in 4m`). |
| Build e Deploy de APK | `cargo tauri android build --target aarch64 --debug` && `adb install -r` | pass | APK universal compilado e instalado sem falhas. |

## Output Excerpts

### 1. Logcat de Confirmação no Android Shell:
```text
11-10 19:21:19.337 11202 11202 I ShellHost: onActivityResult: processing requestCode=19791 resultCode=-1 for .MainActivity
11-10 19:21:19.339 11202 11202 D DocIngest: Persistable URI permission granted
11-10 19:21:19.411 11202 11202 D DocIngest: Snapshot written: /data/user/0/app.mdreader/files/snapshots/f26ea757124c2cb251891f4c1fb44fc4af6f4eb98e277a1957eb6428cb65d689.md
11-10 19:21:19.412 11202 11202 D DocIngest: Trimmed old snapshot: 1c00014c999d7eac361046432c01815cf923a22b30afc0e8fa0121ff42b0289a.md
```

### 2. Estado do WebView via Chrome DevTools Protocol (CDP):
```json
{
  "titleText": "Arquitetura e Engenharia de Áudio",
  "statusView": {
    "title": "Arquitetura e Engenharia de Áudio",
    "section_title": "Arquitetura e Engenharia de Áudio",
    "rate": 1,
    "transport": "completed",
    "position_secs": 0,
    "reason": null,
    "reason_expiry": null,
    "fingerprint": "f26ea757124c2cb251891f4c1fb44fc4af6f4eb98e277a1957eb6428cb65d689"
  }
}
```

## Residual Risks

- Nenhum risco residual identificado. O ciclo de vida do Android agora trata transições do SAF sem perda de contexto da Activity receptora.

## Recommendation

Close the bug — verified end-to-end no dispositivo de referência físico Android com documentos reais.
