# Decision: Ingress via Intent Filters (ACTION_VIEW & ACTION_SEND)

- **Slug**: intent-filter-ingress
- **Date**: 2026-11-11
- **Verdict**: go

## Decisão

Aprovado formalmente (`go`) para promoção imediata ao fluxo de **Spec-Driven Development** como **Spec 019** (`019-intent-filter-ingress`).

## Justificativa

1. **Maior Alavancagem com Menor Esforço (80/20):**
   - Transforma o app em uma extensão natural do ecossistema Markdown no Android com menos de 100 linhas de código no total.
   - Elimina o principal ponto de atrito de adoção diária para os usuários centrais (Obsidian / PKMS).
2. **Segurança e Conformidade:**
   - Mantém rigorosamente todas as regras constitucionais: zero lógica de playback em Kotlin, zero permissão de rede, modelo de áudio inalterado.
   - O pipeline de ingestão de bytes, cálculo de fingerprint Blake3 e persistência atômica em `redb` já existe e está plenamente testado e consolidado no núcleo do Ressoa.
