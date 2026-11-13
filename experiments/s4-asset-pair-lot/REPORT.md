# S4 Asset-Pair+Lot Mutations — report

- Date: 2026-11-14
- Status: multi-failure + inventory mutation differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-asset-pair-lot/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=288 cases=2592 matched=2592 mismatched=0
```

## 2. Findings

- Two pre-failed assets plus one missing accepted lot still yields identical
  Rust/Python verdicts across 2592 cases.
- This is the largest combined mutation corpus so far.
- No new divergence surfaced.

## 3. Limits

- Two assets + one lot.
- No process combinations or timing changes.
- Not a formal equivalence proof.
