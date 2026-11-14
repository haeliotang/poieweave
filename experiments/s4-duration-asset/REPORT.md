# S4 Duration+Asset Mutations — report

- Date: 2026-11-14
- Status: combined timing/equipment differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-duration-asset/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=63 cases=567 matched=567 mismatched=0
```

## 2. Findings

- A failed asset plus a doubled process duration still yields identical
  Rust/Python verdicts across 567 cases.
- This combines the timing axis with equipment failure.

## 3. Limits

- One asset + one process.
- 2× duration only.
- Not a formal equivalence proof.
