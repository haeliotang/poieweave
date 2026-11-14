# S4 Quantity+Asset Mutations — report

- Date: 2026-11-14
- Status: combined partial-inventory/equipment differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-quantity-asset/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=72 cases=648 matched=648 mismatched=0
```

## 2. Findings

- A failed asset plus a halved lot still yields identical Rust/Python verdicts
  across 648 cases.
- This covers partial inventory loss combined with equipment failure.

## 3. Limits

- One asset + one lot.
- One halving ratio.
- Not a formal equivalence proof.
