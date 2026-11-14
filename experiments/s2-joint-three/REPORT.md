# S2 Three-Consumable Frontier — report

- Date: 2026-11-14
- Status: bronze dominates; energy 20-80 kWh not binding at zero load
- Runner: `cargo run --locked -q -p poie-viability --bin poie-joint-three`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
bronze_kg	energy_kwh	survived_days	blocked_by
3.0	20	60	cyclic process dependency at turn-sleeve-p1
3.0	40	60	cyclic process dependency at turn-sleeve-p1
3.0	80	60	cyclic process dependency at turn-sleeve-p1
6.0	20	90	none
6.0	40	90	none
6.0	80	90	none
```

## 2. Findings

- With sharpening enabled and no site load, initial energy 20/40/80 kWh does not
  change the horizon; PV recharge covers the plan's energy use.
- Bronze is the binding consumable: 3 kg -> 60 days, 6 kg -> full 90 days.
- This is consistent with the recharge rule: at zero load, inflow covers demand.

## 3. Significance

- Adding a third consumable does not always add a new frontier; energy stays
  non-binding while recharge covers the schedule.
- The binding frontier remains bronze, and 6 kg reaches the full horizon.

## 4. Limits

- Zero load only; higher load re-enters the energy-limited regime.
- One policy; model-level units.
