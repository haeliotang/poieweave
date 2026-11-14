# S2 Threshold vs Initial Energy — report

- Date: 2026-11-14
- Status: threshold tracks initial energy stock
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-threshold-initial`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
initial_kwh	first_blocked_x
20	41
40	61
60	81
80	101
120	141
```

## 2. Findings

- The threshold increases by 20× for every additional 20 kWh of initial energy:
  exactly one multiplier per kWh.
- This confirms that the 60/61 boundary is set by the initial electric-energy
  lot, not by `BatteryLimits.capacity_kwh`.
- Together with the battery-capacity experiment, this identifies a clear
  semantic gap: capacity bounds storage but does not seed plan energy.

## 3. Limits

- Forced path; one model family.
- Threshold measured in integer multipliers.
