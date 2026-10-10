# DESIGN/components.md — Componentes B (Sinalização)

**Versão:** 1.0.0 · **Data:** 2026-10-06 · **Status:** especificado (escopo fixado pelo mestre `DESIGN.md`)

> Normativas aplicáveis a cada componente: `constitution.md` §VII (conjunto fechado), `spec.md` FR-028/FR-041/SC-029/SC-035, `VERIFICATION.md` (gate 9: build aarch64 + trials 10/10).

---

## 1. NotificationSurface (MediaStyle — superfície real)

**Não é um componente web — é um contrato de conteúdo:** a notificação Android é renderizada pela template padrão do sistema (`NotificationCompat.MediaStyle`), nunca por `RemoteViews` customizado extensivo.

### Elementos permitidos (conjunto fechado)

| Elemento | Token / Fonte | Regra | Fonte / Justificativa |
|---|---|---|---|
| **Título documento** | `--md-ink` (`#F4F6F8`) sobre `#0A0A0A` | Metadado de documento (FR-029); fallback = nome de arquivo estável; **nunca** primeira frase falada (SC-030) | `spec.md:922-927`; `FR-029`; `SC-030` |
| **Seção atual** | `--md-fog` (`#A7B0BA`) | Título do heading mais recente já passado; **vazio** antes do 1º heading / sem headings (FR-028c/SC-030); **limpa** ao parar | `spec.md:883-902`; `FR-028c` |
| **Progresso** | `--md-paint` (`#FFFFFF`) fill sobre `--md-track` (`#5D5D5D`) | **Determinado** sse total conhecido **no início da session**; **indeterminado** = ciclo parcial; **nunca** estimado; **nunca** muda de determinância na session (FR-028a/SC-042) | `spec.md:874-882`; `FR-028a` |
| **Estado** | **Geometria + posição + cor** — nunca só cor | Fluxo = barra cheia + posição; Caution-3s = barra congelada + ponto amarelo + texto curto; Prohibit = barra congelada + ponto vermelho + texto curto | `spec.md:903-921`; `FR-028e`; `FR-037` |
| **Controles** | Plataforma (`PlaybackState` derivado, API 33+) | 5 slots compactos: Play/Pause/Prev/Next/Stop (último só expandido); *skip* = ±5 s exato (FR-018); *rate/voice* **NÃO** aparecem (SC-027) | `spec.md:699-718`; `FR-018`; `SC-027` |
| **Motivo de pausa (transiente)** | `--md-warning` (`#FFD400`) + texto estável ≤ 40 chars pt-BR | **Não rola** — motivo estático curto; ponto amarelo pulsante (in-app); auto-dismiss 3 s; **não** é teleprompter (SC-029: scroll falha) | `spec.md:679-695`; `FR-041`; `SC-029` |
| **Motivo de falha (permanente)** | `--md-prohibit` (`#FF4D63`) + texto estável | Persistente (não expira em 3 s, diferente do motivo de pausa); documentado em `FR-037`/`FR-037a` | `spec.md:802-822`; `FR-037` |

### Elementos proibidos (hard — nunca aparecem)

- **Body text do documento** (FR-028: nem sentença em progresso, nem passagem anterior/posterior).
- **Elapsed / remaining time** (FR-028: "wrong number worse than none" — se total não conhecido, nenhum número). 
- **Rate / voz** (SC-027: só aparece in-app; `spec.md:41`: "current rate is shown inside the application but never on the lock screen").
- **Progressão estimada / projetada** (FR-028a: "A estimated, projected, or guessed total MUST NOT be displayed").
- **Marquee / texto rolante** (SC-029: impossível sob RemoteViews; solução = reason estática + earcon).
- **Progressão como porcentagem** (FR-028b: progress only — não percent-encoded; seção atual é referência de posição, não número).

### Template padrão (não custom)

```kotlin
// Android 13+: PlaybackState deriva ações; não lista de ações
NotificationCompat.Builder(context, CHANNEL_ID)
  .setStyle(MediaStyle().setMediaSession(mediaSession.sessionToken))
  .setVisibility(NotificationCompat.VISIBILITY_PUBLIC) // FR-028b
  .setOngoing(true) // FR-028e (não-dismissable enquanto playing)
  .setContentTitle(docTitle)    // FR-029
  .setContentText(sectionTitle) // FR-028c
  .setProgress(totalKnown ? totalMs : 0, positionMs, !totalKnown) // FR-028a
  // NÃO setCustomContentView (Live Updates proíbe; SC-029 não atende)
  .setSubText(pauseReasonStatic) // FR-041 — estático, ≤40 chars
  .addAction(playAction) // PlaybackState-derived
```

> **Nota técnica:** `PlayerNotificationManager` (Media3) oferece `MediaDescriptionAdapter` para título/seção, mas a **template padrão** é a única garantida em todos os OEM. Custom `RemoteViews` é permitido até API 31, mas Android 12+ reduz altura max (48 dp recolhido, 88 dp heads-up, 252 dp expandido) e não expõe `setSelected` para marquee. **Regra de B:** nunca dependemos de custom view para função crítica.

---

## 2. StatusView (in-app — plan.md:214-220)

**Única tela in-app no escopo** (não há settings, não há library, não há picker). Renderiza apenas as strings do Rust.

### Layout — grade 2-campo (Müller-Brockmann, adaptado)

```
+----------------------------------------------------------+
| [Campo esquerdo ~62%]     | [Campo direito ~38%]       |
| Título doc                |  Taxa (rate) — só in-app   |
| Seção atual               |  Transport (▶/⏸/■)         |
| Progresso (barra monocrom) |  Razão de pausa (scroll)  |
| (barra posição, não %)     |  (dot amarelo se 3s)       |
+----------------------------------------------------------+
```

**Regras:**
- **Grade 4-dp** (M3): gutter 24 dp, margens 16 dp, baseline 4 dp.
- **Tipografia:** Roboto Flex; título doc `20sp/500`; seção `14sp/400`; taxa `16sp/600` (números tabulares); razão `14sp/400`.
- **Progress bar:** trilho `#5D5D5D` 4 dp, preenchimento `#FFFFFF`; **sem animação de preenchimento** — posição é autoridade do Rust; transição de estado apenas (opacidade ≤ 150 ms); `prefers-reduced-motion` trivial.
- **Razão de pausa (SC-029 atendido aqui):** `TextView` com `android:ellipsize="marquee"` + `android:selected="true"` + `marqueeRepeatLimit="marquee_forever"` + `scrollHorizontally="true"`. **Só funciona in-app** porque a view tem foco e é renderizada pelo WebView/Svelte, não pelo sistema de notificação. Quando razão expira (TTL 3 s), texto volta ao estado estático; ponto amarelo some.
- **Dot amarelo pulsante:** `ObjectAnimator` alpha 0.2 ↔ 1.0, período 1.2 s, `ease-in-out`; cessa quando TTL = 0 (estado `reason_expired` do Rust). **Nunca** anima se `prefers-reduced-motion` estiver ativo.
- **Taxa:** escada fechada `[0.75, 1.0, 1.25, 1.50, 1.75, 2.0, 4.0]`; rung ativo `#FFD400`; inativos `#A7B0BA`; **nunca** aparece na notificação (SC-027).

---

## 3. RateView / FocusMirror / Residencies (referência ao spec, não componentes visuais adicionais)

- **Rate:** componente de controle — stepper 48 dp, uma pressão = uma rung, saturação nas extremidades. **Não é um componente visual novo** — é o mesmo texto de taxa com interação.
- **FocusMirror / RoutingMirror:** não têm componente visual próprio — apenas estados do `PlaybackState` (Ganho, Perda, Temporário, Permanente). Os estados são refletidos pela **cor do ponto / cor da barra / texto da razão** — nunca por um widget separado.
- **Residency:** não é uma tela — é o comportamento do `Service` (continuar enquanto resumável; teardown no fim do documento). Não há UI de "residente" além do que a notificação já mostra (paused + resume control).

---

## 4. Notas de engenharia (não design, mas vinculadas)

- **Earcon:** sintetizado pelo engine Rust (`crates/engine-core`) a 22050 Hz, mono f32. Nenhum arquivo `.wav`/`.mp3` externo. Nenhum asset de notificação. Nenhuma dependência de áudio do Android para o earcon — o som vem do mesmo caminho de playback.
- **MediaSession:** espelha `PlaybackState`; não computa posição (SC-035). Positivo: não há risco de divergência entre notificação e playback.
- **Notification channel:** `IMPORTANCE_DEFAULT` (não `MIN` — Live Updates proibidas; `MIN` bloqueia promoção; não precisamos de promoção para media playback).
- **Ongoing flag:** `setOngoing(true)` enquanto playing; `setOngoing(false)` ao pausar (FR-028e permite dismissal pelo pause, não pelo swipe enquanto playing).

---

*Referências dos componentes:* `spec.md` FR-028/FR-028a/FR-028b/FR-028c/FR-028d/FR-028e/FR-029; `spec.md` FR-041 (pausa razão); `constitution.md` §VII (conjuntos fechados); `plan.md` 214-220 (status view); `VERIFICATION.md` 184 (dispositivo Xiaomi API 33); Android 16 `Notification.ProgressStyle` / Live Updates docs.
