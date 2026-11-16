# Canonical Auxiliary Batch 14 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch14`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
adversarial-time day 0   legacy 0/5, canonical 5
adversarial-time day 10  legacy 0/5, canonical 2
regeneration-horizon     8/8 exact
regeneration-yield       60/60/50 exact
```

## 2. Findings

- Day 0 canonical min 5 matches event-driven.
- Day 10 canonical min 2 because the 2-day pump failure interrupts the 5-day
  repair; legacy ignores `at_day` and still places failures at t=0.
- regeneration-horizon and regeneration-yield agree exactly.

## 3. Classification

- adversarial-time day 0: `EXPECTED_AGREEMENT`
- adversarial-time day 10: `SEMANTIC_CORRECTION_LEGACY_IGNORED_AT_DAY`
- other rows: `EXPECTED_AGREEMENT`

## 4. Limits

Sample only.
