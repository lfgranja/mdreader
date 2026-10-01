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
   transiently (a navigation prompt, a notification chime), **Then** the document's volume
   reduces automatically and restores when focus returns
2. **Given** a document is reading aloud, **When** another app requests audio focus
   permanently, **Then** playback pauses and does not resume on its own
3. **Given** focus was lost transiently, **When** it is regained, **Then** playback resumes
   from the position it paused at, not from the beginning of the sentence

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

---

### User Story 4 - Stay in control from the lock screen (Priority: P2)

Luís wants to skip to the next section, slow down, or jump back a paragraph. He does this
from the lock screen notification. He does not unlock the phone, and he does not look away
from the road for longer than a single glance at a known control position.

**Why this priority**: Principle VIII makes this non-negotiable, but it builds on the
playback-and-focus foundation. Once audio survives locking and interruptions, exposing
controls is the natural next step.

**Independent Test**: With the device locked, exercise each transport control from the
notification and confirm the correct effect on playback.

**Acceptance Scenarios**:

1. **Given** a document is playing and the device is locked, **When** the user taps skip
   forward from the notification, **Then** playback advances to the next section and
   continues without interruption
2. **Given** playback is paused and the device is locked, **When** the user taps play from
   the notification, **Then** playback resumes from where it stopped
3. **Given** playback is in progress, **When** the user adjusts the rate from the
   notification, **Then** the change takes effect within two seconds and persists for the
   rest of the session

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

### Edge Cases

- Audio device becomes unavailable mid-playback (headset disconnected by the kernel rather
  than by the user, Bluetooth dropout). Playback must pause, not crash, and must not
  silently resume on the speaker.
- The user presses play rapidly several times. Each press must be honoured or explicitly
  rejected; the app must not enter a state where it reports "playing" while producing no
  audio.
- Audio focus is lost and regained within the same sentence. Playback must resume mid-
  sentence at the correct position, not replay the sentence.
- A navigation prompt arrives in the same moment as a document section boundary. Both
  events are handled; neither is dropped.
- The foreground service is killed by the system under memory pressure. Playback state must
  be recoverable on the next launch rather than reset to the document start.
- The device is locked and the user has granted no notification permission. Transport
  controls are then unavailable; the app must state this rather than leaving the user
  wondering why the controls are missing.
- Audio focus is lost permanently while the screen is off. The app must pause and the
  paused state must be visible when the user next looks at the phone.

## Requirements *(mandatory)*

### Functional Requirements

**Audio output and authority**

- **FR-001**: The system MUST continue producing audible playback while the device screen
  is off and the device is locked
- **FR-002**: Playback MUST be driven from the audio engine, and the engine MUST be the
  single authority for playback position, buffering state, and transport state
- **FR-003**: The system MUST NOT maintain a second independent source of truth for
  playback position; any media-control surface MUST reflect engine state rather than
  computing its own position
- **FR-004**: Playback position MUST survive process death and be recoverable to the last
  spoken position rather than the document start
- **FR-005**: The system MUST NOT report a playing state while producing no audible output

**Background execution and manifest**

- **FR-006**: Audio playback MUST run in a background service declared with the media
  playback foreground-service type, so the system treats it as ongoing user-visible work
- **FR-007**: The application manifest MUST declare that foreground service explicitly, and
  its presence MUST be verifiable by an automated check rather than by inspection
- **FR-008**: The system MUST request the foreground-service permissions required to keep
  media playback alive, and MUST degrade with a stated reason if they are absent
- **FR-009**: The background service MUST start successfully whether the user launches from
  the application screen or from a transport control while the application is not running
- **FR-010**: The system MUST handle foreground-service start restrictions introduced in
  Android 14 and later, and MUST NOT silently fail to start when the platform refuses the
  start
- **FR-011**: Every media transport control MUST remain functional when the background
  service was started from outside the application process

**Audio focus and interruptions**

- **FR-012**: The system MUST yield to audio focus requests from other applications
- **FR-013**: On transient focus loss, the system MUST reduce playback volume rather than
  stop, and MUST restore full volume when focus returns
- **FR-014**: On permanent focus loss, the system MUST pause and MUST NOT resume
  automatically
- **FR-015**: After a transient loss and recovery, playback MUST resume from the position
  it yielded at, not from the start of the current sentence
- **FR-016**: The system MUST pause playback when headphones or a wired headset are
  disconnected, so audio does not move to the loudspeaker
- **FR-017**: The system MUST resume through a reconnected output device when the user
  explicitly resumes

**Transport controls from the lock screen**

- **FR-018**: The system MUST expose play, pause, resume, skip-forward, and skip-backward
  from the lock screen or notification shade without requiring the device to be unlocked
- **FR-019**: Rate adjustment MUST be reachable without unlocking the device
- **FR-020**: Transport controls MUST reflect actual playback state at all times, and MUST
  NOT display a playing state while playback is paused
- **FR-021**: When the notification permission is denied, the system MUST state that
  lock-screen controls are unavailable rather than omitting them silently

**Failure surfacing**

- **FR-022**: When the audio output device cannot be acquired, the system MUST surface a
  clear failure to the user and MUST NOT continue reporting playback
- **FR-023**: Any engine that cannot be reached MUST surface as a named engine failure and
  MUST NOT degrade to silence

**Scope boundaries for this feature**

- **FR-024**: This feature covers the audio playback shell only. Document parsing, text
  segmentation, speech synthesis, and voice selection are separate features, and this
  feature MUST consume them through a defined interface without implementing them
- **FR-025**: Android Auto integration is out of scope for this feature and MUST NOT be
  treated as a requirement
- **FR-026**: The system MUST NOT depend on network connectivity to start or continue
  playback

### Key Entities

- **Playback Session**: One continuous reading of one document. Carries the authoritative
  position, the transport state, and the current rate. The single source of truth for what
  is playing.
- **Playback Position**: Where the reader is within the document, expressed in a form that
  survives engine changes and process restarts.
- **Focus Transition**: An event describing audio focus being lost or regained, classified by
  whether the interruption is temporary. Drives ducking versus pausing.
- **Output Device Event**: A change in available audio output, such as a headset being
  connected or disconnected.
- **Engine Failure**: A named condition where a required engine cannot be reached or
  initialised. Surfaces to the user; never degrades to silence.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 10 out of 10 trials, audio continues playing for a full five minutes with
  the device screen off and manually locked
- **SC-002**: In 10 out of 10 trials, disconnecting headphones pauses playback within one
  second, and audio is never audible through the loudspeaker afterward
- **SC-003**: In 10 out of 10 trials, a transient interruption reduces volume and playback
  resumes from the exact position it yielded at, with no repeated or skipped content
- **SC-004**: In 10 out of 10 trials, every transport control reachable from the lock screen
  produces the correct effect without requiring an unlock
- **SC-005**: Zero occurrences of the notification showing a playing state while no audio
  is being produced
- **SC-006**: An automated check confirms the foreground service declaration and required
  permissions are present in the built application manifest, so a regression in that
  declaration is caught without human review
- **SC-007**: After an unexpected process termination and relaunch, playback can be resumed
  at the last spoken position
- **SC-008**: Zero reports of required behaviour being blocked by the user looking at the
  screen during normal use

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
- Target devices run a recent Android release where background execution is restricted.
  Behaviour on significantly older releases is out of scope.
- The user has granted notification permission, or the system states the consequence of not
  having it. The feature does not design a permission-request flow.
- Playback consumes audio in a form that can be streamed continuously rather than only as a
  complete file.
- Single-document playback is the only supported session shape; playlist and queue
  behaviour is out of scope.
- Sleep timer, playback speed presets, and per-document resume bookmarks beyond "last spoken
  position" are out of scope for this feature.
- The application is sideloaded during development; store-distribution constraints do not
  affect this feature's requirements.