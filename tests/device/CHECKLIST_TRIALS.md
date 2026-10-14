# Checklist Executável — On-Device Trials (T086)

> **Device normativo**: Redmi 2201117TY (spesn_global) — Android 13 (API 33)
> **APKs necessários**:
> - mdreader: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`
> - focus-injector: `focus-injector/app/build/outputs/apk/debug/app-debug.apk`

---

## 📋 Pré-requisitos (marque antes de iniciar)

- [ ] Device conectado via USB, `adb devices` mostra `53cf9aa7  device`
- [ ] mdreader APK instalado (`adb install -r .../app-universal-debug.apk`)
- [ ] focus-injector APK instalado (`adb install -r .../app-debug.apk`)
- [ ] Permissão `POST_NOTIFICATIONS` concedida ao mdreader (`adb shell pm grant app.mdreader android.permission.POST_NOTIFICATIONS`)
- [ ] Fone de ouvido com fio (P2) disponível
- [ ] Headset Bluetooth A2DP pareado e conectado
- [ ] Cabo USB para simular power loss (desconectar device do PC enquanto roda)
- [ ] Cronômetro/segundos (app de celular ou relógio)
- [ ] Documento Markdown de teste (>5 min de leitura) no device

---

## 🎯 T-01 · SC-001 — Playback survives screen-off and manual lock

**Critério**: 10/10 trials — áudio contínuo 5 min com tela desligada/travada, controles no lock screen

| Trial | Passo | Ação | Observação | ✅/❌ |
|-------|-------|------|------------|------|
| 1 | 1 | Iniciar playback no mdreader; confirmar áudio audível | | |
|   | 2 | Pressionar Power (tela off) → travar manual (long-press Power → Lock) | | |
|   | 3 | Aguardar 5 min sem tocar no device | | |
|   | 4 | Acordar tela (sem desbloquear) → verificar controles + áudio | | |
| 2 | 1-4 | Repetir | | |
| 3 | 1-4 | Repetir | | |
| 4 | 1-4 | Repetir | | |
| 5 | 1-4 | Repetir | | |
| 6 | 1-4 | Repetir | | |
| 7 | 1-4 | Repetir | | |
| 8 | 1-4 | Repetir | | |
| 9 | 1-4 | Repetir | | |
| 10 | 1-4 | Repetir | | |

**Evidência**: logcat `mdreader` mostrando `transport: "playing"` contínuo; `position_secs` avançando.

---

## 🎯 T-02 · SC-002 — Headphone / A2DP-loss pause within one second

### Part A — Wired headphones (10 trials)

| Trial | Passo | Ação | Latência pausa (s) | Speaker mudo 10s? | ✅/❌ |
|-------|-------|------|---------------------|-------------------|------|
| 1 | 1 | Play via fone P2; confirmar áudio no fone | | | |
|   | 2 | Despluguar fone P2 | | | |
|   | 3 | Cronometrar pausa desde remoção | | | |
|   | 4 | Confirmar silêncio no speaker por 10s | | | |
| 2 | 1-4 | Repetir | | | |
| ... | | | | | |
| 10 | 1-4 | Repetir | | | |

### Part B — Bluetooth A2DP (10 trials — odd=normal, even=secondary signal only)

> **Even trials**: No focus-injector, arme `fault=disable_noisy_receiver` (se houver fixture) ou apenas desconecte BT

| Trial | Tipo | Ação | Latência pausa (s) | Speaker mudo 10s? | ✅/❌ |
|-------|------|------|---------------------|-------------------|------|
| 1 | Normal | Play via BT headset; confirmar áudio | | | |
|   | | Desligar headset / remover pairing | | | |
| 2 | Secondary | Mesmo, mas apenas sinal secundário | | | |
| 3 | Normal | Repetir | | | |
| 4 | Secondary | Repetir | | | |
| 5 | Normal | Repetir | | | |
| 6 | Secondary | Repetir | | | |
| 7 | Normal | Repetir | | | |
| 8 | Secondary | Repetir | | | |
| 9 | Normal | Repetir | | | |
| 10 | Secondary | Repetir | | | |

**Evidência**: logcat `mdreader` + `FocusMirror`/`RoutingMirror` mostrando `BecomingNoisy` ou `DeviceDisappeared` → `transport: "paused"` ≤1s.

---

## 🎯 T-03 · SC-003 — Duckable transient: 15–25% gain, keeps reading

**Requer**: focus-injector app instalado

| Trial | Passo | Ação | Gain factor observado | Transport `playing`? | Position avança? | Gain 1.00 ao retomar? | ✅/❌ |
|-------|-------|------|------------------------|----------------------|------------------|------------------------|------|
| 1 | 1 | Iniciar playback; registrar gain baseline (1.00) | | | | | |
|   | 2 | No focus-injector: tocar **Duckable Transient (MAY_DUCK)** | | | | | |
|   | 3 | Segurar 8s; ler gain em `focus.duck gain=<factor>` | | | | | |
|   | 4 | Verificar transport `playing` + position monotônica | | | | | |
|   | 5 | No focus-injector: tocar **Abandon Focus** | | | | | |
|   | 6 | Confirmar gain 1.00 restaurado | | | | | |
| 2-10 | | Repetir | | | | | |

**Evidência**: logcat `mdreader` tag `focus` mostrando `duck gain=0.20` (ou 0.15–0.25) durante hold, `gain=1.00` após abandon.

---

## 🎯 T-04 · SC-041 — Silence-demanding interruption pauses; rewind math on resume

**Requer**: focus-injector app + stand-in audio com mapa de tom/gap conhecido

### Part A — Silence-demanding (10 trials)

| Trial | Duração interrupção | Posição yield P (s) | Posição resume R (s) | Cursor back 5%? | Sentence snap? | Total backward ≤30s? | ✅/❌ |
|-------|---------------------|---------------------|----------------------|-----------------|----------------|----------------------|------|
| 1 | 40s | | | | | | |
| 2 | 40s | | | | | | |
| 3 | 40s | | | | | | |
| 4 | 40s | | | | | | |
| 5 | 40s | | | | | | |
| 6 | 15min | | | | | | |
| 7 | 15min | | | | | | |
| 8 | 15min | | | | | | |
| 9 | 15min | | | | | | |
| 10 | 15min | | | | | | |

**Procedimento por trial**:
1. Play até posição P (meio de sentença conhecida)
2. No focus-injector: **Silence Demanding (TRANSIENT)** por D segundos
3. Confirmar: pause (não duck), zero áudio, lock screen mostra razão "Leitura pausada — outro app usando o áudio"
4. **Abandon Focus** no injector → regain
5. Medir posição resume R no logcat/mirror
6. Verificar: cursor = `min(0.05 × D, 30s)` atrás de P; R = início da sentença contendo cursor (ou cap point se sentença > cap); `P - R ≤ 30s`

### Part B — Differentiation (10 trials)
- Mesmo procedimento mas com **Duckable Transient (MAY_DUCK)** → deve duck (gain 0.15–0.25), NÃO pausar

---

## 🎯 T-05 a T-48 — Demais trials (consulte `tests/device/trials.md`)

> **Resumo rápido** — marque ✅ quando completar cada seção:

- [ ] **T-05** SC-004 — Lock-screen controls functional (±5s skip, rate ladder, pause/resume)
- [ ] **T-06** SC-007 — Kill recovery replay ≤30s
- [ ] **T-07** SC-008 — Blind lock-screen protocol (tester não vê tela)
- [ ] **T-08** SC-009 — Cold start play-to-audio ≤10s
- [ ] **T-09** SC-010 — Notification surface: title + section + progress ONLY
- [ ] **T-10** SC-011 — Unexpected end → paused + explicit resume
- [ ] **T-11** SC-013 — Engine failure mid-doc → named failure + 3s offer + fallback
- [ ] **T-12** SC-014 — Externally-started service → play works without app open
- [ ] **T-13** SC-016 — Skip clamp at boundaries
- [ ] **T-14** SC-017 — Rate ladder 7 rungs, one-step, saturate
- [ ] **T-15** SC-018 — Voice cycling with re-anchor
- [ ] **T-16** SC-019 — Stop vs Pause storage (Stop discards, Pause preserves)
- [ ] **T-17** SC-020 — Resume offer auto-start ONLY if foregrounded
- [ ] **T-18** SC-021 — Explicit resume for unexpected end
- [ ] **T-19** SC-022 — Device acquisition failure → named statement
- [ ] **T-20** SC-023 — Force device failure → clear message, never phantom playing
- [ ] **T-21** SC-024 — Diagnostic record trim 30d/10MB visible
- [ ] **T-22** SC-025 — Scanner zero prose
- [ ] **T-23** SC-026 — Determinate progress fixed at session start
- [ ] **T-24** SC-027 — Rate shown in-app = lock-screen source
- [ ] **T-25** SC-028 — Denied permission statement visible in-app
- [ ] **T-26** SC-029 — Heading re-resolve on boundary cross/skip/resume
- [ ] **T-27** SC-030 — No section title before first heading
- [ ] **T-28** SC-031 — Document edit on disk → no resume offer
- [ ] **T-29** SC-032 — Manifest `<service mediaPlayback>` + minSdk 29
- [ ] **T-30** SC-033 — Routing: new device → follow unchanged; private→public pause first
- [ ] **T-31** SC-033b — Coincidence unplug+connect → pause first
- [ ] **T-32** SC-034 — Handoff shape invariant all 7 rungs
- [ ] **T-33** SC-035 — Shell purity: zero playback logic in Kotlin
- [ ] **T-34** SC-036 — Lock-screen non-dismissable while playing
- [ ] **T-35** SC-037 — Zero in-app diagnostic surface
- [ ] **T-36** SC-038 — Fingerprint blake3 content hash
- [ ] **T-37** SC-039 — Engine failure locked → fallback without unlock
- [ ] **T-38** SC-040 — Repeated failure re-offers
- [ ] **T-39** SC-042 — Audible confirmation on every control (earcon-or-speech)
- [ ] **T-40** SC-043 — Residency liveness: Play press produces audio
- [ ] **T-41** SC-044 — DEFERRED (real engines, synthesis feature)
- [ ] **T-42** SC-045 — Completion tears down residency
- [ ] **T-43** SC-046 — Offline start (airplane mode) + 60s continue
- [ ] **T-44** SC-047 — Progress indeterminate whole session if total unknown
- [ ] **T-45** SC-048 — Diagnostic gap marker on storage full
- [ ] **T-46** SC-049 — One resident + one notification while resumable
- [ ] **T-47** SC-050 — Idle resume ≤3s
- [ ] **T-48** SC-050b — Idle drain: zero billing during idle
- [ ] **FR-026** — Airplane mode start + 60s continue

---

## 📝 Registro de Evidência por Trial

Para **CADA trial que falhar**, capture:

```
Trial: T-XX #N
SC: SC-XXX
Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Expected: <critério exato do spec.md>
Actual: <o que aconteceu>
Logcat snippet: <linhas relevantes>
Diag.jsonl snippet: <entrada relevante>
Screenshot/photo: <se aplicável>
```

---

## 📦 Entregável Final

Ao completar todos os 480 trials (48 × 10):

1. Preencher `VERIFICATION.md` com verdict PASS/FAIL/PARTIAL/DEFERRED por SC
2. Anexar logs representativos (logcat + diag.jsonl) para cada SC
3. Registrar trials parciais/falhos com evidência completa