# S2 Energy-Load Sweep under Long Repairs — report

- Date: 2026-11-15
- Status: energy horizons remain time-semantics invariant
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-energy-sweep`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	instant_days	event_driven_days
0.0	30	30
1.0	30	30
2.0	30	30
2.6	14	14
3.0	3	3
```

## 2. Findings

- Even with 5-day repairs, the energy-limited horizons are identical under both
  semantics at every load.
- Loads up to 2 kW reach the full 30-day horizon; 2.6 kW gives 14 days and 3 kW
  gives 3 days.
- Energy, not repair time, remains the binding constraint.

## 3. Significance

- It extends the earlier energy negative control to the long-repair regime.
- It bounds the time-semantics effect to material/consumable limits, not energy.

## 4. Limits

- One long-repair setting; five loads; model-level units.
