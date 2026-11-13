# S4 Asset-Pair Mutations — report

- Date: 2026-11-14
- Status: multi-asset failure differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-asset-pairs/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=36 cases=324 matched=324 mismatched=0
```

## 2. Findings

- Two simultaneous pre-failed assets still preserve Rust/Python verdict
  agreement across 324 cases.
- This extends the single-asset corpus to multi-failure states.

## 3. Limits

- Asset pairs only.
- No inventory/process combinations.
- Not a formal equivalence proof.
