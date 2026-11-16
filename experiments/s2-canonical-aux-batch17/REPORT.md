# Canonical Auxiliary Batch 17 — report

- Status: EXPECTED_AGREEMENT 4/4
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-aux-batch17`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
redundant-only  30
filter-only     30
robot-only      30
filter+robot    90
```

## 2. Classification

`EXPECTED_AGREEMENT` 4/4.

## 3. Limits

`adversarial-rand`, `adversarial-timing`, `recycle-residue`, `regeneration`,
`resupply` remain without canonical evidence; `metrics` and `order-sensitivity`
are definitional/negative records.
