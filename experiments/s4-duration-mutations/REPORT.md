# S4 Duration Mutations — report

- Date: 2026-11-14
- Status: duration differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-duration-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=7 cases=63 matched=63 mismatched=0
```

## 2. Findings

- Doubling any one process duration preserves Rust/Python verdict agreement
  across 63 cases.
- This adds a timing axis to the S4 corpus.

## 3. Limits

- One process at a time.
- Only 2× duration.
- Not a formal equivalence proof.
