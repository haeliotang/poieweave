# S2 Canonical C5 Migration — report

- Date: 2026-11-16
- Status: EXPECTED agreement on horizon and final energy for 4/4 intervals
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c5`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
interval_days	legacy_days	canonical_days	delta_days	legacy_energy_kwh	canonical_energy_kwh	canonical_termination	legacy_events	canonical_events	failure_times	first_divergence	classification
10	40	40	0.0	200.0	200.0	blocked:bronze-bar	6	10	match	event_count	EXPECTED_AGREEMENT
15	60	60	0.0	200.0	200.0	blocked:bronze-bar	6	10	match	event_count	EXPECTED_AGREEMENT
20	80	80	0.0	200.0	200.0	blocked:bronze-bar	6	10	match	event_count	EXPECTED_AGREEMENT
30	90	90	0.0	200.0	200.0	completed	4	8	match	event_count	EXPECTED_AGREEMENT
```

## 2. Findings

- Horizons and final energy agree on all four intervals.
- Failure timestamps match exactly.
- Event counts differ by the canonical `repair_completed` records and the initial
  repair event; this is logging granularity, not a state divergence in the
  compared fields.
- First divergence in raw trace length is `event_count`; it is not hidden.

## 3. Classification

- `EXPECTED_AGREEMENT` for the C5 claim (horizon, energy, failure timing).
- Trace representation is not identical; future certificates must bind the
  canonical trace schema rather than claim byte-identical legacy logs.

## 4. Limits

- Summary/state comparison only; no per-event full-state diff.
- C5 is a deterministic model result, not a physical forecast.
