# Canonical Auxiliary Batch 8 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch8`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
regen-long Base-05  8/12/20/40 -> canonical 2/3/15/35
regen-long Base-08  12/18/30/60 -> canonical 2/3/15/35
joint-double-failure 10 -> 10.25 day
```

## 2. Findings

- Dense 5-day repairs with 2/3/5/10-day failures are interrupted at event time;
  the canonical horizon is the first failure for 2/3-day intervals and 15/35 days
  for 5/10-day intervals.
- Joint double-failure remains day-level 10; canonical 10.25 d reflects the
  scheduled repair chain.

## 3. Classification

- regen-long: `SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`
- joint-double-failure: `SEMANTIC_CORRECTION_REPAIR_SCHEDULE`

## 4. Limits

Sample only.
