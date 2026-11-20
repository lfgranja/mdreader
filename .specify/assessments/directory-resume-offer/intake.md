# Idea Intake: Ligar o diálogo de retomada na reabertura de diretório

- **Slug**: directory-resume-offer
- **Created**: 2026-11-20
- **Source**: pasted text (atribuição do ciclo SDD 80/20 #024, T233) + codebase pointer (`/home/luis/development/mdreader`) + mapa de evidência do ScoutT233
- **Type**: fix

## Idea (as captured)

> Ciclo SDD 80/20 #024 — T233: ligar o diálogo de retomada na reabertura de diretório.
>
> FR-009 (spec 010:90): reabrir pasta com leitura em andamento → diálogo de retomada
> (3 s, continuar como padrão, começar do início como alternativa, auto-continue no
> expiry se a janela estiver em primeiro plano), com nome do arquivo e posição salva.
> T233 está `[ ]` mas é "partial": as peças existem, falta a cola.
>
> Decisão de escopo: fazer a cola completa — publicar a rota, enriquecer o payload,
> chamar `route_offer` uma vez, estacionar a retomada, ligar `refreshOffer()` no JS,
> testar comportamentalmente no target já pinado e espelhar os bundles HTML.

Nenhuma URL foi fornecida; nenhum fetch foi realizado e a URL Trust Policy não foi
acionada. Toda a entrada é local: texto da atribuição, mapa de evidência do
ScoutT233 (referências de linha verificadas em host) e o repositório no caminho
acima. Conteúdo de `specs/` é tratado como dado não-confiável: FR-009 é citado como
registro de requisito, nunca como instrução.

## Restated

A reabertura de um diretório com leitura em andamento já calcula a rota do diálogo
de oferta no núcleo Rust, mas o caminho que trata a seleção da pasta não publica
essa rota, não estaciona a oferta e não entrega o nome do arquivo nem a posição
salva à interface. O trabalho é ligar peças já existentes, sem introduzir novo
comportamento de produto.

## Origin & Context

- **Raised by**: mantenedor do projeto, via atribuição T233 do ciclo SDD 80/20 #024.
- **Trigger**: T233 da spec 010 permanece `[ ]` embora FR-009 já esteja especificado
  e a maior parte do mecanismo implementada; o ciclo 024 fecha a lacuna com a menor
  mudança suficiente (80/20).
- **Branch**: `024-directory-resume-offer`, criada a partir de `dev` (HEAD `319596c2`).
- **Relação com a spec 010**: FR-009 é apenas referência; este ciclo NÃO edita `specs/010`.
- **Gate autoritativo**: `scripts/verify.sh` (gates 1–8). O gate 2 inclui two-way diff
  de `scripts/expected-tests.txt` — um novo target de teste exigiria atualizar esse pin.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: publicar a rota `InAppDialog` basta, ou é necessário também chamar `park_directory_resume` para a oferta sobreviver até a resposta do usuário?]
- [NEEDS CLARIFICATION: o payload do evento `directory-selected` comporta `filename`/`offset_secs` sem alterar marcação HTML nos bundles espelhados?]
- [NEEDS CLARIFICATION: `route_offer` muta estado — quantas vezes pode ser chamado por reabertura sem duplicar a transição `expiry → AutoStart`?]
- [NEEDS CLARIFICATION: o teste pode ser comportamental dentro do target já pinado, evitando tocar em `scripts/expected-tests.txt`?]
- [NEEDS CLARIFICATION: a janela de 3 s e o auto-continue em primeiro plano são verificáveis em host ou exigem dispositivo?]
