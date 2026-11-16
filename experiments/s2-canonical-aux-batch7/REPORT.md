# Canonical Auxiliary Batch 7 — report

- Status: EXPECTED_AGREEMENT 17/17
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch7`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
regen-threshold-battery   capacities 100/200/300/400 -> 60/61
regen-threshold-initial   initial 20/40/60/80/120 -> 41/61/81/101/141
regen-event               Base-05 40/60/80/90, Base-08 60/90/90/90
```

## 2. Classification

`EXPECTED_AGREEMENT` 17/17.

## 3. Limits

Sample only; remaining auxiliary experiments are not covered.
