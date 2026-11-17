# Ressoa — Google Play Store Listing Draft (rc2 v0.1.0)

> Prepared as part of the mdreader next 80/20 cycle. Requires explicit authorization (Play Console account, keystore, ASO assets) before publication. F-Droid publication also pending authorization.

## Identity

- **Package name**: `app.mdreader`
- **App title (30 ch)**: Ressoa — Leitor Markdown Offline
- **Short description (80 ch)**: Leitor de Markdown 100% offline e sem internet. Síntese de voz local, privacidade absoluta.
- **Full description (4000 ch)**: see below
- **Developer**: Ressoa (mdreader)
- **Category**: Books & Reference / Productivity
- **Content rating**: Everyone
- **Privacy policy**: 100% offline — no data leaves the device (no INTERNET permission)

## Full description (PT-BR)

**Ressoa** é um leitor de audiolivros e notas Markdown 100% offline e air-gapped.

Nenhum byte de texto ou áudio sai do seu aparelho. Sem permissão de internet. Sem assinatura. Sem telemetria.

### Por que o Ressoa

- **Leitura de Markdown estruturada**: cabeçalhos, listas, blocos de código e seções são compreendidos, não lidos como texto linear.
- **Síntese de voz local (Piper pt-BR)**: vozes neurais executadas no próprio dispositivo, sem nuvem.
- **Privacidade absoluta**: zero permissão de internet (`android.permission.INTERNET` ausente). Seus documentos nunca saem do aparelho.
- **Marcadores duráveis**: retomada exata da posição, persistida localmente (redb + Blake3).
- **Leitura de pastas completas**: navegação por diretórios com transições gapless entre capítulos.
- **Controle de velocidade**: 0.75x a 4.0x, pitch preservado.
- **Acessibilidade**: TalkBack 10/10, contraste WCAG, `aria-live`, alvos de toque ≥ 48×48 px.

### Modelo de negócio

Freemium + licença perpétua ("Pague uma vez, use para sempre"):
- **Grátis**: leitura ilimitada de notas Markdown avulsas com a voz padrão Piper pt-BR.
- **Pro (pagamento único, R$ 29,90–49,90 / $9,99–14,99)**: leitura de pastas completas, múltiplos catálogos de vozes locais, temas adicionais, relatórios diagnósticos.

## ASO keywords (PT-BR / EN)

leitor markdown, audiolivro offline, TTS local, síntese de voz offline, leitor de notas, markdown reader, audiobook offline, local TTS, privacy first, obsidian, logseq, notion

## Screenshots / assets needed

1. Phone screenshot: document library view (dark theme, alto contraste)
2. Phone screenshot: player with Lente Sinestésica canvas (RMS waveform)
3. Phone screenshot: voice catalog / speed control
4. Feature graphic: Ressoa wordmark + "100% offline · privacidade absoluta"
5. Icon: existing DESIGN-RESSOA logo vector

## Release notes (rc2 v0.1.0)

- Catálogo fechado de vozes locais (faber + cadu, CC0) com ciclo persistente
- Ingestão de arquivos via intent filter (ACTION_VIEW / ACTION_SEND)
- Leitura de pastas com transições gapless entre capítulos
- Conformidade Android 16 (API 36) — acessibilidade TalkBack
- Cache de áudio limitado 32 MiB (LRU)
- Síntese on-device 100% offline — zero permissão de internet

## Monetização

Freemium + licença perpétua via Google Play Billing (one-time purchase) ou F-Droid (grátis). Sem assinatura recorrente.
