# Canonical Auxiliary Batch 11 — report

- Status: EXPECTED_AGREEMENT 13/13
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch11`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
worn-bound      worn 0/2/4/8 -> 50/70/90/90
sharpen-yield   yield 1.0/0.8/0.5 -> 60/60/50
regen-cost-load duration 1/4 x load 0/2/3 -> 60/60/3 (canonical 3.292 at 3 kW)
```

## 2. Classification

`EXPECTED_AGREEMENT` 13/13 at day granularity; 3 kW exact time shifts to 3.292.

## 3. Limits

Sample only.
