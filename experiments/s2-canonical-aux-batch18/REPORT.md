# Canonical Auxiliary Batch 18 — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch18`
- Output: [expected.txt](expected.txt)

## 1. Result

32 timing rows (16 sets x day 0/10): 30 `EXPECTED_AGREEMENT`, 2 semantic.

The two differences are both `pump-p1+robot-m1`:

```text
day 0   legacy 60, canonical 0.125
day 10  legacy 60, canonical 10
```

The pump is failed while its repair executor robot is also failed at the same
timestamp, so the canonical repair cannot proceed.

## 2. Classification

`SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME` for those two rows.

## 3. Limits

The original expected file also prints a `worst` row per day; this batch keeps
the 16 concrete set rows per day.

## 4. Remaining

`adversarial-rand`, `recycle-residue`, `regeneration`, `resupply`; `metrics`
and `order-sensitivity` are definitional/negative.
