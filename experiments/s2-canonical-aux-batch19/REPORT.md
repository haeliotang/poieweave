# Canonical Auxiliary Batch 19 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch19`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
regeneration Base-06  25/24 -> 25/24
regeneration Base-07  25/25 -> 25/24
resupply before       25/24 -> 25/24
resupply after        25/25 -> 25/25
```

## 2. Finding

Base-07 joint coverage loses one pair under canonical t=0 repair-event ordering;
the independent envelope is unchanged at 25. This is a joint-coverage semantic
correction, not an enforcement failure.

## 3. Classification

- Base-07: `SEMANTIC_CORRECTION_JOINT_COVERAGE`
- other rows: `EXPECTED_AGREEMENT`

## 4. Remaining

`adversarial-rand` and `recycle-residue`; `metrics` and `order-sensitivity` are
definitional/negative records.
