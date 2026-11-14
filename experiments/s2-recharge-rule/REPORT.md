# S2 Recharge Rule — report

- Date: 2026-11-14
- Status: causal confirmation of the recharge rule
- Runner: `cargo run --locked -q -p poie-viability --bin poie-recharge-rule`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recharge_kwh	average_kw	first_energy_limited_load_kw
10	1.25	1.3
20	2.50	2.6
40	5.00	5.1
```

## 2. Finding

- The first energy-limited load tracks average recharge power almost exactly:
  +0.1 kW at the 0.1 kW probe resolution.
- This is causal evidence for the rule: sustained load above average recharge
  power eventually depletes storage.
- It also explains why capacity and charge-power caps did not move the boundary:
  the modelled recharge process fixes the average inflow.

## 3. Significance

- The 2.6 kW switch is a consequence of `20 kWh / 8 h`, not a fitted constant.
- The +0.1 kW offset is consistent with probe resolution/rounding, not a second
  mechanism.

## 4. Limits

- One schedule and one horizon.
- Recharge output varied directly; no irradiance model.
