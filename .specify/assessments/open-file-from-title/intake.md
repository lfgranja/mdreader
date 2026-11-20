# Idea Intake: Abrir arquivo pelo botão de título e iniciar reprodução

- **Slug**: open-file-from-title
- **Created**: 2026-11-02
- **Source**: "pasted text"
- **Type**: new-capability

## Idea (as captured)

> "Clicar no botão de título para abrir um arquivo compatível e já inicializar tocando."

## Restated

Permitir que o usuário interaja com a área/botão de título do documento na interface para disparar a seleção de um arquivo de áudio ou documento compatível (via file picker do sistema / SAF no Android) e, ao confirmar o arquivo, iniciar imediatamente a reprodução do áudio (TTS/leitura).

## Origin & Context

- **Raised by**: Usuário / Mantenedor do projeto
- **Trigger**: Necessidade de carregar documentos no aplicativo diretamente pela interface da Lente Sinestésica, permitindo fluxo contínuo de abertura e início imediato de reprodução sem passos intermediários manuais.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Quais são as extensões/formatos de arquivo compatíveis aceitos para leitura (ex: `.md`, `.txt`, `.epub`, etc.)?]
- [NEEDS CLARIFICATION: No Android, o seletor de arquivos deve ser invocado via Storage Access Framework (SAF / Intent `ACTION_OPEN_DOCUMENT`) através de plugin Tauri/JNI ou via canal IPC existente?]
- [NEEDS CLARIFICATION: Se o documento selecionado já possui um bookmark salvo com posição anterior no `session` (redb), a abertura deve tocar do início diretamente ou disparar a oferta de retomar (`offerDialog` / FR-009)?]
- [NEEDS CLARIFICATION: O elemento clicável deve ser a caixa de metadados inteira (`.meta-box`) ou especificamente o elemento de título (`#documentTitle`)?]
