# Concept: Cola da oferta de retomada de diretório

- **Slug**: directory-resume-offer
- **Created**: 2026-11-20
- **Recommended option**: Option A — Cola mínima sobre o mecanismo existente

## Options

### Option A — Cola mínima sobre o mecanismo existente (recomendada)
- **Sketch**: manter o cálculo de rota, o armazém de bookmarks, a máquina de oferta
  e a interface já existentes; ligar o caminho de reabertura ao estado publicado,
  enriquecer o dado que já chega à interface com o nome do arquivo e a posição
  salva, e fazer a interface consumir esse dado ao atualizar a oferta. A evidência
  passa a medir a rota publicada e o conteúdo entregue, não a presença de texto no
  fonte.
- **Appetite**: small (1–2 dias; poucas dezenas de linhas em três arquivos de código
  mais os espelhos HTML).
- **Trade-offs**:
  - *Vantagens*: menor risco de regressão — quase nada é reescrito; usa a máquina
    de oferta já testada; preserva a posição salva como fonte única; cabe no target
    de teste já pinado.
  - *Desvantagens*: exige disciplina de contrato entre Rust e interface (payload
    passa a ser parte da API interna); exige espelhar a mudança de interface em
    cinco superfícies HTML byte-idênticas.
- **Rabbit holes**: publicar a rota em um caminho e esquecer outros ingressos de
  reabertura; chamar a máquina de oferta mais de uma vez e duplicar o expiry; criar
  uma segunda origem de posição ao copiar o offset para a interface sem ligá-lo ao
  bookmark.

### Option B — Reescrever o fluxo de oferta na interface
- **Sketch**: abandonar a máquina de oferta do núcleo e decidir na própria
  interface, a partir do bookmark bruto, se a oferta aparece, quanto tempo dura e
  o que acontece no expiry.
- **Appetite**: medium (1–2 semanas).
- **Trade-offs**:
  - *Vantagens*: remove a dependência da publicação de rota entre camadas.
  - *Desvantagens*: cria segunda autoridade de oferta, duplica regras já
    especificadas (3 s, continuar padrão, auto-continue em foreground) e
    contradiz a direção de manter o Rust como autoridade de transporte/estado.
- **Rabbit holes**: divergência entre o que a interface decide e o que o núcleo
  persistiu; testes que passam na interface mas não no motor; necessidade de
  expor bookmarks diretamente à webview.

### Option C — Não corrigir agora (deferir T233)
- **Sketch**: manter T233 `[ ]` e aceitar que a retomada de diretório existe no
  código mas não é alcançável pelo usuário.
- **Appetite**: none.
- **Trade-offs**:
  - *Vantagens*: custo imediato zero.
  - *Desvantagens*: deixa o requisito FR-009 da spec 010 sem cumprimento efetivo,
    mantém código morto e adia o único benefício visível da funcionalidade de
    diretórios já construída.
- **Rabbit holes**: nenhum — é a opção de não fazer.

## Recommendation

**Option A — Cola mínima sobre o mecanismo existente.**

É a única opção que fecha a lacuna com risco proporcional ao problema: o mapa de
evidência mostra que todas as peças já existem e que a falha é de ligação, não de
projeto. A opção atende diretamente às métricas de sucesso definidas (rota
publicada em 100% dos bookmarks pausados, payload completo, uma única resolução de
oferta, bundles idênticos e pin de testes intacto) e evita a segunda autoridade de
estado que a Option B introduziria. A Option C não atende a nenhuma métrica.

## Out of Scope (for the recommended option)

- Nova janela de tempo, nova alternativa de retomada ou mudança em FR-009 da
  spec 010.
- Edição de `specs/010`, renumeração de T232/T233, ou criação de novo target de
  teste / alteração em `scripts/expected-tests.txt`.
- Reescrita da máquina de oferta, da fila de capítulos ou do armazém de bookmarks.
- Verificação em dispositivo real; toda a evidência deste ciclo é de host, e
  afirmações de device são `[NV]`.
- Novo comportamento visual: a mudança de interface é apenas a leitura do payload
  já existente.

## Assumptions to Validate

- `handle_directory_selected` é o único caminho de reabertura que precisa
  publicar a rota neste ciclo (se surgir outro ingresso, ele será tratado como
  escopo adicional, não como desculpa para adiar).
- Existe, ou pode ser criada sem novo target, uma seam no bridge para observar a
  rota publicada e o payload no teste já pinado.
- Espelhar o arquivo-fonte nos quatro bundles preserva a identidade byte a byte
  exigida pela auditoria C-H8.
- A janela de 3 s e o auto-continue em foreground permanecem corretos porque a
  máquina de oferta já existente continua sendo a única resolutora.
