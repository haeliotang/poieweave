# S4 Degraded+Process Mutations — report

- Date: 2026-11-14
- Status: combined degradation/process differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-degraded-process/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=63 cases=567 matched=567 mismatched=0
```

## 2. Findings

- A degraded asset plus a removed process still yields identical Rust/Python
  verdicts across 567 cases.
- This combines intermediate degradation with missing production/repair.

## 3. Limits

- One asset + one process.
- Not a formal equivalence proof.
