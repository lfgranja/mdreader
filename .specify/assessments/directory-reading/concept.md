# Concept: Leitura Contínua de Diretórios de Markdown

- **Slug**: directory-reading
- **Created**: 2026-11-03
- **Recommended option**: Option B — Playlist Linear de Capítulos com Início Antecipado (Streaming Chapter Queue)

## Options

### Option A — Concatenação Estática Bruta (Synthetic Single Document)
- **Sketch**: Ao selecionar a pasta no Android SAF, o app varre todos os arquivos `.md`/`.txt` de forma síncrona/imediata, concatena todo o texto em um único documento virtual em memória/disco com divisões de títulos (`# Nome do Arquivo`), e o entrega ao motor de playback existente exatamente como se fosse um único arquivo gigante.
- **Appetite**: small (dias)
- **Trade-offs**: 
  - *Vantagens*: Reutiliza 100% da infraestrutura atual de arquivo único do Rust e o modelo de persistência já existente sem nenhuma alteração estrutural no motor de áudio.
  - *Desvantagens*: Viola o limite orçado de memória de `93.636 bytes` (`MAX_DOC_BYTES`) para diretórios médios/grandes; viola o orçamento de latência inicial (TTFA ≤ 3s) ao exigir leitura, hash e concatenação de dezenas de arquivos antes de começar a falar; qualquer adição ou edição em uma única nota invalida o bookmark inteiro da pasta.
- **Rabbit holes**: Esgotamento de memória e travamento no heap ao tentar concatenar vaults com centenas de notas; estouro do tempo limite de inicialização em flash lenta do Android.

### Option B — Playlist Linear de Capítulos com Início Antecipado (Streaming Chapter Queue) [Recomendada]
- **Sketch**: O usuário toca no seletor e escolhe uma pasta via SAF (`ACTION_OPEN_DOCUMENT_TREE`). O Android/Kotlin resolve imediatamente o primeiro arquivo elegível (por convenção numérica `00-`..`04-` ou alfabética caso não haja numeração) e já dispara a síntese e a reprodução dele em menos de 1 segundo (respeitando o TTFA ≤ 3s). Em segundo plano (thread de I/O), o sistema indexa os arquivos restantes em uma fila ordenada de capítulos leves. Ao atingir o final audível de um arquivo, o motor Rust executa a transição perfeita (*gapless transition*) para o próximo capítulo da fila, mantendo o progresso durável individual gravado por arquivo (`<dir_uri>:<file_fingerprint>`).
- **Appetite**: medium (2 a 3 semanas)
- **Trade-offs**:
  - *Vantagens*: Respeita rigorosamente a Constituição (orçamento TTFA ≤ 3s, limite de 93KB por arquivo individual, Rust como autoridade de transporte); zero atrito cognitivo para o ouvinte; preserva marcadores mesmo se arquivos forem renomeados ou novas notas forem salvas na pasta; transição mãos-livres completa.
  - *Desvantagens*: Requer estender a camada de controle de sessão do Rust para suportar conceito de `ChapterQueue` / `PlaylistState` sobre a autoridade do `Player`.
- **Rabbit holes**: Bloqueio de IPC no SAF caso tente varrer centenas de arquivos recursivos de uma vez em vez de fazer indexação preguiçosa/paginada; complexidade de navegação por controle de fone de ouvido (distinguir se o botão "Próximo" avança 30s ou pula para o próximo arquivo/capítulo).

### Option C — Gerenciador Completo de Coleções com Painel Interativo de Playlist (Interactive Vault Explorer)
- **Sketch**: Ao selecionar o diretório, o aplicativo exibe uma tela/modal dedicada de inspeção da pasta com a lista completa de arquivos detectados, permitindo ao usuário reordenar itens manualmente por arrastar-e-soltar, marcar/desmarcar rascunhos para exclusão, visualizar o tempo estimado de fala por arquivo e selecionar qual arquivo iniciar.
- **Appetite**: large (4 a 6 semanas)
- **Trade-offs**:
  - *Vantagens*: Controle total pelo usuário sobre rascunhos e notas antigas; elimina completamente o problema de ler rascunhos acidentalmente.
  - *Desvantagens*: Viola diretamente o critério de simplicidade e baixo atrito (exige múltiplos toques e decisões visuais antes de ouvir o som); cria atrito severo em momentos de direção ou caminhada; transforma o Ressoa em um gerenciador de arquivos em vez de um leitor direto e minimalista.
- **Rabbit holes**: Implementação de árvore interativa de diretórios e drag-and-drop na Lente Sinestésica; gerenciamento de estado complexo entre UI webview e disco no Android.

## Recommendation

**Recomendação: Option B — Playlist Linear de Capítulos com Início Antecipado (Streaming Chapter Queue)**.

*Justificativa*: A Option B é a única que atende simultaneamente aos requisitos de negócio e aos pilares inegociáveis da Constituição do Ressoa:
1. **Atende ao TTFA ≤ 3s**: O primeiro arquivo começa a tocar em < 1s enquanto a árvore é indexada em background.
2. **Respeita o Orçamento de Memória**: Cada arquivo individual permanece sob o teto de 93.636 bytes, processado e descarregado ordenadamente.
3. **Entrega a Promessa Central (Zero Toques)**: O áudio transiciona continuamente entre arquivos sem intervenção do usuário.
4. **Evita o Inchaço de Escopo (Anti-Bloat)**: Não transforma o app em um gerenciador de arquivos complexo (como na Option C), mantendo a Lente Sinestésica focada na experiência auditiva imersiva.

## Out of Scope (for the recommended option)

- Reordenação manual via interface de arrastar-e-soltar (drag-and-drop).
- Edição ou exclusão de arquivos no sistema de arquivos do dispositivo.
- Monitoramento reativo em tempo real via file-watcher para atualizar dinamicamente a fila enquanto o áudio estiver tocando.
- Leitura de subpastas além do 1º nível hierárquico na versão inicial (restringindo-se aos arquivos da pasta selecionada e suas subpastas imediatas ordenadas).

## Assumptions to Validate

- O SAF `ACTION_OPEN_DOCUMENT_TREE` permite obter os metadados do primeiro arquivo e ler seu stream inicial em menos de 500ms no Android 10+ (API 29+).
- A convenção de ordenação natural (prefixo numérico `01-`, `02-` e fallback alfabético) cobre >85% dos casos de uso de audiobooks e cursos em Markdown sem necessidade de reordenação manual na UI.
- O motor Rust consegue encadear a síntese do arquivo subsequente quando o arquivo atual atinge 80% do playback sem introduzir pausas audíveis ou estalos no `CpalBackend`.
