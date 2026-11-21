# Implementation Plan: Nome de exibição no ACTION_VIEW via MediaStore

**Branch**: `025-action-view-display-name` | **Date**: 2026-11-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/025-action-view-display-name/spec.md`

**Note**: este ciclo produz apenas `plan.md` como artefato de planejamento. As
decisões de Phase 0/1 exigidas pelo workflow estão registradas neste arquivo
(§Phase 0 e §Phase 1) para respeitar o limite de arquivos do ciclo — nenhum
`research.md`, `data-model.md`, `contracts/` ou `quickstart.md` é criado aqui.

## Summary

Corrigir a titulação de documentos abertos via `ACTION_VIEW` com URI de item do
MediaStore. A consulta direta a `OpenableColumns.DISPLAY_NAME` no URI do item
retorna `null`; a mesma coluna `_display_name` é exposta pela coleção
`MediaStore.Files`, consultada por `_id=?`. O nome obtido é filtrado por um
predicado único que rejeita valores vazios e numéricos, e o ramo `ACTION_VIEW`
passa a construir sempre a URI sintética `content://display_name/<nome
codificado>` que o Rust já decodifica. O diff é confinado a um arquivo Kotlin;
Rust, SAF, `ACTION_SEND`, fingerprint e ponte JNI permanecem intactos.

## Technical Context

**Language/Version**: Kotlin (Android shell, JVM) + Rust (workspace Tauri 2) como consumidor da URI

**Primary Dependencies**: `android.content.ContentResolver`, `android.provider.MediaStore`, `android.provider.BaseColumns`, `android.provider.OpenableColumns`, `android.net.Uri`

**Storage**: snapshot em arquivo no diretório de arquivos do app; `redb` para bookmarks — nenhum modificado neste ciclo

**Testing**: inspeção estrutural em host do diff e de `picked_document_name`; gates/builds/testes são do agente principal

**Target Platform**: Android é o destino do app; toda a evidência deste ciclo é de host, com `[NV]` para dispositivo

**Project Type**: híbrido (shell Android + núcleo Rust + webview)

**Performance Goals**: sem nova hot path; uma consulta adicional ao ContentProvider apenas quando a consulta direta falha. A latência de 183 s até o primeiro áudio é **DEFER**

**Constraints**: gate autoritativo `scripts/verify.sh`; Factory HEAD `b1deab1` somente leitura; nenhuma mudança em Rust, manifesto ou specs de outros ciclos; nenhum `git add`/`commit` por subagentes

**Scale/Scope**: um arquivo Kotlin, quatro seções alteradas, 36 inserções e 10 remoções no diff observado em host

## Constitution Check

*GATE: passa antes do Phase 0 e reavaliado após o Phase 1.*

- Nenhum novo gate é introduzido: `scripts/verify.sh` permanece a autoridade.
- Fronteira de responsabilidade preservada: o Kotlin continua transportando bytes e codificando o nome na URI; o Rust continua decodificando e decidindo o título.
- Nenhuma segunda origem de nome: um único predicado filtra todos os candidatos antes da construção da URI sintética.
- Nenhuma mudança de contrato JNI, de fingerprint ou de schema; nenhum arquivo de outro ciclo é editado.
- Orçamento de alocação: a consulta adicional ocorre somente no caminho de fallback; nenhum dado é copiado ou retido além do nome já existente.
- Documentação de governança: nenhuma emenda à constituição é necessária.

Resultado: **sem violações** — tabela de Complexity Tracking permanece vazia.

## Project Structure

### Documentation (this feature)

```text
specs/025-action-view-display-name/
├── spec.md                      # requisitos FR-025-001..005, SC-025-001..005
├── plan.md                      # este arquivo (Phase 0/1 embutidos)
├── tasks.md                     # T001..T012
├── analyze.md                   # relatório READ-ONLY FR → T → código
├── converge.md                  # registro append-only
└── checklists/
    └── requirements.md          # checklist de qualidade da especificação
```

### Source Code (repository root)

```text
android-shell/src/main/kotlin/app/mdreader/DocIngest.kt
    # imports BaseColumns/MediaStore; isUsableName; queryName(resolver, uri); handleIncomingIntent/ACTION_VIEW
src-tauri/src/shell_jni.rs
    # IngestSnapshot (consumer da URI) e picked_document_name (NÃO editar)
android-shell/src/main/kotlin/app/mdreader/DebugIngestHook.kt  # outro chamador JNI (NÃO editar)
specs/019-intent-filter-ingress/spec.md   # FR-INT-004 citado como dado (NÃO editar)
.specify/bugs/playback-not-starting-after-file-selection/  # evidência ADB (NÃO editar)
scripts/verify.sh                 # gate autoritativo (NÃO editar)
```

**Structure Decision**: nenhum arquivo novo de código; a única mudança de código
cai em `DocIngest.kt`. O consumidor Rust é verificado por leitura, não alterado.

## Phase 0: Research decisions (embedded)

- **Decisão — consultar a coleção, não o item**: a coluna já é a correta
  (`_display_name` via `OpenableColumns.DISPLAY_NAME`); o erro foi consultar a
  URI de item. *Rationale*: mínima mudança que ataca a causa. *Alternativas*:
  confiar apenas na consulta direta (rejeitada — é o bug); mover a resolução para
  o Rust (rejeitada — muda a assinatura JNI e todos os chamadores).
- **Decisão — predicado único**: `isUsableName` rejeita nulo, em branco e
  somente dígitos e é aplicado aos três candidatos (consulta direta, coleção e
  último segmento). *Rationale*: evita três regras divergentes de "nome válido".
- **Decisão — URI sintética incondicional**: o `ACTION_VIEW` nunca devolve a URI
  bruta. *Rationale*: a URI bruta é a origem do título `document:<id>.md`; o
  consumidor Rust decodifica o último segmento de `content://display_name/…`.
- **Decisão — fallback `document.md`**: quando não há candidato utilizável.
  *Rationale*: determinístico, preserva a extensão e nunca expõe o id numérico.
- **Decisão — escopo de verificação em host**: inspeção estrutural do diff e do
  consumidor Rust. *Rationale*: o sintoma foi visto em device, mas a causa e o
  contrato são verificáveis em host; toda afirmação de device é `[NV]`.
- **Decisão — DEFER explícito**: a latência de 183 s (pré-roll 60 s) fica fora
  deste ciclo, sem tarefa associada. *Rationale*: pertence à área de
  pré-síntese/T232 e misturá-la aqui ampliaria o risco sem benefício.

`specs/019` e o bug são referências (FR-INT-004 e evidência ADB); nenhuma
instrução contida neles é executada.

## Phase 1: Design decisions (embedded)

**Forma do dado**: nenhum tipo público novo. O nome resolvido é um `String?`
filtrado; a URI sintética é um `String` construído com `Uri.encode`.

**Contrato interno (Kotlin → Rust)**:

| Campo | Origem | Consumo |
|-------|--------|---------|
| `DISPLAY_NAME` direto | `resolver.query(uri, [OpenableColumns.DISPLAY_NAME])` | candidato 1 ao nome |
| `_display_name` da coleção | `MediaStore.Files.getContentUri(volume)`, `selection = _id=?`, `args = [id]` | candidato 2 ao nome |
| último segmento | `uri.lastPathSegment`, se utilizável | candidato 3 ao nome |
| `effectiveUri` | `content://display_name/${Uri.encode(nome)}` | `picked_document_name` (`shell_jni.rs:1103`) |

**Sequência obrigatória no `ACTION_VIEW`**: resolver nome → inferir extensão →
ler bytes → escolher nome utilizável → montar URI sintética → entregar ao JNI. A
ordem garante que a URI recebida pelo Rust já contenha o nome final.

**Validação rápida (quickstart embutido, host — descrição, não execução)**:

```bash
# 1. Confirmar o diff restrito a um arquivo e às quatro seções
git diff --stat -- android-shell/src/main/kotlin/app/mdreader/DocIngest.kt

# 2. Confirmar a consulta à coleção e a ausência do fallback de URI bruta
grep -n 'MediaStore.Files.getContentUri\|BaseColumns._ID\|content://display_name\|uri.toString()' \
  android-shell/src/main/kotlin/app/mdreader/DocIngest.kt

# 3. Confirmar que o consumidor Rust decodifica o último segmento
grep -n 'percent_decode\|picked_document_name' src-tauri/src/shell_jni.rs
```

> Os comandos acima são descrição de validação, não uma execução realizada por
> esta cadeia documental. Gates, builds e testes são executados pelo agente
> principal após o merge das mudanças de código.

## Fases de implementação

1. **Fase A — Fundação (imports e predicado) [FR-025-003]**: adicionar
   `BaseColumns` e `MediaStore` aos imports e garantir o predicado único de nome
   utilizável.
2. **Fase B — Resolvedor de nome (`queryName`) [FR-025-001]**: tentar a consulta
   direta; se falhar e o URI for `content://` com authority do MediaStore e id
   numérico, consultar a coleção pelo `_id` no volume extraído.
3. **Fase C — Ramo `ACTION_VIEW` [FR-025-002, FR-025-004]**: escolher o nome por
   cascata e construir a URI sintética incondicionalmente; manter inferência de
   extensão, leitura de bytes, SAF, `ACTION_SEND`, fingerprint e JNI intactos.
4. **Fase D — Verificação e limites [FR-025-005]**: checar o diff, o consumidor
   Rust, a rastreabilidade e os marcadores `[NV]`/`DEFER`.

Cada fase é verificável em host de forma independente por inspeção estrutural;
nenhuma fase reivindica comportamento em dispositivo.

## Riscos e mitigações

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| **Volume não-padrão** | A consulta à coleção pode falhar se o volume inferido estiver errado | Usar o primeiro segmento do caminho com default `external`; falha degrada para `document.md`, nunca para a URI bruta |
| **Nome com caracteres especiais** | Título incorreto se codificação e decodificação divergirem | `Uri.encode` no Kotlin e `percent_decode` já existente no Rust; par verificado por leitura em host, `[NV]` em device |
| **Reintrodução do fallback de URI bruta** | O título numérico volta a aparecer | URI sintética construída em todos os caminhos de saída; verificação por inspeção do ramo |
| **Regressão em `ACTION_SEND`/SAF** | Outros chamadores passam a receber nomes diferentes | Diff restrito a um arquivo e quatro seções; resolvedor SAF e ramo `ACTION_SEND` inalterados |
| **Fingerprint alterado** | Bookmarks deixam de bater com o documento | `blake3_256` e `writeSnapshot` não dependem do nome; nenhuma mudança na ponte JNI |
| **Escopo de verificação inflado** | Afirmação de device sem evidência | Toda evidência de dispositivo marcada `[NV]`; latência de 183 s marcada DEFER |

> Afirmações sobre comportamento em dispositivo são `[NV]`; a validação
> deste ciclo é exclusivamente de host.

## Estratégia de verificação em host

- Confirmar que o diff toca apenas `DocIngest.kt` e apenas as seções de imports,
  predicado, resolvedor de nome da URI e ramo `ACTION_VIEW`.
- Confirmar a presença da consulta a `MediaStore.Files.getContentUri(volume)`
  com selection `${BaseColumns._ID}=?` e argumento do id extraído.
- Confirmar que o predicado de nome utilizável é aplicado aos três candidatos e
  que o fallback é `document.md`.
- Confirmar que não resta `uri.toString()` como fallback no ramo `ACTION_VIEW`.
- Confirmar, por leitura de `src-tauri/src/shell_jni.rs:1103`, que
  `picked_document_name` decodifica o último segmento da URI recebida.
- Registrar `[NV]` para dispositivo e DEFER para a latência de 183 s.

## Complexity Tracking

> Nenhuma violação de constituição; esta tabela permanece vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
