# 5 Candidatos a Design System — mdreader / ressona.dev

> Compilado de 4 subagentes paralelos (A/B/C/D concluídos; E é variação espectral do A com duotone, não repetido) + contexto consolidado da conversa.
> Base: spec 001-android-media-shell, VERIFICATION.md (tela preta esperada), constitution v1.2.0 (conjuntos fechados, SC-035, SC-029 declarado), paletas de contraste verificadas via cálculo WCAG 2.2 (sRGB linear, limite 0.03928), marca ressona.dev (RDAP livre), F-Droid (confirmado pelo usuário), projeto open source / dogfooding / 0 faturamento.

---

## A — MEDIDOR (VU-meter / Ableton transport / SMuFL)

**Tese:** cada superfície é um instrumento de medição de nível, não painel de controle. Metáfora vem de ANSI C16.5-1942 (VU-meter), transporte Ableton, SMuFL (Bravura).

**Paleta:** fundo `#0B0C0E` · tinta `#F2F4F7` (17.76:1) · dim `#9AA3AD` (7.66:1) · acento âmbar `#E8B04B` (10.01:1) · verde `#5FD0A0` (10.27:1) · vermelho `#E5534B` (5.28:1) · trilho corrigido `#555F6B` (3.01:1) · track decorativo `#22262B` (não-texto).

**Tipografia:** Roboto Flex (variável) in-app; sistema Android na notificação (12sp seção, ~16sp título).

**Notificação:** título doc + seção + progresso (barra âmbar/ciclo indeterminado) + estado por cor/posição. Sem body text. **Marquee: declarado NÃO — solução honest é earcon descendente 3 notas + reason estática curta + auto-dismiss 3s.**

**In-app:** grid Müller-Brockmann 2 campos (medidor ~62% / strings 38%); agulha VU real (RMS do Rust 22050 Hz mono f32 via `tauri::ipc::Channel`); taxa ladder fechada visível só aqui; progress monocromático.

**Tokens:** 3 camadas (primitive `--md-ink-*`; semantic `--md-state-*`; component `--notif-fill`, `--notif-track`).

**Integração ressona.dev:** logo = onda + documento; acento âmbar é a zona amarela do VU (única cor viva); "Engine Sona" = nome do motor interno.

**Riscos:** marquee impossível (§6); OEM relayout; Android 13+ PlaybackState deriva ações; máximo 252 dp; determinância travada (nunca muda mid-session); minSdk 29 vs manifesto 24.

---

## B — SINALIZAÇÃO (CTB / DIN 1451 / Semáforo)

**Tese:** código de cor por função, não decoração. Amarelo = aviso que expira; vermelho = falha/perda permanente; azul = informativo. Nenhum verde (estado de repouso = ausência de cor); nenhum amarelo sem TTL 3s.

**Paleta:** `#0A0A0A` fundo · `#FFFFFF` 19.8:1 · `#C9CDD2` 12.39:1 · `#FFD400` 13.83:1 (amarelo exclusivo) · `#FF4D63` 6.12:1 (vermelho corrigido) · `#3D8BE8` 5.71:1 (azul corrigido) · trilho `#5D5D5D` 3.01:1.

**Notificação:** título + seção + progresso (barra branca, não colorida — posição ≠ estado) + estado por cor/forma. **SC-029 resolvido pelo ponto pulsante amarelo (in-app) + razão estática curta + TTL 3s — é o candidato que MAIS SE APROXIMA sem quebrar plataforma.** Nenhum marquee; nenhum body text.

**In-app:** faixa de razão com scroll real (SC-029 atendido só aqui); razão de pausa com dot amarelo pulsante (in-app); progress monocromático; taxa só in-app.

**Acessibilidade:** earcon 2 notas (C5→E5 ascendente, inversão); confirmação audível; 48dp targets; cor nunca única informação (forma/posição + earcon).

**Riscos:** OEM recolor (mitigado por MediaStyle padrão, não custom RemoteViews); tema claro precisa reverificar; pulso in-app pode ser throttled; matriz obrigatória Pixel/Samsung/Xiaomi (gate 9 foi Xiaomi/API 33).

---

## C — EMISSORA (NRK Sans / DR Publik / Yle / Broadcast Discipline)

**Tese:** disciplina institucional — uma geometria, uma voz, uma cor, sobrevivência de 4K a 16px. Sem inovação visual; apenas consistência.

**Paleta:** `#0E1116` fundo · `#F4F6F8` 17.46:1 · `#A7B0BA` 8.61:1 · `#5AC8FA` 9.98:1 (ciano — identidade) · `#FF7A59` 7.37:1 (coral — falha exclusiva) · trilho `#556170` 3.0:1.

**Notificação:** título doc + seção + progresso ciano + falha coral. Sem amarelo semântico (não pertence ao conjunto). Sem marquee; razão estática.

**In-app:** grade Müller-Brockmann 2 campos; progresso ciano; seção atual; taxa in-app; sem animação (mudança de estado apenas).

**Integração:** marca = emissora; "Engine Sona" = motor interno; logo geométrico simples; não confundir com LoL.

**Riscos:** OEM recolor; confusão com LoL se "Sona" for usado como marca pública — por isso este candidato propõe marca = ressona, não Sona.

---

## D — EDITORIAL (Müller-Brockmann / iA Writer / Readwise / Ribbon-marker)

**Tese:** a superfície in-app é um poster editorial; a notificação é um miniposter de 48dp. Não é "bonito" — é legível em branco sobre preto. Fita-ribbon marker como posição no documento (não porcentagem).

**Paleta:** `#101010` estoque · `#F5F2EC` 17.03:1 (tinta principal) · `#A8A196` 7.43:1 · regra `#646057` 3.04:1 (hairline) · vermelho `#C8553D` 4.37:1 só falha. Nenhum acento neutro alternativo.

**Notificação:** título grande + seção subtitle + progresso monocromático + razão estática. **Sem marquee (declarado). Sem acento de cor. Sem display face.** Monocromático + uma regra de separação.

**In-app:** grade assimétrica 2-colunas (campo esquerdo conteúdo, direito controle, gutter 24dp); ribbon-marker físico na borda esquerda; corpo 20px/1.4; focus mode (seção em tinta, resto em dim); scroll de razão no in-app (SC-029 atendido só aqui).

**Riscos:** risco principal = importar tipografia editorial para notificação (não faz sentido — Android já renderiza); risco de expectativa (usuário pode esperar revista); solução = alinhamento com brief (ferramenta, não revista); orçamento 40 chars pt-BR no título.

---

## E — ESPECTRAL (Duotone / Audacity / Sonificação / LinnStrument)

**Tese:** energia como cor. O progresso é uma rampa de calor (cool → hot) ao longo do trilho; indeterminado = trilho cool-only. A faixa in-app pode mostrar envelope de fala (previsão de passagem densa).

**Paleta:** `#000000` fundo · `#F0F0F0` 18.43:1 · `#9AA0A6` 7.95:1 · cool `#2D5F8B` 3.12:1 · hot `#E8A33D` 9.74:1 · trilho `#595959` 3.0:1.

**Notificação:** título + seção + progresso com gradiente cool→hot (posição marcada por ponto quente) + estado por cor e forma. **Marquee: não — razão estática + earcon, mas o gradiente temporal (esfriamento em 3s) é a resposta visual do SC-029 — é a única superfície que entrega movimento temporal sem texto rolante.**

**In-app:** faixa de envelope (3–5px) + progress gradiente + taxa + seção.

**Riscos:** gradiente pode ser lido como glitch por OEM; contraste de cool é baixo (3.12:1) — só aceitável como trilho não-texto; precisa de verificação obrigatória Samsung/Xiaomi/Pixel; mais decoração que os outros — precisa provar que serve à condução, não só que é bonito.

---

## Síntese comparativa (para decisão)

| Critério | A Medidor | B Sinalização | C Emissora | D Editorial | E Espectral |
|---|---|---|---|---|---|
| **SC-029 (texto rolante)** | FAIL declarado | **Mais próximo** (ponto pulsante + TTL) | FAIL declarado | FAIL declarado (scroll só in-app) | **Parcial** (gradiente esfria em 3s) |
| **Contraste verificável** | ✓ (todos ≥3.01) | ✓ (todos ≥3.01) | ✓ (todos ≥3.0) | ✓ (regras corrigidas) | ⚠ (cool 3.12 marginal) |
| **Sobrevivência OEM** | Média | **Alta** (sem custom RemoteViews) | Alta | Média (texto longo risco) | Baixa (gradiente) |
| **Disciplinado / fechado** | ✓ (conjuntos fechados) | **✓✓** (cor como função regulada) | ✓ (identidade institucional) | ✓ (grade como ordem) | ⚠ (gradiente = risco de expansão) |
| **In-app força** | **Alta** (agulha RMS real) | Média | Alta | **Muito Alta** (grade + ribbon) | Média (faixa de envelope) |
| **Notificação força** | Média | **Alta** (semáforo legível) | Alta | Média (monocromático) | Média (gradiente pode confundir) |
| **Risco de confusão LoL** | Nenhum (Sona = engine) | Nenhum | **Real** se "Sona" usado como marca | Nenhum | Nenhum |
| **Referência cultural** | VU-meter / Ableton | CTB / DIN 1451 / Semáforo | NRK / DR / Yle | Müller-Brockmann / iA Writer | Audacity / SMuFL / LinnStrument |

**Recomendação baseada na evidência (não gosto):** B é o mais próximo de resolver o SC-029 sem quebrar a plataforma; D é o mais forte in-app; A é o mais honesto sobre o descumprimento do SC-029; E é o mais arriscado (gradiente + low-contrast cool). A escolha depende do que você valoriza mais: **honestidade técnica** (A), **resolução do SC-029** (B), **disciplina institucional** (C), **legibilidade profunda** (D) ou **surpresa visual** (E).

---

## Arcabouço do design system (aplica a qualquer dos 5)

- **Marca:** `ressona.dev` (RDAP livre); `mdreader` = workspace técnico; `Engine Sona` = motor interno (não marca).
- **Distribuição:** F-Droid (confirmado pelo usuário); não Play Store; metadados F-Droid necessários quando for distribuir.
- **Conjunto fechado de cores:** cada candidato tem seu conjunto; adicionar uma cor = edição coordenada + bump semântico + docs (Princípio VII da constituição).
- **SC-029:** nenhum candidato entrega marquee na notificação; todos declaram o descumprimento com solução (earcon + razão estática + TTL 3s). Isso é **honestidade**, não falha — e o relatório de defeito é obrigatório.
- **Acessibilidade:** todos usam earcon sintetizado pelo engine (22050 Hz mono f32, Rust) — nenhum asset externo, nenhuma dependência de áudio de notificação.
- **Token architecture:** 3 camadas (primitive / semantic / component) — aplicada consistentemente nos 5.
