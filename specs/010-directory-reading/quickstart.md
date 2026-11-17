# Quickstart & Verification Scenarios: Leitura Contínua de Diretórios de Markdown

- **Feature**: `010-directory-reading`
- **Date**: 2026-11-03
- **Status**: Complete

Este documento apresenta os cenários de teste de ponta a ponta (E2E) para verificação prática das garantias funcionais da leitura de diretórios (010-directory-reading).

---

## Scenario 1: Abertura de Diretório com Ordenação Natural e Auto-Play Imediato (< 3s)

**Objetivo**: Validar a seleção de pasta contendo múltiplos arquivos ordenados numericamente e garantir que o áudio do Capítulo 1 inicia em menos de 3 segundos (TTFA).

### Preparação do Ambiente:
1. Criar um diretório temporário de teste com 3 arquivos Markdown:
   - `01-introducao.md`: 500 palavras em pt-BR.
   - `02-arquitetura.md`: 800 palavras em pt-BR.
   - `10-conclusao.md`: 300 palavras em pt-BR.
2. Iniciar o Ressoa na tela principal da Lente Sinestésica.

### Passos de Execução:
1. Tocar no card principal de metadados (`.meta-box`) ou acionar `pick_directory`.
2. No seletor nativo do sistema operacional (SAF), selecionar a pasta criada e confirmar o acesso.
3. Observar o tempo decorrido até a primeira emissão de som PCM e a atualização dos elementos visuais.

### Resultados Esperados:
- **TTFA**: O áudio audível do arquivo `01-introducao.md` inicia em **< 3.0 segundos**.
- **Interface**:
  - O elemento `<h1>` exibe `"01-introducao.md"`.
  - O subtítulo exibe `"Capítulo 1 de 3 • [Primeira Seção]"`.
  - O visualizador de forma de onda (`#waveform`) ondula sincronizado com a fala.
  - A fila `ChapterQueue` mantém os arquivos ordenados corretamente como `[01-, 02-, 10-]` (ordenação natural reconhecendo `10` após `02`, e não ordenação lexicográfica crua onde `10` ficaria antes de `02`).

---

## Scenario 2: Transição Contínua e Gapless Entre Arquivos (Capítulo 1 → 2)

**Objetivo**: Verificar que ao concluir o primeiro arquivo, o sistema transiciona de forma fluida para o segundo arquivo sem estalos de áudio (*pops*) ou engasgos de interface.

### Passos de Execução:
1. Permitir que a reprodução alcance os últimos 5 segundos do arquivo `01-introducao.md`.
2. Observar o comportamento auditivo e a transição visual no momento em que a última frase do arquivo 1 é concluída.

### Resultados Esperados:
- **Áudio**: Intervalo de silêncio entre o final da última palavra do arquivo 1 e o início da primeira palavra do arquivo 2 é **inferior a 1.0 segundo** (SC-004). Sem cliques, ruídos ou reinicialização do dispositivo de áudio `cpal`.
- **Persistência**: O capítulo 1 é marcado como concluído (`completed`) e o capítulo 2 tem seu início gravado no `DIRECTORY_BOOKMARKS`.
- **Interface**:
  - `<h1>` atualiza suavemente para `"02-arquitetura.md"`.
  - Subtítulo atualiza para `"Capítulo 2 de 3"`.
  - Leitor de tela (TalkBack) emite anúncio acessível discreto via `#liveRegion` ("Capítulo 2 de 3: 02-arquitetura.md").

---

## Scenario 3: Pausa Deliberada, Encerramento e Retomada Precisa (Resume Offer)

**Objetivo**: Verificar que uma pausa deliberada no meio de um capítulo longo é preservada após fechar o aplicativo e reabrir a mesma pasta.

### Passos de Execução:
1. Durante a reprodução do arquivo `02-arquitetura.md`, pausar deliberadamente a leitura aos 45,0 segundos.
2. Forçar o encerramento do aplicativo (`kill` do processo ou fechar janela).
3. Reabrir o Ressoa e acionar novamente a seleção da mesma pasta.

### Resultados Esperados:
- **Diálogo de Retomada**: A interface apresenta o diálogo de oferta (`offerDialog`) por 3 segundos com as opções:
  - *"Continuar do arquivo 02-arquitetura.md aos 45s"*
  - *"Recomeçar do início"*
- **Confirmação "Continuar"**: O áudio recomeça imediatamente nos 45s do arquivo 2, mantendo o arquivo 3 enfileirado na sequência.
- **Alternativa "Recomeçar"**: Se o usuário selecionar recomeçar, o playback inicia do segundo 0.0 do arquivo 1 (`01-introducao.md`).

---

## Scenario 4: Navegação Contextual por Fone de Ouvido Bluetooth e Lock Screen

**Objetivo**: Validar a navegação hierárquica usando os botões de skip de hardware ou da notificação `MediaSession`.

### Passos de Execução:
1. Com o arquivo `01-introducao.md` em reprodução em sua primeira seção, acionar o botão `SkipToNext` no fone de ouvido.
   - **Resultado**: O leitor salta para a próxima seção interna do mesmo arquivo.
2. Avançar até a última seção do arquivo `01-introducao.md` e acionar novamente `SkipToNext`.
   - **Resultado**: Reconhecendo que já está na última seção interna, o sistema avança para o início do próximo arquivo (`02-arquitetura.md`).
3. Nos primeiros 2 segundos do arquivo `02-arquitetura.md`, acionar `SkipToPrevious`.
   - **Resultado**: O sistema retrocede diretamente para o início do capítulo anterior (`01-introducao.md`).

---

## Scenario 5: Resiliência Contra Arquivos Inválidos ou com Limite de Tamanho Excedido

**Objetivo**: Garantir que arquivos com problemas não causem falha geral na reprodução da pasta nem bloqueiem os demais arquivos válidos.

### Preparação do Ambiente:
Adicionar à pasta de teste:
- `03-invalido.bin`: Arquivo binário não-UTF8.
- `04-vazio.md`: Arquivo de 0 bytes.
- `05-pesado.md`: Arquivo de texto com 120 KB (> `MAX_DOC_BYTES = 93.636 bytes`).
- `06-final.md`: Arquivo Markdown válido (400 palavras).

### Passos de Execução:
1. Iniciar a leitura da pasta a partir do capítulo 2 e avançar para os capítulos subsequentes.

### Resultados Esperados:
- Ao alcançar `03-invalido.bin` e `04-vazio.md`, o motor Rust identifica a incompatibilidade, emite registro discreto no log de diagnóstico (`DiagnosticKind::Failure` com razão `FormatUndeliverable`) e pula silenciosamente para o próximo arquivo.
- Ao alcançar `05-pesado.md`, o limite de 93KB é detectado, registrando recusa pontual sem crash.
- O sistema alcança com sucesso o arquivo `06-final.md` e inicia a fala do texto normalmente.
- O transporte não trava em estado fantasma nem produz silêncio infinito.
