# DESIGN/risks.md — Riscos e Limitações do Design B (Sinalização)

**Versão:** 1.0.0 · **Data:** 2026-10-06 · **Status:** especificado, não eliminado
> Todo risco aqui é um risco **documentado**, não escondido. Cada um tem uma mitigação, uma verificação, e uma condição de falha. Nenhum é "corrigido depois" — todos são tratados como restrições de projeto.

---

## R1 — SC-029 (texto rolante / marquee) — FAIL declarado

**O que é:** a cláusula de FR-041/SC-029 exige que o motivo de pausa seja "renderizado como moving text". Na notificação Android MediaStyle, isso é impossível (RemoteViews sem `setSelected`, OEM truncam, Live Updates proíbem custom view).

**Mitigação:** (1) in-app: scroll real (TextView com foco); (2) notificação: reason estática curta + ponto amarelo pulsante + earcon + TTL 3s; (3) documentação explícita no spec.

**Verificação:** trial blind-test (SC-008) — se qualquer efeito não confirmado audivelmente, falha.

**Condição de falha:** se a emenda de spec não for aceita e o trial exige marquee real — o candidato B falha nessa cláusula. Aceitação formal de FAIL parcial é a saída honesta.

**Referência:** `DESIGN/components.md` §1; `DESIGN_SYSTEMS_5_CANDIDATOS.md` §B-6; `spec.md` SC-029/FR-041.

---

## R2 — OEM relayout (Samsung One UI 7, Xiaomi HyperOS, Pixel)

**O que é:** OEMs reescrevem layouts de notificação; HyperOS/One UI 7 adotaram split por padrão; HyperOS não oferece volta.

**Mitigação:** usar `NotificationCompat.MediaStyle` padrão (não custom RemoteViews extensos); estado codificado por **forma + posição**, não apenas cor; progresso como barra (forma), não como cor de preenchimento única.

**Verificação:** matrix obrigatória — Pixel (stock), Samsung (One UI), Xiaomi (HyperOS). Trials: 10/10 (lock-screen, focus, routing, ≤3 s play-to-audio, failover).

**Condição de falha:** se o OEM recolorir o amarelo/vermelho de forma que se confunda com outra função (ex.: amarelo = info, vermelho = sucesso); solução = a forma (ponto vs barra vs posição) é independente da cor.

**Referência:** `VERIFICATION.md` 184 (Xiaomi, Android 13, API 33); `research.md` §3 (OEM relayout); `DESIGN/components.md` §4.

---

## R3 — Android 13+ PlaybackState deriva controles (5 slots compactos)

**O que é:** a notificação deriva ações do `PlaybackState`, não da ordem da lista. 6 controles exigidos (FR-018) vs 5 slots compactos — Stop pode ser só expandido.

**Mitigação:** declarar na documentação que Stop pode não estar no compacto; não inventar uma 6ª ação; usar `setShowActionsInCompactView()` corretamente.

**Verificação:** trial de notificação em todos os 3 OEMs; confirmar que play/pause/prev/next estão sempre visíveis; confirmar que skip ±5 s funciona.

**Referência:** `spec.md` FR-018/FR-018b; `developer.android.com` (Android 13 behavior changes).

---

## R4 — Android 16 Live Updates (proibição `customContentView`)

**O que é:** notificações promovidas (Live Updates) exigem estilo permitido (`ProgressStyle`, `Standard`, etc.) e **proíbem** `customContentView`. `MediaStyle` está fora da lista de elegíveis.

**Mitigação:** não usar Live Update para media playback — usar `MediaStyle` padrão com `FLAG_ONGOING_EVENT`. Não tentar promover a notificação como Live Update.

**Verificação:** verificar `Notification.Builder.setRequestPromotedOngoing()` — não usar para media playback.

**Referência:** `developer.android.com` Android 16 docs; `DESIGN_COMPONENTS.md` §4.

---

## R5 — minSdk API 29 vs manifesto gerado 24

**O que é:** `spec.md`/`tasks.md` exigem `minSdk = 29`; manifesto gerado (`src-tauri/gen/android/app/src/main/AndroidManifest.xml`) tem `minSdkVersion="24"`.

**Mitigação:** corrigir manifesto antes de distribuição; verificar por `scripts/check-manifest.sh`; gate 9 (`cargo tauri android build --target aarch64`) falha se manifesto estiver errado.

**Verificação:** `check-manifest.sh`; `tasks.md` linhas 93/228.

**Referência:** `spec.md` SC-032 (`spec.md:1313-1316`); `AGENTS.md`; `VERIFICATION.md`.

---

## R6 — Conjunto fechado de cores (Princípio VII da constituição)

**O que é:** adicionar amarelo a outra função, ou vermelho a outro estado, ou verde como "repouso" = violação do conjunto fechado; requer edição coordenada + bump semântico + docs.

**Mitigação:** documentar a regra no token spec (`DESIGN/tokens.md` §4); verificar em review que nenhuma cor é usada fora de sua função.

**Verificação:** review de design antes de cada commit; `clippy` + `cargo deny` não pegam isso — é regra de design, não de código.

**Referência:** `constitution.md` §VII; `DESIGN.md` §3; `DESIGN/tokens.md` §4.

---

## R7 — Light mode não especificado

**O que é:** apenas dark (`#0A0A0A`) está definido. Se usuário ativar tema claro, a paleta pode falhar (amarelo/vermelho podem perder contraste com fundo claro).

**Mitigação:** não implementar light mode até que seja especificado; se implementado, requer re-verificação completa de todos os contrastes.

**Referência:** `DESIGN.md` §6 (variantes: apenas dark documentado; light = não-especificado, com pré-requisitos listados).

---

## R8 — Throttling / Performance do pulso amarelo

**O que é:** ponto amarelo pulsante (alpha 1.2 s) pode ser throttled pelo Android se o app está em background; pode consumir bateria se re-postado frequentemente.

**Mitigação:** default OFF na notificação; só in-app; usar `ObjectAnimator` com `repeatMode=REVERSE` (não loop infinito); cessa quando TTL = 0 (estado `reason_expired` do Rust); não usar `RemoteViews` animation (não suportado confiável).

**Referência:** `DESIGN/components.md` §6.

---

*Nota:* todos os riscos são **documentados, não eliminados**. O design B aceita esses riscos como restrições do produto e os mitiga por disciplina (conjunto fechado, MediaStyle padrão, nenhum custom RemoteViews, nenhum green, nenhum marquee inventado, acessibilidade eyes-free obrigatória). A decisão de aceitar cada risco é parte do design — não é uma falha que o usuário pode corrigir depois.
