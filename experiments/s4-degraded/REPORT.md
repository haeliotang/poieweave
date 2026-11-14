# S4 Degraded Status Mutations — report

- Date: 2026-11-14
- Status: degraded-status differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-degraded/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=9 cases=81 matched=81 mismatched=0
```

## 2. Findings

- Setting any one asset to `degraded` preserves Rust/Python verdict agreement
  across 81 cases.
- This adds the intermediate `degraded` status to the S4 corpus, distinct from
  `failed`.

## 3. Limits

- One asset at a time.
- Base-05 only.
- Not a formal equivalence proof.
