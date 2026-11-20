# Idea Intake: Gestão Inteligente de Fila e Cache de Áudio Offline (Spec 016/017)

- **Slug**: audio-cache-queue-management
- **Created**: 2026-11-10
- **Source**: pasted text
- **Type**: improvement

## Idea (as captured)

> "O próximo avanço de maior alavancagem com menor esforço no fluxo SpecKit é iniciar o pipeline Idea Assessment para a Especificação 016 (Gestão Inteligente de Fila e Cache de Áudio Offline) ou Otimização de Consumo Energético/Bateria em Background, iniciando formalmente em /speckit.assess.intake."

## Restated

Implementar gestão inteligente de fila e cache local persistente/em memória de buffers de PCM já sintetizados (ou pré-sintetizados concorrentemente), combinada com descarte LRU por cota de armazenamento e suspensão agressiva de threads de síntese (governança de CPU e conservação de bateria em segundo plano no Android).

## Origin & Context

- **Raised by**: Desenvolvedor / Equipe do Projeto Ressoa (MDMV)
- **Trigger**: Conclusão da Spec 010 (Directory Reading) e Specs 014/015 (Normalização e Prosódia). Em leituras contínuas de livros ou pastas com dezenas de capítulos Markdown, a ressíntese repetida ao retroceder/avançar desperdiça bateria e ciclos de CPU (Sherpa-ONNX/Piper), e a ausência de um teto de retenção de PCM sintetizado pode pressionar a memória RAM de dispositivos Android modestos.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Qual deve ser a política de cota de cache: memória RAM transitória (ex: LRU de 30-60 MB / ~15 minutos de áudio) vs. armazenamento flash temporário (`context.cacheDir`)?]
- [NEEDS CLARIFICATION: Como invalidar o cache de áudio caso o arquivo Markdown ou as configurações de prosódia/taxa sejam alteradas?]
- [NEEDS CLARIFICATION: Qual o impacto térmico e de consumo de bateria da síntese antecipada em background vs síntese estritamente just-in-time?]
- [NEEDS CLARIFICATION: O cache deve ser indexado pelo fingerprint content-addressed Blake3-256 já utilizado no subsistema de sessão e bookmarks?]
