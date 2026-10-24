# Clarification F2 — Reduced-motion trigger for the SC-007 freeze leg

> **Feature:** `specs/005-adopt-design-sona` (SC-007 freeze leg: with system "Remove animations" active, the sliding pill freezes at a visible track position — 10/10; SC-001 item (v) host census; S5 movement contract)
> **Question:** Which OS pathway authoritatively exercises the CSS `prefers-reduced-motion` media query on the project's reference device, and if the standard setting is absent or unreliable, what SANCTIONED deterministic trigger should the trial protocol name — without violating the S5 census-clean rule (no render-time forced class)?
> **Scope:** research note; not a spec amendment. Informs the SC-007 freeze-leg trial protocol only.
> **Method (2026-10-24):** MDN/W3C media-queries-5 spec; Chromium blog 74 + crbug 722548 + blink-dev intent-to-ship + Chrome Platform Status; Chromium source (AccessibilityState, `ANIMATOR_DURATION_SCALE == 0.0f`); Android issue tracker 161675988 / 229628568; eevis.codes Android reduced-motion article; web.dev prefers-reduced-motion (T. Steiner); Android developer docs (accessibility, 13 behavior changes); repo pins (tests/device/README.md, quickstart.md, scripts/check-manifest.sh). ISO/NHTSA/IVIS references checked but not verifiable for an animation-minimization clause (see §6).

## 1. Pinned device + floor

- **Reference device:** Redmi 2201117TY (spesn_global), Android 13 / API 33, Snapdragon 680 — `tests/device/README.md` §2 + `quickstart.md` (005). All verdicts run on this device; no device-class asterisk.
- **Floor:** Android 10 / API 29 (`scripts/check-manifest.sh`, SC-032). So every supported device is API 29+, i.e. **Android 9+ accessibility "Remove animations" is always available** (introduced API 28).

## 2. Authoritative OS pathway → Chromium `prefers-reduced-motion`

The CSS media feature is defined by W3C Media Queries Level 5 (`drafts.csswg.org/mediaqueries-5/#prefers-reduced-motion`) and consumed by Chromium since **Chrome 74** (beta 2019-03-22, stable Android 2019-04-25) — Chromium blog (https://blog.chromium.org/2019/03/chrome-74-beta-reducing-unwanted-motion.html), blink-dev intent-to-ship (https://groups.google.com/a/chromium.org/g/blink-dev/c/NZ3c9d4ivA8), crbug.com/722548, Chrome Platform Status (https://www.chromestatus.com/feature/5597964353404928). The intent doc confirms Android + Android WebView coverage: "Windows, Mac, and Android are expected to be covered in the initial implementation."

On Android, Chromium's `AccessibilityState.prefersReducedMotion()` returns `reduce` iff **`Settings.Global.ANIMATOR_DURATION_SCALE == 0.0f`** (Chromium source diff on `ui/accessibility/android/accessibility_state.java`; summarized in the eevis.codes article and in Chromium issue 41319571).

The user-facing setting that drives that value is **Settings → Accessibility → "Remove animations"** (Android 9+). MDN lists it verbatim: "In Android 9+: Settings > Accessibility > Remove animations" (https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion). web.dev (T. Steiner) confirms Android Pie shipped the toggle and Chrome 74 honors it (https://web.dev/articles/prefers-reduced-motion). The accessibility toggle sets `ANIMATOR_DURATION_SCALE = 0` (eevis.codes, "Android, Animations and Reduced Motion", https://eevis.codes/blog/2022-12-12/android-animations-and-reduced-motion/).

**Android WebView inherits the same path:** WebView uses the Chromium accessibility state shared with the Chrome build it ships; the media query is "Widely available" across Android WebView (caniwebview.com). The repo's WebView floor is Chromium 74, so the media query is supported on every target device.

## 3. Pathway comparison (trial trigger options)

| # | Trigger pathway | Sets `ANIMATOR_DURATION_SCALE`? | Exercises real media query? | Reliable on API 29+? | Census-clean? |
|---|---|---|---|---|---|
| 1 | **Accessibility → "Remove animations" (Android 9+)** | Yes (sets to 0) | Yes (Chromium reads it) | Setting exists, but **issue 229628568**: turning off Developer options turns off "remove animations"; OEM menus vary (Samsung "Visibility enhancements", Xiaomi "Additional settings") | Yes (no code change) |
| 2 | **Developer options → "Animation scale = off"** (window/transition/animator all 0) | Yes (sets to 0) | Yes | Only if Developer options enabled; not a user-facing a11y setting | Yes (no code change) |
| 3 | **`adb shell settings put global animator_duration_scale 0`** | Yes (direct) | Yes (Chromium reads the same value) | **Yes — deterministic**; independent of UI state and OEM gating | Yes (no code change) |
| 4 | Render-time forced class (`html.reduced-motion` toggled in JS) | n/a | No — bypasses the media query | n/a | **No — violates S5** (host census counts `animation`/`transform` decls; only the freeze-guard exception is allowed) |
| 5 | Chrome DevTools "Emulate CSS media feature prefers-reduced-motion" | n/a (emulation) | Emulated, not OS | DevTools only, not on-device | Yes, but not a device trial |

**Key caveat (issue 229628568, issuetracker.google.com/issues/229628568):** "Turning off 'Developer options' turns off 'remove animations'." and "Accessibility does seem to be modifying the same settings as the duration scale." So the accessibility toggle's effect is gated on Developer options being enabled on some Android builds — a real reliability gap on OEM skins (Xiaomi HyperOS on the reference device is a candidate). Android 12 also introduced the same "Remove animations" accessibility preference (issue 161675988); Compose honors it from 1.2.0. No separate "reduce motion" setting exists on API 29–33 beyond "Remove animations" (Android 16 adds further motion-sickness options, out of repo scope).

## 4. What Android 12+ / API 29–33 actually exposes

Only one user-facing motion setting exists in this range: **"Remove animations"** (Accessibility). There is no distinct "reduce motion" toggle, no `Settings.Secure` reduced-motion flag consumed by Chromium, and no WebView-specific override. Chromium on Android has historically read only `ANIMATOR_DURATION_SCALE` (issue 41319571), so the accessibility toggle and the developer scale converge on one underlying value — pathways 1 and 2 are two UIs over the same control.

## 5. Census-clean constraint (S5)

The freeze leg must exercise the **real** OS/media-query pathway. Injecting a forced-reduced-motion class at render time (pathway 4) would let the trial pass without the media query firing and would break the host census (SC-001 item (v) counts `@keyframes`/`animation`/`transform` declarations inside the audited file and only allows the freeze-guard exception). It is therefore forbidden as a trial trigger. Pathways 1–3 and 5 are census-clean because they change no surface code.

## 6. ISO/NHTSA/IVIS references (animation minimization)

I could not verify a specific ISO or NHTSA clause that mandates animation minimization in IVIS. ISO 15008 (road-vehicle visual presentation ergonomics) addresses legibility, luminance, color and viewing conditions; NHTSA's Visual-Manual Driver Distraction Guidelines (2013) address glance duration and task time. Neither, in the sources I could retrieve, contains a reduced-motion/animation-minimization requirement. The operative, verifiable support for honoring OS reduced motion in-vehicle is WCAG 2.3.3 (AAA, vestibular severity up to bed rest) + the sanctioned C39/SCR40 mechanism, plus the empirical simulator evidence (AHFE 2024 N=53 cluster-position capture; DDI 2024 detection-response-time) already cited in `clarify-e2-motion-accessibility.md` §4. **Mark the ISO/NHTSA claim as unverified/partial**; do not cite a specific clause.

## Recommendation

**Sanction for the SC-007 freeze leg, in this order:**

1. **Primary (on the reference device, API 33):** enable **Settings → Accessibility → "Remove animations"** (pathway 1), record the toggle state + Developer options state in the trial sheet, and verify the pill freezes at a visible track position (10/10) with D5 announcements unchanged. This is the user-facing, WCAG-sanctioned (C39/SCR40) pathway and matches the spec's literal wording.
2. **Deterministic shim (if pathway 1 is flaky on the device build, or for reproducible re-runs):** `adb shell settings put global animator_duration_scale 0` (and, for completeness, `window_animation_scale 0` + `transition_animation_scale 0`), then re-verify. This sets the exact underlying value Chromium reads (pathway 3) and is independent of the accessibility toggle's OEM/Developer-options gating bug (issue 229628568). It is not a fake trigger — it is the canonical control the accessibility toggle itself writes.
3. **If neither produces `prefers-reduced-motion: reduce` on a given build** (e.g., an OEM skin that ignores the setting): **do not** use a render-time forced class (forbidden, §5). Instead, demote the freeze leg to a **measured exception** documented in the trial sheet (device build, settings state, observed behavior) and keep the host census (SC-001 item (v)) as the regression guard. The freeze leg then reports "not exercisable on this build" rather than a false pass.

**Residual uncertainty:** whether the reference device's HyperOS build honors the accessibility toggle without Developer options enabled (issue 229628568). The trial protocol must record the Developer options state alongside the toggle state so a "freeze not observed" result is attributable to the OS, not to the implementation.

## Suggested spec wording (SC-007 freeze leg, trial protocol)

> Freeze leg — with the reference device's OS reporting `prefers-reduced-motion: reduce` (verified by enabling Settings → Accessibility → "Remove animations"; if the toggle is unreliable on the build, by `adb shell settings put global animator_duration_scale 0` as the sanctioned deterministic shim — never by a render-time forced class), the indeterminate sliding pill freezes at a visible on-track position (no disappear, no opacity trick) in 10/10 trials, and all D5 TalkBack announcements remain unchanged. Record toggle state + Developer options state + trigger pathway used in the trial sheet.