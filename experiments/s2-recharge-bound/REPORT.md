# S2 Recharge Bound — report

- Date: 2026-11-14
- Status: switch explained by average recharge power
- Runner: `cargo run --locked -q -p poie-viability --bin poie-recharge-bound`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recharge_output_kwh=20.0
recharge_duration_h=8.0
average_recharge_kw=2.50
first_energy_limited_load_kw=2.6
delta_kw=0.10
```

## 2. Finding

- The PV recharge process supplies 20 kWh over 8 h, i.e. **2.5 kW average**.
- The first energy-limited load is **2.6 kW**, exactly 0.1 kW above average
  recharge.
- This explains why capacity and charge power did not move the switch: the
  binding quantity is the *average recharge rate*, not storage size or charge
  cap.

## 3. Significance

- The 2.6 kW boundary is not arbitrary; it follows from the modelled energy
  inflow.
- It suggests a derived rule to test: sustained load above average recharge
  power eventually depletes storage.
- Loads at/below 2.5 kW remain material-limited in this schedule.

## 4. Limits

- One model/schedule; 0.1 kW probe resolution.
- No weather/season variation in recharge.
