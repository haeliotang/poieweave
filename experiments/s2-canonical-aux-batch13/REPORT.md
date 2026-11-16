# Canonical Auxiliary Batch 13 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch13`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
adversarial-failures: 15/16 day-level agreement
adversarial-subsets:  day 0 and day 10 minimal filter-1, day-level agreement
```

Single difference: `pump-p1+robot-m1` is 60 days legacy but 0.125 days
canonical. The pump is re-failed at t=0 after initial repair while the repair
robot is also failed, so the repair executor cannot continue.

## 2. Classification

- `pump-p1+robot-m1`: `SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME`
- all other rows: `EXPECTED_AGREEMENT`
- adversarial subsets: day-level filter-1 agreement

## 3. Limits

Sample only.
