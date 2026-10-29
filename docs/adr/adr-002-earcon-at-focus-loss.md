# ADR-002: Earcon emission at focus loss

- **Status**: Accepted
- **Date**: 2026-10-26
- **Feature**: `005-adopt-design-sona`

## Context

Ostinato/Sforzando earcons fire AT the focus-loss state change (same call as the pause), following ADR-001's deliberate platform deviation (Tauri Platform Contract, Audio Focus / Driving-Safety). ISO 15006 short-term-signal class.

## Decision

Emission occurs at the state-change shot; failure latches until state end (`reason_ttl_ms = null`); cleanup ONLY by new document/session/app-restart (F4 contract).

## Consequences

No auto-resume; typed diag event on emission failure.
