# Canonical Auxiliary Batch 9 — report

- Status: EXPECTED_AGREEMENT 10/10
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch9`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
insert-bound 2/3/4/6/8/12 -> 40/50/60/60/60/60
joint-load 2 kW x bronze 3/6 -> 60/90
joint-load 3 kW x bronze 3/6 -> 3 day (canonical 3.292)
```

## 2. Classification

`EXPECTED_AGREEMENT` 10/10 at day granularity.

## 3. Limits

Sample only.
