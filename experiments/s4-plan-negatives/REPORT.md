# S4 Plan Negative Cases — report

- Date: 2026-11-14
- Status: verifier negative controls; model-level only
- Runner: `cargo run --locked -q -p poie-viability --bin poie-plan-export`
  then `python3 experiments/s4-plan-negatives/negative.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
dropped_step: rejected (target lathe-1 was not restored by any step)
wrong_duration: rejected (duration mismatch: steps 10800.0, reported 10801.0)
unknown_process: rejected (unknown process does-not-exist)
mutations=3 rejected=3
```

## 2. Findings

- The verifier rejects a plan that omits the restoring step.
- It rejects a duration mismatch between step sum and reported total.
- It rejects a step naming a process not present in the model.
- This shows the S4 plan check is not vacuously green.

## 3. Limits

- Mutations are hand-designed, not systematically generated.
- No fuzzing or adversarial search.
- The verifier still trusts step order and does not model physical feasibility.
