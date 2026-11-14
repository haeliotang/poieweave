# S4 Verification Mutations — report

- Date: 2026-11-14
- Status: under-verification differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-verification-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=7 cases=63 matched=63 mismatched=0
```

## 2. Findings

- Downgrading any one process to `declared` preserves Rust/Python verdict
  agreement across 63 cases.
- This tests the under-verification path, distinct from process removal.

## 3. Limits

- One process at a time.
- Capability verification map unchanged.
- Not a formal equivalence proof.
