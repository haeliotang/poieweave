# Canonical Auxiliary Batch 10 — report

- Status: EXPECTED_AGREEMENT 18/18
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch10`
- Output: [expected.txt](expected.txt)

## 1. Coverage

- joint-three: bronze 3/6 x energy 20/40/80 = 6 rows
- joint-worn: worn 0/2/4 x bronze 3/6 = 6 rows
- joint-yield: yield 1.0/0.8/0.5 x bronze 3/6 = 6 rows

## 2. Result

All 18 rows agree at day granularity, including the 0.5-yield sharpen
`YieldSpec` path: 50/60/70/90 for joint-worn, and 60/90, 60/80, 50/50 for
joint-yield.

## 3. Classification

`EXPECTED_AGREEMENT` 18/18.

## 4. Limits

Sample only.
