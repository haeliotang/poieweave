# Canonical Auxiliary Batch 3 — report

- Status: EXPECTED_AGREEMENT 14/14
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch3`
- Output: [expected.txt](expected.txt)

## 1. Coverage

- `s2-joint-boundary`: 3 loads x 2 bronze stocks = 6 rows
- `s2-joint-consumables`: 2 insert stocks x 4 bronze stocks = 8 rows

## 2. Result

All 14 rows agree at day granularity:

- joint-boundary: 60/90 at 2.4/2.5 kW; 16 days at 2.6 kW
- joint-consumables: 30/40/60/90 by bronze stock
- 2.6 kW energy-limited rows shift 16 -> 16.25 d, same day

## 3. Classification

`EXPECTED_AGREEMENT`; the 0.25 d shift is the full repair-chain schedule.

## 4. Limits

Sample only; 52 auxiliary experiments remain uncanonicalised.
