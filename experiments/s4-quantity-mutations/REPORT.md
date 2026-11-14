# S4 Quantity Mutations — report

- Date: 2026-11-14
- Status: partial-inventory differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-quantity-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=8 cases=72 matched=72 mismatched=0
```

## 2. Findings

- Halving a single accepted lot preserves Rust/Python verdict agreement across
  72 cases.
- This covers partial inventory loss, distinct from lot removal.

## 3. Limits

- One lot at a time.
- No combinations with asset/process mutations.
- Not a formal equivalence proof.
