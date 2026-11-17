"""RED/GREEN invariant contract for Spec 012 T018/T024/T067 normal device path.

Host-testable static invariant (no adb): hook must clear authoritative debug
cache before each operation capture in normal adb/device-execution path to
prevent stale cross-op attribution (integrity: FR-008 correlation/attribution).
"""
from __future__ import annotations

from pathlib import Path
import re

REPO = Path(__file__).resolve().parents[3]
HOOK = REPO / "scripts" / "automate-t184-t226-hook.sh"


def test_hook_clears_cache_before_t184_s1_document_capture():
    s = HOOK.read_text(encoding="utf-8")
    # T184 Scenario 1 sends document intent and reads cache; require rm/clear of
    # cache/debug_ingest_result.txt before that document intent/capture sequence.
    i = s.find("T184 Scenario 1: open new document")
    assert i >= 0, "T184 Scenario 1 block not found"
    j = s.find('am start -n "$PKG/.DebugIngestActivity" --es mode document --es path $FIX_BASE/t184-teste.md', i)
    assert j > i, "T184 Scenario 1 document intent not found"
    window = s[i:j]
    assert 'cache/debug_ingest_result.txt' in window and ('rm -f' in window or 'rm ' in window), (
        "normal path must clear cache/debug_ingest_result.txt before T184 Scenario 1 capture"
    )


def test_hook_clears_cache_before_each_directory_and_reopen_capture():
    s = HOOK.read_text(encoding="utf-8")
    for label in ["T184 Scenario 2", "T226: directory ingest"]:
        i = s.find(label)
        assert i >= 0, label
        window = s[i : i + 500]
        assert 'rm -f cache/debug_ingest_result.txt' in window or 'run-as $PKG rm -f cache/debug_ingest_result.txt' in window, (
            f"{label} must clear authoritative cache before capture"
        )
