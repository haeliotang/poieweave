# S2 Canonical C10 Migration — report

- Date: 2026-11-16
- Status: EXPECTED_AGREEMENT 4/4
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c10`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
interval  legacy  canonical  energy   termination
10        60      60         200.0    cycle
20        50      50         200.0    cutting-insert
30        40      40         200.0    cutting-insert
60        30      30         200.0    cutting-insert
```

## 2. Findings

- Resupply intervals 10/20/30/60 days map to 60/50/40/30 days.
- Final energy and failure/blocked terminal state match.
- Classification: `EXPECTED_AGREEMENT`.

## 3. Limits

- Single consumable; model-level units.
