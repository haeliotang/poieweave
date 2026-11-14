# S4 Quantity Pairs — report

- Date: 2026-11-14
- Status: paired partial-inventory differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-quantity-pairs/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=84 cases=756 matched=756 mismatched=0
```

## 2. Findings

- Two lots scaled together to 25/50/75% still yields identical Rust/Python
  verdicts across 756 cases.
- This extends the single-lot ratio sweep to interacting inventory loss.

## 3. Limits

- Two lots at a time.
- Shared ratio for both lots.
- Not a formal equivalence proof.
