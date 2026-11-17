# Feature Specification: 016-voz-lula — TTS voz Lula da Silva

**Feature Branch**: `016-voz-lula`

**Created**: 2026-11-08

**Status**: Deferred (Cycle 017, 2026-11-10 — see `decision.md`)

**Input**: Requirement to implement reading using a voice that honours Brazilian President Luiz Inácio Lula da Silva in the mdreader (Ressoa) TTS audiobook reader.

---

## User Scenarios & Testing

### User Story 1 — Selecionar voz Lula para leitura (Priority: P1)

Como usuário do mdreader, quero selecionar uma voz de síntese que homenageie o Presidente Luiz Inácio Lula da Silva, para que a leitura de livros audiobooks use uma voz brasileira reconhecível e respeitosa.

**Why this priority**: A síntese de voz é a função central do app; oferecer uma voz que homenageia o presidente brasileiro é um diferencial cultural e de acessibilidade.

**Independent Test**: Com o adaptador de síntese integrado, selecionar a voz `lula` via CLI ou ponte Tauri e verificar que a síntese usa o modelo de voz Lula (quando o asset estiver disponível) ou falha fechado com `EngineError::InitFailed` (quando o asset ainda não foi obtido/traído).

### User Story 2 — Compatibilidade com formato de handoff (Priority: P1)

A voz Lula deve entregar áudio mono f32 a 22 050 Hz, conforme o contrato de handoff do `engine-core`, sem resample silencioso ou downmix.

**Independent Test**: Verificar que qualquer segmento sintetizado com a voz Lula entrega PCM conforme `DeclaredFormat::SAMPLE_RATE` (22 050 Hz), mono, f32.

---

## Requirements

### Functional Requirements

- **FR-01**: O adaptador de síntese MUST suportar seleção de voz entre `faber` (padrão atual, CC0) e `lula` (homenagem a Lula da Silva).
- **FR-02**: A voz Lula MUST usar o mesmo formato de handoff que a voz faber: mono f32 @ 22 050 Hz, sem resample silencioso.
- **FR-03**: Quando o modelo de voz Lula não estiver disponível (arquivo .onnx ausente), a síntese MUST falhar fechado com `EngineError::InitFailed`, nunca silenciosamente.
- **FR-04**: A seleção de voz MUST ser configurável via CLI (`--voice`) e via ponte Tauri (`set_voice`).
- **FR-05**: O diretório de assets da voz Lula MUST seguir a mesma convenção da voz faber: `assets/voices/pt_BR-lula-medium/` com `.onnx`, `tokens.txt`, `espeak-ng-data/`, `.onnx.json`, `MODEL_CARD`, `PROVENANCE.md`.

### Key Entities

- **Voice**: enum `Faber` | `Lula` — identidade da voz de síntese selecionada.
- **Voice asset dir**: `assets/voices/pt_BR-<voice>-medium/` — árvore de assets do modelo Piper VITS.

---

## Open Questions

- **Q1**: O modelo de voz Lula será obtido via treinamento (Fish Speech / GPT-SoVITS + Piper export) ou via modelo pré-existente? → Pendente; depende de dados de áudio de domínio público do presidente Lula.
- **Q2**: O modelo Lula usa espeak-ng pt-BR para fonemização (reutilizando a árvore da voz faber) ou uma árvore própria? → Recomendado: reutilizar a árvore `espeak-ng-data` da voz faber (mesmo idioma pt-BR).

---

## Assumptions

- A voz faber permanece como padrão (`offline_default`, CC0).
- A voz Lula é uma segunda opção, selecionável explicitamente.
- O modelo .onnx da voz Lula não está incluído no repo (será obtido separadamente, via treinamento ou doação); o diretório existe com documentação de como obtê-lo.
- A arquitetura permanece offline-first (Princípio IX): nada é buscado da rede em tempo de síntese.

---

## Clarifications

Nenhuma ainda.

---

## Success Criteria

- **SC-001**: `cargo build --workspace` compila com suporte à voz Lula (mesmo sem o .onnx presente).
- **SC-002**: `cargo nextest run --workspace` passa (testes existentes inalterados + novos testes de seleção de voz).
- **SC-003**: `cargo clippy --workspace --all-targets -- -D warnings` está limpo.
- **SC-004**: `cargo fmt --all -- --check` passa.
- **SC-005**: Selecionar voz `lula` via CLI produz `InitFailed` quando o .onnx está ausente (fail-closed).

---

## Assinatura de voz

A voz Lula homenageia Luiz Inácio Lula da Silva, Presidente do Brasil, com síntese de fala em português do Brasil (pt-BR), preservando as características prosódicas da fala brasileira através do modelo VITS/Piper compatível com 22 050 Hz mono f32.