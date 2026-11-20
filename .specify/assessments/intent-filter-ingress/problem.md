# Problem Definition: Friction in Document Ingestion

- **Slug**: intent-filter-ingress
- **Date**: 2026-11-11

## 1. Quem é afetado?
Todos os usuários de aplicativos de notas (Obsidian, Logseq, Markor, Zettel Notes) e usuários que gerenciam documentos técnicos ou livros em Markdown no Android.

## 2. O que dói hoje?
1. **Fricção de Fluxo:** Para ouvir uma nota criada no Obsidian, o usuário precisa:
   - Sair do Obsidian.
   - Abrir o Ressoa.
   - Tocar na caixa de título (`.meta-box`).
   - Navegar manualmente na árvore do sistema de arquivos (`/storage/emulated/0/Documents/Obsidian/...`) para reencontrar a mesma nota que estava lendo.
2. **Invisibilidade no Ecossistema Android:** O app não existe como opção no menu "Abrir com..." nem na Share Sheet do Android.
3. **Barreira de Adoção:** O aplicativo parece um silo isolado, gerando atrito no hábito diário de escuta durante caminhadas ou deslocamentos.

## 3. Objetivos
- Permitir abertura em 1 toque a partir de qualquer app terceiro (Obsidian, Files, navegadores) via `ACTION_VIEW`.
- Suportar compartilhamento direto de notas e textos via `ACTION_SEND`.
- Preservar a estabilidade do motor Rust, processando a nova nota e disparando o auto-play com persistência de posição em `redb`.

## 4. Não-Objetivos
- Não editar ou sincronizar arquivos de volta para o Obsidian (o Ressoa é estritamente um leitor de áudio offline).
- Não gerenciar permissões de escrita de arquivos.
