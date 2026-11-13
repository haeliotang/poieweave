# S4 Triple-Lot Mutations — report

- Date: 2026-11-14
- Status: triple-lot differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-triple-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=56 cases=504 matched=504 mismatched=0
```

## 2. Findings

- Removing three inventory items simultaneously still preserves Rust/Python
  verdict agreement across all 504 cases.
- Together with pairs and cross lot/process mutations, the corpus now spans
  increasingly severe inventory depletion.
- No new divergence surfaced.

## 3. Limits

- Exactly three lots.
- No process combinations or malformed models.
- Not a formal equivalence proof.
