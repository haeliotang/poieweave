# S4 Cross Mutations — report

- Date: 2026-11-14
- Status: combined mutation differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-cross-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=56 cases=504 matched=504 mismatched=0
```

## 2. Findings

- Removing one inventory item and one process simultaneously still yields
  identical Rust/Python verdicts across all 504 cases.
- This exercises combined missing-input and missing-production paths.
- The mutation corpus now covers single, pair, and cross lot/process changes.

## 3. Limits

- Exactly one lot and one process.
- No triples, malformed TOML or timing changes.
- Not a formal equivalence proof.
