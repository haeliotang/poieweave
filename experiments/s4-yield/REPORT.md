# S4 Yield Mutations — report

- Date: 2026-11-14
- Status: yield differential; model-level only
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-yield/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=14 cases=126 matched=126 mismatched=0
```

## 2. Findings

- With both implementations applying `yield` (good fraction + reject share) and
  repeating sub-unity-yield producers, Rust and Python agree on all 126 cases.
- Python's solver was extended with `add_outputs` to match the fixed Rust
  closure semantics.
- The mutation also inserts a reject resource matching each output's unit/mass,
  satisfying model validation.

## 3. Limits

- One process at a time; yields 0.9 and 0.5.
- Not a formal equivalence proof.
