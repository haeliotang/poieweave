# Canonical Auxiliary Batch 4 — report

- Status: EXPECTED_AGREEMENT 13/13
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch4`
- Output: [expected.txt](expected.txt)

## 1. Coverage

- `s2-regen-cost`: 3 duration x 3 energy multipliers = 9 rows
- `s2-energy-recharge`: 20/30/40 kWh = 3 rows
- `s2-recharge-bound`: average 2.50 kW vs threshold 2.6 kW = 1 row

## 2. Result

All 13 rows agree at day granularity:

- regen-cost: 60 days for all 9 combinations (cost not binding)
- recharge 20 kWh: 16 days; 30/40 kWh: 60 days
- recharge-bound: first energy-limited load 2.6 kW in both paths
- energy-limited exact time shifts 16 -> 16.25 d

## 3. Classification

`EXPECTED_AGREEMENT`; 0.25 d shift is the full repair-chain schedule.

## 4. Limits

Sample only; 51 auxiliary experiments remain uncanonicalised.
