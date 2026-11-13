# S4 Independent Plan Verification — report

- Date: 2026-11-14
- Status: independent replay check; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-plan-export`
  then `python3 experiments/s4-independent-plan/verify.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
plans=4 verified=4
OK
```

## 2. Findings

- A separate Python implementation reads the same TOML models and replays each
  exported S1 plan step by step.
- It confirms for all four plans:
  - every process input was available at that step,
  - the final target was restored,
  - the step durations sum to the reported duration,
  - the horizon was not exceeded.
- This is independent of the Rust `analyze`/planner code path.

## 3. What is still not independent

- Plan selection/closure search still comes from Rust.
- Yield, capability contention, occupancy interactions and physical execution
  are not re-modelled.
- The verifier trusts the exported step order; it does not search alternatives.

## 4. Next

1. Independently solve a small closure case and compare verdicts.
2. Export full inventory transitions and verify them.
3. Introduce an independent process model or FMI co-simulation.
4. Add negative/mutated plan cases so the verifier is shown to reject bad plans.
