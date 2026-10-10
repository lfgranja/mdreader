# DESIGN/accessibility.md — Acessibilidade / Eyes-Free (B — Sinalização)

**Versão:** 1.0.0 · **Data:** 2026-10-06
> Aplicável a todo o diretório `DESIGN/`. Normativas: `constitution.md` §VIII (Driving-First — NON-NEGOTIABLE), `spec.md` FR-018/FR-018a/FR-018b/SC-008, `VERIFICATION.md` (trials 10/10: lock-screen, focus, routing, ≤3s play-to-audio).

---

## 1. Definição do contrato (não é WCAG / TalkBack)

O projeto **não define** WCAG nível, TalkBack, `contentDescription`, focus-order, touch-target mínimo de 48 dp, ou screen-reader announcements. O único contrato de acessibilidade é **explicitamente** documentado:

> **FR-018a / SC-008:** "operable locked, by feel, with its effect confirmed audibly rather than visually"; blind-test harness: "the tester cannot see the surface and a trial fails on any unlock or any effect left unconfirmed aloud" (`spec.md:707-711`, `spec.md:1320-1324`).

Isso é o que este arquivo governa. Nenhum outro requisito de a11y é adicionado sem emenda de spec.

---

## 2. Earcon — conjunto fechado, sintetizado no engine

**Fonte:** `crates/engine-core` (Rust), 22050 Hz, mono f32, sem asset externo. Nenhum `.wav` / `.mp3` / `.ogg`; nenhum decode; zero dependência de áudio de notificação.

**Conjunto fechado (não adicionável sem coordenação + bump + docs):**

| Ação | Frequência / Notas | Duração | Identificação |
|---|---|---|---|
| **Play / Resume** | C5 → E5 (3ª maior ascendente) | 90 ms / nota | Ascendente = início |
| **Pause** | E5 → C5 | 90 ms / nota | Descendente = parada |
| **Skip ± (forward/back)** | C5 → E5 (mesmo play) + tick curto | 90 ms + 40 ms | Confirmação de salto |
| **Rate change (um rung)** | Tick 1000 Hz / 40 ms | 40 ms | Confirmação de mudança de velocidade |
| **Warning — 3 s (aviso de pausa)** | Duplo tick A5 (dois tons curtos) | 60 ms + 80 ms intervalo | Beep de contagem regressiva (semáforo de pedestre) |
| **Failure / Permanent loss** | D3 quadrado + repetição curta | 200 ms + 150 ms | Falha; não confundir com aviso (amarelo vs vermelho) |
| **Section boundary** | Tick 880 Hz / 60 ms | 60 ms | Fronteira de seção (posicionamento estrutural) |
| **Resume offer (FR-032c)** | C5 → E5 (ascendente) + fala de estado | 90 ms + TTS | Confirmação de estado + opção |

**Regra:** o earcon é **obrigatório** para todo controle alcançável enquanto dirigindo (play, pause, skip ±, rate change, resume — FR-018/FR-018a). Se um controle não tem earcon, é um defeito de SC-008 (falha no blind-test).

---

## 3. Confirmação audível — obrigatória, nunca visual-only

**Princípio VIII (Driving-First)** exige que nenhum controle exija olhar. A confirmação é **auditiva**, não visual:
- Todo botão pressionado produz earcon + mudança de estado + (se aplicável) fala de estado (ex.: "Leitura pausada — outro app usando o áudio" — FR-041, não é body text do documento).
- Se o usuário não pode ver (blind-test), ele ainda sabe o que aconteceu.
- **Não há "visual confirmation" como fallback:** se o earcon falha, o controle falha. Isso é fail-closed, conforme Princípio VI.

---

## 4. Touch targets / gestos

- **In-app:** todos os targets ≥ 48 dp (M3; `spec.md` não define, mas `android.md` da skill exige). Espaço entre targets ≥ 8 dp.
- **Notificação:** controles são da plataforma (`PlaybackState`); o usuário interage via swipe/press do Android, não via targets customizados. Nenhum target custom é desenhado — a template padrão já tem targets de sistema.
- **Gestos in-app:** swipe horizontal na faixa de razão (scroll de texto) é operável sem olhar — gesto de mão, não leitura visual. Confirmação auditiva após o gesto (earcon).

---

## 5. Cor — nunca o único meio (WCAG 2.2 SC 1.4.1 / 1.4.11)

- **Estados:** fluxo (sem cor = ausência), caution (amarelo), prohibit (vermelho), info (azul) — cada um tem **forma** (barra cheia / barra congelada / ponto + texto / posição) + **posição** (slot inicial) + **earcon**.
- **Daltonismo / alta contraste:** o amarelo `#FFD400` (13.83:1) e vermelho `#FF4D63` (6.12:1) têm contraste suficiente mesmo sem cor — a forma e o texto são independentes.
- **Tema claro / escuro:** apenas dark (`#0A0A0A`) está definido; light mode não é especificado; se for implementado, requer re-verificação completa (não apenas inversão).

---

## 6. O que NÃO é acessibilidade neste projeto (deliberado)

Para evitar supostos requisitos que não existem:
- **Não** há requisito de TalkBack `contentDescription` (não há webviews de conteúdo — a superfície in-app é minimal; a notificação é MediaStyle padrão).
- **Não** há requisito de focus-order navegável por teclado (não há lista de itens — só status + transporte).
- **Não** há requisito de screen-reader announcement do conteúdo do documento (proibido pelo spec: body text nunca aparece na notificação; não há superfície de leitura em si — só status).
- **Não** há requisito de simplificação de texto para crianças (não é público infantil — é técnico / adulto).
- **Não** há requisito de redução de movimento (exceto `prefers-reduced-motion` para o ponto amarelo pulsante — ele cessa; não há outra animação).

---

## 7. Testes obrigatórios (gate 9 / trials 10/10)

- **Blind-test harness (SC-008):** testar com tela apagada / olhos fechados; sem desbloqueio do dispositivo; sem olhar para o painel; falha se qualquer efeito não for confirmado audivelmente.
- **Lock-screen (FR-028):** confirmar que título + seção + progresso aparecem; que body text NÃO aparece; que progresso é determinante/indeterminado corretamente; que não há marquis (texto estático).
- **Focus / Routing:** confirmar que perda permanente de foco → pausa + razão + earcon de falha; que perda transitória → ducking (~20%) + earcon de aviso; que regresso → resume offer (3 s) + earcon.
- **Routing / Device removal:** ≤ 1 s para pausa (FR-016); ≤ 3 s play-to-audio (FR-027);
- **OEM:** Samsung One UI 7 (split painel), Xiaomi HyperOS (split obrigatório), Pixel (stock) — todos com notificação visível e controles operáveis.

---

*Referências:* `spec.md` FR-018/FR-018a/FR-018b; `spec.md` FR-041; `spec.md` SC-008/SC-010/SC-035/SC-042/SC-046; `constitution.md` §VIII; `research.md` R6 (audible confirmation); `VERIFICATION.md` 184; `m3.material.io` touch targets; `android.md` (72dp, 48dp); `developer.android.com` MediaStyle docs.
