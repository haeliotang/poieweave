# S2 Held-Out Failure Intervals — report

- Date: 2026-11-14
- Status: model-level held-out check; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-heldout`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
heldout_interval_days	survived_days	blocked_by
12	48	missing resource bronze-bar: need 1.2, have 0.6000000000000001
25	90	none
```

## 2. Held-out conclusion

- The 12-day interval lands at 48 days, inside the predicted 45–55 band and
  between the 10-day (40) and 15-day (60) training points.
- The 25-day interval survives the full 90-day horizon, consistent with the
  broader trend.
- The failure at 12 days is again material (`bronze-bar`), not energy.
- No parameter or policy was changed after the held-out run.

## 3. What this supports

The survival-horizon curve is not just a lookup of the four training intervals:
two unseen intervals follow the same monotone trend and material-failure mode.

## 4. Limitations

- Only two held-out points, one synthetic model, one policy.
- No statistical confidence interval or external replication.
- No operating demand, stochastic timing or physical calibration.
