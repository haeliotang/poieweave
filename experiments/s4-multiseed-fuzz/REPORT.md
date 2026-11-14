# S4 Multi-Seed Fuzz — report

- Date: 2026-11-14
- Status: multi-seed combined fuzz; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-multiseed-fuzz/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
seeds=5 mutated_models=500 cases=4500 matched=4500 mismatched=0
```

## 2. Findings

- Five independent seeds of combined mutations show no further divergence.
- Together with the single-seed run, this strengthens confidence in the
  capability-verification fix, though it is not a proof.

## 3. Limits

- Five seeds, one model family.
- Same mutation generator.
- Not a formal equivalence proof.
