# S2 Energy Interventions at the Switch — report

- Date: 2026-11-14
- Status: capacity and charge power do not move the 2.6 kW horizon
- Runner: `cargo run --locked -q -p poie-viability --bin poie-energy-interventions`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	capacity_kwh	charge_kw	survived_days	blocked_by
2.6	100	3	16	energy_depleted
2.6	100	10	16	energy_depleted
2.6	100	30	16	energy_depleted
2.6	200	3	16	energy_depleted
2.6	200	10	16	energy_depleted
2.6	200	30	16	energy_depleted
2.6	300	3	16	energy_depleted
2.6	300	10	16	energy_depleted
2.6	300	30	16	energy_depleted
```

## 2. Findings

- All nine combinations stop at 16 days with `energy_depleted`.
- Capacity 100-300 kWh and charge 3-30 kW do not change the horizon.
- In this model, the binding quantity is the recharge process's average output
  (20 kWh / 8 h), not storage size or charge cap.

## 3. Significance

- This is a negative intervention result: at the energy switch, storage/charge
  knobs are inert in the tested range.
- It reinforces the recharge rule: to move the switch, change recharge output or
  reduce sustained load, not battery size.

## 4. Limits

- One load; model-level units.
