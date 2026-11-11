# Release Record: v0.1.0-rc1 (Ressoa Candidate Release)

- **Slug**: `v0.1.0-rc1`
- **Product**: Ressoa (`app.mdreader`)
- **Date**: 2026-11-10
- **Target Platform**: Android (minSdk 29, ARM64 / universal)
- **Status**: Release Candidate (Candidate 1)
- **Approved by**: Automated SDD Quality Gate & Hardware Verification Suite

---

## 1. Release Highlights & Executive Summary

Ressoa é o primeiro leitor de áudio focado em arquivos e diretórios Markdown estruturados, operando sob o princípio de soberania absoluta de dados e execução **100% offline e air-gapped** (`android.permission.INTERNET` rigorosamente ausente).

Nesta versão `v0.1.0-rc1`, o motor soberano em Rust gerencia todo o ciclo de síntese, transporte, buffer e persistência de marcadores de áudio, com espelhamento declarativo no shell Android (Kotlin).

---

## 2. Converged Features Matrix (Specs 001 a 017)

| Spec | Nome da Feature | Status | Evidência Chave |
|---|---|---|---|
| **001** | Android Media Shell & Service Mirror | ✅ Convergido | Espelhamento declarativo via JNI de MediaSession e controle de lock-screen. |
| **002** | FR-024 Synthesis Adapter (Sherpa-ONNX / Piper) | ✅ Convergido | Síntese neural local pt-BR (`pt_BR-faber-medium`). |
| **005 / 007** | Design System SONA & HUD Visual | ✅ Convergido | Contraste externo, conformidade WCAG e visualizador sinestésico em canvas. |
| **006** | Lente Sinestésica & RMS Lock-Free | ✅ Convergido | Buffer atômico `AtomicU32` a ~20 Hz sem impacto no thread de áudio. |
| **009** | Abertura de Arquivo e Bookmarks Duráveis | ✅ Convergido | Android SAF, cálculo de hash Blake3-256 e gravação a cada 20s em `redb`. |
| **010** | Leitura Contínua de Diretórios | ✅ Convergido | Suporte a árvore SAF (`ACTION_OPEN_DOCUMENT_TREE`), transições gapless e comandos de avanço/recuo de capítulos. |
| **013** | Pipeline TTFA e Agendamento Assíncrono | ✅ Convergido | Latência TTFA comprovada em hardware real (Xiaomi Redmi Note 11): **1.551 ms** (< teto de 3.000 ms). |
| **014** | Normalização de Símbolos Técnicos Markdown | ✅ Convergido | Eliminação de fadiga auditiva de sintaxe técnica de markdown (`normalize.rs`). |
| **015** | Prosódia Semântica Estruturada | ✅ Convergido | Pausas naturais em cabeçalhos `#`, listas `-` e quebras de bloco (`prosody.rs`). |
| **017** | Cache de Chunks de Áudio em Memória (LRU) | ✅ Convergido | Sub-50ms rewind/repeat e teto estrito de 32 MiB com PBT. |

---

## 3. Verification Gates & Supply Chain Audit

- **Gate 1 (Format):** PASS (`cargo fmt --all -- --check`).
- **Gate 2 (Test Census & Regression):** PASS (1.058 testes passando sem falhas no workspace; diff exato com `expected-tests.txt`).
- **Gate 3 (Documentation):** PASS (`cargo doc --workspace --no-deps`).
- **Gate 4 (Clippy Pedantic):** PASS (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Gate 5 (Supply Chain Security):** PASS (`cargo deny check advisories licenses bans sources` & `cargo audit`).
- **Gate 6 (Deny-list Scan):** PASS (sem dependências de rede ou crates não autorizados).
- **Gate 7 (Closed Sets):** PASS (sem curingas `_ =>` e sem variantes `Other`/`Unknown`).
- **Gate 8 (Shell Purity):** PASS (zero lógica de reprodução/transporte em Kotlin; rastreabilidade de 135 tokens estrita).

---

## 4. Hardware Verification & Real Device Telemetry

- **Dispositivo de Referência**: Xiaomi Redmi Note 11 (`2201117TY`, Android 13 SDK 33).
- **Conexão**: ADB Wireless TLS (`adb-53cf9aa7-4VfOJ6`).
- **TTFA Medido**: `play_to_audio_ms=1551` (evidenciado no arquivo `diagnostic-record.jsonl`).
- **Navegação de Diretórios**: 3 capítulos verificados via CDP (`01-introducao.md`, `02-arquitetura.md`, `10-conclusao.md`) com transições auditadas no HUD e persistência em `redb`.

---

## 5. Artifact Hashes & Build Information

- **APK Universal Debug**: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`
- **SHA-256**: `cb452b4e52c0043c1ec58967a055eb0eb4610feae92335dcaf647d3ba6d56d5c`
- **Tamanho**: 187.934.693 bytes (~179 MiB incluindo modelo Piper ONNX e espeak-ng-data)
- **Git Commit de Referência**: `5d2c7971`
