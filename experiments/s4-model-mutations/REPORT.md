# S4 Model Mutations — report

- Date: 2026-11-14
- Status: differential model-mutation check; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-model-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=8 cases=72 matched=72 mismatched=0
```

## 2. Findings

- Removing any one accepted lot from Base-05 yields 72 target/mutation verdicts.
- Rust and the independent checker agree on all 72.
- This adds a different mutation axis from the asset-failure corpus: inventory
  removal rather than failure injection.
- It also exercises missing-resource paths across multiple targets.

## 3. Limits

- Only accepted-lot removal.
- No process removal, model corruption, multi-lot mutation or timing changes.
- Not a formal equivalence proof.
