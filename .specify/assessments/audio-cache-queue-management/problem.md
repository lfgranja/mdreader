# Problem Definition: Gestão Inteligente de Fila e Cache de Áudio Offline

- **Slug**: audio-cache-queue-management
- **Created**: 2026-11-10
- **Inputs used**: intake.md, research.md

## Problem Statement

Durante a leitura de audiolivros e diretórios Markdown, qualquer comando de repetição de seção, retrocesso temporal ou reabertura de capítulos recentes obriga o motor neural a resintetizar todo o áudio do zero, desperdiçando bateria preciosa do usuário e introduzindo atrasos acústicos desnecessários, enquanto a falta de uma cota de memória para a fila de capítulos e buffers transitórios arrisca pressão excessiva de RAM no Android.

## Affected Users & Stakeholders

- **Usuários:** Ouvintes de livros e textos técnicos que usam frequentemente os botões de retroceder (15s/30s) ou repetir seção, e que enfrentam congelamento temporário do som e drenagem rápida de bateria do smartphone.
- **Engenharia / Produto:** Responsáveis por manter o app leve, responsivo e seguro contra o OOM (Out Of Memory) killer em dispositivos modestos (Android com 3GB/4GB de RAM).

## Goals

- Eliminar a latência de ressíntese em saltos para trás (rewind ≤ 60s) e repetição de parágrafos imediatos servindo o áudio diretamente do cache.
- Reduzir o consumo de CPU e bateria em segundo plano ao evitar a inferência neural repetida de blocos de texto idênticos.
- Garantir um teto determinístico de consumo de memória RAM (Bounded LRU Cache), prevenindo qualquer vazamento ou crash por OOM em sessões de leitura prolongadas (2h+).

## Non-Goals

- Não adicionar codecs de compressão pesados (como FFmpeg ou Opus encoders) que aumentem a superfície de ataque e o tamanho do binário APK.
- Não persistir arquivos de áudio descriptografados permanentemente para exportação como MP3 (o Ressoa é um leitor em tempo real, não um conversor de arquivos offline para pirataria).
- Não violar o princípio do `PositionClock` (o relógio de posição continua avançando estritamente sobre áudio audível emitido).

## Success Metrics

- **Latência de Retorno (Rewind / Repeat):** < 50ms para áudio em cache (contra 400ms-1500ms de ressíntese neural).
- **Consumo de Memória RAM:** O cache em memória deve respeitar um teto máximo configurável rígido (ex: ≤ 32 MB / ~6 minutos de áudio bruto).
- **Taxa de Acerto de Cache (Cache Hit):** > 90% em operações de rewind e repetição de seções imediatas.

## Cost of Inaction

Sem essa gestão, o aplicativo continuará reprocessando texto repetidamente em CPU/NNAPI a cada toque de rewind, degradando a experiência auditiva contínua do usuário e reduzindo a autonomia de bateria do celular.

## Open Questions

- [NEEDS CLARIFICATION: Qual o limite de RAM seguro padrão para a política LRU em Android minSdk 29? Sugestão técnica: 32 MiB (~6 minutos de áudio f32 a 22.050 Hz).]
