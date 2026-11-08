# Android Device Automation — Research Update

## Research 4: Other feedback mechanisms

Scope: Ressoa/mdreader Android app. Covers screenshot diff, logcat parsing, monkey stress, accessibility inspector, custom debug hooks. Pros/cons per mechanism.

### 1. Screenshot diff / visual regression

Tools:
- Android Screenshot Testing (official): https://developer.android.com/training/testing/ui-tests/screenshot
- Maestro `assertScreenshot`: https://docs.maestro.dev/reference/commands-available/assertscreenshot
- Appium image comparison: https://browserstack.com/guide/appium-visual-testing
- WebdriverIO visual testing: https://webdriver.io/docs/visual-testing

Pros for Ressoa:
- Catches UI regressions (layout, theming, audio player controls) not covered by functional tests.
- Baseline images can be committed; CI can diff.
- Maestro assertScreenshot integrates with existing Maestro flows.

Cons:
- Flaky due to anti-aliasing, fonts, animations, device-specific rendering; requires tolerance thresholds.
- Baselines per device/density multiply storage.
- Audio feedback not visible; screenshot alone can't verify playback.

### 2. Logcat parsing

Tools:
- Android logcat CLI: https://developer.android.com/tools/logcat
- AOSP GenericLogcatEventParser: https://source.android.com/reference/tradefed/com/android/tradefed/util/GenericLogcatEventParser

Pros:
- Low overhead; captures crashes, ANRs, warnings, audio subsystem errors.
- Can assert absence of ERROR/exception lines after flows.
- Works headless in CI.

Cons:
- Noisy; requires filtering by tag/pid; false positives.
- Timing races; logs may arrive after test ends.
- Doesn't capture visual state.

### 3. Monkey stress (UI/Application Exerciser)

Tools:
- Android Monkey: https://developer.android.com/studio/test/other-testing-tools/monkey

Pros:
- Finds crashes/ANRs under random input cheaply.
- Reproducible via seed; good for smoke/regression.
- No test code needed.

Cons:
- Random; low coverage of specific paths; hard to assert semantic correctness (e.g., audio playback).
- Can leave app in odd state; cleanup needed.
- Not suitable for precise feedback parsing.

### 4. Accessibility inspector

Tools:
- Android Accessibility Suite / Accessibility Scanner: https://developer.android.com/guide/topics/ui/accessibility/views/testing-views
- Compose inspect & debug: https://developer.android.com/develop/ui/compose/accessibility/inspect-debug
- Espresso accessibility checking: https://developer.android.com/training/testing/espresso/accessibility-checking

Pros:
- Ensures screen reader labels, touch targets, contrast for Ressoa UI.
- Compose Layout Inspector helps debug accessibility tree.
- Automated checks catch regressions in CI.

Cons:
- Manual/subjective aspects (usability) not fully automatable.
- Adds test runtime overhead.
- May not reflect real assistive tech behavior.

### 5. Custom debug hooks

Approach: expose debug endpoints/intents in app (e.g., broadcast intents, debug menu, logging hooks) to query internal state (playback position, buffer, errors).

Pros:
- Precise, app-specific assertions (audio detection, performance/battery metrics).
- Can expose metrics not visible externally.
- Fast, deterministic.

Cons:
- Requires code changes and maintenance; risk of shipping debug code.
- Must be gated (build variants) to avoid production impact.
- Increases attack surface if not secured.

### Recommendation for Ressoa
Combine: Maestro flows + assertScreenshot for UI, logcat assertion for crashes, Monkey for nightly stress, accessibility checks in CI, and debug hooks (debug build) for audio/playback and performance/battery metrics. Use logcat + debug hooks for audio detection since screenshots cannot verify playback.

Sources:
- Android screenshot testing: https://developer.android.com/training/testing/ui-tests/screenshot
- Maestro assertScreenshot: https://docs.maestro.dev/reference/commands-available/assertscreenshot
- Appium visual testing: https://browserstack.com/guide/appium-visual-testing
- logcat: https://developer.android.com/tools/logcat
- GenericLogcatEventParser: https://source.android.com/reference/tradefed/com/android/tradefed/util/GenericLogcatEventParser
- Monkey: https://developer.android.com/studio/test/other-testing-tools/monkey
- Accessibility testing: https://developer.android.com/guide/topics/ui/accessibility/views/testing-views
- Compose inspect/debug: https://developer.android.com/develop/ui/compose/accessibility/inspect-debug
- Espresso accessibility: https://developer.android.com/training/testing/espresso/accessibility-checking
