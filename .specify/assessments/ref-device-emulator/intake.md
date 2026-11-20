# Idea Intake: Local free emulator for reference-device visual verification

- **Slug**: ref-device-emulator
- **Created**: 2026-10-29
- **Source**: pasted text (session message — research request on integrating local/light/free emulation/visual-verification for reference appliance without mandatory physical adb device)
- **Type**: exploration

## Idea (as captured)

Investigação extensa (multilingue, alta confiabilidade, todas as ferramentas/skills) para determinar se é possível, viável, interessante e desejável integrar uma opção LOCAL, LEVE, gratuita, para emulação e verificação visual do aparelho de referência (API 29+), sem depender obrigatoriamente do aparelho físico conectado ao `adb devices`. A pergunta origina do fluxo `003-reference-device` e do gargalo `verify.sh` / `gate-8-full`.

## Restated

Propor uma camada de verificação local baseada no emulador Android SDK (gratuito, local, API 29+) para o spec `003-reference-device`, avaliando o que ela pode confirmar estruturalmente (manifest, `MediaSession`, notificação, foco, diag) versus o que exige dispositivo físico (áudio real, `SC-009`, interrupções reais, Bluetooth/car, 10/10 on-device).

## Origin & Context

- **Raised by**: usuário (sessão atual, após conclusão do plano `speckit-80-20-roadmap-plan.md` e atualização do `ROADMAP.md`)
- **Trigger**: necessidade de reduzir barreira de verificação do dispositivo de referência (`003`) e avaliar se o emulador pode acelerar `gate-8-full` sem substituir o físico
- **Related artifacts**: `.specify/assessments/003-reference-device/` (não existe; `003` está em `specs/003-reference-device/` — Draft); `AGENTS.md` (SC-035, SC-051); `constitution.md` (gates 1–8, closed-set); `specs/003-reference-device/plan.md`; `scripts/verify.sh`; `ROADMAP.md` linha 27 (atualizado para `VERIF-EMULATOR` + `gate-8-full` DEFERRED)

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Qual emulador exato é o alvo? Android Emulator SDK (oficial, via Android Studio / `avdmanager`), Genymotion, BlueStack, Waydroid, ou outro? Cada um tem limitações diferentes de áudio, foco, Bluetooth, notificação de lock-screen.]
- [NEEDS CLARIFICATION: A verificação via emulador deve ser uma camada adicional (`VERIF-EMULATOR`) ao lado do físico (recomendado pelo roadmap atualizado), ou uma substituição do físico para `gate-8-full`? O usuário parece querer ambos; precisa confirmar.] 
- [NEEDS CLARIFICATION: O consent/consent de cloud TTS (`SC-044`, `002`) é relevante para esta ideia? Não diretamente, mas se o emulador acelerar `003`, ele desbloqueia `001` `gate-8-full` que depende de `002` — há interação entre as cadeias.] 
- [NEEDS CLARIFICATION: Qual é o critério de sucesso do emulador: apenas `verify.sh` estrutural, ou também benchmark `SC-009` ≤3 s (que requer hardware real)? A pesquisa já indicou que `SC-009` não é simulável fielmente.] 
- [NEEDS CLARIFICATION: Há restrição de tempo/budget para manter o físico? Se não, o emulador é apenas camada intermediária, não substituto.] 
- [NEEDS CLARIFICATION: O usuário deseja que a ideia seja tratada como novo assessment (`ref-device-emulator`) ou apenas como nota de verificação dentro de `003`? A criação deste intake assume novo assessment; pode ser revertido.] 

## Source / Context Notes

- Nenhuma URL fornecida; fonte é texto colado (pergunta de pesquisa).
- Nenhum segredo, token, senha ou credencial presente no texto original (verificado: apenas referência a `adb devices`, `gate-8-full`, `SC-009`, `VERIFY-EMULATOR` — todos termos internos do projeto).
- Nenhum comando executado, nenhum arquivo fonte editado por esta intake.
