# Quickstart & Validation Guide: Feature 009 (open-file-from-title)

- **Feature**: `009-open-file-from-title`
- **Date**: 2026-11-02
- **Status**: Complete

Este guia orienta a validação ponta a ponta da funcionalidade de abertura de documentos e início de reprodução pelo título.

## Pré-requisitos
- Dispositivo Android conectado via ADB (ex: `53cf9aa7`) ou emulador ARM64.
- Documento de teste `.md` ou `.txt` transferido para o armazenamento do dispositivo (ex: `/sdcard/Download/teste.md`).

## Cenário 1: Abertura de Novo Documento com Auto-play Imediato (P1)
1. Instale e abra o aplicativo Ressoa.
2. Na tela inicial da Lente Sinestésica, toque na caixa de título (`.meta-box`).
3. **Resultado Esperado 1**: O seletor nativo de documentos do Android (SAF) é exibido, filtrando por arquivos Markdown e texto.
4. Navegue e selecione o arquivo `teste.md`.
5. **Resultado Esperado 2**: O seletor fecha, o título do documento na Lente Sinestésica é atualizado para o nome do arquivo selecionado e o áudio da narração (TTS) começa a tocar imediatamente, com a forma de onda (waveform) reagindo ao RMS de reprodução.

## Cenário 2: Reabertura de Documento com Histórico de Pausa (P2 / FR-009)
1. Durante a reprodução do documento do Cenário 1, toque no botão central de Pausa (`#btnPlayPause`).
2. Aguarde 2 segundos para gravação durável da posição na base de dados (`redb`).
3. Toque novamente na caixa de título (`.meta-box`) e selecione o mesmo arquivo.
4. **Resultado Esperado**: O diálogo de oferta de retomada (`offerDialog`: "Continuar de onde parou ou começar do início?") é exibido sobre a tela por 3 segundos. O áudio não toca enquanto a oferta aguarda resposta.
5. Ao selecionar "Continuar", a reprodução retoma exatamente do ponto pausado.

## Cenário 3: Cancelamento de Seleção no SAF (P3)
1. Inicie a reprodução de um áudio.
2. Com o áudio tocando, toque na caixa de título (`.meta-box`).
3. Quando o seletor de arquivos do Android abrir, pressione o botão "Voltar" ou o botão Cancelar.
4. **Resultado Esperado**: O seletor é fechado, o áudio que estava tocando continua sem engasgos ou interrupção e nenhum metadado na tela é corrompido.

## Cenário 4: Acessibilidade e Teclado
1. Navegue pelos controles usando a tecla `Tab` de um teclado conectado ou navegação acessível TalkBack.
2. Verifique se o foco visual (`:focus-visible`) destaca a `.meta-box` com anel contrastante (`var(--sona-etwahl)`).
3. Pressione `Enter` ou `Space`. O seletor de arquivos SAF deve abrir normalmente.
