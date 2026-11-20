# Problem Definition: Carregamento de Documentos e Início Direto de Reprodução pelo Título

- **Slug**: open-file-from-title
- **Created**: 2026-11-02
- **Inputs used**: intake.md | research.md | user input

## Problem Statement

Ouvintes e leitores que utilizam o Ressoa para ouvir textos e audiolivros não possuem um meio direto e intuitivo dentro da interface gráfica da Lente Sinestésica para selecionar e carregar seus próprios arquivos de texto compatíveis (`.md`, `.txt`). Atualmente, trocar de leitura exige comandos externos ou mecanismos fora do fluxo do aplicativo, e mesmo quando um arquivo é carregado, o usuário enfrenta atrito desnecessário de múltiplos passos manuais para iniciar a escuta de um texto recém-selecionado.

## Affected Users & Stakeholders

- **Users**: Ouvintes de audiolivros e leitores de markdown em dispositivos móveis — impedidos de abrir e alternar entre seus documentos de estudo ou lazer diretamente pela interface do aplicativo em execução.
- **Users**: Usuários em trânsito ou com mobilidade reduzida — afetados pelo excesso de toques e fricção na tela para carregar um arquivo e começar a escutar.
- **Stakeholders**: Mantenedores e desenvolvedores do Ressoa — responsáveis por assegurar que o fluxo de carregamento respeite a autoridade exclusiva de playback do Rust, o modelo de segurança offline e as políticas de preservação de progresso/sessão (FR-009).

## Goals

- Permitir que os usuários selecionem e carreguem seus arquivos de texto suportados (`.md`, `.txt`) diretamente a partir de um toque na área de identificação/título do documento na interface ativa.
- Reduzir a fricção no consumo de conteúdo, iniciando a reprodução do áudio (TTS) imediatamente após a seleção bem-sucedida de um novo documento.
- Manter o respeito estrito à política de retomada e integridade de bookmarks quando o documento selecionado já tiver sido lido anteriormente.
- Assegurar conformidade com diretrizes de acessibilidade móvel (alvo de toque mínimo de 48×48dp, anúncio TalkBack e controle imediato de pausa/parada conforme WCAG 1.4.2).

## Non-Goals

- Criar um gerenciador de arquivos interno ou navegador de pastas personalizado dentro do aplicativo (o sistema operacional Android já provê o Storage Access Framework).
- Adicionar suporte a formatos binários complexos fora do escopo do leitor (ex: PDFs com layout complexo, DOCX, formatos de vídeo ou áudio proprietários).
- Alterar as regras fundamentais de autoridade de playback do Rust ou introduzir chamadas de rede no pipeline de ingestão.
- Modificar o comportamento de reprodução contínua de um documento que já está tocando caso a seleção de arquivo seja cancelada no sistema operacional.

## Success Metrics

- **Redução de passos para ouvir novo documento**: Passos manuais necessários para carregar um novo arquivo e escutar o áudio reduzidos de 3+ passos/ações para 1 toque e 1 confirmação no seletor de arquivos (baseline: impossível diretamente pela UI na versão atual).
- **Tempo para áudio após seleção**: Início da síntese/reprodução em menos de 1 segundo após o retorno do arquivo pelo sistema operacional (baseline: N/A).
- **Taxa de retenção de posição**: 100% de preservação dos bookmarks prévios ao reabrir documentos já lidos anteriormente (baseline: 100% no redb).

## Cost of Inaction

Se este problema não for resolvido, o aplicativo Ressoa permanece como um leitor restrito a documentos embutidos ou dependente de intents externos do sistema para troca de conteúdo. O usuário não consegue utilizar o aplicativo de forma autônoma no dia a dia como seu leitor padrão de markdown/audiolivros, tornando a interface da Lente Sinestésica um visualizador isolado em vez de uma ferramenta de leitura funcional completa.

## Open Questions

- [NEEDS CLARIFICATION: A área de toque deve abranger todo o contêiner de metadados (`.meta-box`) ou restringir-se estritamente ao texto do título (`#documentTitle`)?]
- [NEEDS CLARIFICATION: Quando o documento selecionado já possuir bookmark salvo no histórico (ex: lido até a metade), a reprodução deve iniciar imediatamente do ponto pausado ou deve apresentar o diálogo de oferta (`offerDialog`) conforme o contrato FR-009?]
- [NEEDS CLARIFICATION: Deve haver affordance visual explícita (como ícone sutil de pasta/documento ou cursor de botão) indicando que o título é interativo?]
- [NEEDS CLARIFICATION: Qual deve ser o comportamento em caso de recusa por arquivo acima do limite orçado de 93.636 bytes (alerta discreto na tela ou toast informativo)?]
