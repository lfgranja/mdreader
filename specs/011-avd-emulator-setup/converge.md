# Converge: 011 AVD Emulator Setup for mdreader/Ressoa

Status: **CONVERGED** (Harness operacional, 9/23 tasks) — configuração/verificação do AVD `mdreader_avd` e procedimentos de uso documentados; bootstrap do emulador validado, mas build/instalação/validação estrutural completa no emulador permanecem parcialmente executados.

## Implementation Summary

Preparado o harness de emulação Android para testes, validações e gates do mdreader/Ressoa em host Linux x86_64 com aceleração KVM disponível. O trabalho cobre verificação do SDK/ADB/KVM e da configuração do AVD (`mdreader_avd`, API 33 x86_64), registro de parâmetros de inicialização estáveis para ambientes restritos (`-gpu swiftshader_indirect`, flags de boot) e documentação operacional (quickstart, notas de startup, revisão de config, pesquisa de hooks de debug). O objetivo é viabilizar acesso reproduzível ao emulador e suportar validações estruturais (alinhadas à Spec 008) com caveats explícitos para medições como SC-009.

### Evidence (tasks, artifacts, git)

- `specs/011-avd-emulator-setup/tasks.md`: **9 checked / 14 unchecked (23 total)**. Concluídos T001–T009: verificação de SDK/KVM/AVD/ADB, revisão de configuração (`config.ini`), tentativa e documentação de startup do emulador, boot/connectivity e registro de issues. Em aberto: T010–T023 (seleção de target/build, instalação/launch no emulador, procedimentos de validação estrutural Spec 008, atestação VERIF-EMULATOR, documentação/polish/quickstart end-to-end e revisão de checklists).
- Artifacts/spec: `spec.md`, `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `quickstart.md`, `avd-config-review.md`, `emulator-startup-notes.md`, `research-debug-hooks.md`, `contracts/`, `checklists/` presentes no diretório da spec.
- `emulator-startup-notes.md`: registra tentativa `emulator -avd mdreader_avd -no-boot-anim -no-audio -gpu swiftshader_indirect -no-window`, constraints de ambiente (sem `/dev/kvm`, boot não concluído dentro do timeout em container), e conclusão de que a configuração do AVD está correta mas o boot foi bloqueado por limitações do ambiente.
- Git evidence: nenhum commit com referência direta `011`/`avd` no recorte recente; evidência principal reside nos artefatos da spec e no estado ROADMAP (harness operacional). Commit de governança relacionado ao ecossistema de specs: `47ef032b docs(converge): add converge records for specs 001-012, 016, 017`.
- ROADMAP alignment: **CONVERGED (Harness operacional, 9/23 tasks)** — scripts e configs AVD.

## Gate results

- Gates 1–8 (host): **[not applicable / not re-run in this pass]** — spec de infraestrutura/harness; sem alterações de código de produto neste registro e sem execução de suíte de gates aqui.
- Gate 9 (Android build): **[not re-run in this pass]** — build/instalação no emulador (T010–T013) permanecem em aberto.
- Validação de emulador/ADB: **[verified in prior cycles, parcial]** — pré-requisitos e configuração/startup documentados (T001–T009); boot completo e validação estrutural end-to-end dependem de ambiente com KVM/nested virt ou dispositivo alternativo.

## Non-Changes

- Nenhuma modificação de código-fonte do app, execução de gates, formatters, builds ou testes realizada neste registro de convergência (edit-only).
- Sem expansão de escopo além do consolidado em `spec.md`/`plan.md`/`tasks.md` e do estado ROADMAP.

## Unresolved / Deferred

- **T010–T014**: definir target adequado (x86_64 para emulador / aarch64 para gate 9), build, instalação, launch e documentação de considerações de ABI.
- **T015–T018**: executar procedimentos de validação estrutural (Spec 008 quickstart: notificação, foco/ducking/interrupção, resume por fingerprint, diag JSONL), documentar caveats SC-009 e produzir atestação `VERIF-EMULATOR`.
- **T019–T023**: polish documental, troubleshooting/cleanup, validação end-to-end do quickstart e revisão de checklists.
- Bloqueio ambiental registrado: ausência de `/dev/kvm`/nested virtualization impediu boot completo no ambiente onde o startup foi tentado; harness permanece operacional em configuração/procedimentos.

## Acceptance Criteria Status

- [x] Verificação de SDK/ADB/KVM e configuração do AVD `mdreader_avd` documentada [verified in prior cycles].
- [x] Processo de inicialização e constraints/caveats de ambiente registrados (`quickstart.md`, `emulator-startup-notes.md`, `avd-config-review.md`) [verified in prior cycles].
- [ ] Build/instalação/launch do app no emulador com ABI apropriado — **pendente** (T010–T014).
- [ ] Validação estrutural completa no emulador e atestação `VERIF-EMULATOR` (incl. caveats SC-009) — **pendente** (T015–T023).
