# S4 Quantity Ratios — report

- Date: 2026-11-14
- Status: multi-level partial-inventory differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-quantity-ratios/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=24 cases=216 matched=216 mismatched=0
```

## 2. Findings

- Inventory levels at 25%, 50% and 75% all yield identical Rust/Python
  verdicts across 216 cases.
- This adds a small quantity-resolution axis to the S4 corpus.

## 3. Limits

- One lot at a time.
- Three ratios only.
- Not a formal equivalence proof.
