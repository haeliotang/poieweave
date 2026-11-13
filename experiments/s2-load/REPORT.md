# S2 Site Load Sensitivity — report

- Date: 2026-11-14
- Status: model-level sensitivity; not a physical power model
- Runner: `cargo run --locked -q -p poie-viability --bin poie-load`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
load_kw	survived_days	final_energy_kwh	blocked_by
0.0	40	200.0	missing resource bronze-bar: need 1.2, have 0.6
1.0	40	200.0	missing resource bronze-bar: need 1.2, have 0.6
2.0	40	200.0	missing resource bronze-bar: need 1.2, have 0.6
3.0	3	0.0	energy_depleted
4.0	1	0.0	energy_depleted
```

## 2. Findings

- With PV averaging about 2.5 kW, loads of 0–2 kW do not change the material
  limit: the outpost still fails at day 40 on `bronze-bar`.
- At 3–4 kW the battery depletes in days; energy becomes the limiting factor.
- This is a clean **regime switch**: the same system can be material-limited or
  energy-limited depending on load.
- It strengthens the S2 conclusion that "energy is not the limit" is
  assumption-dependent, not universal.

## 3. Limits

- Constant load, no demand profile or duty cycle.
- PV is modelled as a constant process output; no irradiance variability.
- Battery capacity/charge are study parameters.
- No load-shedding or service-quality tradeoff.
- Model-level only; no physical power claim.
