# S4 Pair-Lot Mutations — report

- Date: 2026-11-14
- Status: pair-lot differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-pair-mutations/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=28 cases=252 matched=252 mismatched=0
```

## 2. Findings

- Combining missing inventory items does not break the Rust/Python agreement in
  this model family.
- 28 pair mutations × 9 targets = 252 verdicts, all matching.
- This extends the one-lot corpus to interacting inventory shortages.

## 3. Limits

- Pairs only; no triples or process+lot combinations.
- No malformed models or timing changes.
- Not a formal equivalence proof.
