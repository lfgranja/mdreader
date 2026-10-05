# Device Fixture Traces — U1 Resolution (choice a)

Each file below is a committed, replayable trace of device-level events
that the integration/contract suites (T022b, T034, T043, T053, T070)
reference instead of requiring live Android hardware. Format: JSON Lines,
one event per line (`{ts_ms, kind, payload?, result}`), sorted by `ts_ms`.

These fixtures do not contain document text (FR-045 / SC-056 respect) and
are never transmitted off-device (FR-040). They are the evidence base for
host-runnable red-green on the device-trace replay path; the 10/10 on-device
trial path (T083 / T086) remains unchanged and is not replaced.

References:
- T022b external-start (refused / granted) → external_start_refused.jsonl
- T034 focus duck-vs-pause → focus_duck_pause.jsonl + focus_transient_permanent.jsonl
- T043 routing/becoming-noisy / device-removal → routing_noisy_removal.jsonl + headphone_disconnect.jsonl
- T053 lock-screen blind trial → lockscreen_blind.jsonl
- T070 failover + kill/power-loss → service_kill_power_loss.jsonl
