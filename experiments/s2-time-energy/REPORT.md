# S2 Time-Semantics Energy Boundary — report

- Date: 2026-11-14
- Status: energy horizons are unchanged at 16/3/1 days
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-energy`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	instant_days	event_driven_days
2.6	16	16
3.0	3	3
4.0	1	1
```

## 2. Findings

- Unlike the grease and insert boundaries, the energy-limited horizons are
  unchanged by clock-advanced repair time in this scenario.
- The extra load during 5-day repairs is too small relative to the stored energy
  to move the integer-day horizon.

## 3. Significance

- This is a negative control: time semantics does not shift every boundary.
- It localizes the effect to material/consumable-limited regimes where repair
  time changes how many failures fall inside the horizon.

## 4. Limits

- One repair duration; three loads; model-level units.
