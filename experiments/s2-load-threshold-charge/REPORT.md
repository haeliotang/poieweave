# S2 Load Threshold vs Charge — report

- Date: 2026-11-14
- Status: switch invariant to charge power in tested range
- Runner: `cargo run --locked -q -p poie-viability --bin poie-load-threshold-charge`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
charge_kw	first_energy_limited_load_kw
3	2.6
10	2.6
30	2.6
```

## 2. Findings

- The material-to-energy switch stays at 2.6 kW for 3/10/30 kW charge power.
- Charge power does not move the boundary at this horizon.
- Together with the capacity result, the switch appears load-dominated in the
  tested range.

## 3. Limits

- Three charge values; one capacity; one schedule.
- Model-level units.
