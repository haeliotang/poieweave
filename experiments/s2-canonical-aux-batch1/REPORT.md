# Canonical Auxiliary Batch 1 — report

- Status: EXPECTED_AGREEMENT 20/20
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch1`
- Output: [expected.txt](expected.txt)

## 1. Coverage

- `s2-energy-boundary`: 2/3 kW x 100/200/300 kWh x 3/10/30 kW = 18 rows
- `s2-heldout`: Base-05 intervals 12 and 25 days

## 2. Result

All 20 rows agree at day granularity:

- load 2 kW: 60 days, capacity/charge inert
- load 3 kW: 3 days legacy, 3.292 canonical (energy-limited)
- held-out interval 12: 48 days
- held-out interval 25: 90 days

## 3. Classification

`EXPECTED_AGREEMENT`; exact energy-limited times differ only because canonical
schedules the full repair chain (3 vs 3.292 d).

## 4. Limits

This is a sample, not migration of all 59 auxiliary experiments.
