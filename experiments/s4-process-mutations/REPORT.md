# S4 Process Mutations — report

- Date: 2026-11-14
- Status: process-removal differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-process-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=7 cases=63 matched=63 mismatched=0
```

## 2. Findings

- Removing any one of the seven Base-05 processes gives 63 target/mutation
  verdicts.
- Rust and Python agree on all 63.
- Combined with one-lot removal, the mutation corpus now covers both missing
  inventory and missing production/repair processes.

## 3. Limits

- One process at a time.
- No malformed TOML, multi-process removal or timing mutation.
- Not a formal equivalence proof.
