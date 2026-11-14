# S4 Duration+Quantity Mutations — report

- Date: 2026-11-14
- Status: combined timing/partial-inventory differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-duration-quantity/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=56 cases=504 matched=504 mismatched=0
```

## 2. Findings

- Doubling one process duration while halving one lot still yields identical
  Rust/Python verdicts across 504 cases.
- This combines timing with partial inventory loss.

## 3. Limits

- One process + one lot.
- One duration/quantity ratio.
- Not a formal equivalence proof.
