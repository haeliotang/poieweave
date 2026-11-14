# S4 Degraded Triples — report

- Date: 2026-11-14
- Status: three-asset degradation differential
- Runner: `cargo build -p poie-viability --bin poie-model-corpus`
  then `python3 experiments/s4-degraded-triples/mutate.py target/debug/poie-model-corpus`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutated_models=84 cases=756 matched=756 mismatched=0
```

## 2. Findings

- Every triple of degraded assets yields identical Rust/Python verdicts across
  756 cases.
- This extends multi-asset degradation from pairs to triples.

## 3. Limits

- Triples only.
- Base-05 only.
- Not a formal equivalence proof.
