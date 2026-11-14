# S2 Threshold vs Battery Capacity — report

- Date: 2026-11-14
- Status: threshold is invariant to declared battery capacity
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-threshold-battery`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
capacity_kwh	max_safe_x	first_blocked_x
100	60	61
200	60	61
300	60	61
400	60	61
```

## 2. Findings

- The 60/61 threshold does not change with battery capacity.
- The plan's energy feasibility is limited by the initial electric-energy lot
  (40 kWh / 144 MJ plus recharge flows), not by the declared battery capacity.
- This identifies a modeling gap: capacity is a bound, not an initial energy
  seed, so it cannot shift the immediate plan threshold.

## 3. Limits

- Model-level units; one forced path.
- Capacity may still matter for long-horizon recharge/load behavior.
