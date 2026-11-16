# S2 Canonical C13 Migration — report

- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c13`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load  instant  event   canonical  energy  termination          classification
2.6   16       16.250  16.250     0.0     energy_depleted     EXPECTED_AGREEMENT
3.0   3        3.292   3.333      0.0     energy_depleted     SEMANTIC_CORRECTION_REPAIR_SCHEDULE
4.0   1        1.097   1.111      0.0     energy_depleted     SEMANTIC_CORRECTION_REPAIR_SCHEDULE
```

## 2. Findings

- Day-level energy boundary is invariant: 16/3/1 days.
- Final energy is zero in all cases.
- Exact depletion shifts by at most 1 h (0.041 d) because canonical schedules the
  full repair chain, including the lathe/gauge steps, while the legacy
  event-driven initial repair uses the closure plan duration directly.
- No horizon-order change.

## 3. Classification

- 2.6 kW: `EXPECTED_AGREEMENT`.
- 3.0/4.0 kW: `SEMANTIC_CORRECTION_REPAIR_SCHEDULE` (day-level invariant, exact
  time shifted).

## 4. Limits

- Three loads; exact shift depends on the scheduled repair chain.
