# Concept: Carregamento de Documentos e Início Direto de Reprodução pelo Título

- **Slug**: open-file-from-title
- **Created**: 2026-11-02
- **Recommended option**: Option A — Toque na Caixa de Título com Disparo Direto do SAF e Início Automático

## Options

### Option A — Toque na Caixa de Título com Disparo Direto do SAF e Início Automático (Recomendada)
- **Sketch**: O usuário toca na caixa de título (`.meta-box` / `#documentTitle`), que passa a exibir affordance tátil e semântica acessível (cursor pointer, `role="button"` e indicação sutil de toque). Esse toque invoca diretamente o seletor nativo do sistema Android (Storage Access Framework `ACTION_OPEN_DOCUMENT`) já configurado para `.md` e `.txt`. Assim que o usuário escolhe o documento, o sistema valida os bytes, atualiza a interface com o novo título e dispara automaticamente o início da leitura (TTS) — a menos que o documento possua um bookmark anterior salvo no banco de dados, caso em que a oferta de retomada padrão (`offerDialog`) é apresentada para honrar a escolha do usuário entre continuar ou recomeçar.
- **Appetite**: `small` (1 a 2 dias). A infraestrutura de ingestão (`DocIngest.kt`), validação de MIME, hashing Blake3-256 e gravação de snapshot já está implementada e testada no shell Android e JNI; resta apenas a rota IPC de disparo e o início imediato de reprodução no carregamento.
- **Trade-offs**: 
  - *Vantagens*: Implementação direta, leve e de altíssimo valor de uso; aproveita 90% do código já construído e testado; zero dependências externas ou bibliotecas adicionais; atende exatamente à requisição do usuário com menor atrito possível.
  - *Desvantagens*: Se o usuário tocar inadvertidamente no título enquanto navega na tela, o seletor do sistema é aberto (embora possa ser cancelado sem efeito colateral no áudio atual).
- **Rabbit holes**: Tentar implementar um seletor ou navegador de pastas próprio dentro do aplicativo em vez de usar o SAF do Android; tentar ignorar ou sobrescrever incondicionalmente o histórico de bookmarks existente sem respeitar o contrato de retomada FR-009.

### Option B — Botão de Ação Dedicado ("Abrir") no Cabeçalho
- **Sketch**: Em vez de tornar o texto/caixa de título clicável, um botão de ação com ícone de pasta/documento é posicionado ao lado do título ou no canto superior da Lente Sinestésica. O toque nesse botão dispara o seletor de arquivos SAF, seguido pelo início automático da reprodução do arquivo selecionado.
- **Appetite**: `small` (2 a 3 dias). Requer ajustes no layout visual da Lente Sinestésica para acomodar um novo controle interativo na barra superior, garantindo alinhamento e contraste segundo o design token SONA.
- **Trade-offs**: 
  - *Vantagens*: Elimina qualquer ambiguidade ou risco de clique acidental no título; deixa explícito para qualquer usuário que existe um botão de importação.
  - *Desvantagens*: Ocupa espaço visual na interface limpa e minimalista da Lente Sinestésica; não atende de forma literal ao desejo expresso do usuário de "clicar no título".
- **Rabbit holes**: Discussões extensas sobre posicionamento, ícones, tamanhos e quebra de simetria do design do cabeçalho da Lente Sinestésica.

### Option C — Modal/Gaveta de Gerenciamento de Documentos e Biblioteca
- **Sketch**: O toque no título abre um drawer ou modal de biblioteca/histórico ("Documentos Recentes", "Importar Novo Arquivo", "Favoritos"). A partir desse painel, o usuário escolhe se quer selecionar um documento recente ou acionar o explorador de arquivos para importar um novo.
- **Appetite**: `medium` (1 a 2 semanas). Exige criação de uma nova camada visual de biblioteca, listagem de arquivos recentes do `redb`, estados de seleção e gerenciamento de permissões persistidas.
- **Trade-offs**: 
  - *Vantagens*: Oferece visão completa da biblioteca local e histórico de leituras.
  - *Desvantagens*: Muito além do escopo solicitado; adiciona etapas intermediárias antes da reprodução, contrariando o objetivo de "já inicializar tocando".
- **Rabbit holes**: Persistência de URIs de longo prazo no Android (Scoped Storage), sincronização de metadados de arquivos e proliferação de telas de navegação.

## Recommendation

**Option A (Toque na Caixa de Título com Disparo Direto do SAF e Início Automático)**.
- Atende diretamente à dor e ao pedido explícito do usuário ("clicar no botão de título para abrir arquivo compatível e já inicializar tocando").
- Aproveita a arquitetura existente de `DocIngest.kt` e `shell_jni.rs` sem inflar a complexidade do aplicativo.
- Mantém o apetite estritamente `small`, com esforço concentrado na ligação entre o frontend e a chamada nativa do picker, além do disparo imediato de reprodução.
- Respeita os princípios de segurança, integridade de dados e autonomia do Rust definidos na Constituição do projeto.

## Out of Scope (for the recommended option)

- Criação de uma tela de gerenciamento de biblioteca ou explorador de arquivos próprio dentro do app.
- Suporte a formatos além dos já estabelecidos (`text/markdown` e `text/plain`).
- Edição do conteúdo do documento ou renomeação de arquivos via interface.
- Sobrescrever silenciosamente o bookmark de um documento pausado sem acionar a política de retomada FR-009.

## Assumptions to Validate

- O clique na caixa `.meta-box` (ou `#documentTitle`) consegue invocar com segurança uma chamada Tauri que dispara `DocIngest.pickDocument(activity)` através da JNI/ShellHost.
- O cancelamento da seleção de arquivos pelo usuário no SAF do Android fecha a janela sem interromper ou perturbar o áudio do documento que já estava em reprodução.
- A inicialização automática de reprodução (`play`) após a carga do novo arquivo respeita o tempo de buffer e síntese do motor de áudio sem travamento de UI.
