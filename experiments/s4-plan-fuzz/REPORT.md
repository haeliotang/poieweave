# S4 Systematic Plan Fuzzing — report

- Date: 2026-11-14
- Status: negative mutation suite; model-level only
- Runner: `cargo run --locked -q -p poie-viability --bin poie-closure-corpus-export`
  then `python3 experiments/s4-plan-fuzz/fuzz.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
mutations=130 rejected=130
```

## 2. Findings

- Every RESTORABLE plan in the 112-case corpus was mutated five ways.
- All 130 mutations were rejected by the independent plan verifier.
- This generalises the earlier three hand-written negative cases to the full
  corpus and shows the verifier is not vacuous.

## 3. Limits

- Mutations are structural; no search for physically plausible errors.
- The verifier still trusts step order and does not re-solve closure.
- No yield, capability contention or physical execution model.
