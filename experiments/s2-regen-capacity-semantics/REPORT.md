# S2 Capacity Semantics — report

- Date: 2026-11-14
- Status: capacity does not cap initial inventory
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-capacity-semantics`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
initial_kwh	capacity_kwh	first_blocked_x
500	200	521
500	400	521
500	600	521
```

## 2. Findings

- With 500 kWh initial stock, all capacities give the same 521× threshold.
- Capacity 200 kWh does **not** cap the initial stock at 200; the plan can use
  the full 500 kWh.
- This confirms the semantic gap: `capacity_kwh` is applied to interval
  charge/discharge flows, not to the initial inventory.

## 3. Significance

- The earlier “threshold invariant to capacity” result is not a coincidence; it
  follows from capacity not capping initial SOC.
- If capacity is intended as a physical storage bound, the initial energy lot
  should be clamped to it (or the model should reject inconsistent inputs).

## 4. Limits

- Model-level units; forced path.
