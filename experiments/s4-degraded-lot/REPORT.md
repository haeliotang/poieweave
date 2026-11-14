# S4 Degraded+Lot Mutations — report

- Date: 2026-11-14
- Status: combined degradation/inventory differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-degraded-lot/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=72 cases=648 matched=648 mismatched=0
```

## 2. Findings

- A degraded asset plus a missing accepted lot still yields identical
  Rust/Python verdicts across 648 cases.
- This combines the intermediate degraded status with inventory loss.

## 3. Limits

- One asset + one lot.
- Not a formal equivalence proof.
