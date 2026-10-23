# DESIGN/brand.md — Sistema de Identidade Visual Oficial: Dupla Assinatura Ressoa (v2.1.0)

**Versão:** 2.1.0 · **Data:** 2026-10-16 · **Status:** Homologado e Integrado ao `DESIGN-SONA`

> Aplicável a todo o ecossistema `DESIGN-SONA`.  
> **Regra Fundamental de Governança:** A marca é referência e garantia de autenticidade, nunca conteúdo intrusivo. Nenhum texto de documento é substituído pela marca; na notificação do sistema, prevalece a informação do documento lido.

---

## 1. Nomenclatura e Arquitetura de Marca

- **Workspace de Engenharia:** `mdreader` — preservado como nome estrutural do repositório, namespace Rust (`crates/*`), package Android (`app.mdreader`) e Tauri (`tauri.conf.json`). Nunca utilizado em materiais de comunicação pública.
- **Marca Oficial do Produto:** `Ressoa` (e domínio institucional `ressoa.dev`).
- **Motor Acústico Subjacente:** `Engine Sona` — denominação estritamente interna dos crates de síntese (`crates/engine-core`, `crates/player`, `crates/diag`).
- **Arquitetura Visual:** **Sistema de Dupla Assinatura (Dual-Mark Architecture)**. Combina uma **Marca Primária Tipográfico-Bioacústica** (para reconhecimento imediato de app e identidade móvel) com um **Selo de Engenharia Acústica** (para o motor de áudio hi-fi, mostradores de reprodução e chassi).

---

## 2. A Dupla Assinatura Visual Oficial

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                       SISTEMA DE DUPLA ASSINATURA RESSOA                        │
├────────────────────────────────────────┬────────────────────────────────────────┤
│  MARCA PRIMÁRIA: A CONCHA HARMÔNICA    │  SELO DO MOTOR: O RESSONADOR CANÔNICO  │
│  (Baseada na Proposta 03 — 9.62 pts)   │  (Baseada na Proposta 01 — 8.95 pts)   │
├────────────────────────────────────────┼────────────────────────────────────────┤
│ • Ícone Oficial do App Android (M3)    │ • Emblema da "Engine Sona" (Rust Core) │
│ • Google Play Store & F-Droid (1024px) │ • Mostrador de Status / Diagnostics    │
│ • Favicon Web (16px, 32px, SVG)        │ • Carimbo de Áudio Hi-Fi / Hardware    │
│ • Avatar GitHub & Redes Sociais        │ • Selo de Calibração / Mídia Física    │
│ • Monograma Institucional ("R")        │ • Variante Circular de Instrumentação  │
└────────────────────────────────────────┴────────────────────────────────────────┘
```

---

## 3. Especificação dos Símbolos

### 3.1. Marca Primária: A Concha Harmônica (Monograma "R")
* **Arquivos Oficiais:**
  * Master PNG 1024px (Studio Photocraft): [`DESIGN-SONA/assets/logo/logo-primary-concha-r.png`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/logo-primary-concha-r.png)
  * Vetor Canônico SVG: [`DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/logo-primary-concha-r.svg)
  * Adaptive Icon Foreground: [`DESIGN-SONA/assets/logo/ic_launcher_foreground.xml`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/ic_launcher_foreground.xml)
  * Adaptive Icon Background: [`DESIGN-SONA/assets/logo/ic_launcher_background.xml`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/ic_launcher_background.xml)
  * Material You Monochrome Mask: [`DESIGN-SONA/assets/logo/ic_launcher_monochrome.xml`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/ic_launcher_monochrome.xml)
* **Conceito & Anatomia:**
  1. **Coluna de Leitura (Haste Marfim `#F2F0E6`):** Representa o cursor de bookmark determinístico do motor Rust sobre a superfície do documento.
  2. **Tríade de Arcos Concêntricos:** As três auras da Sona (*Hymn of Valor* `#8FD8E8`, *Aria of Perseverance* `#5AD6A8`, *Song of Celerity* `#B9A6F5`) curvam-se formando a espiral de uma cóclea auditiva e a silhueta da letra "R" de Ressoa.
  3. **Pérola Nodal Central:** Ponto de foco e convergência harmônica no centro da cóclea.
* **Comportamento em Escala:** Reconhecimento neural imediato da letra "R" desde favicons de 16px até telas retina 4K.

---

### 3.2. Selo de Engenharia: O Ressonador Canônico (Lente Semafórica)
* **Arquivos Oficiais:**
  * Master PNG 1024px (Studio Photocraft): [`DESIGN-SONA/assets/logo/seal-engine-ressonador.png`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/seal-engine-ressonador.png)
  * Vetor Canônico SVG: [`DESIGN-SONA/assets/logo/seal-engine-ressonador.svg`](file:///home/luis/development/mdreader/DESIGN-SONA/assets/logo/seal-engine-ressonador.svg)
* **Conceito & Anatomia:**
  1. **Chassi Cilíndrico em Titânio/Obsidiana Fosca:** Câmara de contenção acústica com chanfros usinados de alta precisão.
  2. **Fendas Polares Abertas:** Permitem que as ondas sonoras de Fourier atravessem o corpo ressonador.
  3. **Tríade Senoidal de Fourier:** Três ondas puras que vibram horizontalmente em harmonia com o nó perolado central.
* **Aplicações Específicas:**
  * Cabeçalho da visualização de status / diagnóstico do motor (`crates/diag`).
  * Emblema central de rotação de mídia e buffer de reprodução in-app.
  * Carimbo de alta fidelidade e autenticidade da síntese PCM nativa a 22050 Hz.

---

## 4. Paleta Cromática Institucional (Tríade Sinestésica Sona)

| Token Semântico | Hexadecimal | Papel no Sistema | Aplicação na Marca e Selo |
|---|---|---|---|
| `--sona-void` | `#0B0F1A` | Vácuo Base OLED | Fundo infinito de alto contraste (0% consumo de energia OLED) |
| `--sona-stage`| `#141B2D` | Superfície Elevada| Chassi e anéis de contenção |
| `--sona-voice`| `#F2F0E6` | Voz / Estrutura Marfim | Haste de leitura, cursor, nó perolado central |
| `--sona-hush` | `#A9B8CE` | Silêncio / Metadados | Linhas de construção, legendas auxiliares |
| `--sona-etwahl`| `#8FD8E8` | Frequência Alta / Foco | **Onda 1 (Azul Turquesa):** *Hymn of Valor* |
| `--sona-aria`  | `#5AD6A8` | Frequência Média / Cura | **Onda 2 (Verde Jade):** *Aria of Perseverance* |
| `--sona-celeste`| `#B9A6F5`| Frequência Baixa / Ritmo | **Onda 3 (Roxo Ametista):** *Song of Celerity* |

> [!IMPORTANT]
> **Separação Rigorosa de Domínios:** A tríade cromática vibrante (Azul, Verde, Roxo) pertence exclusivamente à **Identidade Visual, Ícone e Selo**. Na **interface operacional do player** (transporte de áudio) e na **notificação do sistema**, a sinalização funcional austera prevalece: o verde permanece expressamente vedado na telemetria semafórica do player, e o amarelo (`#FFD400`) é exclusivo do aviso de pausa de 3 segundos.

---

## 5. Matriz de Superfícies de Aplicação

| Superfície | Qual Símbolo Utilizar? | Justificativa e Regra de Uso |
|---|---|---|
| **Ícone do App Android (Launcher / Drawer)** | **Marca Primária ("R")** | Reconhecimento instantâneo do nome Ressoa na tela inicial; conformidade com a Safe Zone de 66dp e suporte nativo ao Material You. |
| **Google Play Store / F-Droid Listing** | **Marca Primária ("R")** | Monograma com altíssima taxa de conversão (ASO) e memorabilidade categórica. |
| **In-App Player Header (Lente Sinestésica)** | **Marca Primária ("R")** | Posicionada no topo esquerdo ao lado do wordmark `ressoa`, com luminescência sutil. |
| **In-App Mostrador Central & Buffer Ring** | **Selo do Motor (Ressonador)** | Coração acústico central do player (P1/P2); anel de buffer e VU meter concêntrico. |
| **Diálogo de Diagnóstico & Telemetria** | **Selo do Motor (Ressonador)** | Centro da malha de Chladni no modal de instrumentação Rust (P5). |
| **Relatórios de Diagnóstico (`crates/diag`)** | **Selo do Motor (Ressonador)** | Selo técnico de auditoria e conformidade da telemetria determinística. |
| **Notificação MediaStyle (Android OS)** | **NENHUM (Regra FR-028)** | A notificação do sistema Android exibe apenas o título do documento, seção e progresso numérico. A marca não polui a lock screen. |
| **Gravação a Laser em Hardware / Dispositivo** | **Selo do Motor (Ressonador)** | Geometria circular ideal para usinagem CNC em potenciômetros, botões rotativos ou tampas de chassi. |
| **Terminal CLI / Logs (`cargo run`)** | **Marca Primária ("R")** | Renderização em arte ASCII / ANSI monocromática do "R" no banner de inicialização. |

---

## 6. Histórico de Versões
- **v1.0.0 (2026-10-06):** Definição original do Candidato B (Sinalização austera semafórica).
- **v2.0.0 (2026-10-16):** Integração da Tríade de Ondas da Sona e tokenização `--sona-aria`.
- **v2.1.0 (2026-10-16):** Homologação do Sistema de Dupla Assinatura (Marca Primária "R" + Selo do Motor Acústico).
- **v2.2.0 (2026-10-17):** Integração com a Arquitetura Multi-Modo UI/UX (Lente Sinestésica como player default, Cockpit Eyes-Free, Sumário Suíço e Matriz de Telemetria).

