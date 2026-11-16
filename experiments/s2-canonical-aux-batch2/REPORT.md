# Canonical Auxiliary Batch 2 — report

- Status: EXPECTED_AGREEMENT 19/19
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch2`
- Output: [expected.txt](expected.txt)

## 1. Coverage

- `s2-resupply-time`: 4 interval rows
- `s2-resupply-delivery`: 3 stock/resupply rows
- `s2-regen-cross`: 12 yield x interval rows

## 2. Result

All 19 rows agree exactly:

```text
resupply-time      60/50/40/30
resupply-delivery  30/60/40
regen-cross        yields 1.00/0.75/0.50, intervals 10/15/20/30
```

The `one-insert-resupply` row is 40 days with deliveries at day 30 and 60.

## 3. Classification

`EXPECTED_AGREEMENT` for all 19 rows.

## 4. Limits

Sample only; 54 auxiliary experiments remain uncanonicalised.
