# S4 Degraded Pairs — report

- Date: 2026-11-14
- Status: two-asset degradation differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-degraded-pairs/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=36 cases=324 matched=324 mismatched=0
```

## 2. Findings

- Every pair of degraded assets yields identical Rust/Python verdicts across
  324 cases.
- This adds multi-asset degradation, not just one-at-a-time.

## 3. Limits

- Pairs only.
- Base-05 only.
- Not a formal equivalence proof.
