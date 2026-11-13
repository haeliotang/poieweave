# S2 Self-Sustaining Metrics — report

- Date: 2026-11-14
- Status: formal metric definitions; model-level
- Runner: `cargo run --locked -q -p poie-viability --bin poie-metrics`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
survival_horizon_base05_days=40.0
survival_horizon_base08_days=60.0
recovery_coverage=5/10 ratio=0.500000
regenerative_capacity_days=20.0
```

## 2. Definitions

- Survival Horizon: service survival seconds from a `SurvivalReport`.
- Recovery Coverage: recoverable/total cases from a `FailureEnvelope`.
- Regenerative Capacity: regenerated minus baseline survival.

## 3. Findings

- The three metrics now have explicit types and tests, instead of only
  experiment-specific prints.
- Base-08 extends the 10-day horizon by 20 days with local recycling.
- Coverage remains an independent envelope (5/10), not joint survival.

## 4. Limits

- Model-level and definitional.
- No uncertainty intervals or physical calibration.
- Coverage is independent, not joint.
