# Canonical Auxiliary Batch 12 — report

- Status: EXPECTED_AGREEMENT 8/8
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch12`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
regen-forced-cost  1/10/50/100 -> 10/10/10/0
regen-capacity     200/400 rejected; 500/600 -> first blocked 521
```

## 2. Classification

`EXPECTED_AGREEMENT` 8/8, including the initial-energy-over-capacity rejection
and the 521 multiplier threshold.

## 3. Limits

Sample only.
