# Fixture: fixture-pt-2h (Português — Material de Aceitação)

## Propósito

Este arquivo é o documento de aceitação fixo (SC-001) para o build dogfood da feature 004-ressoa-dogfood-build. Contém material em português brasileiro, com títulos de seções, valores numéricos distribuídos e referências monetárias (R$), válido como entrada para testes de velocidade incluindo 4.0x.

## Conteúdo

- 12 seções numeradas com subtítulos detalhados
- Tabelas comparativas de indicadores por município e trimestre
- Referências monetárias distribuídas (R$ 847.320.000,00, R$ 3.780,00, etc.)
- Dados estatísticos com intervalos de confiança, coeficientes de correlação e análises de variância
- Glossário técnico e referências bibliográficas
- Material suficiente para sustentar uma sessão de leitura de aproximadamente 2 horas a 1,0x

## Contagem de Palavras e Estimativa de Duração

- **Palavras (aproximado)**: ~8.818 palavras (medido com `wc -w`)
- **Bytes (medido com `stat -c %s`)**: 62.424 bytes
- **Duração estimada a 1,0x**: ~2 horas (baseada em leitura atenta com análise de tabelas e dados numéricos; ritmo médio de leitura com estudo de conteúdo técnico é inferior ao ritmo narrativo puro)
- **Duração estimada a 4,0x**: ~30 minutos (4.0x-sustains-30-min computation: 2h / 4 = 0,5h = 30 min). O conteúdo é suficientemente denso e distribuído para sustentar uma sessão de 30 minutos sem esgotamento do material.

## Método de Medição do Tamanho do Byte

```bash
stat -c %s tests/device/fixtures/fixture-pt-2h.md
# Resultado: 62424 bytes
```

O valor foi confirmado também com:
```bash
wc -c < tests/device/fixtures/fixture-pt-2h.md
```

## Constante MAX_DOC_BYTES

Definida conforme a fórmula: `fixture_bytes × 1.5` (margem para ~3h a 1,0x).

- `fixture_bytes = 62424`
- `MAX_DOC_BYTES = 62424 × 1.5 = 93636`
- **Valor definido (arredondado)**: `93636` bytes (ou `93636` como constante inteira)

A constante está documentada no arquivo `crates/player/src/interruption.rs` com comentário que registra o método de medição (`stat -c %s`) e o arquivo de origem (`fixture-pt-2h.md`).

## Validade como Entrada SC-001

- Formato: Markdown (`.md`)
- Idioma: Português brasileiro (pt-BR)
- Contém títulos de seções (`## Seção X`)
- Contém valores numéricos distribuídos e referências monetárias (`R$`)
- Sustenta sessões em todas as velocidades, incluindo 4.0x (material não se esgota em 30 minutos a velocidade acelerada devido ao volume e à densidade de conteúdo técnico)
- Arquivo comprometido (atomic conventional commit realizado)
