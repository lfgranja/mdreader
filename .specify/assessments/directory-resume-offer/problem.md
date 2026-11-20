# Problem Definition: Retomada de Diretório Inalcançável na Reabertura

- **Slug**: directory-resume-offer
- **Created**: 2026-11-20
- **Inputs used**: intake.md, research.md

## Problem Statement

Ouvintes que interrompem a escuta de uma pasta e a reabrem mais tarde não recebem
a oferta de retomada prometida: o diálogo não aparece e, quando apareceria, não
traz o nome do arquivo em andamento nem a posição salva. O mecanismo de progresso
existe e foi persistido, mas o caminho de reabertura não o torna visível nem
utilizável, forçando o usuário a recolocar manualmente a escuta no ponto em que
parou — exatamente a situação que o requisito de retomada deveria evitar.

## Affected Users & Stakeholders

- **Users**:
  - **Ouvinte de diretórios Markdown/áudio local (hands-free)**: depende da retomada
    automática ao trocar de sessão, dispositivo ou após pausa; perde o fio da leitura
    quando a oferta não é exibida.
  - **Ouvinte em trânsito/direção**: não pode inspecionar a tela para descobrir em
    que arquivo estava; a ausência do nome do arquivo e do ponto salvo agrava o
    custo da interrupção.
- **Stakeholders**:
  - **Mantenedor do projeto**: precisa fechar T233 da spec 010 sem alterar o
    comportamento já especificado e sem introduzir novo target que perturb o gate 2.
  - **Responsável pela qualidade de build (gate owner)**: depende da integridade
    dos bundles espelhados e da estabilidade do two-way diff de
    `scripts/expected-tests.txt`.

## Goals

- Fazer com que a reabertura de uma pasta com leitura em andamento apresente a
  oferta de retomada no mesmo prazo e com as mesmas alternativas já definidas
  para a retomada em geral.
- Tornar visíveis, no momento da oferta, o arquivo ativo e o ponto salvo, para que
  a decisão "continuar" ou "começar do início" seja informada.
- Preservar a posição salva como única origem de verdade, sem duplicá-la em estados
  de interface que possam divergir.
- Evidenciar o comportamento com teste que meça a rota publicada e o payload,
  em vez de apenas verificar a presença literal de trechos de código.
- Manter os cinco bundles HTML byte-idênticos e o pin de testes inalterado.

## Non-Goals

- Criar novo comportamento de produto, nova janela de tempo ou nova alternativa de
  retomada além do já especificado em FR-009 da spec 010.
- Alterar a spec 010 ou renumerar requisitos/tarefas existentes dela.
- Adicionar novo target de teste ou modificar `scripts/expected-tests.txt`.
- Redesenhar a máquina de oferta, a fila de capítulos ou o armazém de bookmarks.
- Validar a experiência em dispositivo real neste ciclo (cobertura host apenas;
  qualquer afirmação de device é `[NV]`).

## Success Metrics

- **Oferta alcançável**: 100% dos casos de bookmark `Paused` na reabertura resultam
  em rota de diálogo publicada e janela de 3.000 ms (baseline em host: 0% — rota
  `none`).
- **Contexto completo**: 100% das ofertas in-app exibem nome de arquivo não vazio
  e offset salvo igual ao bookmark (baseline: campos ausentes no payload).
- **Chamada única da máquina**: exatamente 1 chamada de resolução de oferta por
  reabertura, com zero re-derivações em poll (baseline: 0 chamadas no caminho).
- **Integridade dos bundles**: 5 de 5 superfícies HTML byte-idênticas após a
  mudança (baseline: 5 de 5 idênticas em host).
- **Estabilidade do gate**: zero linhas alteradas em `scripts/expected-tests.txt`
  (baseline: pin `src-tauri::test_directory_resume` presente na linha 113).

## Cost of Inaction

T233 permanece `[ ]` e a funcionalidade de retomada de diretório continua
inacessível ao usuário apesar de implementada em grande parte. O código morto
continua suprimido por `expect(dead_code)`, o teste continua provando apenas que
strings existem no fonte (não que a oferta funciona), e qualquer esforço futuro
de retomada terá que refazer a mesma cola com menos contexto.

## Open Questions

- [NEEDS CLARIFICATION: `handle_directory_selected` é o único caminho de reabertura a publicar a rota, ou existem outros ingressos a cobrir?]
- [NEEDS CLARIFICATION: a seam de teste para observar rota publicada e payload já existe no bridge, ou precisa ser criada?]
- [NV] A janela de 3 s e o auto-continue em primeiro plano se comportam como
  especificado em dispositivo real? (fora de escopo de verificação deste ciclo)
