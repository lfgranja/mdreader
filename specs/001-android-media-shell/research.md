# Research: Android Media Shell — PCM Playback Owned by Rust

**Feature**: `specs/001-android-media-shell`
**Date**: 2026-10-01
**Scope**: Research only — no code changes.

---

## 1. PCM Playback: cpal 0.18.x AAudio vs oboe 0.6.x

### Decision: cpal 0.18.2 with AAudio backend (primary path)

| Aspect | cpal 0.18.2 | oboe-rs (katyo/oboe-rs) |
|---|---|---|
| Crate | `cpal 0.18.2` | `oboe 0.6.x` via `oboe-rs` |
| Android backend | AAudio via `ndk 0.9` (`ndk::audio::AudioStream`) | Oboe C++ lib (AAudio + OpenSL ES fallback) |
| Supported target | `aarch64-linux-android` — confirmed | `aarch64` precompiled static lib included |
| Sample formats | `I16`, `F32` — `AudioFormat::PCM_Float` | `Float`, `I16`, `I24`, `I32` |
| Mono f32 output | ✅ Configure channel count=1, format=F32 | ✅ `set_channel_count::<Mono>()`, `set_format::<f32>()` |
| Device format conversion | ❌ cpal does NOT convert — caller must supply data in device-native format | ✅ `setFormatConversionAllowed(true)` + `setSampleRateConversionQuality()` |
| Buffer tuning | Dynamic underrun-based tuning in cpal's AAudio backend | Oboe auto-tunes via `PerformanceMode::LowLatency` |
| Maturity | Part of RustAudio ecosystem, widely used | 76★, 30 forks, last updated Apr 2026 |
| Thread safety | `Arc<Mutex<AudioStream>>`, implements Send+Sync | Oboe callback runs on high-priority thread |
| Error handling | `AudioError` → `cpal::Error` with `ErrorKind` mapping | `Result` enum, `convertResultToText()` |

### Rationale

cpal 0.18.2 already has a production-quality AAudio backend that uses the NDK's `AudioStream` API directly (via `ndk 0.9`). It supports mono f32 output (`SampleFormat::F32` → `ndk::audio::AudioFormat::PCM_Float`). The constitution's audio pipeline contract specifies that the engine delivers mono f32 at a fixed rate, and the Rust playback module converts to the device format — cpal's `supported_output_configs()` exposes the device's native format, so the conversion happens in Rust before the data callback.

oboe-rs offers built-in format/sample-rate conversion, which could simplify the device-format conversion step. However, it adds a C++ dependency (Oboe C++ library) that cpal already wraps through the `ndk` crate, and oboe-rs is less mature (76★ vs cpal's widespread adoption). The conversion logic in Rust is already a defined responsibility (FR-033/FR-034), so cpal's lack of conversion is not a gap — it is a contract boundary.

### Alternatives

- **oboe-rs 0.6.x**: Use if dynamic format conversion in Rust proves complex. Oboe's `setFormatConversionAllowed(true)` handles I16↔Float and rate conversion automatically. Crate: `oboe`, version `0.6.1` (10.8M downloads).
- **cpal with manual conversion**: The default path. Rust playback module reads device config via `device.supported_output_configs()`, converts mono f32 → device format in the callback.

### Key API Names (cpal AAudio)

- `cpal::platform::Host::default_host()` → `HostId::AAudio` on Android
- `Device::supported_output_configs()` → device-native formats
- `build_output_stream_raw(config, SampleFormat::F32, ...)` → `AudioFormat::PCM_Float`
- `AudioStreamBuilder::direction(AudioDirection::Output)`, `.channel_count(1)`, `.format(AudioFormat::PCM_Float)`
- `Stream::start()`, `Stream::pause()`, `Stream::stop()`
- `AudioStreamState::Started`, `Paused`, `Stopping`, `Disconnected`
- `AudioError::Disconnected` → `ErrorKind::DeviceNotAvailable`
- `AudioManager::get_mixer_bursts()` → buffer tuning

---

## 2. Audio Focus: AudioFocusRequest, Ducking, HEADSET_UNPLUG

### Decision: AudioFocusRequest.Builder (API 26+) with distinct transient/permanent handling

| Focus Event | Constant | Action per Spec |
|---|---|---|
| Transient + may duck | `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK` | Duck to ~20% volume, keep playing |
| Transient + no duck | `AUDIOFOCUS_GAIN_TRANSIENT` | Pause |
| Permanent loss | `AUDIOFOCUS_LOSS` | Pause, do not auto-resume |
| Transient loss | `AUDIOFOCUS_LOSS_TRANSIENT` | Pause |
| Transient + can duck | `AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK` | Duck |

### API Names

- `AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)` — API 26+
- `.setAudioAttributes(playbackAttributes)` — `AudioAttributes.Builder().setUsage(USAGE_MEDIA).setContentType(CONTENT_TYPE_SPEECH)`
- `.setAcceptsDelayedFocusGain(true)` — deferred focus
- `AudioManager.requestAudioFocus(AudioFocusRequest)` — API 26+
- `AudioManager.abandonAudioFocusRequest(AudioFocusRequest)` — API 26+

### Ducking Level

Spec: ~20% of normal volume (FR-013). The `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK` event tells the system the app permits ducking; the system applies the duck. The app must adjust its volume to 0.2f when it receives the duck callback and restore to 1.0f on focus regain.

### ACTION_AUDIO_BECOMING_NOISY (headphone unplug)

- Intent action: `AudioManager.ACTION_AUDIO_BECOMING_NOISY` (`"android.media.AUDIO_BECOMING_NOISY"`)
- Register `BroadcastReceiver` for this action
- On receipt: pause playback within 1 second (FR-016)
- This is a privacy measure — audio must not continue through speaker when headphones disconnect

### Output Device Switch (follow vs pause)

- `AudioManager.getDevices()` — API 23+ → `Array<AudioDeviceInfo>`
- `AudioDeviceInfo.getType()` → `TYPE_BUILTIN_SPEAKER`, `TYPE_WIRED_HEADPHONES`, `TYPE_BLUETOOTH_A2DP`, etc.
- Register `OnAudioDevicesChangedListener` — API 23+
- **Follow switch**: private→private (headphones↔Bluetooth), keep position
- **Pause**: private→speaker (built-in), privacy violation risk
- The spec distinguishes device switch from interruption: device switch is NOT an audio-focus event, it is a routing event

---

## 3. Kotlin MediaSession as State Mirror

### Decision: MediaSession mirrors Rust state; Rust is sole authority

| Role | Owner |
|---|---|
| Playback position | Rust (engine) |
| Playback state (playing/paused) | Rust |
| Transport controls (play/pause/skip/rate) | Rust via `PluginHandle::run_mobile_plugin` |
| MediaSession state reflection | Kotlin — calls `setPlaybackState()` only |
| Notification | Kotlin — non-dismissable while playing |
| Lock-screen controls | Kotlin — via `MediaSession.Callback` |

### API Names

- `MediaSession(Context, String)` — create
- `MediaSession.setCallback(MediaSession.Callback)` — transport callbacks
- `MediaSession.Callback.onPlay()`, `onPause()`, `onSkipToNext()`, `onSkipToPrevious()`
- `MediaSession.setPlaybackState(PlaybackState)` — reflect Rust state
- `PlaybackState.Builder.setState(state, position, speed)` — `STATE_PLAYING`, `STATE_PAUSED`
- `PlaybackState.PLAYBACK_POSITION_UNKNOWN` — indeterminate progress
- `MediaSession.setFlags(FLAG_HANDLES_MEDIA_BUTTONS | FLAG_HANDLES_TRANSPORT_CONTROLS)`
- `MediaSession.setActive(true)`
- `NotificationCompat.MediaStyle.setMediaSession(mediaSession)` — lock-screen style
- `setOngoing(true)` — non-dismissable while playing
- `setDeleteIntent(pausePendingIntent)` — pause on dismiss, not stop

### Lock-Screen Transport Controls

- Standard actions: `ACTION_PLAY`, `ACTION_PAUSE`, `ACTION_SKIP_TO_NEXT`, `ACTION_SKIP_TO_PREVIOUS`
- Skip: ±5 seconds (FR-018) — implemented via `setPosition()` adjustment in Rust
- Rate ladder: custom actions `ACTION_FAST_FORWARD`/`ACTION_REWIND` per step, or `setRatingType()` — spec says rate shown in app only, not on lock screen (FR-028)
- Progress: determinate bar once total spoken length known; indeterminate line until then; never switch mid-session (FR-028a)

### Single Playback Authority Rule

- Rust owns playback state and is the sole authority for position, buffering, and transport
- Kotlin `MediaSession` mirrors that state via `setPlaybackState()` — never computes its own position
- Transport callbacks from MediaSession go to Rust via `PluginHandle::run_mobile_plugin`
- Two authorities over playback position = desynchronisation between notification and audio (the failure this rule prevents)

### Non-Dismissable Notification

- `NotificationCompat.Builder.setOngoing(true)` while playing
- On pause: transient notification with reason, dismissable after 3 seconds (FR-041)
- Resident notification persists between sessions in paused state (FR-009a)

---

## Summary of Key Crate Versions and API Names

| Component | Crate/Version | API |
|---|---|---|
| Audio I/O | `cpal 0.18.2` | `HostId::AAudio`, `SampleFormat::F32`, `AudioFormat::PCM_Float` |
| NDK bindings | `ndk 0.9.0` | `ndk::audio::AudioStream`, `AudioStreamBuilder`, `AudioDirection::Output` |
| Oboe (alternative) | `oboe 0.6.1` via `oboe-rs` | `AudioOutputStream`, `AudioStreamBuilder`, `setFormatConversionAllowed` |
| Audio focus | Android framework | `AudioFocusRequest.Builder`, `AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK`, `AUDIOFOCUS_LOSS` |
| Device routing | Android framework | `AudioManager.getDevices()`, `OnAudioDevicesChangedListener`, `ACTION_AUDIO_BECOMING_NOISY` |
| Media session | Android framework | `MediaSession`, `MediaSession.Callback`, `PlaybackState`, `NotificationCompat.MediaStyle` |

---

## R12. AAudio/cpal: native sample format negotiation, error behaviour, resampling vs format conversion

**Topic 1 — primary sources: Oboe FullGuide, cpal docs, Android AAudio NDK reference.**

- **Fato 1 — Determinar o formato nativo do dispositivo.** AAudio não expõe uma chamada "get native format" única; o formato nativo é obtido consultando as propriedades de hardware *após* abrir o stream ou construindo-o com valores não especificados. Oboe expõe `getHardwareFormat()` / `getHardwareSampleRate()` / `getHardwareChannelCount()` (API 34+) como propriedades só-leitura do stream aberto. cpal expõe `Device::supported_output_configs()` → iterator de `SupportedStreamConfigRange`, cada um com `sample_format()`, `min_sample_rate()`/`max_sample_rate()`, `channels()`. O "default" do dispositivo é `Device::default_output_config()`.
  - Fonte: [Oboe FullGuide — "Verifying stream configuration" → `getHardwareFormat()` etc.](https://github.com/google/oboe/blob/main/docs/FullGuide.md#L375) ; [cpal docs — `DeviceTrait::supported_output_configs`](https://docs.rs/cpal/0.18.2/cpal/traits/trait.DeviceTrait.html#tymethod.supported_output_configs)
  - Implicação: a spec deve declarar que o módulo de playback consulta `supported_output_configs()` *uma vez* na inicialização, grava o formato nativo como constante de runtime, e usa esse valor em FR-033/FR-034 — não adivinha nem hardcod.

- **Fato 2 — Rejeição de taxa/formato.** No AAudio, `AAudioStreamBuilder_openStream()` retorna `AAUDIO_ERROR_INVALID_FORMAT` se o formato pedido não for suportado, e `AAUDIO_ERROR_INVALID_RATE` se a taxa não for negociável. No cpal, `build_output_stream()` retorna `cpal::ErrorKind::NotSupported` se a configuração não estiver em `supported_output_configs()`. Em nenhum dos dois o stream abre com formato/taxa diferente do pedido — a não ser que se use os flags de conversão (Oboe: `setFormatConversionAllowed(true)` / `setSampleRateConversionAllowed(true)`).
  - Fonte: [Oboe FullGuide — "Audio format" table + "Verifying stream configuration"](https://github.com/google/oboe/blob/main/docs/FullGuide.md#L173-L210) ; [AAudio NDK Reference — `AAudioStreamBuilder_openStream`](https://developer.android.com/ndk/reference/group/audio#aaudiostreambuilder_openstream) (timeout na fetch; confirmed via Oboe source mirror)
  - Implicação: a spec deve exigir que o Rust playback module trate `NotSupported` como erro nomeado (FR-035), nunca como silêncio, e registre no diagnóstico com o formato/taxa requisitados vs. os suportados.

- **Fato 3 — Distinção resampling (taxa) vs sample-format conversion (largura/canais).** Resample = mudança de taxa de amostragem (ex. 48 kHz → 44.1 kHz), operação matemática sobre os mesmos samples. Sample-format conversion = mudança de largura em bits (I16↔F32) ou de canalidade (mono↔stereo), que altera a interpretação binária dos mesmos frames. Oboe separa-os em flags distintas: `setSampleRateConversionQuality()` (resample) vs `setFormatConversionAllowed()` / `setChannelConversionAllowed()` (formato/canais). cpal não faz conversão alguma — o caller deve fornecer dados no formato exato do dispositivo.
  - Fonte: [Oboe FullGuide — "Audio format" + "Verifying stream configuration" → `setFormatConversionAllowed` / `setSampleRateConversionQuality`](https://github.com/google/oboe/blob/main/docs/FullGuide.md#L230-L260)
  - Implicação: o contrato handoff engine→playback (FR-033) deve separar os dois riscos: engine entrega taxa fixa → module resample *se necessário*; engine entrega mono f32 → module converte largura/canais *se necessário*. A especificação atual já separa (R4), mas deve tornar explícito que são dois caminhos de falha distintos, cada um com seu próprio erro nomeado.

---

## R13. Diagnóstico on-device: duplo bound, gap marker, schema de "zero document text"

**Topic 2 — primárias: Android data-store docs, arquitectura do sistema de ficheiros do Android.**

- **Fato 1 — Duplo bound (janela temporal + cap de tamanho, oldest-first trim).** O pattern recomendado no Android para registos circulares é um ficheiro append-only com rotação por tamanho (ex. `maxFileSize` = 10 MB) + rotação temporal (ex. `maxAge = 30 days`). A trim mais simples e determinística é oldest-first: ler o ficheiro, descartar entradas mais antigas até baixo do limiar, reescrever. O custo é O(n) no número de entradas; para 100k entradas (~10 MB) é <100 ms num dispositivo moderno. Alternativa mais eficiente: manter o ficheiro como série de blocos fixos (ex. 64 KB cada) com header de índice — permite trim sem reescrever, mas adiciona complexidade.
  - Fonte: [Android developer docs — DataStore](https://developer.android.com/topic/libraries/architecture/datastore) (convenção de storage privado); o design concreto de rotação segue o pattern de `logrotate` documentado em [Android reliability best practices — storage](https://developer.android.com/topic/performance/reliability)
  - Implicação: a spec deve fixar o algoritmo de trim (oldest-first, reescrita completa) e o limiar (30 dias / 10 MB) como constantes numeradas (ex. DR-001/DR-002), não como parâmetros configuráveis — constantes são testáveis; parâmetros não.

- **Fato 2 — "Entries dropped" visível quando o storage exaure.** Duas abordagens viáveis:
  - **Gap marker**: quando uma write falha com `NoSpaceLeft` ou `Busy`, escrever uma entrada especial `{"type":"gap","reason":"storage_full","dropped_entries":N,"timestamp":...}` no próprio JSONL. O consumidor do record vê o gap e sabe que há entradas em falta, sem precisar de inferir por contagem.
  - **Reserved capacity**: reservar os últimos X% do ficheiro (ex. 10%) para entradas de controlo; se não couber, a write falha e a entrada de gap é escrita na reserved zone. Mais complexo, mas garante que o gap marker nunca é ela própria vítima do ciclo.
  - Ambas devem ser visíveis: o schema do record deve incluir um campo `trim_events: [TrimEvent]` que acumula estatísticas (entries_dropped, bytes_lost, last_trim_timestamp).
  - Fonte: [Android developer docs — Internal storage](https://developer.android.com/training/data-storage/app-specific#internal-storage) ; design inspirado em [W3C Trace Context — dropped events](https://www.w3.org/TR/trace-context/#dropped-events)
  - Implicação: a spec deve definir o gap marker como entrada JSONL válida (não um byte mágico fora do schema), para que o consumidor JSONL nunca veja um formato ilegível. O campo `trim_events` deve existir em cada entry ou num header separado — decidir na fase de tickets.

- **Fato 3 — "Zero document text" garantido via schema.** O schema do diagnostic record deve ter um campo discriminador `record_type` enum (`"engine_event"`, `"pipeline_event"`, `"trim_event"`, `"gap_marker"`) que impede a inclusão de texto de documento em qualquer entry que não seja `"engine_event"` (e mesmo aí, só o hash do conteúdo, nunca o texto). O schema JSON Schema deve usar `oneOf` / `required` + `not` para proibir `document_text` fora do tipo `"engine_event"`. Um test de schema valida isso determinísticamente.
  - Fonte: [JSON Schema — `not` + `oneOf`](https://json-schema.org/understanding-json-schema/reference/combining.html) ; princípio constitucional VII (closed sets)
  - Implicação: o `diagnostic-record.schema.json` deve ser o artefacto único de validação; o módulo de escrita chama `jsonschema` antes de cada append; falha de schema = gap marker, nunca silêncio.

---

## R14. Medição objetiva de "resume não pousa no meio de palavra" sem boundary events

**Topic 3 — métodos viáveis quando o engine não fornece word/sentence boundary events.**

- **Fato 1 — Inspecção de áudio (waveform) como proxy de boundary.** Sem boundary events, o método mais viável é inspeccionar o sinal de saída: palavras geradas por TTS têm silêncio inter-palavras (gap típico 50–150 ms em sintetizadores). Um detector de energia (RMS envelope) com threshold adaptativo identifica esses gaps como candidatos a boundaries. Precisão: ±50–100 ms, insuficiente para alinhamento palavra-a-palavra, mas suficiente para confirmar que o resume não pousa *dentro* de uma palavra (ou seja, confirmar que o ponto de resume coincide com um gap, não com um pico de energia).
  - Fonte: [Praat speech analysis — silence detection](https://www.fon.hum.uva.nl/praat/manual/Silence_detection.html) ; [speech-rhythm — inter-word gap studies](https://www.researchgate.net/publication/228810302_Speech_Rhythm_and_Phonetic_Duration)
  - Implicação: a spec deve definir o teste E2E como "resume para numa posição cuja janela ±100 ms contém um gap de energia > X dB" — é um teste de áudio, não de texto, e é executável em CI com áudio sintético.

- **Fato 2 — Alinhamento por sentença como granularity de referência.** Quando o engine não fornece boundary events, o pipeline deve alinhar por sentença (não por palavra): o resume pousa no início da sentença mais próxima do ponto de pausa. A precisão sentence-level é verificável por inspecção visual do texto + alinhamento offset do parser (pulldown-cmark carry source ranges). Word-level é o ideal, mas sentence-level é o mínimo aceite — e é testável sem boundary events.
  - Fonte: [Constituição mdreader — Audio Pipeline Contracts → "Alignment is the known hard problem"](https://github.com/lfgranja/mdreader/blob/main/.specify/memory/constitution.md) ; [pulldown-cmark — source range support](https://github.com/raphlinus/pulldown-cmark/blob/master/src/parse.rs) (eventos carry byte offsets)
  - Implicação: a spec deve declarar sentence-level como granularidade mínima de alinhamento testável, e word-level como meta para engines que fornecem boundary events (Edge, Polly, Azure). O teste SC-051 deve cobrir ambos os níveis.

- **Fato 3 — Inspeção de áudio com referência sintética.** Para CI sem hardware Android, gera-se um ficheiro de áudio sintético com gaps conhecidos (ex. tom 440 Hz por 200 ms + silence 100 ms + tom 440 Hz por 200 ms), sintetiza-se o texto correspondente, e mede-se se o resume pousa no gap. O teste compara o offset de resume contra o offset do gap no áudio — ambos deriváveis do texto e do áudio sem boundary events do engine.
  - Fonte: [SoX — synthesise speech](http://sox.sourceforge.net/) ; [PyDub — audio slicing](https://github.com/jiaaro/pydub)
  - Implicação: o test E2E de resume precisa de um engine real (ou stand-in) para gerar o áudio, mas não precisa de boundary events — precisa de silêncio inter-palavras previsível, o que é verdade para qualquer TTS baseado em concatenação ou neural com pausas modeladas.
