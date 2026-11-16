# S2 Canonical C19 Migration — report

- Date: 2026-11-16
- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c19`
- Seed/sample: [SEED.md](SEED.md)
- Output: [expected.txt](expected.txt)

## 1. Result

```text
trials  old_zero  new_zero  canonical_zero  old_mean  new_mean  canonical_mean
200     13        0         0               4.4       14.0      2.2

series     p10  p50  p90
old        1.0  6.0  6.0
new        10.4 15.6 15.6
canonical  2.0  2.0  2.0
```

## 2. Findings

- Zero-day rate agrees with event-driven: 0/200.
- Mean changes 14.0 → 2.2 under canonical.
- p10/p50/p90 all become 2.0: the delayed subset failure reaches the pump repair
  window and interrupts it at the event time.
- Classification: `SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`.

## 3. Limits

- One fixed seed; no seed sweep.
- Model-level; no physical claim.
