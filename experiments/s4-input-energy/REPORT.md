# S4 Energy Input Mutations — report

- Date: 2026-11-14
- Status: energy-input differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-input-energy/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=7 cases=63 matched=63 mismatched=0
```

## 2. Findings

- Doubling any one process's electric-energy input preserves Rust/Python verdict
  agreement across 63 cases.
- This adds an input-quantity axis to the S4 corpus.

## 3. Limits

- One process at a time.
- Only electric-energy input.
- Not a formal equivalence proof.
