# S4 Asset+Lot Mutations — report

- Date: 2026-11-14
- Status: combined equipment/inventory mutation differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-asset-lot-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=72 cases=648 matched=648 mismatched=0
```

## 2. Findings

- One failed asset plus one missing lot still yields identical Rust/Python
  verdicts across 648 cases.
- This combines equipment failure and inventory depletion in the same model.
- No new divergence surfaced.

## 3. Limits

- One asset + one lot.
- No multi-asset/process combinations.
- Not a formal equivalence proof.
