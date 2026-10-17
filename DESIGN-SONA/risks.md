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

**O que é:** adicionar cores arbitrárias fora da paleta homologada viola o Princípio VII; o verde na interface do player é terminantemente proibido.

**Mitigação:** a introdução do token de marca `--sona-aria` (`#5AD6A8`) é restrita e confinada à **camada de identidade (Onda 2)**; no player e na notificação, a ausência de verde é mantida como invariante. Qualquer nova cor requer edição coordenada + bump semântico + docs.

**Verificação:** code review de design antes de cada commit; validação de tokens em `tokens.md`.

**Referência:** `constitution.md` §VII; `briefing-logo-ressoa-design-sona.md`; `DESIGN-SONA/tokens.md`.

---

## R7 — Light mode não especificado

**O que é:** apenas dark (`#0B0F1A`) está definido. Se usuário ativar tema claro, a paleta pode falhar (amarelo/vermelho podem perder contraste com fundo claro).

**Mitigação:** não implementar light mode até que seja especificado; se implementado, requer re-verificação completa de todos os contrastes.

**Referência:** `DESIGN.md` §6 (variantes: apenas dark documentado; light = não-especificado, com pré-requisitos listados).

---

## R8 — Throttling / Performance do pulso amarelo

**O que é:** ponto amarelo pulsante (alpha 1.2 s) pode ser throttled pelo Android se o app está em background; pode consumir bateria se re-postado frequentemente.

**Mitigação:** default OFF na notificação; só in-app; usar `ObjectAnimator` com `repeatMode=REVERSE` (não loop infinito); cessa quando TTL = 0 (estado `reason_expired` do Rust); não usar `RemoteViews` animation (não suportado confiável).

**Referência:** `DESIGN/components.md` §6.

---

## R9 — Risco de Confusão de Marca com LoL (Sona)

**O que é:** O lore e a sinestesia musical de Sona são fontes conceituais. Se a logo evocasse fantasia medieval de videogame, a credibilidade técnica como leitor de documentos em Rust seria comprometida.

**Mitigação:** A marca pública é **Ressoa** (`ressona.dev`), não Sona. O logotipo utiliza geometria pura de ondas senoidais de Fourier, arcos matemáticos e sinalética inspirada na HfG Ulm / Otl Aicher, sem adereços figurativos ou estéticas de entretenimento gamer.

**Verificação:** Homologação visual contra o checklist de anti-padrões do briefing v2.0.0.

---

## R10 — Fusão das Três Ondas em Baixa Resolução (16px)

**O que é:** Em favicons ou chips de status in-app (16×16 px), três curvas coloridas próximas poderiam fundir-se em uma massa borrada de pixels.

**Mitigação:** Cada onda tem espessura de 4.5 unidades vetoriais com **espaçamento negativo (*gap*) de 3.5 unidades** entre elas. Isso garante que mesmo em 16px haja uma linha nítida de separação física entre as curvas.

**Verificação:** Teste de rasterização 16×16 px e 32×32 px com inspeção visual pixel a pixel.

---

## R11 — Violação da Inviolabilidade da Notificação Android (Lock-Screen)

**O que é:** Tentação de exibir o logotipo com as ondas coloridas na notificação `MediaStyle` do Android, violando o princípio FR-028/FR-029 de respeitar a atenção e não roubar espaço do título do documento.

**Mitigação:** O componente `NotificationSurface` proíbe expressamente a presença do logotipo da marca. A logo vive apenas no cabeçalho in-app da `StatusView`, no ícone do lançador e na web.

**Verificação:** Gate de verificação em dispositivo real (Wave B / Wave C).

---

## R12 — Degradação no Android Material You (Themed Icons Monocromático)

**O que é:** Em launchers modernos (Android 13+), o usuário pode ativar ícones temáticos baseados no wallpaper, descartando as cores azul, verde e roxo. Se a distinção do logo dependesse exclusivamente da cor, o símbolo perderia o sentido.

**Mitigação:** O asset `ic_launcher_monochrome.xml` utiliza recortes vetoriais transparentes com espaçamento de respiro. As três ondas são identificáveis pela sua morfologia e posição espacial, garantindo que o ícone funcione com 100% de inteligibilidade em silhueta plana 1-bit.

**Verificação:** Inspeção do Vector Drawable compilado no emulador ou dispositivo com tema dinâmico ativado.

---

*Nota:* todos os riscos são **documentados, não eliminados**. O design SONA v2.0.0 aceita esses riscos como restrições do produto e os mitiga por disciplina (conjunto fechado, MediaStyle padrão, separação de camadas Marca vs. Player, gaps matemáticos de onda e acessibilidade eyes-free).
