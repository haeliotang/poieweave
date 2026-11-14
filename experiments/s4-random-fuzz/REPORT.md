# S4 Random Combined Fuzz — report

- Date: 2026-11-14
- Status: deterministic combined fuzz; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-random-fuzz/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=100 cases=900 matched=900 mismatched=0
```

## 2. Findings

- After the capability-verification fix, 100 seeded random combined mutations
  show no further Rust/Python divergence across 900 verdicts.
- The fuzz mixes equipment, inventory, process-evidence and capability-evidence
  mutations in one model.

## 3. Limits

- One seed and 100 models.
- No time/occupancy/yield mutations.
- Not a formal equivalence proof.
