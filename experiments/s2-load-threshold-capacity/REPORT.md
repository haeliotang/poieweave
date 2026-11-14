# S2 Load Threshold vs Capacity — report

- Date: 2026-11-14
- Status: switch invariant to capacity in tested range
- Runner: `cargo run --locked -q -p poie-viability --bin poie-load-threshold-capacity`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
capacity_kwh	first_energy_limited_load_kw
100	2.6
200	2.6
300	2.6
```

## 2. Findings

- The material-to-energy switch stays at 2.6 kW for 100/200/300 kWh capacity.
- Capacity does not move the boundary at this horizon/charge-power setting.
- This is consistent with the earlier energy-boundary result: the switch is
  load-dominated in the tested range.

## 3. Limits

- Three capacities; one charge power; one schedule.
- Model-level units.
