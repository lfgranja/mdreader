# Concept: Nome de exibição no ACTION_VIEW via MediaStore

- **Slug**: action-view-display-name
- **Created**: 2026-11-21
- **Recommended option**: Option A — fallback MediaStore localizado no Kotlin

## Options

### Option A — fallback MediaStore localizado no Kotlin (recomendada)
- **Sketch**: manter a consulta direta a `OpenableColumns.DISPLAY_NAME` como
  primeiro tentativa; se ela falhar e o URI for `content://` com authority
  `MediaStore.AUTHORITY`, extrair `id` do último segmento e `volume` do primeiro
  (default `external`) e consultar `MediaStore.Files.getContentUri(volume)`
  projetando `DISPLAY_NAME` com selection `${BaseColumns._ID}=?`. Filtrar o
  resultado com o mesmo predicado que rejeita vazio e somente dígitos. No ramo
  `ACTION_VIEW`, montar o URI sintético incondicionalmente a partir de
  `displayName ?: lastPathSegment?.takeIf(isUsableName) ?: "document.md"`.
- **Appetite**: small — uma dezena de linhas em um único arquivo Kotlin; nenhuma
  mudança em Rust, manifesto, contrato JNI ou fingerprint.
- **Trade-offs**:
  - *Vantagens*: usa a mesma coluna que já funcionava, muda apenas a coleção
    consultada; converte o `content://display_name/` que o Rust já decodifica;
    remove a única origem do título numérico (a URI bruta como fallback); diff
    facilmente auditável em host.
  - *Desvantagens*: o `volume` inferido do primeiro segmento pode precisar de
    validação futura; o nome obtido depende de permissão de leitura no
    MediaStore; a prova final continua sendo em dispositivo (`[NV]`).
- **Rabbit holes**: consultar o URI de item mais de uma vez sem necessidade;
  aceitar o último segmento numérico como "nome"; reintroduzir `uri.toString()`
  como fallback "seguro"; tocar no ramo `ACTION_SEND` por similaridade.

### Option B — resolver o nome no Rust e passá-lo pelo JNI
- **Sketch**: parar de codificar o nome no URI; estender `onDocumentIngested`
  com um parâmetro de nome resolvido e fazer `picked_document_name` consumi-lo.
- **Appetite**: medium — muda a assinatura JNI nos dois lados, o `IngestSnapshot`
  e os testes que chamam a ponte.
- **Trade-offs**:
  - *Vantagens*: elimina o "URI sintético" como transporte de metadado.
  - *Desvantagens*: quebra a compatibilidade com todos os chamadores do JNI
    (SAF, `ACTION_SEND`, hooks de debug) e contradiz a fronteira atual, em que o
    Kotlin transporta bytes e o Rust decide título a partir da URI.
- **Rabbit holes**: migração incompleta de chamadores; regressão no fingerprint
  se o nome passar a participar do hash; necessidade de novos testes em Rust.

### Option C — deferir e aceitar o título numérico
- **Sketch**: não corrigir neste ciclo; tratar o título como cosmético e
  concentrar esforço na latência de 183 s.
- **Appetite**: none.
- **Trade-offs**:
  - *Vantagens*: custo imediato zero.
  - *Desvantagens*: mantém a regressão aberta, deixa o usuário sem identificar
    o documento e mistura dois problemas independentes (titulação e pré-roll).
- **Rabbit holes**: nenhum — é a opção de não fazer.

## Recommendation

**Option A — fallback MediaStore localizado no Kotlin.**

É a menor mudança que ataca a causa exata: a coluna já é a correta, apenas a
coleção consultada estava errada. O URI sintético resultante já é decodificado
por `picked_document_name` (`src-tauri/src/shell_jni.rs:1103`), então nenhuma
outra camada precisa mudar. A opção atende às cinco métricas de sucesso e
mantém o diff confinado a um arquivo, com verificação estrutural em host; as
opções B e C, respectivamente, ampliam a superfície sem necessidade e não
corrigem o sintoma. A latência de 183 s permanece **DEFER**, fora deste ciclo.

## Out of Scope (for the recommended option)

- Mudanças em `queryName(resolver, treeUri, docId)` (SAF/diretório) e no ramo
  `ACTION_SEND`.
- Mudanças em `blake3_256`, `writeSnapshot`, `onDocumentIngested` ou em
  `picked_document_name` no Rust.
- Edição de `specs/019`, `specs/009`, do bug ou de `converge.md` de outra spec.
- Redução da latência de primeiro áudio (183 s / pré-roll 60 s) — **DEFER**.
- Qualquer afirmação de verificação em dispositivo; toda a evidência aqui é de
  host, com `[NV]` para device.

## Assumptions to Validate

- O `volume` extraído do primeiro segmento do caminho, com default `external`,
  é suficiente para os URIs do MediaStore encontrados na prática.
- `Uri.encode` no Kotlin e `percent_decode` no Rust formam um par simétrico
  para nomes com espaços e acentos (verificado por leitura estrutural em host;
  device `[NV]`).
- O predicado `isUsableName` cobre os casos que importam: nulo, em branco e
  somente dígitos.
- O fallback `document.md` é aceitável como último recurso, pois preserva a
  forma de arquivo e evita o título numérico.
- Nenhum novo target de teste é necessário: a evidência é inspeção estrutural
  do diff em host, e gates/builds ficam com o agente principal.
