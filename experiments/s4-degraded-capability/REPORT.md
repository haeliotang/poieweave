# S4 Degraded+Capability Mutations — report

- Date: 2026-11-14
- Status: combined degradation/capability differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-degraded-capability/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=45 cases=405 matched=405 mismatched=0
```

## 2. Findings

- A degraded asset plus a weakened (declared) capability entry still yields
  identical Rust/Python verdicts across 405 cases.
- This combines intermediate degradation with capability-evidence loss.

## 3. Limits

- One asset + one capability.
- Not a formal equivalence proof.
