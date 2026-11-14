# S4 Capability Subsets — report

- Date: 2026-11-14
- Status: multi-capability downgrade differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-capability-subsets/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=31 cases=279 matched=279 mismatched=0
```

## 2. Findings

- All 31 non-empty capability subsets agree between Rust and Python.
- This extends the single-capability fix to combined capability-evidence loss.

## 3. Limits

- Base-05 only.
- Process evidence unchanged.
- Not a formal equivalence proof.
