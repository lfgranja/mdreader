# Clarification D3 — Overlap policy when transport actions arrive faster than earcon durations

> **Feature:** `specs/005-adopt-design-sona` (FR-009/FR-010/SC-002: additive earcon mix over narration; ≤100 ms action→audible; seven confirmation types; drop-and-refire on pause for user-initiated confirmations only).
> **Question:** FR-009 mandates that every triggered transport action sounds its earcon (additively mixed over narration; suppression prohibited unless physically impossible) but defines no behaviour when actions arrive faster than the earcon durations (Adagio/Andante 120 ms, Unison 60 ms, Allegro skip chirps 40 ms). Which of (a) additive mix / overlap, (b) truncate-replace (latest wins), (c) drop if an earcon is in flight, (d) queue, is supported by evidence?
> **Scope:** research note; not a spec amendment. Informs a clarification decision only.
> **Method (2026-10-24):** primary-source review of auditory-scene-analysis/streaming (Bregman), simultaneous and temporal masking (Moore via citable previews; ansys/Synopsys masking note; Wikipedia auditory masking), FM-sweep interval evidence (Luo et al. 2007; Kung et al. 2020), earcon design guidance (Brewster, Wright & Edwards 1992–1995; McGookin & Brewster 2003/2004; Bonebright & Nees 2009; Sonification Handbook ch. 14), alert redundancy/consolidation doctrine (FAA AC 25.1322-1; 14 CFR 25.1322; Horvitz et al. 1999/2003), and UI sound-design practice (Material sound; product-sound-design literature). No fabricated citations; paywalled standards treated as direction only.

## 1. The problem and the four candidates

The earcon durations are fixed by the shipped 004 grammar: **Adagio play / Andante pause = 120 ms; Unison rate tick = 60 ms; Allegro skip chirps = 40 ms (880→1108 Hz up / 880→660 Hz down)**. A driver can press play, pause, skip and change rate in well under 120 ms (a single double-tap or a fast notification swipe). Four policies are on the table:

- **(a) additive mix** — every earcon sounds in full, overlapping.
- **(b) truncate/replace** — a new action cuts the in-flight earcon short; the new one plays in full ("latest wins").
- **(c) drop if in flight** — the new earcon is suppressed while one is playing.
- **(d) queue** — earcons play back-to-back in arrival order.

Constraints that any policy must satisfy (from the spec and constitution): (i) **every triggered action sounds its earcon** — suppression is prohibited unless physically impossible (FR-009); (ii) **≤100 ms** from action to first consumed earcon sample (FR-010); (iii) narration is **never** ducked, paused or delayed for an earcon (FR-009); (iv) **system-initiated** warning (Ostinato) / failure (Sforzando) earcons fire immediately, never queued, dropped or delayed (FR-009, 2026-10-23 clarification); (v) drop-and-refire applies to **user-initiated** confirmations only, when a pause is in force at trigger time.

## 2. Auditory perception of overlapping brief sounds

- **Bregman, *Auditory Scene Analysis* (1990)**: the auditory system parses overlapping energy into discrete "streams" using simultaneous (spectral proximity, harmonicity, onset synchrony) and sequential (pitch proximity, tempo, timbre) grouping. When two brief sounds share onset time, frequency region and level, they fuse into a single percept or one masks the other; they are *not* heard as two independent events unless segregated by a cue (timbre, location, onset asynchrony). (Citable via the Timbre and Orchestration Resource summary, https://timbreandorchestration.org/writings/timbre-lingo/2019/2/12/auditory-scene-analysis; napier blog, http://blogs.napier.ac.uk/sounddesign/2026/03/02/auditory-scene-analysis-albert-bregman.)
- **Simultaneous masking (Moore, *An Introduction to the Psychology of Hearing*, 6th ed.)**: a louder (or equal-level) sound raises the threshold for a simultaneous softer sound in the same critical band; two equal-level brief tones in the same frequency region partially mask each other (informational + energetic masking). Moore's text is paywalled; the mechanism is corroborated by the Synopsys/ansys masking note (https://ansys.synopsys.com/blog/what-is-auditory-masking) and the Timbre and Orchestration masking page (https://timbreandorchestration.org/writings/timbre-lingo/masking).
- **Temporal masking (Wikipedia, "Auditory masking", https://en.wikipedia.org/wiki/Auditory_masking)**: forward masking (masker precedes signal) decays over **≈100 ms** after the masker ends; backward masking (masker follows signal) is effective only within **≈20 ms**. So even *sequential* earcons with no gap can suffer forward masking for ~100 ms — but simultaneous overlap is the worst case (energetic + informational masking at the same instant).
- **Luo, Boemio, Gordon & Poeppel 2007, *Hearing Research* 224(1-2):75–83** (PDF https://www.psy.pku.edu.cn/static/main/uploads/faculty/luo_huan/(10)Luo_Hearingresearch_2007.pdf): FM-sweep direction-identification threshold ≈ **20 ms** for Mandarin/English listeners; downward sweeps are harder than upward. **Honesty note: Luo tests single sweeps in isolation; it says nothing about intervals or overlap between presentations.** The ISI is not a variable.
- **Kung, Wu, Hsu & Hsieh 2020, *Frontiers in Psychology* 11:389** (PMC7078663, https://pmc.ncbi.nlm.nih.gov/articles/PMC7078663/): MEG + behavioural; ISI fixed at **700 ms**, not varied. Confirms the 20 ms elementary window but, again, **no overlap/interval manipulation**. Neither paper licenses any claim about two earcons 40–120 ms apart.
- **The only direct evidence on rapid/overlapping structured sounds** is McGookin & Brewster (§3 below): a **300 ms onset-to-onset delay** between concurrent earcons significantly improves identification.

## 3. Earcon design guidance on overlap and presentation rate

- **Brewster, Wright & Edwards 1992/1993/1994/1995** (earcon guidelines; parallel earcons, *Int. J. Human-Computer Studies* 43:153–175; https://www.dcs.gla.ac.uk/~stephen/papers/IJHCS.pdf): earcons can be presented in parallel to raise the rate of information transfer, but only when segregated (unique timbre, spatialisation, onset asynchrony); without segregation they interfere. Guidelines do not endorse blind overlap of same-timbre, same-channel icons.
- **McGookin & Brewster 2004, *ACM Trans. Applied Perception* 1(2):130–155** (https://eprints.gla.ac.uk/3281): two experiments on concurrently presented earcons. (1) Fewer concurrent earcons → higher identification. (2) Two manipulations significantly raised identification: **unique timbre per earcon** and **a 300 ms onset-to-onset delay** between earcons. Conclusion: concurrent/rapid earcons interfere unless separated by ~300 ms or made spectrally distinct. **This is the load-bearing citation for D3.**
- **McGookin & Brewster 2003, ICAD** (http://www.icad.org/Proceedings/2003/McGookinBrewster2003.pdf): baseline for the above; four simultaneous earcons identified poorly.
- **Bonebright & Nees 2009, *Applied Cognitive Psychology* 23(3):431–445**: stable-pitch earcons do not impair connected-speech comprehension — supports additive (non-ducking) mixing with narration, already adopted in FR-009. Does not address earcon-earcon overlap.
- **Nees & Liebman 2023 meta-analysis** (*Auditory Perception & Cognition*): earcons are among the least-interfering brief alerts, but "thorough testing … in their target environment" is required.
- **Sonification Handbook ch. 14** (https://sonification.de/handbook/chapters/chapter14): earcon identification baselines ~10% untrained / ~80% after 5–10 min training; reiterates the 300 ms delay finding.

## 4. Alert redundancy / consolidation doctrine in high-tempo environments

- **FAA AC 25.1322-1** (https://www.faa.gov/documentLibrary/media/Advisory_Circular/AC_25.1322-1.pdf), §b "Multiple Aural Alerts": **"Aural alerts should be prioritized so that only one aural alert is presented at a time."** "An active aural alert should finish before another aural alert begins. However, active aural alerts must be interrupted by alerts from higher urgency levels if the delay to annunciate the higher-priority alert impacts the timely response." If interrupted, the lower-urgency alert may repeat once the higher one completes. (Direction: aviation-grade doctrine, paywalled-adjacent; quoted text retrieved from the AC PDF.)
- **14 CFR 25.1322** (https://www.ecfr.gov/current/title-14/chapter-I/subchapter-C/part-25/.../section-25.1322): alerts must be "readily and easily detectable and intelligible … including conditions where multiple alerts are provided" — intelligibility under multiplicity is the regulatory floor.
- **Horvitz, Jacobs & Hovel 1999, UAI** (http://erichorvitz.com/attend.htm) and **Horvitz et al. 2003, CACM 46(3)** (https://erichorvitz.com/Models_of_attention_in_computing.pdf): attention-sensitive alerting — chunking/coalescing multiple messages into a single compound alert can *raise* the value of the information delivered; critical events bypass deferral (already cited in this spec for the system-vs-user-initiated boundary). For multi-event collisions the doctrine is **coalesce/prioritize**, not stack.
- **Automotive:** FMVSS 141 / UN R138 (already cited in this spec for AVAS pause-function removal) and ISO 15006 (warning time classes) address *pausing* of AVAS, not earcon overlap; **ISO 15623 / FCMW and SAE ARP automotive warning-management standards were not located in this pass** — treated as a dead end here, not as evidence. (Honesty: no automotive source on overlap was retrieved; do not cite one.)
- **Scope boundary:** this note addresses only the *presentation* policy. It does not revisit the 40 ms chirp length question (owned by B1) nor the drop-and-refire-on-pause rule (settled 2026-10-23); it assumes both stand.

## 5. UI sound-design practice for point-of-interaction feedback

- **Material sound** (https://m2.material.io/design/sound/about-sound.html; https://design.google/library/ux-sound-haptic-material-design): sounds are short, functional, family-coherent feedback; the design guidance stresses *concision* and avoiding unnecessary cues — i.e., each interaction earns at most one short sound, and piling sounds is an anti-pattern. No explicit "overlap" rule was found in the retrieved pages; the principle is implicit (one sound per action, short).
- **Product sound design literature** (IntechOpen ch. 43375, https://www.intechopen.com/chapters/43375): discrete feedback sounds "only sound once after a key has been pressed"; continuous monitoring (parking beeps) varies *rate*, not overlap. Rapid repeated presses are served by re-triggering, not by stacking.
- **Game audio / voice-stealing doctrine** (well-established practice, not a single document): when a new sound of equal-or-higher priority arrives, the engine **steals the voice** of the in-flight sound (truncate/replace) rather than (a) mixing into mud or (c) dropping the new sound. This is the industry default for short UI SFX on limited voice counts. (No single citable spec retrieved; flagged as practice, not standard.)

## 6. Synthesis: which policy the evidence supports

| Policy | Compliance with FR-009 (no suppression) | ≤100 ms under rapid input | Perceptual evidence | Verdict |
|---|---|---|---|---|
| (a) additive mix / overlap | yes | yes | **Against** — McGookin 2004: 300 ms separation needed; simultaneous same-channel tones fuse/mask (Bregman, Moore) | Reject |
| (c) drop if in flight | **No** — suppresses a triggered action's earcon | yes | n/a | Reject (violates FR-009) |
| (d) queue | yes | **No** — backlog delays starts beyond 100 ms | McGookin: 300 ms separation still not met unless gaps added | Reject (violates FR-010) |
| **(b) truncate/replace (latest wins)** | **yes** — every triggered action initiates its earcon | **yes** — latest starts within one render slice | Avoids simultaneous overlap; consistent with FAA "one aural alert at a time" + voice-stealing | **Adopt** |

**Recommendation: adopt (b) truncate/replace ("latest wins") for user-initiated transport confirmations**, with two carve-outs carried over from the spec:

1. **System-initiated warning/failure earcons (Ostinato, Sforzando) always interrupt** any in-flight confirmation earcon and sound immediately — they are higher urgency and are never queued, dropped or delayed (FAA: higher-urgency interrupts; Horvitz: critical events bypass deferral). This is the only case in which a user confirmation earcon is cut short by something other than a newer user action.
2. **Drop-and-refire when paused** (already in FR-009) remains scoped to user-initiated confirmations: if a pause is in force at trigger time, the earcon is dropped and re-fired once playback resumes; warning/failure fire immediately regardless.

The "additive" in FR-009's "additive mix over narration" is **earcon↔narration only** (non-ducking, full gain on the duck-exempt short-sound channel) — it must not be read as licensing earcon↔earcon overlap. Earcon↔earcon is governed by this D3 sub-rule.

**Why not queue with a cap:** a bounded queue would still push starts past 100 ms under burst input (FR-010) and would not achieve the 300 ms separation McGookin shows is needed for identification — it would buy nothing over truncate/replace while adding latency. Queueing is also the policy the spec already forbids for system alerts.

## 7. Honesty, uncertainty and limits

- **No source tests our exact stimuli.** McGookin 2004 used multi-note earcons with 300 ms onset delays; our earcons are single events 40–120 ms. The 300 ms figure is an identification threshold for *concurrent* earcons, not a measured property of our set. Applying it to our case is an extrapolation, flagged as such.
- **Forward masking (~100 ms, Wikipedia/ansys) means truncate/replace does not fully eliminate perceptual interaction:** a new earcon starting immediately after a truncated one can still be forward-masked for up to ~100 ms. Because the previous sound is *truncated* (shorter effective masker) and often lower in accumulated energy, the effect is smaller than under additive mix, but it is not zero. Rapid presses <~100 ms apart may still show some masking; the ≤100 ms bound prevents us from inserting the 100 ms gap that would fully clear it.
- **Luo 2007 and Kung 2020 do not test intervals** — they are cited here only to bound the single-sweep direction-ID risk (40 ms downward chirp), not to support any overlap claim. Do not cite them as overlap evidence.
- **Automotive warning-management standards were not located** (dead end); FMVSS 141/UN R138 address AVAS pausing, not overlap.
- **The blind test (SC-002) presents single stimuli in isolation** and therefore cannot detect an overlap problem. A rapid-sequence sub-condition is needed (§8).

## 8. Falsifiable considerations for the maintainer's blind test (SC-002)

The SC-002 confusion matrix (single stimuli, ≥15 trials/type, no two identical consecutively) will not by itself adjudicate D3. Which policy, if wrong, would show up where:

- **If (a) additive mix were adopted and is wrong:** systematic confusions cluster in **descending-pitch pairs** — skip-backward Allegro (880→660 Hz, 40 ms) with Andante pause (E5→C5, 120 ms), both descending — and in **shared-fundamental pairs** — Unison tick (880 Hz, 60 ms) with Allegro skip-forward (880→1108 Hz, 40 ms), same onset frequency. The 40 ms downward chirp (already the riskiest stimulus per B1/Luo) would be the first to be lost inside a 120 ms play/pause tail. Detectable only in a **rapid-sequence condition** (two or three actions <150 ms apart), not in the single-stimulus matrix.
- **If (b) truncate/replace is adopted and is wrong:** no overlap confusions, but **missed intermediate confirmations** — the user hears only the last of a rapid burst. Symptom is "I didn't hear play" reports, not confusions. Falsifiable via a diagnostic counter of truncated earcons and a rapid-sequence identification task (expect "last only"; if users report hearing multiples, an additive-mix bug is present).
- **If (c) drop is adopted and is wrong:** "no earcon heard on rapid presses" — trivially detected by a suppressed-event diagnostic; also a direct FR-009 violation.
- **If (d) queue is adopted and is wrong:** action→audible >100 ms — caught directly by the FR-010 host measurement; also by users perceiving lag.

**Recommendation to the maintainer:** add a small **rapid-sequence sub-test** (2–3 triggered actions within 100–150 ms, e.g., play→rate→skip) to the SC-002 session, scoring "which actions were confirmed" in addition to single-stimulus identification. This is the only way D3 is empirically falsifiable; the single-stimulus matrix alone cannot.

## Implication for the spec

**Recommendation: adopt policy (b) truncate/replace ("latest wins") as an FR-009 sub-rule**, with system-initiated interrupt and the existing pause drop-and-refire carve-outs. Draft wording (pt-BR, consistent with the spec's voice):

> **FR-009 (acrescentar sub-regra, registro 2026-10-24, D3):** Quando ações de transporte iniciadas pelo usuário chegam mais rápido do que a duração do earcon em curso, o earcon em curso é **truncado e substituído** pelo da ação mais recente ("latest wins"): cada ação disparada inicia seu earcon (nenhum é suprimido), e o novo earcon começa no próximo espaço de renderização do canal curto, dentro do limite de 100 ms da FR-010. **Nenhum earcon de confirmação de transporte se sobrepõe a outro** — a mistura aditiva da FR-009 aplica-se exclusivamente entre earcon e narração em andamento (ganho pleno, sem duck), nunca entre earcons. Earcons iniciados pelo sistema (Ostinato de aviso de pausa, Sforzando de falha permanente) **interrompem** qualquer earcon de confirmação em curso e soam imediatamente — prioridade maior, nunca enfileirados, descartados ou atrasados. A regra de descarte-e-re-disparo por pausa permanece exclusiva dos earcons de confirmação iniciados pelo usuário. A supressão de um earcon disparado continua proibida; a truncagem por ação mais recente não é supressão (o earcon da ação disparada soa). Falha tecnológica de emissão segue a degradação da B2 (máximo uma re-emissão, evento diagnóstico tipado).

Concretamente:
1. **Não adotar (a)** — a mistura aditiva entre earcons sobrepõe ícones de 40–120 ms no mesmo canal e os mescla/mascara (McGookin 2004: 300 ms de separação necessários; Bregman/Moore: fusão e mascaramento simultâneo). O chirp descendente de 40 ms (já o estímulo mais arriscado do B1) seria o primeiro a se perder dentro da cauda de 120 ms de play/pausa.
2. **Não adotar (c)** — viola a proibição de supressão da FR-009 (um gatilho de ação de transporte não soaria seu earcon).
3. **Não adotar (d)** — viola a FR-010 (backlog empurra o início além de 100 ms) e não alcança a separação de 300 ms de qualquer forma.
4. **Manter a distinção earcon↔narração (aditiva, sem duck) vs. earcon↔earcon (truncagem)** na redação da FR-009, para que a palavra "aditivamente" não seja lida como licença de sobreposição entre earcons.
5. **Acrescentar ao SC-002 um sub-teste de sequência rápida** (2–3 ações em <150 ms) pontuando "quais ações foram confirmadas", além da matriz de estímulo único — único meio de falsificar esta sub-regra. Registrar a contagem de earcons truncados como evento diagnóstico (sem texto de documento, padrão de identificador, ≤200 chars — disciplina do registro diagnóstico da 001).
6. **Não reespecificar pitches/timbres** — a gramática da 004 permanece intacta; esta sub-regra trata apenas de política de apresentação.
7. **Registrar como exceção/ADR** a interrupção de um earcon de confirmação por um earcon de sistema (prioridade), análoga ao ADR-001 (ducking) já registrado na constituição.