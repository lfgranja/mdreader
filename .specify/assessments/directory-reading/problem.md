# Problem Definition: Leitura de Diretórios Inteiros como Sessão Contínua

- **Slug**: directory-reading
- **Created**: 2026-11-03
- **Inputs used**: intake.md, research.md

## Problem Statement

Ouvintes de textos e notas técnicas que organizam seus conteúdos em pastas e vaults locais (como no Obsidian ou diretórios de documentação Markdown) são obrigados a abrir e selecionar manualmente cada arquivo individualmente a cada poucos minutos, interrompendo o fluxo cognitivo e inviabilizando a escuta passiva em deslocamentos ou tarefas manuais. Quando tentam contornar essa limitação combinando arquivos manualmente, perdem tempo e sofrem com a poluição auditiva de rascunhos obsoletos e notas auxiliares lidas fora de ordem.

## Affected Users & Stakeholders

- **Users**:
  - **Pesquisadores, desenvolvedores e estudantes com vaults no Obsidian/Markdown**: possuem dezenas ou centenas de notas encadeadas ou organizadas por pastas que gostariam de ouvir sequencialmente como audiobooks.
  - **Ouvintes em trânsito/direção (Hands-free / Driving-time listeners)**: não podem interagir com a tela para trocar de arquivo a cada capítulo sem violar a segurança e a ergonomia de uso.
- **Stakeholders**:
  - **Mantenedor do projeto Ressoa**: necessita expandir a utilidade real do leitor para além de arquivos unitários sem comprometer as garantias da Constituição (Rust como autoridade exclusiva, orçamento de latência ≤ 3s, limite orçado de memória e operação estritamente offline).

## Goals

- Permitir a audição contínua e sequencial de um conjunto de arquivos de texto/Markdown agrupados em uma pasta sem necessidade de interação tátil entre o término de um arquivo e o início do próximo.
- Respeitar a ordem lógica e cronológica pretendida pelo ouvinte (reconhecendo sequências numéricas declaradas e índices explícitos quando disponíveis).
- Preservar o progresso individual e coletivo de leitura de forma durável, permitindo pausar e retomar exatamente no trecho e arquivo corretos sem risco de recomeçar a pasta do zero.
- Proteger a experiência auditiva contra o consumo acidental de arquivos corrompidos, arquivos não textuais ou rascunhos duplicados indesejados.

## Non-Goals

- Não ser um gerenciador de arquivos (file manager) completo, visualizador de árvore de diretórios ou editor de notas.
- Não executar conversão de formatos não textuais complexos (PDFs diagramados, planilhas, binários).
- Não implementar sincronização em nuvem ou streaming de diretórios remotos (o leitor permanece estritamente offline e local).
- Não monitorar em tempo real (file-watcher agressivo) alterações em lote de centenas de arquivos enquanto a tela estiver desligada.

## Success Metrics

- **Zero toques manuais entre arquivos**: 100% de transição automática audível do arquivo `N` para o arquivo `N+1` sem intervenção do usuário durante a sessão contínua (baseline atual: 1 toque por arquivo para navegar e selecionar).
- **Tempo até o primeiro som (TTFA)**: Primeira palavra audível iniciada em menos de 3,0 segundos após selecionar a pasta, independentemente do volume total de arquivos do diretório (baseline: atendido em arquivos únicos; meta: mantido em pastas).
- **Fidelidade de Retomada**: 100% de restauração precisa da posição de audição (arquivo exato e segundo exato) ao reabrir o app após fechamento deliberado ou reinicialização do dispositivo.

## Cost of Inaction

Se este problema não for resolvido, o Ressoa continuará restrito ao nicho de "leitor de arquivo único isolado", desqualificando-se para o maior público orgânico de ouvintes de Markdown (usuários de PKM, cursos técnicos e vaults de anotações). O usuário é forçado a abandonar o app em favor de scripts manuais de concatenação no desktop ou leitores com permissões invasivas de nuvem.

## Open Questions

- [NEEDS CLARIFICATION: Qual o comportamento padrão de ordenação em diretórios que não possuem convenção numérica nem arquivo de índice (`README.md`): ordenação alfabética simples ou ordenação por data de modificação mais recente?]
- [NEEDS CLARIFICATION: Em diretórios profundos com múltiplas subpastas, a travessia deve achatar todos os arquivos (depth-first traversal) ou limitar-se estritamente à pasta raiz selecionada?]
- [NEEDS CLARIFICATION: A unidade de bookmark durável deve ser atrelada a uma impressão digital global do diretório inteiro (invalidada caso um arquivo seja adicionado) ou um índice composto por arquivo individual?]
- [NEEDS CLARIFICATION: Deve ser oferecida uma tela/diálogo intermediário leve para confirmação ou exclusão de rascunhos antes de iniciar o áudio, ou a reprodução deve iniciar imediatamente a partir do primeiro arquivo detectado?]
