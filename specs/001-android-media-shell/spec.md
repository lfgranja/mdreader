# Feature Specification: Android Media Shell

**Feature Branch**: `001-android-media-shell`

**Created**: 2026-10-01

**Status**: Draft

**Input**: User description: "Android media shell for a driving-first Markdown
read-aloud app. PCM playback owned by Rust, with the Kotlin surface reduced to what the
Android platform structurally requires. Covers: PCM playback via cpal/oboe with Rust as the
single authority on position and buffering; Kotlin MediaSession as a mirror, never a second
source of truth; audio focus with ducking on transient loss and pause on permanent loss;
ACTION_AUDIO_BECOMING_NOISY so unplugging headphones pauses rather than playing out of a
speaker; playback with the screen off and the device locked; the manual <service> manifest
declaration as a verifiable requirement; and Service bootstrapping from
Application/Activity, including interaction with Android 14+ foreground-service start
restrictions."

## Clarifications

### Session 2026-10-01

- Q: How long may it take between the user pressing play and the first audio actually coming out of the speaker? → A: 3 seconds, lenient
- Q: What text should the lock-screen notification display about the document being read? → A: The document title and the title of the section currently being read; no document body text. This answer supersedes both earlier answers on this question ("title only" and "title plus a teleprompter of the spoken text")
- Q: When the system kills the foreground service under memory pressure, what should the user see when they next open the app? → A: Show a paused state at the last spoken position with an explicit resume control; never auto-resume
- Q: What format and sample rate should the audio engine hand to playback, and who decides it? → A: Engine emits mono f32 already resampled to a fixed rate the shell declares; the shell only converts to the device format
- Q: What should happen to the playback position while another app holds audio focus transiently and the document is ducked? → A: Rewind 5% of the paused duration on resume, capped at 30 seconds, so a half-cut sentence is picked up without replaying far back. **Narrowed later in this session:** the 5% is how far the cursor moves, and the audio then restarts from the start of the sentence the cursor lands in; see the later entry on the rewind
- Q: When the user taps skip-forward or skip-backward from the lock screen, what should the reader jump to? → A: Fixed 5-second increments: +5s forward, -5s back, measured against the current playback position and independent of document structure
- Q: When the user presses Play from the lock screen while the background service is not running, how should playback be started given Android's rules about starting services from the background? → A: Keep a lightweight always-resident component so play can start immediately; if even that component has been killed, state the reason and require one app open
- Q: When the user changes the playback rate, should the audio keep its current pitch and voice character, or should a faster rate sound higher-pitched? → A: Preserve pitch and voice character; faster speech sounds quicker, not higher, because intelligibility under noise is the point of rate control in a car
- Q: When the app starts reading a document it has read before, should it begin from the very first spoken word, or from wherever the user last stopped in that same document? → A: Resume from the last spoken position unless the user pressed Stop; Pause, app close, service kill, phone shutdown, battery loss, and network loss all preserve state, and Stop discards it so the next open starts from the beginning
- Q: When a resume position is stored for a document, should the reader be told where they left off, or should the document simply start reading from that spot? → A: Show a 3-second resume offer with "continue" as the default choice and a visible alternative to start over; if no choice is made within 3 seconds, reading begins automatically from the stored position
- Q: After the 3-second countdown on the resume offer expires, should the app start reading aloud, or stop at the offer and wait for a tap? → A: Reading starts automatically, but only when reopening a document the user paused deliberately; after an unexpected end such as a system-killed service, recovery still stops at a paused state and never auto-plays
- Q: What uniquely identifies a document for the purposes of remembering where the reader stopped? → A: A fingerprint derived from the document's content itself, so that a stored position is still found after the file is moved or renamed
- Q: Where is the reason shown when playback cannot start or fails while the phone is locked? → A: A short reason on the lock-screen notification, with the full reason shown in the app when it is next opened
- Q: What happens to a stored position when the reader switches to a different voice or engine? → A: The position is kept but re-anchored to the nearest sentence or node boundary for the new engine, so reading resumes at the same point in the document rather than at the same elapsed second in a different narration
- Q: What diagnostic record must the shell keep so a failed or refused playback can be diagnosed afterwards? → A: A local record of session starts, transport changes, focus transitions, and failures, retained on the device and never transmitted anywhere
- Q: On which Android versions must this app work correctly? → A: Android 10 (API 29) and later. That is where foreground services became mandatory for background playback and where the media-playback foreground-service type exists, so the whole design rests on one model rather than two. Older releases are out of scope rather than degraded
- Q: While another app speaks for less than a second, how quiet should the ducked document get relative to its normal reading volume? → A: About 20% of normal volume. Quiet enough to sit under a navigation prompt in a cabin, still audible so the reader is not pushed to turn the volume up. Ducking is not pausing: a transient interruption ducks and keeps reading, because a pause on every "turn right" would chop the document mid-sentence for the whole commute
- Q: How should the playback rate be set and adjusted from the lock screen, and what values can the user actually reach? → A: A fixed ladder of discrete steps — 0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x — where each press advances or retreats one step. The current rate is shown inside the application but never on the lock screen, because FR-028 already limits that surface
- Q: A new output device becomes available while audio is already playing, such as car Bluetooth connecting mid-sentence — what should happen? → A: Follow the switch to the new device automatically, keeping the playback position, because connecting car audio is an act the driver took deliberately
- Q: The local diagnostic log records every session, transport change, focus change, and failure — how long must those entries be kept before being discarded? → A: 30 days, discarding oldest first, which keeps the evidence available long enough to investigate a problem the reader reports later while capping growth on a device nobody will clear
- Q: If the phone is killed or loses power mid-sentence, how much already-spoken audio may be heard again when the reader resumes? → A: At most 30 seconds of replay, so a hard kill that prevents any final write is still recoverable while the reader does not re-hear a long passage
- Q: While a document sits paused, the audio is kept ready ahead of the resume point — what is the largest amount the app may hold? → A: 60 seconds of pre-roll audio, replacing the previous unbounded "MUST be bounded in memory" wording with a number that SC-013 can actually assert
- Q: When another app takes audio focus permanently and the notification must say why the document stopped, what should that notification actually say? → A: State the pause and the reason — for example "Leitura pausada — outro app usando o áudio" alongside the document title. This supersedes the earlier wording "Retomando leitura de *&lt;título&gt;*", which promised an automatic resume that FR-014 forbids
- Q: Beyond the title and transport controls, what else may the lock-screen notification show? → A: A progress indication and the title of the section currently being read. A mini-teleprompter showing the spoken text was considered and rejected in favour of the section heading, which gives the same orientation without putting document prose on the lock screen or asking the reader to read while driving. No elapsed or remaining time, rate, or voice name
- Q: When the reader changes the playback rate, who applies the speed change to the audio? → A: The engine offers a pitch-preserving time-stretch operation that the Rust playback module calls; the shell does not stretch audio itself. This keeps FR-033 coherent, because the engine still delivers mono f32 at the one fixed declared rate and a rate change therefore never alters the delivered sample rate, and it makes FR-019a's pitch guarantee a property of the engine contract rather than of platform code
- Q: How much of this feature may live in the Android/Tauri surface itself? → A: Nothing beyond exposing the application to the platform. The shell's entire content is the Android and Tauri surface; every behaviour — transport, rate, position, buffering, focus response, and conversion to the device's sample format — is resolved by the Rust backend behind appropriate modules and submodules. This corrects the earlier wording of FR-034, which assigned that conversion to "the shell" as though the shell were a component with logic of its own
- Q: Which rate does a new session start at, and does a chosen rate carry over into the next session or the next document? → A: A new session starts at the last rate used anywhere in the app, or 1.0x on first ever use, and that rate is remembered across both sessions and documents until the reader changes it again. The rate is one app-wide setting rather than a per-document one, so reopening a document does not silently return the reader to a speed they did not choose. 1.0x is the starting rung because it is the only one meaning the voice as recorded
- Q: How is the 30-day diagnostic record actually read — is there a place in the app where the reader can see it, or is it only reachable by a developer? → A: Developer-only in this feature: a record written on the device and readable through tooling, with no in-app screen. An in-app screen showing the record is wanted next, and a reader-initiated export to share it is wanted after that; both are explicitly out of scope here and are named as deferred rather than left as an unstated gap. Writing the record for a screen nobody can open yet is deliberate — the record must exist before it is displayed, because a failure the reader reports afterwards cannot be diagnosed from events that were never kept
- Q: When a document is edited on disk while the reader is not looking, should that automatically affect the stored position for it? → A: A changed document is a different document. Because FR-036 keys the position by content, an edited file's fingerprint no longer matches, the stored position becomes unreachable, and no resume is offered. This is the consequence FR-036 already implies rather than a new rule: offering a resume at a position that no longer means anything in the file the reader now holds would be worse than starting over, and keeping the old position alive would reintroduce the location-keyed behaviour the fingerprint exists to prevent
- Q: What should happen when a document fails partway through speaking, and should the reader be offered a different voice or engine? → A: Present the named failure and offer the reader the choice, then fall back automatically if no choice is made within 3 seconds — the same bounded-offer shape as FR-032c, applied to engine recovery. The reader must not have to stop driving to diagnose or to keep listening. Choosing an alternative engine continues from the last spoken position rather than restarting the document
- Q: When another app takes audio focus, should every interruption be treated the same way, or should a call or alarm pause the document while a navigation prompt only makes it quieter? → A: Honour the two platform signals separately. Duck to ~20% only when the interrupting app permits ducking, such as a navigation prompt or a notification chime, and pause immediately for an interruption that demands silence, such as a phone call or an alarm, as well as for permanent loss. This supersedes the earlier FR-013 wording, which ducked every temporary interruption regardless of what the interrupting app had asked for: a document still audible at 20% under a phone call is unintelligible to the person on the call and socially wrong in a shared cabin, which is the context this feature exists for
- Q: While a document is playing, what should the progress indicator on the lock screen actually show, given that the document's total spoken length may not be known before it has been read? → A: Show a determinate bar or percentage once the document's total spoken length is actually known, and an indeterminate line until then. No estimated or guessed total may ever be shown, because a bar computed from a guess is a wrong number presented as a fact, and a wrong number on a glanceable surface is worse than an honest absence. FR-028 required a progress indication without saying which; this fixes what the indication is allowed to assert
- Q: After the reader has played something once and then stopped, what has to stay alive so that a later Play press on the lock screen still works, and what does the reader see while it does? → A: The playback service itself stays resident between sessions in a paused state, showing one ongoing notification. A second resident component was considered and rejected: it would mean a second lock-screen notification competing for the same glance and a second place for the two playback authorities this specification forbids from diverging. The reader therefore sees one persistent paused notification, which is the cost of making SC-014 unconditional, and that notification carries the paused state rather than a reason once the 3-second window of FR-041 has passed
- Q: To prove that engine-failure recovery works, how many speech engines have to be actually configured and working on the test phone? → A: Two engines — one offline and one cloud — so that a real failure and a real takeover are demonstrated on real hardware, with the rarer repeated-failure case produced by forcing the second engine to fail as well rather than by provisioning a third engine. Two is the smallest configuration in which a real engine stops and a real engine takes over; one engine would reduce SC-039 and SC-040 to checking that a failure was reported, and three would add credential and setup burden to evidence a forced failure already provides
- Q: When a document finishes playing while the phone is locked, should anything on the lock screen say so? → A: **Superseded by a later answer on the same question.** The finished-state notification described here is withdrawn in favour of tearing the resident service down at document end; see the later entry on service teardown at completion
- Q: When the document reaches its end, does the always-resident playback service stay alive in its finished state, or does it shut down and require the reader to open the application to start again? → A: The service shuts down at document end. The resident notification and its controls are torn down rather than left in a finished state, and starting the document again requires the reader to open the application. This supersedes the earlier finished-state answer, which required a notification whose play control produced no audio of its own — a control the reader would press and hear nothing. A completed document is not a resumable session: its stored position is already cleared by FR-032b, so keeping a surface alive to offer a restart keeps a resident service and a battery cost for a state that is equivalent to "nothing to play". The cost is that the reader cannot tell "the document ended" from "the notification went away for some other reason" by glancing at the lock screen, which the earlier answer accepted to avoid; that ambiguity is accepted instead, because the reader who wants another document has to open the application to choose one anyway
- Q: Who supplies the 60 seconds of pre-roll audio that FR-015a requires to be held ready while paused, given that a cloud engine is charged per character and would otherwise be spending the reader's quota while nobody is listening? → A: The pre-roll is delivered by the audio engine and the 60-second bound is a property of the pipeline, measured against audio actually handed to the playback module. This specification deliberately does not say whether the engine or the playback module drives the bound, because that is the handoff's business; what is fixed is that a bound exists and that pre-roll does not advance the position. Preparing audio that is never played stays inside the budget a reader was granted rather than consuming it silently
- Q: When is the section title on the lock screen re-read from the document, given that it must track playback, must follow headings that change on disk, and must be cleared when playback stops? → A: Re-resolved from the document currently being read on every playback state change that can alter which section the reader is in — crossing a section boundary, resume, a skip in either direction, and re-anchoring after an engine or voice change — with no polling of the file while nothing has changed
- Q: Where should the 30-day on-device diagnostic record be written, given that this feature requires it to exist and be readable through developer tooling but provides no in-app screen? → A: In the application's own private storage, trimmed oldest-first at the 30-day window with a size cap as a second bound. It needs no permission, is unreadable by other applications, and survives as long as the app is installed
- Q: When a document's opening paragraphs come before its first heading, does the notification show the most recent preceding section title as FR-028 states, or show no section title as an edge case and SC-030 state? → A: Show the most recent preceding section title when the current position comes after at least one heading, and show no section title when the current position comes before the first heading or the document has no headings at all. FR-028's "most recent preceding" fallback governs only the first case, and the two readings are reconciled rather than competing
- Q: Must the notification show the document title and section title on the lock screen, or must it yield to the phone's own "hide notification contents on lock screen" setting? → A: Show the content and never yield. The app declares its notification visible so that the title and section title appear on the locked screen, and adds no redaction, hide-on-lock, or in-app privacy mode of its own
- Q: When a document pauses because another app took audio focus permanently, is the full reason for that pause still shown in the app the next time it is opened, or does it expire along with the lock-screen statement after 3 seconds? → A: It expires everywhere after 3 seconds. FR-041's expiry governs a focus-loss pause reason on both surfaces, and FR-037's in-app persistence applies only to genuine failures — an engine failure, an audio-device failure, or a refused start — not to a focus loss. A focus loss is not a malfunction the reader cannot see the cause of, and the diagnostic record of FR-039 still carries the focus transition and its reason, so nothing needed for diagnosis is lost
- Q: While a document is paused, how long may the reader leave it there before the app does something — and if it does something, does it stay audible or stop? → A: No time limit and no automatic expiry, but no continuous pre-roll either. A paused document stays resumable for as long as the reader leaves it there and the resident service persists; pre-roll audio is prepared only after a Play press rather than continuously while idle, so a long pause spends none of the reader's synthesis budget on audio nobody will hear. Pre-roll resumes when the reader presses Play again. This narrows FR-015a, which as written required continuous preparation while paused
- Q: Success criterion SC-044 requires two real working speech engines on the phone, but FR-024 places synthesis in a different feature — so what happens to SC-044 while this feature is being built? → A: Split it. SC-044 stays the on-hardware evidence against two real engines and is re-verified once they are delivered; a new SC-051 is the mechanism criterion, verifiable in this feature against two stand-in engines that honour the same engine contract, so FR-023a is testable now rather than deferred whole
- Q: When an engine fails and FR-023a's 3-second window expires with no choice made, which engine does the system fall back to — in what declared order? → A: The constitution's registry tiers, as a closed declared sequence: the remaining engines of the current tier first, then escalate tiers in registry order, with the Edge evaluated fallback last and never a default. An engine the reader has not consented to is never selected by the fallback, so the unattended switch cannot move document content to a provider the reader did not authorise
- Q: If the lock-screen notification is dismissed while a document is playing, must the audio keep playing? → A: No — dismissal is blocked while audio plays, and pausing is the only route to dismissal, with a pause control on the surface for as long as audio plays. On pause, a transient surface carrying a stated reason is dismissed after 3 seconds, while the persistent resident surface merely becomes dismissable and is not dismissed automatically, so its resume control survives
- Q: What total size cap applies to the on-device diagnostic record, given that FR-039b requires one but states no number? → A: 10 MB, on the order of 100,000 entries, trimmed oldest-first. Sized by entry volume rather than entry bytes, and set at specification level for the same reason the 30-day window is there: a bound that no criterion can name is a bound any implementation may satisfy with any value. SC-025 now asserts the cap rather than only its existence

### Session 2026-10-02

- Q: When the audio-focus request is answered with something other than an immediate grant — a deferred grant, or a refusal — what should the reader see, and how is the one-second pause on losing the output device reached for a Bluetooth route as well as a wired one? → A: Both answers become named outcomes rather than silence. A deferred grant behaves as a granted focus that has not started yet: audio begins only once the grant actually arrives, and nothing is played before that. A refusal is a named failure under FR-037 and never a quiet session. The one-second pause on losing the output device is reached through two named mechanisms rather than one — `ACTION_AUDIO_BECOMING_NOISY` as the primary signal and `AudioDeviceCallback.onAudioDevicesRemoved` as the secondary signal — each held to the one-second target on its own, because a wired removal and a Bluetooth loss do not deliver the same signal and a bound held by a single signal is a bound the platform can decline to signal (FR-013b, FR-016a)
- Q: What sample rate does the shell declare for playback, and what happens when a device refuses it? → A: 22050 Hz, declared here as a decision of this specification rather than left to planning, because it is the native rate of the offline pt-BR voices the constitution's engine registry admits. The output stream requests exactly that rate and no other; a device that refuses it is a named failure under FR-035 — never playback at a substituted format and never a silent resample, so the rate a position is recorded against cannot differ from the rate it is heard at
- Q: What does the letter in a requirement identifier such as FR-019a mean, does a parent requirement stay in force once it has a derived one, and does every such derived requirement have to be traceable? → A: A letter marks a requirement derived from its parent, the letter being allocated in order of creation and implying no reading order; the parent remains in force and the derived requirement states its relation to the parent in prose rather than by renumbering it; and every lettered requirement is cited by at least one success criterion or cites one, so no derived requirement exists only as prose
- Q: Which sets of values in this feature are closed, and what has to happen when one of them has to grow? → A: The rate rungs, the audio-focus outcomes, the transport states, the engine identifiers, the error kinds, and the session-end reasons are governed closed sets. Engine identifiers are the single exception to being enumerated here: they live in the constitution's engine registry and are referenced by name and version, so that a second list cannot drift from the first. Extending any of these sets is a coordinated edit across every file that enumerates it, with a semantic-version bump and a documentation update — never a catch-all arm added to absorb whatever case was not anticipated. Two reconciliations are fixed rather than deferred: focus regained is an event of direction rather than a fourth outcome classification, and the session-end reason and the transport state are one closed set of four values shared with the record (FR-044)
- Q: How can it be shown that a session reported as playing was actually audible, given that the playback backend in use exposes no playback counters? → A: By a position rule over the diagnostic record rather than by a backend counter: a `transport: "playing"` entry conforms only if `position_secs` advances within the window between that entry and the next entry of the same session. Every transport change therefore carries `position_secs`, which is what makes the rule checkable at all. Adopting a backend that exposes real playback counters would make the evidence stronger and is recorded as a later adapter decision, not a requirement of this feature (FR-039, SC-005)
- Q: May the diagnostic record contain any text taken from the document? → A: No entry may contain document text at all. An entry's content is limited to event kinds, playback position, time, identifiers, and named failures, and a scanner verifies that zero entries carry prose. The record is kept on the device and read by tooling, so what is written to it determines what can leave it later, which is why the prohibition is on the entry rather than on the storage the entry lives in (FR-045)
- Q: The record's shape carries a free-text `detail` field, and requires a named error only for a `failure` — should both stand? → A: Neither stands as written. `detail` is either removed in favour of typed fields or retained under a bound of at most 200 characters and a pattern that admits identifiers and values rather than prose, and whichever of the two is chosen must be declared rather than left to whichever an implementation finds convenient, because an unbounded field is exactly where a document sentence would end up by accident. And `error` becomes required for a `refusal` as well as a `failure`, since both name a condition rather than merely marking that something occurred (FR-045)

### Session 2026-10-05

- Q: When a transient focus loss (ducked or silence-paused) is superseded by a permanent loss before any regain, what is the authoritative state, the rewind basis, the reason-window behavior, and the record shape? → A: Latest-event-wins (recommended option A). The newest loss fully supersedes: authoritative state is paused permanent loss with no auto-resume (FR-014 clears any transient resume/duck-restore flag and un-ducks gain, explicit Play required); rewind on eventual manual resume is cumulative from the first yield with the FR-015 formula unchanged; the FR-041 3-second reason window restarts at the permanent loss; the record carries two ordered entries (transient then permanent with supersede link), never a merged rewrite (FR-039, FR-044)
- Q: Which AudioAttributes, Builder flags, and API-level facts underwrite FR-013/FR-013b/FR-016a? → A: Spec Assumptions (recommended option A). Focus request assumes AUDIOFOCUS_GAIN with USAGE_MEDIA + CONTENT_TYPE_SPEECH, setWillPauseWhenDucked(true), setAcceptsDelayedFocusGain(true) plus OnAudioFocusChangeListener; duck depth 15–25% is the app-applied gain factor (system auto-duck depth not relied upon); output stream exact-requests 22050 Hz / PCM float / mono, never UNSPECIFIED, refusal is FR-035; removal signals are independent with each alone sufficient; all cited APIs exist below the API-29 floor (AudioFocusRequest 26, AAudio 26, setUsage/setContentType 28, device callback 23)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Keep reading with the phone locked (Priority: P1)

Luís locks the phone, puts it in the cup holder, and starts his commute. The document keeps
reading aloud. The audio never stops because the screen went dark, and he never has to
unlock the phone to keep listening.

**Why this priority**: This is the reason the project exists. Every other story is worthless
if playback dies when the screen turns off, because the primary context is a moving vehicle.

**Independent Test**: Start playback, lock the device, wait five minutes, and confirm audio
is still playing and the media controls remain reachable from the lock screen.

**Acceptance Scenarios**:

1. **Given** a document is reading aloud, **When** the device screen turns off or is
   manually locked, **Then** playback continues uninterrupted for the duration of the
   document or until the user pauses
2. **Given** playback continues with the screen off, **When** the user pulls the
   notification shade without unlocking, **Then** transport controls for play, pause, and
   skip are visible and functional
3. **Given** the device is locked mid-sentence, **When** the system suspends background
   work, **Then** no audible gap, restart, or repeated phrase occurs

---

### User Story 2 - Hand the phone back to the system (Priority: P1)

A navigation app announces a turn. The document should yield politely — get quieter rather
than stop dead — and come back when the announcement finishes. When the interruption is not
temporary, such as another app taking the audio stream for its own session, the document
should pause rather than fight for the speaker.

**Why this priority**: Routing interruptions are how real driving gets dangerous. A document
that talks over a turn-by-turn prompt teaches the user to turn the app off permanently.

**Independent Test**: Trigger a transient interruption and a permanent one separately, and
confirm the volume and transport-state response differs correctly in each case.

**Acceptance Scenarios**:

1. **Given** a document is reading aloud, **When** another app requests audio focus
   permitting continued audio at a reduced level (a navigation prompt, a notification chime),
   **Then** the document's volume reduces automatically and restores when focus returns
2. **Given** a document is reading aloud, **When** another app requests audio focus
   permanently, **Then** playback pauses and does not resume on its own
3. **Given** focus was lost transiently, **When** it is regained, **Then** playback resumes
   from the start of the sentence containing the point 5% of the paused duration back from
   where it yielded, capped at 30 seconds in total, so the listener hears the thought that was
   cut off rather than resuming mid-word
4. **Given** a document is reading aloud, **When** an interrupting app demands that other
   audio stop, such as a phone call or an alarm, **Then** playback pauses rather than ducks,
   and the same pause reason is stated as for a permanent loss (FR-013a)

---

### User Story 3 - Unplug headphones and do not disturb the cabin (Priority: P1)

Luís finishes a call, pulls the headphones out of his ear, and the document stops
speaking. He does not want a private document read aloud to whoever is in the car.

**Why this priority**: Unplugging headphones means the audio would move to the speaker.
That is a privacy failure and a social failure simultaneously, and it is the kind of thing
that happens constantly in a car.

**Independent Test**: Play through headphones, disconnect them, and confirm playback pauses
rather than continuing through the speaker.

**Acceptance Scenarios**:

1. **Given** audio is playing through headphones, **When** the headphones are disconnected,
   **Then** playback pauses within one second
2. **Given** headphones are reconnected after an automatic pause, **When** the user resumes
   from the lock screen, **Then** audio resumes through the reconnected device
3. **Given** audio is playing and a new output device becomes available, such as car
   Bluetooth connecting, **When** the device connects, **Then** audio continues on the new
   device from the unchanged position, with no pause, restart, or repeated phrase (FR-017a)

---

### User Story 4 - Stay in control from the lock screen (Priority: P2)

Luís wants to skip forward, slow down, jump back a few seconds, or stop for good and
come back to the same spot tomorrow. He does this from the lock screen notification. He does not unlock the phone, and he does not look away
from the road for longer than a single glance at a known control position.

**Why this priority**: Principle VIII makes this non-negotiable, but it builds on the
playback-and-focus foundation. Once audio survives locking and interruptions, exposing
controls is the natural next step.

**Independent Test**: With the device locked, exercise each transport control from the
notification and confirm the correct effect on playback.

**Acceptance Scenarios**:

1. **Given** a document is playing and the device is locked, **When** the user taps skip
   forward from the notification, **Then** playback advances five seconds and continues
   without interruption
2. **Given** a document is playing and the device is locked, **When** the user taps skip
   backward from the notification, **Then** playback returns five seconds and resumes from
   the resulting position
3. **Given** playback is paused and the device is locked, **When** the user taps play from
   the notification, **Then** playback resumes from where it stopped
4. **Given** a document was paused partway through, **When** the user reopens that same
   document later, **Then** a resume offer appears offering to continue from the last spoken
   position with a visible option to start over
5. **Given** the resume offer is showing and the user does nothing, **When** three seconds
   pass, **Then** reading begins automatically from the stored position
6. **Given** the resume offer is showing, **When** the user chooses to start over,
   **Then** reading begins from the first spoken position and the stored position is
   discarded
7. **Given** a document was paused partway through, **When** the user presses Stop, **Then**
   the stored position is discarded and reopening the document starts from its beginning
8. **Given** a document is playing and the device is locked, **When** the user adjusts the rate from the
   notification, **Then** the rate moves exactly one step along the declared ladder, the change
   takes effect within two seconds, the pitch is unchanged, the new rate persists for
   the rest of the session and into later sessions on other documents, and a document opened
   while a rate is already in effect begins at that rate rather than at 1.0x (FR-019b)
9. **Given** a document is playing and the notification is visible, **When** the user
   glances at the lock screen, **Then** the document title, the title of the section being
   read, and a progress indication are shown, and no body text from the document is shown
10. **Given** a document is playing and the device is locked, **When** the document reaches
    its end, **Then** the notification is dismissed and the reader is offered nothing to
    resume; starting that document again requires opening the application (FR-032f)
11. **Given** a document is playing, **When** the user glances at the lock screen or tries to
    swipe the notification away, **Then** the notification cannot be dismissed while audio is
    playing, a pause control is present to do that instead, and pressing pause keeps the
    resumable surface available (FR-028e)

---

### User Story 5 - Be told when the system cannot play audio at all (Priority: P3)

If the audio device cannot be opened, the app says so plainly instead of appearing to play
while producing silence.

**Why this priority**: A silent failure is the failure mode this whole project exists to
fix. It is last in build priority but first in honesty, and it is cheap once the rest
exists.

**Independent Test**: Force an audio-device failure and confirm the app surfaces a clear
message rather than reporting normal playback.

**Acceptance Scenarios**:

1. **Given** the audio device cannot be acquired, **When** the user presses play, **Then**
   the app reports the failure and does not display a playing state
2. **Given** playback stops unexpectedly mid-document, **When** the user returns to the app,
   **Then** it offers to resume from the last spoken position
3. **Given** the service was killed under memory pressure, **When** the user opens the
   app, **Then** the session is shown paused at the last spoken position and no audio
   plays until the user activates resume
4. **Given** the engine fails partway through a document, **When** the failure is presented,
   **Then** the named failure and the available alternative engines are offered, and either a
   choice made within 3 seconds or the expiry of that window leads to playback continuing from
   the last spoken position rather than restarting the document
5. **Given** an engine fails while the device is locked, **When** the 3-second offer expires
   without a choice, **Then** the fallback engine takes over without an unlock, and the
   notification has stated both that the failure occurred and that another engine continued

### Edge Cases

- An interrupting app demands that other audio stop — a phone call, an alarm, a voice message
  being played back — rather than merely permitting it at a reduced level. Playback MUST pause
  rather than duck, and the pause MUST carry the same stated reason as a permanent loss, so
  that a document is never still audible under a conversation the reader is having
  (FR-013a, FR-014, FR-041).
- An interruption arrives while the document is already ducked. The system MUST apply the
  outcome the new interruption asks for rather than compounding it with the existing duck, so
  that a call arriving during a navigation prompt pauses rather than ducking twice
  (FR-013, FR-013a).
- A new output device becomes available while a session is playing, such as car Bluetooth
  connecting or a wired headset being plugged in. Playback MUST follow the switch to the new
  device from the unchanged position, with no pause, restart, or repeated phrase, and MUST
  record the switch as an output-device event. This MUST NOT be treated as an interruption
  requiring a resume, and MUST NOT be permitted to move audio out of a private output to a
  loudspeaker, which remains a pause under FR-016 (FR-017a).
- Audio device becomes unavailable mid-playback (headset disconnected by the kernel rather
  than by the user, Bluetooth dropout). Playback must pause, not crash, and must not
  silently resume on the speaker. Whichever of the two removal mechanisms delivers the event,
  the pause is reached within the one-second bound, because neither signal is load-bearing
  alone (FR-016, FR-016a).
- The user presses play rapidly several times. Each press must be honoured or explicitly
  rejected; the app must not enter a state where it reports "playing" while producing no
  audio.
- Audio focus is lost and regained within the same sentence. Playback must resume at the
  rewind position defined in FR-015, which is what recovers the words the interruption
  covered, not the beginning of the sentence (FR-015).
- A pause lasts long enough for the 5% rewind to reach its 30-second cap. The rewind MUST
  saturate at the cap rather than continuing to grow, the sentence snap MUST NOT push the resume
  further back than the cap even where the sentence containing the cursor is longer than 30
  seconds, and a paused session left
  indefinitely MUST NOT accumulate pre-roll audio beyond the 60-second bound of FR-015a
  (FR-015, FR-015a). A paused session left indefinitely MUST also prepare no audio at all, so
  the buffer drains to empty on becoming idle rather than sitting at its 60-second bound, and
  no synthesis may be billed for a session the reader has not resumed (FR-015c).
- A paused document is resumed after a long delay — hours, or a day. The system MUST begin
  pre-roll on the press and MUST NOT synthesise anything while the reader was away, so the
  resume is prepared within the FR-027 startup bound rather than waiting on audio prepared
  during a pause that has long since ended (FR-015a, FR-015c, FR-027).
- A navigation prompt arrives in the same moment as a skip press from the notification.
  Both events are handled; neither is dropped, and the focus rule governs volume while the
  skip still applies to the position (FR-013, FR-018).
- The foreground service is killed by the system under memory pressure. Playback state must
  be recoverable on the next launch rather than reset to the document start, and must
  never auto-resume into audible speech (FR-030, FR-031).
- The device is locked and the user has granted no notification permission. Transport
  controls are then unavailable; the app must state this rather than leaving the user
  wondering why the controls are missing.
- The device is locked and audio focus is lost permanently while the screen is off. The app must pause and the
  paused state must be visible when the user next looks at the phone.
- The document's total spoken length is not knowable before playback, because it is synthesised
  as it is read. The progress indication MUST remain indeterminate for the whole session and
  MUST NOT adopt a projected total once a partial estimate exists, so that the surface never
  presents an estimate as an established fact (FR-028a).
- The document's total spoken length becomes known partway through a session — because a
  pre-computed duration arrives late, or because the synthesis boundary is finally passed. The
  indication MUST NOT switch from indeterminate to determinate mid-session, since a bar that
  changes its own denominator while the reader is looking at it asserts a total that was not
  available when the reading began (FR-028a).
- The document has no usable title (no metadata and no first heading). The notification
  MUST show a stable placeholder rather than falling back to speaking or displaying the
  first sentence, so that the no-body-text rule is never traded away to fill the gap.
- The document has no section title at or before the current position, because it begins
  before the first heading or carries no headings at all. The notification MUST show no
  section title rather than falling back to the body text around the current position
  (FR-028, FR-028c).
- The current position falls before the document's first heading while earlier content is
  absent, and the reader glances at the lock screen. The notification MUST show no section
  title rather than the heading ahead of the reader, because a title names the section being
  entered rather than the one being read, and both cases — this one and the preceding-title
  fallback — are settled by FR-028c rather than left to implementation preference.
- The document's headings change on disk while a session is active. The section title on the
  lock-screen surface MUST follow the document currently being read, and MUST NOT keep
  displaying the title of a section that no longer exists (FR-028, FR-028d).
- The document is edited on disk while no session is active, so a stored position exists for
  the content the reader last saw. The edited file MUST be treated as a different document and
  MUST begin from its first spoken position, with no resume offered and no partially matched
  position (FR-036a). Editing on disk during playback is a different case, governed by
  FR-032d, where the session recovers to a paused state rather than restarting.
- A skip press would carry the position past the end of the document, or back before its
  start. The position MUST clamp to the nearest document boundary rather than wrap around,
  skip into silence, or fail, and a forward skip landing at the end MUST settle into a
  paused end-of-document state (FR-018).
- The user presses Play from the lock screen after the always-resident component has been
  killed by the system and the platform refuses a cold foreground-service start. The press
  MUST surface the reason and MUST NOT present a playing state; audio begins only after the
  user opens the application, and the offered resume position is the last spoken position
  (FR-010, FR-030).
- The always-resident component is killed by the system while no session is active. This MUST
  NOT itself be treated as an interrupted session, because nothing was playing and nothing was
  lost; the next launch starts clean (FR-032).
- The reader stops the resident service from the notification, or the platform stops it, and
  presses Play from the lock screen afterwards. The press MUST be handled by the stated
  fallback of FR-010 — reason given, no playing state, playback begins after the user opens
  the application — and MUST NOT be presented as working while silently starting nothing
  (FR-009a, FR-010).
- The document ends while the service is resident. The resident component and its notification
  MUST be torn down, the stored position MUST have been cleared, and the completed document
  MUST NOT be offered for resume anywhere (FR-009a, FR-032b, FR-032e, FR-032f, SC-019). No
  surface is left showing a finished state with a play control, so on the lock screen a
  completed document looks the same as a notification that went away for any other reason; that
  ambiguity is a stated consequence of FR-032f rather than an unhandled case. Starting the
  document again requires an application open, and the platform refusing that start while
  locked MUST be handled as FR-010 requires — reason stated, no playing state presented —
  rather than leaving a control that silently begins nothing (FR-010, FR-032f).
- The document ends while the device is locked. The teardown MUST NOT be recorded as an
  interruption, a service kill, or a failure, and the next launch MUST NOT present a paused
  state or a resume offer for that document, because completion cleared its stored position
  deliberately rather than losing it (FR-032b, FR-032f, SC-019).
- A document ends while an engine-failure offer or a resume offer is still within its 3-second
  window. Completion and those offers are mutually exclusive states — a document that has
  reached its end is not one still awaiting a reader choice about how to continue — so the
  window MUST NOT outlive the session it belonged to, and teardown MUST NOT wait for a choice
  that can no longer apply (FR-023a, FR-032c, FR-032f).
- The device loses power entirely, or the battery dies, while a document is partway through.
  The stored position MUST survive this, so the document offers to resume on the next open
  rather than restarting from the beginning, and the resume MUST replay at most 30 seconds of
  already-spoken audio rather than requiring a write that a power loss cannot guarantee
  (FR-032a, FR-004).
- The network drops while a document is paused. This MUST NOT clear the stored position:
  a connectivity loss is not an intent to abandon the document (FR-032a, FR-026).
- The stored position was recorded under one voice and the reader resumes under another, so
  the stored offset lands between two words in the new narration. The system MUST re-anchor
  to the nearest sentence or node boundary rather than speak from the raw offset, and MUST
  state which boundary it resolved to in the application, so the reader is not silently given
  different text than the position implies (FR-038).
- Playback reaches the end of the document. The stored position MUST be cleared at that point,
  so the next open starts from the beginning rather than resuming at the final word
  (FR-032b).
- An engine delivers audio at a sample rate other than the declared playback rate. The
  system MUST surface a named engine failure rather than accepting the mismatch or resampling
  the stream outside the engine, because accepting it would put the two responsibilities —
  the engine's resampling and the playback module's sample-format conversion — on the wrong
  side of the contract (FR-033, FR-034, FR-035, FR-042).
- An engine fails mid-document and the fallback engine succeeds. Playback MUST continue from
  the last spoken position rather than restarting, and the transition MUST be recorded as an
  engine change so that a later position is re-anchored rather than replayed against the
  failed engine's timeline (FR-023a, FR-038).
- An engine fails mid-document and no alternative engine is available. The 3-second window
  still elapses, and expiry MUST leave the session in a paused state with a stated reason
  rather than attempting a fallback that does not exist. A window that expires into silence
  with no stated cause is the FR-005 failure, so the no-alternative case is reported rather
  than passed over in silence (FR-023a, FR-037).
- The reader selects an alternative engine from the 3-second offer and that engine also fails.
  The offer MUST be presented again for the remaining engines rather than cycling silently,
  and each attempt MUST be recorded, so that repeated failure is visible in the diagnostic
  record as several named failures rather than one (FR-023a, FR-039).
- The reader changes engine while playback is playing, rather than in response to a failure.
  Playback MUST continue from the position the change resolved to and MUST NOT restart, so
  that a voluntary engine change behaves like the automatic one in FR-023a (FR-038).
- An engine delivers stereo audio, or an unexpected sample width, where mono f32 was
  declared. This MUST also surface as a named engine failure. Silently downmixing or
  reinterpreting the buffer would make an adapter conform by accident rather than by
  contract (FR-033).
- The user presses Stop and then presses Play on the same document without leaving it. Play
  MUST begin from the first spoken position, because Stop discarded the stored position, and
  MUST NOT offer or perform a resume (FR-032).
- The resume offer is showing and the user chooses start-over during the 3-second window.
  Reading MUST begin from the first spoken position and the stored position MUST be
  discarded, so that a later reopen of the document starts clean rather than offering the
  position the user just rejected (FR-032c).
- The device is locked while the resume offer is showing, or the user switches away from the
  app during the window. The countdown MUST NOT leave audio playing unattended once the app
  is no longer in the foreground, so that a document never begins reading aloud into a pocket
  or a glovebox (FR-032c, FR-031).
- The rate is raised to its maximum while a session is playing, then lowered again. Pitch
  MUST remain unchanged across both transitions and across any pause in between, so that
  repeated adjustment does not accumulate an audible pitch drift (FR-019a).
- The rate is changed while audio is pre-rolled but not playing. The pre-roll buffer MUST be
  re-prepared at the new rate rather than played back at a mismatched one, and the position
  MUST NOT shift as a result of the change (FR-019a, FR-015a).
- The document is moved or renamed on the device between sessions. The stored position MUST
  still be found and offered, because the document is identified by its content and not by
  where it happens to sit on storage (FR-036).
- A playback failure occurs while the device is locked and the notification has been
  dismissed or cannot be shown. The reason MUST still be available in the application on the
  next open, and FR-037 MUST NOT depend on a lock-screen surface being present in order to be
  satisfied (FR-037).
- The device-wide "hide notification contents on the lock screen" preference is enabled while a
  document is playing. The application MUST NOT attempt to override it, surface a workaround
  that reveals the content some other way, or claim the content is visible when the platform
  has suppressed it, and a reader who has asked for private notifications MUST be obeyed. FR-028b
  forbids only the application's own redaction and its own hide-on-lock mode; it does not
  instruct the app to defeat a preference the reader set.
- The diagnostic record reaches its total size cap within the 30-day retention window, or a
  second application on the device attempts to read it. The record MUST be trimmed
  oldest-first with the trim visible as a trim rather than as an absence of events, and the
  record MUST NOT be placed anywhere a second application can read it (FR-039b, FR-039).
- The reader swipes the notification away while a document is playing. The surface MUST NOT be
  dismissable in that state, so the gesture leaves playback running with its controls intact
  rather than removing the only surface able to stop the audio; where a platform or OEM build
  permits the dismissal regardless, the document MUST pause rather than go on speaking with no
  control surface, and the stored position MUST be preserved either way
  (FR-028e, FR-009a).
- The reader presses pause on the persistent resident surface and then swipes it away, which is
  the intended way out of residency. Playback is already paused, so the gesture MUST leave the
  document resumable in the ordinary way, and the next Play press from a restored surface MUST
  resume it rather than presenting the cold-start refusal of FR-010 (FR-009a, FR-010,
  FR-028e).
- The device runs out of storage during a session and the diagnostic record cannot be written.
  Playback MUST NOT be interrupted by that failure — losing diagnostic entries is preferable
  to losing audio — and entries that were dropped MUST be visible as such in the record rather
  than leaving a gap that reads as "nothing happened" (FR-039). A small fixed reservation
  exists for gap and trim markers alone, so the marker for a write that found no room is
  deferred with its cutoff timestamp to the next successful write rather than lost with the
  entries it reports on.

## Requirements *(mandatory)*

### Functional Requirements

**Requirement identifier convention**: An identifier carrying a letter — FR-019a, FR-032b,
FR-016a — denotes a requirement **derived from** the requirement whose identifier has no
letter, the letter being allocated in order of creation and implying no reading order. The
parent requirement remains in force alongside its derived ones; the derived requirement states
its relation to the parent in prose rather than by renumbering or superseding it. Every
lettered requirement is cited by at least one success criterion or cites one, so that no
derived requirement exists only as prose.

**Audio output and authority**

- **FR-001**: The system MUST continue producing audible playback while the device screen
  is off and the device is locked
- **FR-002**: Playback MUST be driven from the Rust playback module (the player crate),
  and that module MUST be the single authority for playback position, buffering state, and
  transport state. Where this specification says "the engine" as the holder of playback
  authority, it means the Rust playback module; synthesis adapters are suppliers of PCM
  without playback state of their own
- **FR-003**: The system MUST NOT maintain a second independent source of truth for
  playback position; any media-control surface MUST reflect the Rust playback module's
  state rather than computing its own position
- **FR-004**: Playback position MUST survive process death and be recoverable to the last
  spoken position rather than the document start, replaying at most 30 seconds of
  already-spoken audio in the process. This bound is what makes the requirement true under a
  hard kill, where no final write can be relied upon: a reader who hears up to 30 seconds
  again is being rewound, while a reader who restarts a paragraph or a page is being sent
  back. A replay beyond the bound is a defect, and so is discarding the position entirely
- **FR-005**: The system MUST NOT report a playing state while producing no audible output
- **FR-042**: The Android and Tauri surface MUST contain no playback logic of its own. Every
  behaviour this feature specifies — transport, rate, position, buffering, the response to
  audio-focus events, and conversion into the output device's sample format — MUST be
  implemented in the Rust backend behind its own modules or submodules, and the platform
  surface MUST do nothing beyond exposing the application to Android. A behaviour reachable
  from the notification MUST reach the same code as the identical behaviour driven from
  elsewhere, because a surface that decides anything is a second place for the two authorities
  that FR-002 and FR-003 prohibit to diverge
- **FR-043**: The engine MUST expose the playback rate as a pitch-preserving time-stretch
  operation that the Rust playback module invokes, and the Rust playback module MUST NOT
  time-stretch audio by itself. A rate change MUST therefore never alter the sample rate
  the engine delivers, so the single fixed rate of FR-033 holds across every rung of the
  FR-019 ladder, and the pitch guarantee of FR-019a is a property of the engine contract
  rather than of platform code. Stretching the buffer in the platform layer instead would put
  a pitch-preserving algorithm on the wrong side of the boundary, where it could neither be
  tested without an engine nor be replaced when a better one becomes available

**Background execution and manifest**

- **FR-006**: Audio playback MUST run in a background service declared with the media
  playback foreground-service type, so the system treats it as ongoing user-visible work
- **FR-007**: The application manifest MUST declare that foreground service explicitly, and
  its presence MUST be verifiable by an automated check rather than by inspection
- **FR-008**: The system MUST request the foreground-service permissions required to keep
  media playback alive, and MUST degrade with a stated reason if they are absent. On Android 13
  (API 33) and later, where notification permission became a runtime grant, the system MUST
  treat that permission as required for the lock-screen surface and MUST state the consequence
  of its absence rather than proceeding as though the surface were available (FR-021)
- **FR-009**: The background service MUST start successfully whether the user launches from
  the application screen or from a transport control while the application is not running.
  To make a transport-control press viable under Android's background-start restrictions,
  the system MUST keep a lightweight always-resident component alive while playback has ever
  been started, so that the press is handled by a live process rather than requiring a cold
  foreground-service start from the background (FR-009a)
- **FR-009a**: The always-resident component MUST be the playback service itself, retained
  between sessions in a paused state, and MUST NOT be a second component beside it. Only one
  resident component and one ongoing notification may exist. Two resident components would
  put two lock-screen surfaces in competition for the same glance and would create a second
  place for playback state to be tracked, which is the divergence FR-002 and FR-003 prohibit.
  While resident and paused, the notification MUST show the paused state and the resume
  control, and MUST NOT assert an elapsed or remaining time or a section title, because
  playback is not at any position within the document while paused. The notification MAY
  persist indefinitely after playback has been paused; that persistence is the cost of
  making the lock-screen Play press of SC-014 unconditional, and the reader's way out of it
  is the resident-stop control defined here: a Stop action on the resident notification that
  ends residency while preserving the stored bookmark (distinct from the transport Stop of
  FR-032, which discards it). Swiping the paused resident surface away likewise ends residency
  and preserves the bookmark. Only the resident-stop control, a swipe-away while paused, or
  document end under FR-032f ends residency; a swipe-away while playing pauses first under the
  FR-028e fallback rather than leaving audio without a surface. Residency is conditional on there being something
  to resume: the component MUST be torn down when the document reaches its end, per FR-032f,
  because a completed document has no resumable position (FR-032)
- **FR-010**: The system MUST handle foreground-service start restrictions introduced in
  Android 14 and later, and MUST NOT silently fail to start when the platform refuses the
  start. The refusal signal is `ForegroundServiceStartNotAllowedException` (API 34+;
  `BackgroundStartNotAllowedException` on API 31–33), and catching it MUST take the
  stated-reason path below rather than retrying silently. Support begins at Android 10 (API 29), so this requirement concerns the Android 14
  behaviour specifically rather than a range of older start rules. When the platform refuses, or when the always-resident component has itself been
  killed and no cold start is permitted, the system MUST state the reason and MUST NOT
  present a playing state; playback then begins only after the user opens the application.
  Silent refusal is prohibited because a control that appears to work but starts nothing is
  indistinguishable from the silent failure this feature exists to prevent (FR-005)
- **FR-011**: Every media transport control MUST remain functional when the background
  service was started from outside the application process

**Audio focus and interruptions**

- **FR-012**: The system MUST yield to audio focus requests from other applications
- **FR-013**: When audio focus is lost temporarily and the interrupting app permits other
  audio to continue at a reduced level, the system MUST reduce playback volume to
  15–25% of normal gain rather than stop, and MUST restore full volume when focus
  returns. The percentage is measured on the gain factor applied by the application,
  not on acoustic output, and binds unconditionally: where the device's own minimum volume
  prevents the 15–25% band from being observable, that device-volume edge is recorded as a
  best-effort edge case and the gain-factor requirement still holds. Ducking is distinct from pausing: a permitted interruption ducks and keeps
  reading, because pausing on every navigation prompt would cut the document mid-sentence
  for the length of a commute (FR-013a)
- **FR-013a**: The system MUST classify a focus loss by what the interrupting app has asked
  for rather than treating every interruption alike, and MUST produce different outcomes
  accordingly. Where the interrupting app permits continued audio at a reduced level, such
  as a navigation prompt or a notification chime, the outcome is ducking under FR-013.
  Where the interrupting app demands that other audio stop, such as a phone call or an
  alarm, the outcome is a pause, with the same consequences as a permanent loss under
  FR-014 and the same stated reason requirement under FR-041. Classifying by the
  interrupting app's stated request rather than by the system's own reading of the
  interruption is what keeps a document from remaining audible under a phone call, which
  would be unintelligible to the person on the call and socially wrong in the shared cabin
  this feature exists for, while leaving the navigation prompt ducked
- **FR-013b**: An audio-focus request MUST be resolved against a closed set of platform
  outcomes, and each outcome MUST be handled explicitly. Where focus is granted immediately,
  FR-013, FR-013a, and FR-014 govern as written. Where focus is granted but **deferred** — the
  request was accepted and the audio has not been handed over yet — the system MUST behave as
  holding a granted focus that has not begun: it MUST NOT produce audio before the grant
  arrives, and because a deferral is a delay rather than a loss, no rewind under FR-015 is
  owed on the eventual grant and no pause reason under FR-041 is shown. Where the request is
  **refused** outright, the refusal MUST surface as a named failure with a stated reason under
  FR-037, MUST NOT degrade to silence, and MUST NOT present a playing state
  (FR-005, FR-037). The set is closed because a focus answer the system carries no rule for
  ends in one of exactly two ways the reader experiences badly — a document fighting for a
  speaker it was refused, or a document going quiet without saying why — and both are failures
  this feature exists to prevent (FR-012, FR-013a)
- **FR-014**: On permanent focus loss, the system MUST pause and MUST NOT resume
  automatically. Where a transient loss (ducked under FR-013 or silence-paused under FR-013a) is superseded by a permanent loss before any regain, the newest loss wins in full: the system MUST clear any transient resume-on-gain or duck-restore flag, restore full gain, and require an explicit Play press; a late gain attributable to the superseded transient MUST NOT authorize resume (FR-014, FR-041)
- **FR-015**: After a transient loss and recovery, playback MUST resume from a rewind of 5% of
  the paused duration, capped at 30 seconds, measured back from the position at which playback
  yielded. That proportional figure is where the reading cursor moves back to; the position the
  audio actually resumes from is the start of the sentence containing that cursor position, so
  the resume lands on a sentence boundary rather than part-way through a word. Total backward
  movement from the yield point MUST NOT exceed 30 seconds under any pause length: where the
  sentence containing the cursor begins earlier than that, the resume lands at the point 30
  seconds before the yield instead. Where that cap point falls part-way through a word, it snaps forward to the start of the nearest following inter-word gap, except where that gap lies beyond the 30-second cap point, in which case the resume lands exactly at the cap point; the 30-second cap is a hard ceiling that no snap may exceed, so the cap and the zero-mid-word guarantee hold together with the cap taking precedence. Both parts are load-bearing and neither replaces the other.
  The proportional term scales how much context is replayed with the length of the interruption,
  so a short navigation duck replays little and a long interruption replays more; the sentence
  snap is what makes that replay intelligible rather than starting inside a word, which is the
  same defect FR-038 prohibits across a voice change; and the cap bounds the result, so neither a
  long pause nor a sentence longer than the cap can send the reader further back than 30 seconds
  (FR-015a, FR-015c). Where the active engine declares no sentence or node boundary granularity,
  the snap degrades to the raw cursor position — declared and observable, never to the document
  start — because a parser-estimated sentence start has no audio timing behind it. Verification
  uses synthetic audio with known inter-word gaps: the resume MUST land in a gap, never in an
  energy peak. Where a transient loss is superseded by a permanent loss before any regain, the rewind clock MUST NOT reset at the supersede point: paused duration is the continuous non-advancing interval from the first yield to the eventual manual resume, and the 5%-capped-at-30s plus sentence-snap plus hard-cap rule applies unchanged against that cumulative duration
- **FR-015a**: Once the reader has acted to hear audio — a Play or resume press, or playback
  beginning — the system MUST continue preparing audio ahead of the playback point without
  playing it. This pre-roll buffering MUST NOT
  advance the playback position, MUST hold no more than 60 seconds of audio at any moment, and
  MUST NOT grow while a session remains paused, so that an indefinitely long pause cannot
  exhaust device memory, and MUST be safely abandoned when the session ends.
  Pre-roll exists so that resuming does not stall on synthesis
- **FR-015c**: While a document sits paused and the reader has taken no action, the system MUST
  NOT prepare audio, and MUST NOT spend synthesis budget, until a Play or resume press
  arrives. On that press, pre-roll MUST begin as FR-015a requires. Preparing audio
  continuously for an idle session would bill a per-character engine for minutes the reader
  will never hear, and a document left paused for a day would otherwise bill a day; suspending
  preparation while idle is invisible to the reader, because the audio was not audible in either
  case, and it removes a cost that would otherwise be incurred without a request. This
  requirement narrows FR-015a, which as written required continuous preparation from the moment
  of the pause: pre-roll follows the reader's intent to listen, not the existence of a paused
  session (FR-015, FR-015b)
- **FR-015b**: Pre-roll audio MUST be delivered by the audio engine, and the 60-second bound
  of FR-015a MUST be a property of the pipeline measured against audio actually handed to the
  playback module. This specification does not fix which side of the handoff drives the bound,
  because that is the engine's and the playback module's arrangement to make and FR-024 places
  synthesis in another feature. What is fixed here is that preparing audio that is never played
  MUST remain inside the synthesis budget the reader was granted: an engine billed per character
  must not spend a reader's free quota on minutes that will be discarded, so pre-roll is bounded
  rather than open-ended even where nothing is playing, and FR-015c stops it entirely while an
  idle session awaits the reader's next press
- **FR-016**: The system MUST pause playback within one second when headphones, a wired
  headset, or a connected Bluetooth A2DP device are disconnected, so audio does not move to
  the loudspeaker. A Bluetooth disconnect carries exactly the same consequence as a wired
  removal: pause with the stored position preserved. Where a disappearance and an appearance
  coincide in one route change — unplugging wired while car Bluetooth connects — the
  disappearance takes precedence: the system MUST pause first, then re-evaluate the newly
  available device under FR-017a before resuming, so audio never moves to a device the
  reader did not choose. The one-second pause is reached through two named mechanisms rather
  than one, as FR-016a specifies, because a wired removal and a Bluetooth loss do not deliver
  the same signal
- **FR-016a**: The one-second pause of FR-016 MUST be reached through two named mechanisms
  rather than one. `ACTION_AUDIO_BECOMING_NOISY` is the **primary** signal and MUST be handled
  on every device removal it covers; `AudioDeviceCallback.onAudioDevicesRemoved` is the
  **secondary** signal and MUST be handled independently of the first, so that a Bluetooth
  route reporting the removal only through the device callback still pauses. Each mechanism
  MUST reach the pause within one second of the removal being observable, and each MUST be
  sufficient on its own: neither waits for the other, and no removal may pass unhandled
  because the platform happened to deliver only one of them. The two mechanisms exist because
  the one-second bound is the whole privacy guarantee of this feature — it is what keeps a
  private document from moving to a loudspeaker — and a guarantee held by a single signal is a
  guarantee the platform may decline to signal at all (FR-016, FR-039, SC-002)
- **FR-017**: The system MUST resume through a reconnected output device when the user
  explicitly resumes
- **FR-017a**: When a new output device becomes available while audio is already playing, the
  system MUST follow the switch to that device automatically and continue from the unchanged
  position, without a pause, a restart, or a repeated phrase. This is the mirror image of
  FR-016: a device *appearing* is an action the driver took deliberately — plugging in a
  headset, connecting car Bluetooth — and interrupting the document to ask would be the wrong
  response to an intentional act. The direction of travel matters, and the privacy intent of
  FR-016 is unaffected: a switch *out of* a private output to a public one still pauses, so
  following a newly available device never permits audio to move to a loudspeaker the reader
  did not choose. Private outputs are wired headsets, wired headphones, and Bluetooth
  headsets (A2DP/SCO worn by the reader); public outputs are the built-in loudspeaker and
  car Bluetooth in shared-cabin use. Privacy takes precedence over convenience: any route
  change that would move audio out of a private output pauses first under FR-016 and resumes
  only by explicit user action, even where a public device newly appears. The switch MUST be recorded as an output-device event so that a route change
  is distinguishable from a stall or a failure afterwards (FR-039)
- **FR-041**: A pause caused by permanent focus loss while the device is locked MUST be
  reported on the lock-screen or notification-shade surface, because a document that stops
  without saying so is the silent failure FR-005 prohibits. The surface MUST show a short
  scrolling statement of the form "Leitura pausada — outro app usando o áudio", rendered as
  moving text so it is noticeable without being read (FR-028). The statement MUST state that
  reading is paused rather than resuming, because the platform has taken audio focus and
  FR-014 forbids automatic resumption, so a statement promising a resume would describe a
  transition the system will not make. The statement MUST name
  the document title and MUST NOT contain any other text drawn from the document (FR-037).
  The application MUST additionally show this reason on its own surface, but only if the user
  opens the application within 3 seconds of the pause; after that window the reason has aged
  out and the application MUST NOT show it, because a screen the user last looked at some time
  ago is stale and a message on it explains nothing. This expiry governs both surfaces: a
  focus-loss pause reason does not persist in the application the way a failure reason does
  under FR-037, and FR-037a states that boundary explicitly rather than leaving the two
  requirements free to contradict each other. Paused state itself remains visible after
  that window (FR-030); only the reason expires. Where a transient loss is superseded by a permanent loss, the 3-second window MUST restart at the permanent-loss callback and the authoritative reason is the permanent-loss statement on both surfaces; a stale transient reason MUST NOT linger

**Transport controls from the lock screen**

- **FR-018**: The system MUST expose play, pause, resume, skip-forward, skip-backward, and
  Stop from the lock screen or notification shade without requiring the device to be
  unlocked. Pause and Stop MUST be distinct controls with distinct consequences for stored
  state, and the surface MUST NOT present one as the other (FR-032, FR-032a).
  Skip MUST move the playback position by exactly 5 seconds — forward for skip-forward,
  backward for skip-backward — measured against the position at the moment the control was
  pressed, and MUST NOT depend on document structure, so that the same press produces the
  same displacement regardless of headings, paragraphing, or engine
- **FR-018a**: No control reachable while driving — play, pause, resume, skip in either
  direction, rate change, voice change — MUST demand looking at the screen. Every one of
  them MUST be operable locked, by feel, with its effect confirmed audibly rather than
  visually, because a control that needs eyes is a control that cannot be used where this
  feature exists to be used (FR-018, FR-019)
- **FR-019**: Rate adjustment MUST be reachable without unlocking the device. The reachable
  rates are a fixed, closed ladder: 0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, and 4.0x. Each rate
  control press MUST advance or retreat exactly one step on that ladder, and a press at either
  end MUST saturate (stay at that end) rather than wrapping, failing, or yielding a rate
  outside the ladder. The ladder is closed
  because a rate the reader cannot anticipate is a control they cannot use without looking
  (FR-028)
- **FR-019a**: Changing the playback rate MUST preserve pitch and voice character. Faster
  playback MUST sound quicker rather than higher, and MUST NOT be implemented by resampling
  the audio without correction. A rate change that is audible as a pitch change is a defect,
  because intelligibility under cabin noise degrades as speed rises, which would make the
  control counterproductive in the environment this feature exists for. The pitch guarantee
  applies to every rung of the FR-019 ladder, including the largest step between 2.0x and 4.0x
- **FR-019b**: A new session MUST begin at the last rate the reader used anywhere in the
  application, or at 1.0x if no rate has ever been chosen, and that rate MUST be remembered
  across sessions and across documents until the reader changes it again. The rate is one
  app-wide setting rather than a per-document one: a reader who slowed the document down
  because the cabin was noisy is not silently returned to full speed by reopening a
  different file, and per-document rate memory is not in scope. 1.0x is the opening rung
  because it is the only one that means the voice as recorded. A rate chosen before the
  application was last closed MUST be the rate a new session opens with (FR-019)
- **FR-019c**: A rate change MUST take audible effect within 2 seconds of the control press,
  measured under the same preconditions as FR-027 (mid-range arm64 device, document already
  parsed). A slower change is treated as a failure of the control, not as normal behaviour,
  because a rate press made while driving that answers late is a press the reader cannot
  confirm without looking (FR-019)
- **FR-020**: Transport controls MUST reflect actual playback state at all times, and MUST
  NOT display a playing state while playback is paused
- **FR-021**: When the notification permission is denied, the system MUST state that
  lock-screen controls are unavailable rather than omitting them silently. That statement is
  made inside the application, because the denied permission is precisely what removes the
  surface it would otherwise appear on (FR-037)

**Failure surfacing**

- **FR-022**: When the audio output device cannot be acquired, the system MUST surface a
  clear failure to the user and MUST NOT continue reporting playback
- **FR-023**: Any engine that cannot be reached MUST surface as a named engine failure and
  MUST NOT degrade to silence
- **FR-023a**: When an engine fails mid-document, the system MUST present the named failure and
  MUST offer the reader the available alternative engines, and MUST fall back to the next engine
  in the declared order if the reader makes no choice within a 3-second window. A choice made
  within that window MUST be honoured instead of the fallback, and either path MUST continue
  from the last spoken position rather than restarting the document. This is the bounded-offer
  shape of FR-032c applied to engine recovery rather than to reopening a document: the reader
  is driving, so requiring a decision before the audio continues would mean stopping the
  vehicle, and requiring no decision at all would mean a failed document simply stops. The
  window is what makes both avoidable, and the fallback being automatic rather than silent is
  what keeps FR-023 satisfied — the reader is told which engine failed and that another has
  taken over, rather than hearing one voice stop and a different one start unexplained (FR-005,
  FR-037)
- **FR-023c**: The engine fallback order MUST be a closed, declared sequence derived from the
  constitution's engine registry tiers: the remaining engines of the failing engine's own tier
  are tried first, the higher tiers follow in registry order, and the evaluated Edge fallback is
  last and is never a default. Descent to a lower tier is permitted, but only behind a fresh
  3-second offer in the shape of FR-023a naming the lower-tier engines, so a descent is always
  a choice the reader could have refused rather than a silent demotion. An engine the reader has not consented to MUST NOT be selected by
  the fallback, so an escalation can never move document content to a provider the reader did
  not authorise: consent state lives in application-private storage and is read on every
  fallback attempt, and the automatic fallback is bounded by consent exactly as any other engine
  selection is, and the offer of FR-023a MUST name the engines it may switch to, so the order
  is visible before it is used rather than discovered afterwards. A closed order rather than a
  walk over whatever happens to be configured is what keeps the unattended switch auditable,
  because every failure then resolves to a path the reader could have been told about in
  advance rather than to whichever engine happened to be installed next on the device
  (FR-023a, FR-023b)
- Q: After a transient interruption, should the resume rewind land on a sentence boundary rather than at 5% of the paused duration? → A: Both, in that order. The 5% of the paused duration is how far back the reading cursor moves, capped at 30 seconds; the audio then restarts from the beginning of the sentence the cursor now sits in. Total backward movement never exceeds 30 seconds whatever the pause length
- **FR-023b**: Verification of engine recovery MUST be performed on a device carrying at least
  two working engines, one offline and one cloud, so that a real engine stops and a real engine
  takes over on real hardware rather than only against a stub. A single-engine device is not an
  acceptable substitute, because with one engine there is nothing to fall back to and the
  recovery behaviour reduces to reporting a failure. A third engine is not required: the case
  where every engine fails is exercised by forcing the second engine to fail as well, which
  produces the same observable evidence as a third engine at lower setup cost. The choice of
  which two engines is not fixed by this specification and comes from the declared engine
  order (FR-023a). Because FR-024 places synthesis in another feature, this requirement is
  discharged in two parts rather than one: the recovery mechanism MUST be verifiable within
  this feature against two stand-in engines that honour the same engine contract,
  and the on-hardware evidence against two real working engines MUST be re-verified once those
  engines are delivered. That is an ordering property, not a relaxation — the single-engine
  objection above still applies to the hardware evidence, while the double pair is what keeps
  FR-023a from depending on another feature's schedule to be tested at all (FR-023a, SC-044,
  SC-051)
- **FR-037**: A failure reason that arises while the device is locked MUST appear as a short
  statement on the lock-screen or notification-shade surface, without requiring an unlock, and
  the full reason MUST remain available in the application when it is next opened. A failure
  reason MUST NOT include text drawn from the document, because it is written to a surface
  that FR-028 limits what document-derived content that surface may show, because a failure
  reason is app-generated text and has no reason to quote the document. Reporting only
  "no playing state" while giving no reason anywhere the user can
  reach is prohibited, because a control that explains nothing is indistinguishable from a
  broken one (FR-010, FR-021, FR-022)
- **FR-037a**: The persistence FR-037 requires applies to a failure reason only — an engine
  failure, an audio-device failure, or a refused start. A pause reason arising from audio-focus
  loss is NOT a failure and MUST NOT persist in the application: it expires under FR-041 on
  both surfaces, appearing only if the application is opened within 3 seconds of the pause.
  Scoping persistence this way keeps a permanent focus loss from accumulating indefinitely in
  the application, because a focus loss is a state the reader can see the cause of and that the
  platform itself reports, whereas a failure is an app-level fault whose cause is invisible to
  the reader and must therefore survive until it is read. A focus transition and its reason
  remain recorded in the diagnostic record of FR-039, so a lost focus is still diagnosable
  afterwards even though its reason is no longer displayed. This settles what FR-037 and FR-041
  each govern: FR-037 owns failure reasons and their persistence, FR-041 owns focus-loss pause
  reasons and their expiry, and the two sets of reasons are disjoint (FR-013a, FR-041)

**Scope boundaries for this feature**

- **FR-024**: This feature covers the audio playback shell only. Document parsing, text
  segmentation, speech synthesis, and voice selection are separate features, and this
  feature MUST consume them through a defined interface without implementing them
- **FR-025**: Android Auto integration is out of scope for this feature and MUST NOT be
  treated as a requirement. This is a deliberate deferral rather than an omission: a
  document reader has no media-browsing category under Auto, so integration waits for a
  later feature rather than being overlooked here
- **FR-026**: The system MUST NOT depend on network connectivity to start or continue
  playback

**Playback start latency**

- **FR-027**: The system MUST begin producing audible audio within 3 seconds of the user's
  play action, measured on the reference device defined in plan.md §Target Platform with the document already parsed. A
  longer silence is treated as a failure to start, not as normal startup

**Notification content and privacy**

- **FR-028**: The lock-screen and notification-shade surfaces MUST display the document title,
  the transport controls, a progress indication, and the title of the section currently being
  read. No document body text MUST be displayed on those surfaces: neither the sentence in
  progress, nor any preceding or following passage. The section title MUST track playback,
  advancing as playback crosses a section boundary, and MUST clear when playback stops.
  Section titles are structural metadata rather than document prose, so showing the current one
  gives the reader orientation without putting the document's own words on a screen that may be
  visible to a passenger. Where the current position falls after at least one heading, the
  surface MUST show the most recent preceding section title; where it falls before the first
  heading, or the document carries no headings at all, the surface MUST show no section title.
  In neither case may it fall back to body text. Elapsed time, remaining time, rate, and voice
  name MUST NOT be shown on these
  surfaces, because a total duration may not be knowable before a streamed document has been
  synthesised and a wrong number is worse than none (FR-028a, FR-028c)
- **FR-028b**: The lock-screen surface MUST declare its notification as visible, so that the
  document title and section title FR-028 requires are actually shown while the device is
  locked, and the application MUST NOT redact, hide, or substitute that content when the
  device is locked. An in-application privacy setting that hides document-derived content from
  the lock screen MUST NOT be provided in this feature. Two forces meet on that surface and the
  specification resolves them in favour of visibility: Principle VIII makes the glanceable
  lock-screen surface the reason a driving reader never touches the phone, and the privacy this
  feature protects against is a private document read aloud to a passenger — which is answered
  by pausing rather than by speaker (FR-016), not by blanking a title the reader needs in order
  to know which document stopped. A phone-wide "hide contents" preference remains the user's to
  set and the system MUST NOT attempt to override it; what this requirement fixes is that the
  app itself neither redacts nor offers a mode in which it would
- **FR-028a**: The progress indication MUST be determinate only when the document's total
  spoken length is actually known, and MUST be indeterminate otherwise. An estimated,
  projected, or guessed total MUST NOT be displayed as though it were established, and the
  indication MUST NOT switch between determinate and indeterminate within a single session,
  because a bar whose denominator is a guess is a wrong number presented as a fact, and a
  glanceable surface that changes its own denominator mid-playback is worse than one that
  honestly declines to assert a total. Where the total is never knowable — a document
  synthesised as it is played — the indication remains indeterminate for the whole session
  rather than inventing a denominator to satisfy the appearance of one
- **FR-028c**: The section title shown on the lock-screen surface MUST be the most recent
  heading the playback position has already passed. Where the position is before the document's
  first heading, or the document contains no headings, no section title MUST be shown. FR-028
  states the preceding-title fallback and an earlier edge case stated an absolute absence; both
  are correct over their own case, and this requirement separates them so an implementation does
  not have to choose between them: a title for a section the reader has not entered yet asserts
  where they are going rather than where they are, and no title at all is honest about the
  opening of a document (FR-028)
- **FR-028d**: The section title on the lock-screen surface MUST be re-resolved from the
  document currently being read on every playback state change capable of altering which section
  the reader is in: crossing a section boundary during playback, resuming a session, a skip in
  either direction, and re-anchoring after an engine or voice change. On every resume the
  stored content fingerprint MUST additionally be checked, so a document edited on disk
  re-resolves against the current headings rather than the remembered ones. The document MUST NOT be
  polled for heading changes while no such event has occurred, and no file watcher is kept:
  between two such events a stale title is tolerated and declared rather than silently
  refreshed. Re-resolving on the events that
  can change the answer is what keeps the surface truthful without a watcher, and a title left
  stale across a five-second skip would be a document-derived string naming a section the reader
  has already left, on the one surface they glance at while driving (FR-028, FR-028c)
- **FR-028e**: While a document is playing, the lock-screen surface MUST NOT be dismissable, and
  pausing MUST be the only route by which it can be dismissed; the platform mechanism is the
  ongoing flag (`setOngoing(true)`), and a pause control MUST be present on
  that surface for as long as audio is playing. Where a platform or OEM build permits
  dismissal regardless of the flag, the Edge Case pause-without-surface rule governs rather
  than continued audio with no controls. What happens to the surface on pause depends on
  which kind it is. A transient surface carrying a stated reason — a failure reason under FR-037
  or a focus-loss pause reason under FR-041 — MUST be dismissed 3 seconds after the pause. The
  persistent resident surface MUST instead become dismissable and MUST NOT be dismissed
  automatically, so that its paused state and resume control remain available until the reader
  chooses to swipe it away, stops the resident component, or the document ends under FR-032f.
  Blocking dismissal while audio plays keeps the reader's controls and the audio they govern on
  the same surface: a dismissable notification that vanished while the document was still speaking
  would leave the reader unable to stop what is being read aloud, which is the driving hazard
  Principle VIII exists to prevent. Splitting the two kinds on pause is what lets the transient
  reason surface expire as FR-041 requires while the resident surface keeps the lock-screen Play
  press of FR-010 and SC-014 working, since a resident surface that dismissed itself on every
  pause would restore the cold-start problem FR-009a was written to solve
  (FR-009a, FR-010, FR-028, FR-032f, FR-037, FR-041)
- **FR-029**: The title surfaced in the notification MUST come from the document
  metadata rather than from the document's first spoken sentence, and MUST be
  re-sourced on engine change without being recomputed from spoken text. Where the
  document carries no metadata, the surface MUST show the file name as the stable
  placeholder, independent of whether the document has headings; the first spoken
  sentence MUST NOT be used, and no other document text may substitute

**Session recovery**

- **FR-030**: When a previous session ends unexpectedly because the service was
  killed, the next launch MUST present a paused state positioned at the last spoken
  position, with an explicit resume control
- **FR-031**: Following an unexpected session end, the system MUST NOT begin producing audio
  on its own. Recovery to a paused state is the only permitted automatic transition, because
  unsolicited audio on launch is unacceptable while driving. The sole exception is the
  deliberate-pause reopen path defined in FR-032c, which is bounded by a visible 3-second
  offer carrying a working alternative; every other recovery path remains silent
  (FR-032d)
- **FR-032**: A session the user ended with Stop MUST NOT be presented as an interrupted
  session on next launch, and MUST NOT be resumable. Stop discards the stored position for
  that document, so a subsequent open of the same document starts from its first spoken
  position. Stop is the only action that discards stored state
- **FR-032a**: A position MUST be preserved across every other way a session can end,
  including Pause, closing the application, the system killing the service, the device being
  powered off, the battery dying, and the network dropping. Opening a document that was left
  this way MUST offer to continue from the last spoken position rather than restarting it,
  because pausing a document and returning to it later is the ordinary way a reader picks a
  book back up, not an interruption
- **FR-032c**: When a document left by a deliberate Pause is reopened, the system MUST show a
  resume offer offering to continue from the stored position with a visible alternative to
  start from the beginning. The offer MUST remain visible for 3 seconds. If the user makes no
  choice within that window, reading MUST begin automatically from the stored position,
  provided the application is in the foreground when the window expires; if it has left the
  foreground, the offer instead converts to the explicit-resume form of FR-032d and MUST NOT
  begin audio on its own, so that a document never begins reading aloud unattended. Choosing
  "start over" MUST begin from the first spoken position instead. The 3-second
  window is a deliberate departure from FR-031 and applies to deliberate pauses only; the
  prohibition on unsolicited audio in FR-031 continues to govern every other path without
  exception
- **FR-032d**: The automatic start permitted by FR-032c MUST NOT apply to a document left by
  an unexpected end, a device power-off, a battery death, or a network loss. Those cases
  recover to a paused state with an explicit resume control and never produce audio on their
  own, because after an interruption the stored position may be stale — the file may have
  changed on disk, or the engine may differ — and silently starting speech from a position the
  user did not confirm is the failure FR-031 exists to prevent
- **FR-032b**: Reaching the end of a document is equivalent to Stop for storage purposes:
  the stored position MUST be cleared so that reopening the document starts from the
  beginning rather than offering to resume at the final word
- **FR-032e**: When a document reaches its end, the system MUST NOT advance to any other
  document and MUST NOT offer to resume the completed document, because FR-032b has already
  cleared its stored position and a completed document is equivalent to a stopped one
  (FR-032). The lock-screen and notification-shade surface for that document is governed by
  FR-032f rather than by a finished state of its own.
- **FR-032f**: When a document reaches its end, the system MUST tear down the always-resident
  component and MUST dismiss its notification, and starting that document again MUST require
  the reader to open the application. The component MUST NOT be retained in a finished state
  with a play control, because there is nothing for such a control to resume: the stored
  position is already cleared by FR-032b, so the state is equivalent to having no session at
  all, and a resident service with an ongoing notification is a battery and attention cost paid
  for nothing. The cost accepted here is that the lock screen cannot distinguish "the document
  ended" from "the notification went away", which an earlier finished-state answer avoided; the
  ambiguity is accepted because a reader who wants another document must open the application
  to choose one regardless, and because a play control that started audio from a torn-down
  component would either be refused by the platform or produce silence, and both are the
  failure FR-005 prohibits. Teardown MUST NOT itself be reported as an interruption or a
  failure on the next launch, since nothing was lost: the stored position was cleared
  deliberately by completion (FR-010, FR-032)
- **FR-036**: A stored position MUST be keyed by a fingerprint derived from the document's
  content, and MUST NOT be keyed by the file's location. Moving or renaming the document
  MUST NOT orphan the stored position, because refiling documents is an ordinary act on a
  phone and losing a bookmark to a rename is an invisible loss
- **FR-036a**: When no session is active, a document whose content has changed MUST be treated as a different document
  from the one a stored position was recorded against. The stored position MUST NOT be
  offered, retargeted, or approximately matched, and the changed document MUST begin from its
  first spoken position. This is the consequence of FR-036 rather than a rule competing with
  it: the fingerprint identifies the content, so an edited file no longer resolves, and a
  position offered against changed prose would place the reader at a word that is no longer
  the one they stopped on. The unreachable position is discarded rather than retained for a
  possible reversion, because holding it would reintroduce the location-keyed behaviour the
  fingerprint exists to prevent. Identical content at any path is still the same document.
  Where the change is detected while a session is active, the pending stored position is
  invalidated rather than offered: the session recovers to a paused state with an explicit
  resume control from the first spoken position of the changed content, and no approximate
  match is offered (FR-032d, FR-036)
- **FR-038**: A stored position MUST remain valid when the voice or engine changes, and MUST
  be re-anchored to the nearest sentence or node boundary in the new engine's timeline
  before reading resumes. The stored elapsed time MUST NOT be replayed unchanged across a
  change of voice, because a different voice reaches that second at a different word, and
  resuming mid-word is the same defect the bounded rewind in FR-015 exists to prevent. If no
  boundary is available at or before the stored position, the position MUST fall back to the
  start of the document rather than to an arbitrary offset

**Audio handoff contract**

- **FR-033**: The system MUST declare a single fixed playback sample rate, and every
  engine MUST deliver mono f32 PCM resampled to exactly that rate. **The declared rate is
  22050 Hz.** The output stream MUST be requested at exactly that rate, and the device's own
  preferred default rate MUST NOT be adopted in its place: a device default is not this
  specification's declared rate, and adopting one silently would put a different timeline
  behind every stored position, so the rate a position is recorded against and the rate it is
  heard at would no longer be the same number. A device that refuses the declared rate is a
  named failure under FR-035, never a substituted format and never a silent resample. The
  declaration belongs to the playback side of the handoff and not to the platform surface,
  which declares nothing and decides nothing (FR-042, SC-034)
- **FR-034**: The Rust playback module MUST be solely responsible for converting engine output
  into the output device's native sample format. The native format is the format negotiated
  for the output stream at initialisation — queried from the platform, recorded as a runtime
  constant, never assumed — and conversion covers sample width and channel layout, including
  mono-to-stereo upmixing. Resampling MUST NOT be that module's responsibility: it belongs to the engine, so that no device knowledge leaks into engine
  adapters (FR-042)
- **FR-035**: An engine that cannot deliver the declared sample rate MUST surface as a
  named engine failure rather than being accepted with a mismatched rate. A device-side
  rejection of the declared rate or shape at stream-negotiation time falls under the same
  rule: it MUST surface as a named failure with a stated reason under FR-037, never as
  silence and never as playback at a substituted format

**Diagnosability**

- **FR-039**: The system MUST retain a local record of playback session starts and ends,
  transport changes, audio-focus transitions, output-device events, and every failure or
  platform refusal, carrying the position and time at which each occurred. Entries MUST be
  retained for 30 days and MUST then be discarded oldest-first. Every transport change MUST
  carry the playback position at the moment of that change, and a session-end entry MUST carry
  both the transport state and the reason the session ended, because those two facts are what
  let a claimed playing state be checked against the record rather than against the surface
  that reported it. A retention window is required
  because the record is the only evidence available for the zero-occurrence success criteria
  below, and a device kept for a year would otherwise accumulate events indefinitely, while a
  window long enough to cover a problem the reader reports afterwards is what keeps a diagnosis
  possible (FR-044, FR-045, SC-005). A superseded focus loss MUST be recorded as two ordered append-only entries — the transient entry then the permanent entry carrying a supersede link — each with timestamp, position, and outcome; the first entry MUST NOT be merged or rewritten, so replay answers what was true at any point (FR-044)
- **FR-040**: The diagnostic record MUST remain on the device and MUST NOT be transmitted to
  any external service. Several success criteria are stated as zero-occurrence counts
  (SC-005, SC-010, SC-012, SC-019), and a count of events that must never happen cannot be
  verified without a record of the events that did happen; no outbound reporting is
  permitted, so that verifying the record cannot itself become a channel of egress
- **FR-039b**: The diagnostic record MUST be stored in the application's own private storage
  and MUST NOT be written to a device-wide or shared log area. Private storage is chosen because
  it needs no permission, is not readable by other applications, and survives as long as the
  application is installed — which is the whole lifetime a reader's problem report can arrive
  in. Bounded twice over, by the 30-day window and by the 10 MB cap, because either bound alone
  can be defeated by the other's opposite condition: a day of heavy play can outrun the window,
  and a device read for months cannot be bounded by days alone (FR-039, FR-039b). The record MUST additionally be bounded by a total size cap of 10 MB, on the order of
  100,000 entries, with oldest-first trimming,
  as a second bound alongside the 30-day window of FR-039, because a device that is played
  heavily within a single day could otherwise accumulate entries faster than the window admits
  them and grow without limit. Where a cap is reached, dropped entries MUST remain visible as
  dropped rather than leaving a gap that reads as "nothing happened", as the storage-exhaustion
  edge case already requires. Gap and trim markers carry a fixed reservation held outside the
  10 MB cap, and a deferred marker MUST be flushed before any trim of retained entries, so a
  marker can never itself be trimmed before it lands
- **FR-039a**: This feature MUST provide no application surface for reading the diagnostic
  record. The record is written on the device and read through developer tooling, and the
  application MUST NOT present it, list it, or offer to export it. An in-application view of
  the record and a reader-initiated export are wanted, in that order, and are deferred to
  later features rather than built here. This is a deliberate deferral rather than an
  omission: the record has to be written before it can be displayed, because a failure the
  reader reports afterwards cannot be diagnosed from events that were never kept — and a
  screen that existed with nothing behind it would be a surface reporting on data it had not
  yet collected
- **FR-044**: The value sets this feature depends on are **closed sets**, and every one of
  them is enumerated in exactly one place: the rate-ladder rungs, the audio-focus outcomes,
  the transport states, the engine identifiers, the error kinds carried in the record, and the
  session-end reasons. No catch-all, "other", "internal", or "unknown" arm may appear on any
  of them, and a condition no member covers MUST surface as a named failure under FR-023 and
  FR-037 rather than through a branch that absorbs it, because a catch-all converts an
  unhandled condition into a silently wrong one and silence is the failure this project exists
  to fix. The **engine identifiers** are the one set this specification does not enumerate:
  they live in the constitution's engine registry and are referenced from here by name and
  version, so that a second list cannot drift from the first. Extending any of these sets is
  a coordinated edit across every file that enumerates it, accompanied by a semantic-version
  bump and a documentation update — never an addition made in one place and not in another,
  because a set that grows unevenly is a set whose meaning differs per implementation. Two
  reconciliations are fixed here rather than left to planning. **Focus**: `regained` is an
  event of *direction*, not a fourth classification of loss — a focus transition is recorded as
  the kind of loss that occurred and, when it happens, as a separate regained event, so that
  "it ducked and came back" is two ordered entries rather than one ambiguous state.
  **Session end, transport, and the record**: transport is a closed set of five values — playing, paused, stopped, completed,
  interrupted — and the session-end reason is the closed four-value subset paused, stopped, completed, interrupted shared with it.
  A session-end entry carries `transport` constrained to that shared four-value subset plus `session_end_reason` from the same subset (never `playing` on a `session_end` entry), so a session cannot end in a condition the transport vocabulary cannot express
  (FR-019, FR-013a, FR-023c, FR-039)
- **FR-045**: No entry in the diagnostic record may contain text drawn from the document. An
  entry's content is limited to **event kinds, playback position, time, identifiers, and named
  failures**; a sentence, a heading, a fragment of prose, or a passage of the document's body
  is prohibited, and a title already shown on the lock-screen surface is no more admissible in
  the record than in a failure reason. The record's shape MUST therefore offer no open
  free-text field: the `detail` field it carries is **retained bounded to at most 200 characters and the pattern `^[A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+( [A-Za-z0-9_]+=[A-Za-z0-9_.+\-]+)*$` admitting identifiers and values rather than prose** (declared choice; the remove-alternative is rejected because `contracts/diagnostic-record.schema.json` and SC-056 already normatively retain the bounded field), because an
  unbounded field is precisely where a document sentence would end up being written down by
  accident and then read, stored, or exported by tooling. Every entry whose kind is `failure`
  and every entry whose kind is `refusal` MUST carry `error`, since both name a condition
  rather than merely marking that something happened. A scanner over the whole retained record
  verifies that zero entries carry document text, so the prohibition is mechanical rather than
  a matter of inspection (FR-028, FR-037, FR-039)

### Key Entities

- **Playback Session**: One continuous reading of one document. Carries the authoritative
  position, the transport state, and the current rate. The single source of truth for what
  is playing. Records how the session ended — paused, stopped, completed, or interrupted, one
  closed set shared with the transport state a record entry carries (FR-044) —
  because that distinction decides whether the next launch presents a resume offer or starts
  clean (FR-032, FR-032a, FR-032b), and because completion ends the session and tears down the
  resident component rather than leaving a surface with nothing to resume (FR-032f).
- **Playback Position**: Where the reader is within the document, expressed as a time-offset
  in seconds from the start of the document's spoken audio. Chosen as a time-offset because it
  makes the 5-second skip increments a constant displacement and keeps the representation
  independent of headings, paragraphing, and engine, so that a position recorded by one engine
  can be resumed by another (FR-004, FR-018, FR-030). Persisted such that a session killed
  mid-playback resumes at the last spoken position rather than the document start. Engine-
  independence holds only up to re-anchoring: the offset identifies a point in the *narration*,
  and crossing to a different voice resolves that offset to the nearest sentence or node
  boundary rather than to the same second (FR-038).
- **Resume Offer**: The prompt shown when a document with a stored position is reopened,
  offering to continue from that position or to start over, held visible for 3 seconds
  (FR-032c). In its deliberate-pause form it auto-starts on expiry; after an unexpected end
  it is a paused state awaiting an explicit action and never auto-starts (FR-032d, FR-031).
  The two forms are deliberately distinct because a position the user set is trustworthy
  while a position recovered from an interruption may not be.
- **Stored Position**: The last spoken position kept for a document between sessions, keyed
  by that document's fingerprint rather than by its location (FR-036). Recoverable to within
  30 seconds of replay, so that an abrupt loss of power or process does not send the reader
  back to the paragraph start (FR-004). Survives Pause,
  application close, service kill, device power-off, battery loss, and network loss
  (FR-032a), and is cleared only by Stop or by reaching the end of the document
  (FR-032, FR-032b). Pausing is therefore not the same act as stopping, and the two must
  never be conflated in storage or in the transport surface.
- **Document Fingerprint**: A value derived from a document's content that identifies it for
  stored-position lookup (FR-036). Chosen over file location because a document that is moved
  or renamed is the same document, and a stored position keyed by path would silently become
  unreachable. Two documents with identical content share a fingerprint, and therefore share a
  stored position, which is acceptable because their spoken audio is identical. Editing a
  document changes the fingerprint, so an edited document resolves to no stored position and is
  read from its beginning rather than from an approximate match (FR-036a).
- **Always-resident Component**: The playback service retained between sessions in a paused
  state so that a lock-screen transport press is handled by a live process rather than
  requiring a cold foreground-service start from the background (FR-009, FR-009a). Retained
  only while there is something to resume, and torn down on completion, so that residency
  tracks resumable state rather than the history of playback (FR-032f). It is the
  playback service itself, not a second component beside it, so there is exactly one resident
  component and one ongoing notification. It holds no playback authority distinct from the
  session it retains — the engine remains the single authority — and MUST NOT be treated as
  an active session when it is killed while nothing is playing (FR-010, FR-032).
- **Playback Rate**: How much faster or slower than natural the speech is delivered, applied
  to already-synthesised audio without altering pitch or voice character (FR-019a). Held on
  the session rather than the audio, so that changing it does not move the playback position.
  Takes one of seven closed values — 0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x — and moves
  one step per control press (FR-019). Free adjustment is not in the set. It is one
  app-wide setting remembered across sessions and documents rather than a per-document one,
  so a new session opens at the last rate used and at 1.0x only on first ever use
  (FR-019b). Applied by the engine through a pitch-preserving time-stretch
  operation the Rust playback module calls, never by stretching the buffer in the platform
  layer, so that the delivered sample rate is the one fixed declared rate whatever the rate is
  (FR-043).
- **Focus Transition**: An event describing audio focus being lost or regained, classified by
  what the interrupting app requested rather than by the system's own reading of the event.
  Three outcomes result from that classification: ducking, which lowers the volume to roughly
  20% and keeps reading; a transient pause, which stops and will resume; and a permanent
  loss, which stops and will not resume on its own (FR-013, FR-013a, FR-014). A phone call
  and a navigation prompt are both focus losses and produce different outcomes because the
  app that raised them asked for different things (FR-013a). Carries the paused duration that
  determines the rewind applied on resume (FR-015). A pause that happens while the device is
  locked — whether from a silence-demanding interruption or from a permanent loss — also
  produces a Pause Reason, which is shown on the lock screen as scrolling text and expires 3
  seconds after the pause (FR-041). Before any loss can occur the request itself is resolved
  against its own closed set — granted immediately, granted but deferred until the audio
  arrives, or refused — and each of those is handled explicitly rather than falling through to
  the classification above (FR-013b). Focus regained is an event of direction rather than a
  fourth kind of loss, and the record states the kind of loss and the regained moment as two
  ordered entries (FR-044).
- **Pause Reason**: The short statement explaining why a document is not playing — for
  example "Leitura pausada — outro app usando o áudio", shown as scrolling text on the
  lock-screen surface when a silence-demanding interruption or a permanent focus loss
  interrupts a locked session (FR-013a, FR-041). States that reading is paused, never that
  it is resuming, because neither interruption resumes on its own (FR-014). Names the
  document title and nothing else from the document (FR-037). Expires 3 seconds after the
  pause: within that window the application also shows it, after it only the paused state and
  its resume control remain. Distinct from a failure reason, which describes something that
  went wrong (FR-022, FR-037) rather than an interruption that did not.
- **Pre-roll Buffer**: Audio prepared ahead of the resume point once the reader has acted to
  hear it. Never
  advances position, holds at most 60 seconds, and abandoned when the session ends (FR-015a).
  Empty while a paused session is idle, because preparing audio for a session nobody has
  resumed bills a per-character engine for minutes that will be discarded; a Play press
  refills it (FR-015c).
  Delivered by the audio engine and bounded as a pipeline property rather than by a chosen
  side of the engine handoff, so that audio prepared but never played stays inside the
  synthesis budget rather than consuming it silently (FR-015b).
- **Notification Visibility**: Whether the lock-screen notification shows its content while the
  device is locked. Always visible, never redacted by the application and never a subject of an
  in-app privacy setting (FR-028b). Fixed rather than configurable because the lock screen is
  the only surface a driving reader can read without touching the phone, so a mode that blanks
  it would remove the orientation the feature exists to provide; the privacy the reader actually
  needs is privacy of the audible output, answered by FR-016. Distinct from the permission
  state of FR-008, which decides whether the notification may exist at all rather than what it
  shows.
- **Progress Indication**: What the lock-screen surface shows about how far playback has got.
  Determinate only when the document's total spoken length is actually known, and
  indeterminate otherwise, because a streamed document's total may not be knowable before it
  has been synthesised (FR-028a). Never asserts an estimated or projected total, and never
  changes determinacy within a session. Distinct from the section title, which orients by
  document structure rather than by position (FR-028).
- **Output Device Event**: A change in available audio output, such as a headset being
  connected or disconnected. A device appearing mid-playback carries the automatic
  switch requirement of FR-017a; a device disappearing is governed by FR-016. Both MUST be
  recorded so a route change is distinguishable from a failure afterwards (FR-039).
- **Engine Failure**: A named condition where a required engine cannot be reached,
  initialised, or deliver audio conforming to the declared handoff contract. Surfaces to
  the user; never degrades to silence (FR-023, FR-035). Carries two presentations: a short
  statement on the lock-screen surface and the full reason in the application (FR-037),
  neither of which may quote document content. Recoverable in the sense that it leads to an
  Engine Offer rather than to a dead session (FR-023a).
- **Engine Offer**: The bounded choice presented when an engine fails mid-document, naming
  the failure and offering the alternative engines, held for 3 seconds and then resolving to
  the next engine in the declared order if no choice is made (FR-023a). Resolved against the
  closed declared order of FR-023c rather than against whatever is configured, and naming only
  engines the fallback may actually select, so the sequence the reader is being offered is the
  sequence that will be followed. Both the offer and the
  automatic fallback continue from the last spoken position, so the document is never
  restarted by an engine change. The same bounded-offer shape as the Resume Offer, applied to
  recovery rather than to reopening — distinguished from it because a Resume Offer follows a
  pause the reader chose and this follows a failure the reader did not. Expires into a paused
  state with a stated reason when no alternative engine exists, so that a window never
  resolves into unexplained silence.
- **Session Diagnostic Record**: The local history of session starts and ends, transport
  changes, focus transitions, device events, and failures, each carrying its position and
  time (FR-039). Held in the application's own private storage, bounded by the 30-day window
  and by a total size cap, and unreadable by other applications (FR-039b). Exists so that the feature's zero-occurrence success criteria can be
  verified, and never leaves the device (FR-040). Its contents are event kinds, positions,
  time, identifiers, and failure names — **never document text**, and no free-text field wide
  enough to carry any (FR-045). A focus transition is recorded as the kind of loss that
  occurred plus a separate regained event rather than as one ambiguous state, and a session-end
  entry carries both the transport state and the reason, from one shared closed set of four
  values (FR-044). Entries older than 30 days are discarded
  oldest-first, and the record states that expiry occurred rather than leaving a silent gap
  that reads as "nothing happened" (FR-039). Has no application surface in this feature: it
  is written for reading through developer tooling, with an in-application view and then a
  reader-initiated export deferred to later features (FR-039a).
- **Declared Playback Format**: The single sample rate the system declares — **22050 Hz** —
  together with the mono f32 PCM shape every engine must deliver. The contract that makes
  adapters interchangeable, and the reason resampling lives in the engine rather than the Rust
  playback module (FR-033, FR-034, FR-043). Requested on the output stream as exactly that
  rate; a device that refuses it is a named failure rather than a substituted format, so that
  the rate a position is recorded against is the rate it is heard at.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 10 out of 10 trials, audio continues playing for a full five minutes with
  the device screen off and manually locked
- **SC-002**: In 10 out of 10 trials, disconnecting headphones pauses playback within one
  second, and audio is never audible through the loudspeaker afterward; and in 10 out of 10
  trials a Bluetooth A2DP route loss pauses playback within one second even where the platform
  delivers the removal only through the secondary device-removal signal rather than through
  the becoming-noisy signal, so that neither of the two mechanisms is load-bearing alone
  (FR-016, FR-016a)
- **SC-003**: In 10 out of 10 trials, a permitted interruption reduces playback gain to 15–25%
  of its normal level without stopping the document, full volume returns when focus is
  regained, and playback resumes at the start of the sentence containing the point 5% of the
  paused duration back from where playback yielded — with total repetition of already-spoken
  audio never exceeding 30 seconds, no content skipped, and zero instances of the resume landing
  part-way through a word — measured against synthetic audio with known inter-word gaps,
  where the resume MUST land in a gap and never in an energy peak (FR-013, FR-015)
- **SC-041**: In 10 out of 10 trials, an interruption that demands silence — a phone call, an
  alarm — pauses playback rather than ducking it, produces no audible document audio for the
  duration of the interruption, and states the same reason a permanent loss states; and in 10
  out of 10 trials an interruption that merely permits reduced audio still ducks instead of
  pausing, so the two signals produce different outcomes (FR-013a)
- **SC-055**: In 10 out of 10 trials where a focus request is answered with a deferred grant,
  the document produces no audio at all until the grant arrives and then plays from the point
  it would otherwise have played from, with no rewind applied and no pause reason shown; and in
  10 out of 10 trials where a focus request is refused outright, the refusal is presented as a
  named failure with a stated reason on the lock-screen surface and in the application, the
  session presents no playing state, and zero instances occur of the refusal resolving into
  silence (FR-013b, FR-037)
- **SC-004**: In 10 out of 10 trials, every transport control reachable from the lock screen
  produces the correct effect without requiring an unlock
- **SC-005**: Zero occurrences of a `transport: "playing"` entry in the diagnostic record that
  is not followed, in the same session, by a later entry carrying a strictly greater
  `position_secs` — the window being the interval between that entry and the next entry
  recorded for the same session — except where the entries that intervene are a completion, a
  stop, a pause, a device loss, or a named failure, each of which legitimately ends the
  advance. The check is a scan over the record and needs no playback-backend counters, which
  the backend in use does not expose; a backend that did expose them would be a stronger
  adapter choice, and that choice is a later decision rather than a requirement here. This is
  what turns "no playing state while producing no audio" from an assertion into a rule a
  machine can check (FR-005, FR-039)
- **SC-006**: An automated check confirms the foreground service declaration and required
  permissions are present in the built application manifest, so a regression in that
  declaration is caught without human review
- **SC-032**: The application declares a minimum supported Android version of 10 (API 29), and
  in 10 out of 10 trials on a device running that version or later, a foreground service with
  the media-playback type starts and keeps playback alive with the screen off. No requirement
  in this specification is verified on a release older than that floor
- **SC-007**: After an unexpected process termination and relaunch, playback can be resumed
  at the last spoken position, replaying at most 30 seconds of already-spoken audio, and in
  10 out of 10 trials never replaying more than that bound
- **SC-008**: In 10 out of 10 locked trials with the screen covered (blind-test harness:
  the tester cannot see the surface and a trial fails on any unlock or any effect left
  unconfirmed aloud), every driving-time task — play, pause, resume,
  skip in either direction, rate change — completes without an unlock
  and zero trials require visual confirmation of the effect (FR-018a)
- **SC-009**: In 10 out of 10 trials, audible audio begins within 3 seconds of the play
  action on a mid-range arm64 device with the document already parsed
- **SC-010**: Zero instances of document body text appearing in the notification, on the
  lock screen, or in the notification shade, with the only document-derived content permitted
  there being the document title and the current section title
- **SC-046**: In 10 out of 10 trials on a locked device with no phone-wide notification-content
  redaction enabled, the lock-screen surface shows the document title and the section title
  rather than a generic "new notification" placeholder, and zero instances occur of the
  application redacting that content, hiding it, or presenting a control that would (FR-028b)
- **SC-042**: In 10 out of 10 trials, the progress indication on the lock-screen surface is
  determinate only when the document's total spoken length is actually known and is
  indeterminate otherwise, zero instances occur of an estimated or projected total being shown
  as established, and zero instances occur of the indication switching between determinate and
  indeterminate within one session (FR-028a)
- **SC-047**: In 10 out of 10 trials, the section title on the lock-screen surface is correct
  immediately after a boundary crossing, a resume, a skip in either direction, and a re-anchor
  following an engine change, and zero instances occur of a title naming a section the position
  has already left or a section it has not yet entered (FR-028d)
- **SC-053**: In 10 out of 10 trials, a surface carrying playing audio offers no dismissal
  affordance and presents a working pause control, and pressing pause on a transient reason
  surface dismisses that surface within 3 seconds while pressing pause on the persistent resident
  surface leaves it present and dismissable with its paused state and resume control intact, so
  that a Play press from it still produces audio (FR-028e)
- **SC-011**: In 10 out of 10 trials where the service is killed under memory pressure,
  the next launch shows a paused state at the last spoken position and produces no audio
  until the user activates resume
- **SC-012**: Zero instances of audio starting without a user action, in any recovery path,
  the two permitted exceptions being the deliberate-pause reopen path of FR-032c after its
  3-second offer expires with the application in the foreground, and the engine-fallback
  continuation of FR-023a after its 3-second offer expires (a continuation of an already
  started session, not a session start)
- **SC-020**: In 10 out of 10 trials, reopening a deliberately paused document shows the
  resume offer with both continue and start-over available, and reading begins within 4
  seconds of the offer appearing whether or not the user acts, provided the application is
  in the foreground when the window expires; where it is not, no audio begins until the
  user activates resume (FR-032c, FR-032d)
- **SC-021**: In 10 out of 10 trials, reopening a document left by an unexpected end, a
  power-off, a battery death, or a network loss produces no audio at all until the user
  activates resume (FR-032d)
- **SC-013**: Zero instances of pre-roll buffering advancing the playback position, and zero
  instances of the pre-roll buffer holding more than 60 seconds of audio or growing while a
  session remains paused indefinitely, and zero instances of synthesis being performed while
  paused beyond that bound (FR-015b)
- **SC-050**: In 10 out of 10 trials, a document left paused for at least one hour produces
  zero synthesis and holds an empty pre-roll buffer throughout the idle period, and zero
  instances occur of a per-character engine being billed for audio during that idle period; and
  in 10 out of 10 trials, pressing play after that idle period begins pre-roll and produces
  audible audio within the 3-second bound of FR-027 (FR-015a, FR-015c)
- **SC-050b**: In 10 out of 10 trials, discarded pre-roll (audio prepared but never played)
  bills zero characters on a per-character engine beyond the smallest documented per-request
  byte budget, so pre-roll never spends the reader's free quota on minutes that are discarded
  (FR-015b)
- **SC-014**: In 10 out of 10 trials, a Play press from the lock screen with the application
  process not running produces audible audio without the user opening the application, and
  where the platform refuses the start, the refusal is stated and no playing state is shown
- **SC-043**: In 10 out of 10 trials, after playback has been paused or interrupted and a
  resumable position remains, exactly one resident component and one ongoing notification exist
  and no second
  lock-screen surface competes with them, the resident notification shows the paused state and
  a resume control while asserting no elapsed time, remaining time, or section title, and a
  Play press from that notification still produces audio without the application being opened
  (FR-009a)
- **SC-015**: Zero trials in which a rate change, or a sequence of rate changes including the
  step from 2.0x to 4.0x, is audible as a change in pitch or voice character. Verification is
  deterministic and host-runnable: the fundamental-frequency contour and spectral-peak
  positions of identical PCM rendered at each of the seven rungs MUST match within ±50 cents
  of F0 and equivalent spectral-peak alignment, so pass or fail is decided by measurement rather than by listening (FR-019a)
- **SC-054**: In 10 out of 10 trials, a rate control press takes audible effect within 2 seconds
  on a mid-range arm64 device with the document already parsed (FR-019c)
- **SC-016**: In 10 out of 10 trials, a skip press from the lock screen moves the playback
  position by exactly five seconds in the pressed direction, and a press that would cross a
  document boundary clamps to that boundary without error or silence
- **SC-017**: In 10 out of 10 trials, the stored position for a paused document survives
  application close, service kill, device power-off, and network loss, and reopening the
  document offers to continue from that position (FR-032a)
- **SC-018**: In 10 out of 10 trials, Stop discards the stored position such that reopening
  the document begins from its first spoken position, with no resume offer presented
- **SC-019**: Zero instances of a Stopped or completed document offering to resume (FR-032b)
- **SC-049**: In 10 out of 10 trials, after a paused session has been playing and has been
  paused, exactly one resident component and one ongoing notification exist, and the same holds
  after an interrupted session; and in 10 out of 10 trials where the document reached its end,
  zero resident components and zero ongoing notifications remain, so that residency is
  observable as tracking whether there is something to resume rather than tracking that
  playback has ever been started (FR-009a, FR-032f)
- **SC-045**: In 10 out of 10 trials, a document reaching its end while the device is locked
  results in the resident notification being dismissed and the resident component being torn
  down, with zero instances of a surface remaining that offers to play or resume the completed
  document, zero instances of audio produced at the moment of completion, and zero instances of
  the next launch presenting a paused state or a resume offer for that document (FR-032e,
  FR-032f)
- **SC-022**: In 10 out of 10 trials, a document that is moved or renamed on the device is
  still recognised as the same document and offers to resume at its previously stored
  position, and in 10 out of 10 trials a document with different content at the same path
  offers no resume
- **SC-038**: In 10 out of 10 trials, a document edited while no session is active offers no
  resume, begins from its first spoken position, and zero instances occur of a stored position
  being offered or approximately matched against content it was not recorded against (FR-036a)
- **SC-023**: In 10 out of 10 trials, a refusal to start or an audio-device failure occurring
  while the device is locked presents a reason on the lock-screen surface without an unlock,
  and in 10 out of 10 trials that reason contains no text drawn from the document
- **SC-024**: In 10 out of 10 trials, resuming a document after a voice or engine change
  begins at a sentence or node boundary rather than mid-word, and the boundary chosen is
  reported to the reader
- **SC-025**: In 10 out of 10 trials, a forced audio-device failure leaves a local record
  naming the failure and the position at which it occurred, zero instances of any
  diagnostic record leaving the device, and zero instances of entries older than 30 days
  remaining in the record, and zero instances of the record being readable by another
  application through a shared or device-wide log area; and in 10 out of 10 trials in which the
  record is driven past 10 MB, the oldest entries are trimmed so the record does not exceed the
  cap and the trim remains visible as a trim rather than as an absence of events (FR-039b)
- **SC-056**: A scanner over the whole retained record reports zero entries containing text
  drawn from the document, zero entries whose `detail` value exceeds 200 characters or fails the
  declared non-prose pattern where that field is retained at all, and zero entries whose kind
  is `failure` or `refusal` without an `error` value, so that the no-document-text rule is
  verified mechanically over every entry rather than by inspecting a sample (FR-045)
- **SC-037**: Zero instances of the application presenting, listing, or exporting the
  diagnostic record, so that FR-039a is verified as an absence rather than left to inspection
- **SC-039**: In 10 out of 10 trials, an engine failing mid-document presents the named
  failure with the available alternatives, and playback continues from the last spoken
  position rather than the document start, whether the reader chooses within 3 seconds or
  lets the window expire, and zero trials leave an unresolved failure presenting as silence
- **SC-040**: In 10 out of 10 trials, an engine failing while the device is locked resolves
  through the 3-second offer without an unlock, and the notification has stated both the
  failure and the fallback; and in 10 out of 10 trials where no alternative engine exists, the
  session ends paused with a stated reason rather than continuing or failing silently
- **SC-044**: On a device carrying at least two working engines, one offline and one cloud, an
  engine failing mid-document is followed by the other engine taking over with no document
  restart, and forcing the second engine to fail as well leaves the session paused with a
  stated reason rather than in silence. This evidence requires real engines and therefore cannot
  be produced while synthesis is delivered by another feature under FR-024; it is the hardware
  criterion, re-verified once those engines exist, and it is NOT discharged by SC-051
  (FR-023b)
- **SC-051**: In 10 out of 10 trials within this feature, using two stand-in engines that honour
  the same engine contract, an engine failing mid-document is followed by the
  other engine taking over with no document restart, and forcing the second engine to fail as
  well leaves the session paused with a stated reason rather than in silence. The pair exists
  because FR-024 places synthesis in another feature, so this criterion is what makes FR-023a
  verifiable here rather than deferred whole. It verifies the recovery mechanism only; SC-044
  remains the evidence that the mechanism behaves the same way against real engines
  (FR-023b, FR-024)
- **SC-052**: In 10 out of 10 trials, a failure resolved through the expired 3-second fallback
  selects an engine the declared order places after the failed one rather than restarting the
  walk at the top of the sequence, the engine selected is one the offer named, and zero instances
  occur of an engine the reader has not consented to being selected by the fallback
  (FR-023c)
- **SC-026**: In 10 out of 10 trials, each rate control press moves the rate exactly one step
  along the declared ladder of 0.75x, 1.0x, 1.25x, 1.5x, 1.75x, 2.0x, 4.0x, a press at either
  end of the ladder holds that end, and no press ever yields a rate outside the ladder
- **SC-036**: In 10 out of 10 trials, a new session opens at the last rate used anywhere in the
  application, at 1.0x when none has been chosen, the chosen rate survives the application being
  closed, and zero instances occur of a document opening at a rate different from the one in
  effect before it was opened (FR-019b)
- **SC-027**: In 10 out of 10 trials, a rate reached from the lock screen is the same rate shown
  inside the application, and the rate value never appears on the lock screen or in the
  notification shade
- **SC-028**: In 10 out of 10 trials where the notification permission is denied, the
  application states that lock-screen controls are unavailable and playback can still be
  started, paused, and stopped from inside the application
- **SC-029**: In 10 out of 10 trials, a permanent focus loss reached while the screen is off
  results in a paused state that is visibly reported on the lock screen or in the
  notification shade, so the reader learns the document stopped without unlocking the device,
  and the reported statement scrolls rather than sitting static, names the document title,
  states that reading is paused rather than resuming, and
  carries no other document text
- **SC-031**: In 10 out of 10 trials, opening the application within 3 seconds of that pause
  shows the reason, and in 10 out of 10 trials opening it later than 3 seconds after the pause
  shows no such reason while still showing the paused state and its resume control, and zero
  instances occur of a focus-loss pause reason being shown in the application after that
  3-second window in any of those trials (FR-037a)
- **SC-048**: In 10 out of 10 trials, a genuine failure arising while the device is locked —
  an engine failure, an audio-device failure, or a refused start — still shows its full reason
  in the application when it is opened at any later time, with no expiry window applied, so
  that the persistence FR-037 requires is distinguishable in testing from the focus-loss
  expiry of SC-031 rather than the two surfaces agreeing on everything (FR-037a)
- **SC-030**: In 10 out of 10 trials, a document with no usable title shows a stable
  placeholder on the notification surface, a document with no section title at the current
  position shows no section title rather than body text, and a position before the document's
  first heading shows no section title while a position after a heading shows that heading
  (FR-028c)
- **SC-033**: In 10 out of 10 trials, a new output device becoming available during playback
  causes audio to continue on that device from the unchanged position, with zero instances of
  a pause, a restart, a repeated phrase, or audio moving to a loudspeaker as a result
  (FR-017a)
- **SC-033b**: In 10 out of 10 trials where a disappearance and an appearance coincide in one
  route change (unplugging wired while car Bluetooth connects), the system pauses first under
  FR-016 and resumes only under the FR-017a privacy-precedence rule, with zero instances of
  audio moving unattended to the newly appearing device (FR-016, FR-017a)
- **SC-034**: Zero instances of the audio the engine delivers arriving at a sample rate other
  than the single declared rate of 22050 Hz, including during and after every rate-ladder step
  of FR-019, so that a rate change is observable only as a change in speed; and zero instances
  occur of the output stream being opened at a device default rate in place of the declared one
  (FR-033)
- **SC-035**: An automated check confirms that the Android and Tauri surface contains no
  playback logic, so that every behaviour named in a functional requirement resolves in the
  Rust backend, and a behaviour added to the platform layer without a Rust counterpart fails
  the check rather than passing review unnoticed. Detection is by static scan of the
  platform surface for transport, rate, position, buffering, and focus-response logic
  (state mutations, gain application, skip arithmetic); any match outside a declared
  allowlist fails the check
- **SC-057**: An automated check enumerates each closed set this specification governs — rate
  rungs, focus outcomes, transport states, error kinds, and session-end reasons — and fails if
  any of them contains a catch-all, "other", "internal", or "unknown" arm, and fails if an
  engine identifier referenced in the record or in the fallback order is not present in the
  constitution's engine registry at the version the reference names, so that neither an
  unhandled condition nor a second engine list can reach a build unnoticed (FR-023c, FR-044)

## Assumptions

<!--
  The Architecture Decision note below is context for the maintainer and future readers.
  It is NOT part of this specification: no requirement depends on it, and changing any
  technology named there does not invalidate a single functional requirement above.
-->

### Architecture Decision (context only, not normative)

Recorded here so a future reader understands why the shell is split as it is. The
maintainer chose this direction explicitly after the alternative was presented:

- **Audio output and playback authority are owned by the audio engine**, implemented in
  Rust, driving the platform audio backend directly (via a Rust audio crate rather than
  through the platform's media player).
- **The Kotlin surface is reduced to the platform shell**: the background service, the
  media session, the notification, and the audio-focus request. The media session mirrors
  engine state and never computes its own position.
- **The shell carries no logic at all.** Its entire content is exposing the application to
  Android. Every behaviour is resolved by the Rust backend behind its own modules or
  submodules, including the rate operation and the conversion into the device's sample
  format. The maintainer corrected an earlier framing of this feature, in which the shell
  was treated as a component that could hold responsibilities of its own (FR-042).
- **Only the service class is structurally forced to be Kotlin.** The Android runtime
  instantiates a service by reflecting on the class named in the manifest, so a
  service implemented entirely in Rust cannot exist. Every other element of the Kotlin
  surface is a design decision, not a platform constraint.
- **Known risk, accepted deliberately**: driving the audio track from Rust while a Kotlin
  media session mirrors state creates two potential authorities over playback position.
  This is why FR-003 exists, and why the failure it prevents — the notification drifting
  out of sync with the audio — is called out as a named defect rather than treated as a
  tuning problem.

### Feature Assumptions

- The audio engine, speech synthesis, and document parsing already exist or are delivered
  by other features; this feature defines the shell that plays what they produce and does
  not define their behaviour.
- Supported devices run Android 10 (API 29) or later. That floor is where foreground services
  became mandatory for background playback and where the media-playback foreground-service type
  exists, so every requirement above assumes that model and none of them carries a legacy
  branch. Releases older than Android 10 are out of scope: they are neither supported nor
  degraded gracefully, and the app is not required to state why on them (FR-008).
- The audio-focus request assumes `AUDIOFOCUS_GAIN` with `AudioAttributes(USAGE_MEDIA, CONTENT_TYPE_SPEECH)`, `setWillPauseWhenDucked(true)`, `setAcceptsDelayedFocusGain(true)`, plus an `OnAudioFocusChangeListener` (the listener is mandatory for both flags; without them the system auto-ducks music content with no callback and a locked focus returns `FAILED`, making FR-013/FR-013b unobservable). Duck depth 15–25% is the app-applied gain factor per FR-013; the system's own auto-duck depth is not relied upon.
- The output stream exact-requests 22050 Hz / PCM float / mono and never `AAUDIO_UNSPECIFIED`; a refusal is FR-035's named failure, because an exact AAudio request is honored or the open fails rather than silently substituted.
- Removal handling assumes `ACTION_AUDIO_BECOMING_NOISY` and `AudioDeviceCallback.onAudioDevicesRemoved` are independent signals with neither guaranteed to co-fire per transport; each alone suffices for the FR-016 one-second pause (FR-016a).
- All cited audio APIs exist below the API-29 floor: `AudioFocusRequest` (API 26), AAudio (API 26), `setUsage`/`setContentType` (API 28), `AudioDeviceCallback` (API 23).
- The user has granted notification permission, or the system states the consequence of not
  having it. The feature does not design a permission-request flow.
- Playback consumes audio in a form that can be streamed continuously rather than only as a
  complete file.
- The audio handoff contract is fixed by this specification (FR-033 to FR-035): mono f32 PCM
  at a single declared rate of 22050 Hz, resampled by the engine, with only sample-format
  conversion done by the playback module. The rate's numeric value is fixed here rather than
  left to planning, because the offline pt-BR voices the constitution's registry admits are
  natively 22050 Hz and asking the engine to resample away from its own native rate would spend
  the reader's synthesis budget to no purpose. Two things remain plan-level: which native rate
  the device negotiates for the output stream, and the conversion path used to reach it — and
  neither may be obtained by adopting the device's own default rate in place of the declared
  one, because FR-033 requires the declared rate to be the rate requested. A device that
  refuses the declared rate is FR-035's named failure, not an invitation to substitute
  (FR-033, FR-035).
- Section titles come from the document's own structure as supplied by the document feature;
  this feature displays them and does not derive them. A document with no headings still
  satisfies FR-028, which requires no section title rather than a substitute.
- Single-document playback is the only supported session shape; playlist and queue
  behaviour is out of scope.
- Sleep timer and per-document resume bookmarks beyond "last spoken position" are out of scope
  for this feature. The single stored position per document required by FR-032a is the extent
  of bookmarking in scope. Rate adjustment is not out of scope: FR-019 declares a fixed seven-step
  rate ladder, which is a control the reader can operate without looking at a list of named
  presets, and free or continuous rate adjustment remains out of scope. The rate is stored as
  one app-wide setting rather than per document (FR-019b); what is out of scope is a
  *per-document* rate, not the memory of the last rate used.
- Suppressing the dismissal of a playing notification is requested behaviour, not a guarantee the
  platform offers uniformly: enforcement differs across Android versions and OEM builds. FR-028e
  therefore fixes the observable contract — pause is the route to dismissal, a pause control is
  present while audio plays, and audio never goes on speaking with no surface able to stop it —
  and leaves the mechanism for suppressing dismissal itself to `plan.md`, together with the
  stated fallback that a dismissal which happens anyway pauses the document.
- The 10 MB cap below is set by entry volume rather than by entry size, and the entry's own
  serialisation is therefore a `plan.md` matter provided it remains text and carries no document
  content, as FR-039 requires.
- Storage of the stored position is per document and holds one position, keyed by the
  document's content fingerprint (FR-036). The fingerprint's construction, the storage
  mechanism, where it lives, and how it survives process death belong in `plan.md`; FR-032a
  fixes only the observable requirement that the position survive and that Stop and
  completion clear it. The diagnostic record is not left to `plan.md` in the same way: FR-039b
  fixes it to the application's own private storage, because where the record physically lives
  is itself a privacy and permission question rather than an implementation preference, and an
  unbounded device-wide log would be a wider surface than this feature is entitled to open. The
  cap's size is fixed here on the same footing as the window, for the same reason: a bound that
  no criterion can name is a bound any implementation may satisfy with any value (FR-039b).
- The application is sideloaded during development; store-distribution constraints do not
  affect this feature's requirements.
- The diagnostic record is developer-facing in this feature. An in-application screen showing
  the record and a reader-initiated export of it are wanted in that order, and both belong to
  later features; FR-039a records the absence as a requirement so it is a stated deferral
  rather than an oversight (FR-039a).
- At least two engines are available to fall back to when the current one fails mid-document,
  because recovery on a single-engine device could only be observed as a reported failure with
  nothing taking over. The set of admissible engines comes from the constitution's engine
  registry, and FR-023c derives the fallback order from that registry's tiers rather than
  leaving it to a plan-level arrangement; this feature governs the recovery behaviour — present,
  offer for 3 seconds, then fall back — and not the choice of which engines are configured on
  a given device, except for the two-engine minimum that makes the behaviour observable at all
  (FR-023a, FR-023b).
- The two-engine minimum behind FR-023b is met differently in the two halves of its
  verification. Within this feature it is met by two stand-in engines that honour the same
  engine contract, which is what makes SC-051 possible; for SC-044 it must be met by a real
  offline engine and a real cloud engine on the phone, which arrives with the feature that
  delivers synthesis and is therefore an obligation this feature carries forward rather than one
  it can discharge now (FR-023b, SC-044, SC-051).