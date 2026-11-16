# Canonical Auxiliary Batch 16 — report

- Status: EXPECTED_AGREEMENT 10/10
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch16`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
grease-cost       0.10/0.15/0.20 kg x cheap/expensive -> 20/0, 30/0, 30/10
yield-accumulation 1.00/0.90/0.75/0.50 -> 60/60/60/50
```

## 2. Classification

`EXPECTED_AGREEMENT` 10/10.

## 3. Limits

`adversarial-rand`, `adversarial-timing`, `recycle-residue`, `regeneration`,
`resupply`, `two-restorers` remain without canonical evidence; `metrics` and
`order-sensitivity` are definitional/negative records.
