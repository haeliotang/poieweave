# Canonical Auxiliary Batch 6 — report

- Status: EXPECTED_AGREEMENT 9/9
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch6`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
redundancy baseline        60 -> 60
redundancy redundant-pump  90 -> 90
long-horizon               40 -> 40
recharge-rule Base-08      1.3/2.6/5.1 -> same
recharge-rule Base-05      1.3/2.6/5.1 -> same
```

## 2. Classification

`EXPECTED_AGREEMENT` 9/9.

## 3. Limits

Sample only; remaining auxiliary experiments are not covered.
