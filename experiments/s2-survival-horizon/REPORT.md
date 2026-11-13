# S2 Survival Horizon — report

- Date: 2026-11-14
- Status: model-level survival curve; not a physical or safety claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-horizon`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
interval_days	survived_days	failed_at	final_energy_kwh	blocked_by
10	40	day 40	200.0	missing resource bronze-bar: need 1.2, have 0.6000000000000001
15	60	day 60	200.0	missing resource bronze-bar: need 1.2, have 0.6000000000000001
20	80	day 80	200.0	missing resource bronze-bar: need 1.2, have 0.6000000000000001
30	90	day 90	200.0	none
```

## 2. Findings

- The survival horizon is controlled by material closure, not energy: the
  battery stays at its 200 kWh cap while the repairs fail on `bronze-bar`.
- More frequent failures consume the finite local stock sooner:
  - 10-day failures: 40 days
  - 15-day failures: 60 days
  - 20-day failures: 80 days
  - 30-day failures: survives the 90-day window
- The day-35 lathe failure is repairable independently with the spare nut and
  does not change the endpoint of this curve.
- This is a **negative** self-maintenance result: the outpost cannot regenerate
  its material stock within the model; it can only consume the initial bars.

## 3. Interpretation

The curve is not a forecast of a real plant. It is a parameter study showing
where a deterministic repair policy crosses the material boundary. It gives a
concrete target for the next S2 iteration: add local material regeneration or
explicit external resupply assumptions and measure how the curve changes.

## 4. Limitations

- Only pump failures vary; one lathe event is fixed.
- Greedy multi-service order, not a joint optimiser.
- No operating demand/water consumption or wear.
- No stochastic failure timing.
- Synthetic quantities and no physical safety claim.
