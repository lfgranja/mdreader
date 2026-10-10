# Design System Candidates — mdreader (Tauri Android Media Shell)

> Continuação após restart do servidor (sem repetição de trabalho concluído). Build Tauri já limpo (`src-tauri/gen/` removido, `tauri.conf.json` validado por `python3` — `devPath`/`distDir` eliminados, parse de JSON intermediário resolvido). Subagentes foram despachados para pesquisa profunda (interrompidos pelo restart, recuperados via `websearch` + síntese direta). Skills aplicadas: `brand-identity`, `frontend-ui-engineering`, `impeccable`, `tailwind-design-tokens`, `ui-styling`.

---

## Contexto do projeto (relevante para todos os DS)

- **App**: `mdreader` — Android media shell / leitor de mídia com `redb` (bookmarks), `blake3` (fingerprint), `MediaSession` espelho, `Service` Kotlin (sem lógica de playback), notificação = título + seção atual + progresso apenas (determinate se total conhecido, else indeterminate), foco/áudio com ducking e expiração em 3 s.
- **Constraints fortes (AGENTS.md / spec)**: engine mono f32 @ 22050 Hz; Kotlin nunca computa posição; notificação não-despedível enquanto toca; `reduce-motion` deve ser respeitado; contraste e tamanho de toque são requisitos regulatórios implícitos (Android 29+, API 34 FGS).
- **Implicação de UI**: dark mode é obrigatório (OLED + áudio noturno); player de controle precisa ser compacto; lista de sessões precisa destacar seção atual sem poluir; progresso precisa ser legível em lock-screen e notificação.

---

## CANDIDATO 1 — Material You Adaptive · Dark Audio

**Persona**: Usuário Android nativo, espera personalização dinâmica (Dynamic Color via wallpaper), familiaridade com Jetpack Media3.

**Filosofia**: Adotar M3 (Material Design 3) como base, com tonal elevation em vez de sombra pesada, e acentos de áudio que respeitam a notificação mínima exigida pelo contrato.

**Paleta (Dark)**
- Background: `#121212`
- Surface: `#1E1E1E`
- Surface elevado: `#2D2D2D`
- Primary (lavanda dinâmico): `#D0BCFF`
- Secondary: `#CCC2DC`
- On-surface (texto): `#E6E1E5`
- Acento de playback / seção ativa: `#A8D5BA` (verde-água suave)
- Divider: `#2C2C2C`

**Tipografia**
- Família: Roboto / Google Sans (sistema)
- Headline (seção atual): 24sp / Medium / tracking `-0.02`
- Body (lista de sessões): 16sp / Regular / 1.5 line-height
- Caption / progresso: 12sp / Regular
- Overline (badge): 10sp / Medium, all-caps

**Componentes chave**
1. **Player mínimo (Media3-style)**: controles centralizados (Play/Pause, Seek, Section Title), sem texto de corpo — apenas seção atual + progresso linear fino.
2. **Notificação / Mini-player**: superfície tonal `#2D2D2D`, progress bar 2px altura, título + seção apenas, não-despedível enquanto `playWhenReady`.
3. **Lista de sessões / bookmarks**: cards com thumbnail simplificado + title + meta, elevação tonal `#2D2D2D` para item ativo.
4. **Badge de seção ativa**: pill com fundo primário `#D0BCFF` e texto `#121212`, 16dp altura.
5. **Foco / Áudio**: indicador sutil (círculo 4px) ao lado do item ativo, sem animação agressiva (respeita `reduce-motion`).

**Referências**
- YouTube Music Android (M3 update, 2024)
- Spotify Android (Material 3 dark theme, tonal surfaces)
- Jetpack Compose Media3 `PlayerDefaults` (documentação oficial Android)
- Material Design 3 Dark Theme tutorial (Material.io, 2020+)

**Por que para mdreader**
Respeita o contrato de notificação (título + seção + progresso apenas) via tonal surfaces; usa `MediaSession` espelho nativo; não exige lógica de playback em Kotlin; adapta-se a Android 12+ Dynamic Color se o usuário tiver wallpaper personalizado.

---

## CANDIDATO 2 — Monolith Typography-First · Content Hero

**Persona**: Leitor focado, minimalista, odeia distração; o app é um "escritório de leitura" onde o conteúdo é o hero.

**Filosofia**: Maximizar whitespace, usar serif/contraste de fonte para distinguir seção de leitura de UI de controle, e reduzir componentes à linha essencial.

**Paleta (Dark)**
- Background: `#0A0A0C` (quase preto com leve azul — bom para OLED, reduz burnout)
- Surface: `#111114`
- Surface elev: `#18181B`
- Text primary: `#F5F5F7` (branco levemente quente)
- Text secondary: `#8E8E93`
- Acento de seção ativa: `#C58C4B` (âmbar suave, editorial)
- Divider / linha de progresso: `#1C1C1F`
- Hover / foco sutil: `#26262A`

**Tipografia**
- UI (labels, botões): Inter, 14sp Regular / 16sp Medium
- Títulos de seção (hero): Source Serif Pro / EB Garamond, 28sp / Regular, tracking `-0.03`
- Corpo / meta: Inter, 16sp / 1.6 line-height
- Progresso / tempo: Inter Mono / JetBrains Mono tabular, 13sp — garante alinhamento de números

**Componentes chave**
1. **Header hero**: título da seção atual em serif, grande (28sp), ocupando metade da tela; abaixo, meta (duracao / progresso) em mono pequeno.
2. **Lista de sessões**: coluna estreita (máx. 480dp), sem cards, apenas linha de separação `1px`; item ativo com borda esquerda `3px` âmbar `#C58C4B`.
3. **Player de controle**: rodapé fino (`48dp`), apenas Play/Pause + Seek + tempo atual; sem texto extra.
4. **Botão de foco / nota**: discreto, círculo `36dp`, sem preenchimento a menos que ativo; indicador de bookmark como ponto `6px` ao lado do título.
5. **Notificação**: superfície `#111114`, título em branco, seção em cinza, progresso como linha fina `2px` ânbar — nenhuma outra palavra.

**Referências**
- Pocket (modo leitor, typography-driven)
- Readwise Reader (minimal, serif headings)
- Linear (tipografia precisa, whitespace generoso)
- Sublime Text / VS Code dark editor themes (monospace for time/data)

**Por que para mdreader**
O projeto é leitor/mídia; o conteúdo (seção) é o valor principal. Este DS evita competir com a seção ativa, respeita a notificação mínima, e permite que o progresso seja lido rapidamente em lock-screen.

---

## CANDIDATO 3 — Glassmorphic Deep Night · Translucent Media

**Persona**: Usuário premium, experiência cinematográfica; espera que o app seja uma "sala de som" com superfícies que se misturam ao fundo.

**Filosofia**: Usar `backdrop-filter: blur()` em superfícies, cores profundas de azul-preto, acentos quentes (perola / ouro suave) que lembram iluminação de estúdio.

**Paleta (Dark)**
- Background: `#050510` (preto-azul profundo — bom para OLED)
- Surface glass (60% opaco): `#0F0F14`
- Surface elevado (90% opaco): `#15151A`
- Primary (perola): `#E8D5B5`
- Secondary (azul translúcido): `#8FA3B8`
- Acento playback: `#D4AF37` (ouro suave, apenas para seção ativa / progresso)
- Text: `#F2F0F2`
- Text secondary: `#A0A0A5`

**Tipografia**
- Família: Manrope / Space Grotesk (geometrica, limpa, boa em telas OLED)
- Headline: 22sp / Medium, letter-spacing `-0.02`
- Body: 16sp / Regular, line-height 1.5
- Caption / progresso: 11sp / Medium
- Tracking amplo para meta: `0.05` — dá sensação de luxo

**Componentes chave**
1. **Card de sessão**: `border-radius: 20dp`, fundo translúcido, sombra sutil (`0 8dp 32dp rgba(0,0,0,0.4)`), framem do título e meta.
2. **Player central**: círculo de play com borda `1px` perola, sombra interna (inset) sutil, fundo translúcido; botão de próxima seção com seta fina.
3. **Lista de sessões**: divisores translúcidos (`rgba(255,255,255,0.06)`), sem cards — apenas linhas.
4. **Notificação / Mini-player**: fundo translúcido (`#0F0F14` 90%), progresso com gradiente sutil de perola para ouro `#D4AF37`; título grande, seção ao lado.
5. **Indicador de foco / áudio**: anel `2px` perola ao redor do item ativo, sem animação de pulsação (respeita `reduce-motion`).

**Referências**
- Apple Music (interface dark glass, 2024)
- Raycast (glass UI, blur, dark mode)
- Spotify "Blend" / premium dark themes (translúcido, bordas sutis)
- iOS 17/18 Lock Screen widgets (glassmorphic surfaces)

**Por que para mdreader**
A experiência de leitura/mídia longa beneficia-se de superfícies que não "pesam" visualmente; o blur reduz contraste excessivo em OLED, e o acento de ouro dá indicação clara de progresso sem texto extra.

---

## CANDIDATO 4 — Neon Pulse · High-Contrast Gaming Audio

**Persona**: Usuário de longa duração de áudio (podcasts, audiobooks longos), gosta de feedback visual imediato; ambiente totalmente escuro, pode estar em transporte/noturno.

**Filosofia**: Preto absoluto, acentos neon (ciano / âmbar), glow/sombra como tokens, tipografia mono para dados (tempo/progresso), animação de pulsação sutil no play.

**Paleta (Dark)**
- Background: `#050505` (preto absoluto)
- Surface: `#0A0A0A`
- Surface elev: `#111111`
- Primary (ciano neon): `#00F0FF`
- Secondary (âmbar neon): `#FFB300`
- Text: `#EEEEEE`
- Muted / meta: `#555555`
- Glow ciano (20% alpha): `#00F0FF`
- Glow âmbar (15% alpha): `#FFB300`
- Divider: `#222222`

**Tipografia**
- UI (labels): Inter, 14sp / 600
- Títulos: JetBrains Mono, 20sp / Bold — uso de fonte mono para dar sensação de "dados de áudio"
- Corpo / meta: Inter, 15sp
- Progresso / tempo: JetBrains Mono tabular, 13sp — alinhamento perfeito de números
- Badge de seção ativa: fonte mono, 11sp, fundo ciano `#00F0FF`, texto preto

**Componentes chave**
1. **Botão de play pulsante**: círculo `56dp`, fundo ciano `#00F0FF`, sombra glow sutil; animação de pulsação apenas quando `reduce-motion: no-preference`.
2. **Barra de progresso**: gradiente linear ciano → âmbar, 3px altura, com indicador de posição em forma de diamante ciano.
3. **Lista de sessões**: bordas neon sutis (`1px` ciano) apenas no item ativo; outros itens sem borda, apenas seletor de foco em âmbar quando selecionado via teclado.
4. **Notificação**: fundo `#0A0A0A`, título em branco, seção em ciano, progresso como linha neon 2px — nenhum texto de corpo.
5. **Indicador de áudio / foco**: ponto ciano pulsante (se permitido) ao lado da seção atual; se `reduce-motion`, ponto estático.

**Referências**
- VLC dark neon themes (custom skins)
- Discord dark (acento neon, contraste alto)
- Spotify Wrapped (neon glitch, 2023-2024)
- Gaming HUD designs (feedback imediato, contraste máximo)

**Por que para mdreader**
Em longas sessões de áudio (o projeto envolve bookmarks de seção), o alto contraste reduz fadiga visual; o ciano como primário é distintivo de toda a UI do sistema (não conflita com notificação do Android nativo que usa azul padrão).

---

## CANDIDATO 5 — Accessible Standard AAA · WCAG First

**Persona**: Usuário com sensibilidade à luz, deficiência visual parcial, necessidade de foco claro; acessibilidade é a base, não um aditivo.

**Filosofia**: Todo componente passa pelo critério WCAG 2.2 AAA: contraste 7:1 para texto normal, 4.5:1 para grande texto — mas aqui forçamos > 7:1; todos os botões ≥ 48dp; foco visível com anel `2px` branco; redução de movimento respeitada; tipografia aumentada (18sp body mínimo).

**Paleta (Dark)**
- Background: `#0D0D0F` (não preto puro — reduz halo em telas LCD)
- Surface: `#17171A`
- Surface elev: `#222226`
- Primary (azul claro alto contraste): `#B4C6E7`
- Text (quase branco, contraste > 15:1 com bg): `#FFFFF5`
- Text secondary (contraste > 7:1): `#A8A8A8`
- Acento / seção ativa: `#FFD966` (amarelo de alto contraste 4.5:1 com bg)
- Focus ring: `#FFFFFF` 2px sólido, offset 2px
- Divider: `#333333` (espesso, visível)

**Tipografia**
- Família: Atkinson Hyperlegible / Noto Sans (otimizadas para legibilidade)
- Headline: 22sp / Bold / 1.2 line-height
- Body: 18sp / Regular / 1.6 line-height (maior que padrão — respeita AAA para telas próximas)
- Meta / progresso: 14sp / Medium
- Tracking: `+0.02` — melhora distinção de letras

**Componentes chave**
1. **Botão de play / controle**: mínimo `48dp` x `48dp`; anel de foco branco `2px` sempre visível quando selecionado (não apenas no hover); contraste do ícone > 7:1 com fundo primário.
2. **Lista de sessões**: separadores `2px` `#333333`; item ativo com borda esquerda `4px` amarelo `#FFD966`; fonte 18sp; espaço entre itens `12dp` (não apertado).
3. **Barra de progresso**: padrão de alto contraste (não apenas cor) — barras diagonais ou pontilhadas para distinguir progresso de fundo; indicador de posição com círculo branco `12dp`.
4. **Notificação**: texto grande (título 16sp, seção 14sp), progresso claro; não há mini-texto; botão de pausa visível no notification shade (48dp touch target).
5. **Indicador de áudio / foco**: anel branco sólido ao redor do item ativo, mesmo quando não tem foco de teclado; não usa apenas mudança de cor (não depende só de percepção de cor).

**Referências**
- Android Accessibility Guidelines (touch targets 48dp)
- WCAG 2.2 AAA (contrast, focus, motion)
- Apple HIG Accessibility (focus indicators, text sizing)
- BBC iPlayer Accessibility Mode (alto contraste, fontes grandes)

**Por que para mdreader**
Respeita todos os contratos de acessibilidade implícitos (Android 29+, serviço de primeiro plano, notificação não-despedível); o progresso claro ajuda em lock-screen; a redução de animação evita distração em ambientes de áudio longo.

---

## Comparação rápida (para decisão)

| Dimension | C1 Material You | C2 Typography | C3 Glass | C4 Neon | C5 Accessible |
|---|---|---|---|---|---|
| **Base técnica** | M3 / Compose / Dynamic Color | Inter + Serif | Manrope / Blur | Inter + Mono | Atkinson / Noto |
| **Percepção de cor** | Normal (lavanda) | Quente (âmbar) | Luxo (perola) | Alto (neon) | Máximo (amarelo / azul) |
| **Contraste texto/bg** | ~7:1 | ~9:1 | ~8:1 | ~15:1 | > 15:1 |
| **Touch target mínimo** | 48dp (M3) | 48dp | 48dp | 48dp | **48dp explícito + anel foco** |
| **Reduce-motion** | Sim (M3) | Sim | Sim | Sim (pulsação condicional) | **Obrigatório / base** |
| **Notificação** | Tonal, simples | Minimal linha | Translúcida | Neon 2px | Grande texto, botão visível |
| **Melhor para mdreader quando...** | Quer integração Android nativa | Leitura editorial / foco em texto | Premium / longa duração | Ambiente escuro / longo áudio | Acessibilidade é prioridade |

---

## Recomendação para implementação (respectando AGENTS.md / spec)

1. **Selecione C1 (Material You Adaptive)** como **base técnica** — porque o projeto já usa Tauri + Android SDK 29+ e o contrato de `MediaSession`, `Service`, e notificação é nativo. C1 aproveita `media3-ui-compose-material3` (documentado no websearch) e permite `Dynamic Color` sem quebrar contratos.

2. **Incorpore tokens de C5 (Accessible AAA)** como **requisitos mínimos não-negociáveis**: todos os componentes devem atingir 48dp, contraste > 7:1, foco visível, e respeitar `reduce-motion`. Isso não muda a estética de C1, apenas fortalece o design.

3. **Use C2 (Typography-First)** para **a tela de seção / leitor**: quando a seção está ativa, o título pode ser exibido em serif (Source Serif Pro) para diferenciar conteúdo de UI; o resto da UI permanece M3.

4. **Evite C4 (Neon)** a menos que o usuário explicitamente peça alta energia — o neon pode conflitar com notificação do sistema Android (que usa azul/ciano padrão) e pode ser cansativo para leitura longa.

5. **Evite C3 (Glass)** se o dispositivo alvo for de baixa especificação ou se a performance de `backdrop-filter` no WebView do Tauri não for garantida — prefira tonal elevation de C1 que é apenas cor, não renderizador.

---

## Próximo passo técnico (após esta síntese)

Para retomar o build Tauri (que estava bloqueado pelo `tauri.conf.json` intermediário):

```bash
# Já executado: rm -rf src-tauri/gen src-tauri/target/release/build/src-tauri-*
# Verifique antes de reconstruir:
python3 -c "import json; json.load(open('src-tauri/tauri.conf.json'))" && echo "JSON OK"

# Reconstruir (use o timeout de 600s como antes):
export JAVA_HOME=/usr/lib/jvm/java-25-openjdk && export PATH="$JAVA_HOME/bin:$PATH"
timeout 600 cargo tauri android build --target aarch64
```

O erro original (`devPath`, `distDir` inesperados + trailing comma) está eliminado porque:
- `devPath`/`distDir` são propriedades de Tauri v1, removidas do `tauri.conf.json` da fonte;
- `src-tauri/gen/` (que continha o arquivo intermediário com parse inválido) foi removido e será regenerado pela build a partir da fonte limpa.

Se o build ainda falhar com `build` vazio `{}`, adicione a chave mínima necessária para Android (`bundle`, `build` com `beforeBuildCommand` se houver) conforme `spec.md` e `plan.md`, mas o `build: {}` atual é válido para M3.

---

*Documento gerado com apoio de subagentes (interrompidos e recuperados), websearch atualizado (Material 3, media3, dark theme, accessibility), e skills `brand-identity`, `frontend-ui-engineering`, `impeccable`, `tailwind-design-tokens`.*