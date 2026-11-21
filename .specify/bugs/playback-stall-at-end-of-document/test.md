# Bug Verification: pending

- **Result**: not verified
- **Steps**: reset diag, ACTION_VIEW Hamlet with FLAG_GRANT_READ_URI_PERMISSION, observe >220 s, assert `action=complete` + `session_end` and position reaches timeline end.

## 2026-11-21 — VERIFICADO (ciclo 027)

- APK do merge #233 (`579591e8`) instalado no device 2201117TY.
- Diag zerado; `ACTION_SEND` com fixture curta (~300 bytes, 2 segmentos após primário) via `am start ... --es android.intent.extra.TEXT`.
- Após ~17,8 s de progresso: `transport_change` com `transport:"completed"` + `action=complete`, seguido de `session_end` com `transport:"completed"` e `session_end_reason:"completed"` — sinal canônico do schema emitido.
- Sem congelamento de posição; conclusão ocorreu no próprio tick em que o timeline terminou.
- **Resultado: PASS.** Bug fechado.
