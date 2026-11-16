# S2 Canonical C18 Migration — report

- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c18`
- Seed/sample: [SEED.md](SEED.md)
- Output: [expected.txt](expected.txt)

## 1. Result

```text
trials  old_zero  new_zero  canonical_zero  old_mean  new_mean  canonical_mean
200     129       0         0               1.4       9.9       5.4

series     p10  p50  p90
old        0.0  0.0  4.0
new        5.2  10.4 15.7
canonical  5.2  5.2  6.0
```

## 2. Findings

- Zero-day rate agrees with event-driven (0/200), not with instant (129/200).
- Mean changes 9.9 → 5.4 under canonical.
- Median changes 10.4 → 5.2; p90 changes 15.7 → 6.0.
- Cause: the event-driven legacy defers failures that occur during the 5-day
  repair, while canonical interrupts at the real event time. This shortens the
  distribution but does not create zero-day outcomes.
- Classification: `SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`.

## 3. Limits

- One fixed seed; no seed sweep.
- Model-level; no physical claim.
