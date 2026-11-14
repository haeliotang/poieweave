# S2 Recharge Output at the Switch — report

- Date: 2026-11-14
- Status: positive; recharge output is the effective intervention
- Runner: `cargo run --locked -q -p poie-viability --bin poie-energy-recharge`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recharge_kwh	average_kw	survived_days	blocked_by
20	2.50	16	energy_depleted
30	3.75	60	cyclic process dependency at turn-sleeve-p1
40	5.00	60	cyclic process dependency at turn-sleeve-p1
```

## 2. Findings

- At 3.75/5.0 kW average recharge the 2.6 kW load is covered and the horizon
  jumps from 16 to 60 days.
- The blocker moves from `energy_depleted` back to the material/cycle limit.
- This is the positive counterpart to `s2-energy-interventions`: changing the
  inflow works, changing storage/charge caps does not.

## 3. Significance

- The energy regime boundary is controlled by average recharge, not by battery
  size or charge cap.
- Interventions should target generation/recharge or load, not storage alone.

## 4. Limits

- One load; three recharge values; model-level units.
