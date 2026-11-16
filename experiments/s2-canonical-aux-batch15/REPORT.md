# Canonical Auxiliary Batch 15 — report

- Status: EXPECTED_AGREEMENT 11/11
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch15`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load Base-05 0/1/2/3/4 kW -> 40/40/40/3/1
load-threshold-capacity 100/200/300 kWh -> 2.6 kW
load-threshold-charge 3/10/30 kW -> 2.6 kW
```

## 2. Classification

`EXPECTED_AGREEMENT` 11/11; 3/4 kW exact times are 3.292/1.097 d.

## 3. Limits

`yield-accumulation`, `two-restorers`, `recycle-residue`, `adversarial-rand`,
`adversarial-timing`, `regeneration`, `resupply`, `grease-cost`, `metrics`,
`order-sensitivity` remain without canonical evidence.
