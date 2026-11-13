# S2 Long-Horizon Survival — report

- Date: 2026-11-14
- Status: model-level greedy-policy result; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-longevity`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
service=potable-water horizon_days=90 survived_days=40 final_energy_kwh=200.0
event	day=0	failed=none	repaired=pump-p1	status=ok	energy_kwh=39.5
event	day=10	failed=pump-p1	repaired=pump-p1	status=ok	energy_kwh=199.5
event	day=20	failed=pump-p1	repaired=pump-p1	status=ok	energy_kwh=197.4
event	day=30	failed=pump-p1	repaired=pump-p1	status=ok	energy_kwh=197.4
event	day=40	failed=pump-p1	repaired=none	status=missing resource bronze-bar: need 1.2, have 0.6000000000000001	energy_kwh=200.0
```

## 2. Findings

- The accepted sleeves and initial stock carry the pump through the first
  failures, but the locally available bronze bar runs out.
- Energy is not the limiting resource in this synthetic run once a 200 kWh
  battery capacity and 10 kW charge cap are imposed; the battery reaches its
  cap by day 10. The remaining failure is material, not electrical.
- The first unrecoverable event is a **material** shortage, not a control or
  planning failure. This is a useful negative result for the self-maintenance
  thesis: enough energy does not imply enough material closure.
- The run also demonstrates why "service survives today" is insufficient: a
  repair performed now can consume the stock needed for the next repair.

## 3. What this establishes

The S2 loop can carry state across events:

- repairs consume shared inventory,
- production plans consume energy and material,
- a failed repair stops the horizon,
- and the report identifies the concrete blocking resource.

## 4. Limitations

- Greedy, deterministic policy; no lookahead or optimisation.
- Only one asset type fails repeatedly; no multi-failure schedule.
- Battery capacity/charge cap is a study parameter, not an ontology field;
  values are synthetic.
- No stochastic failures, maintenance delay, or operating demand.
- Synthetic Base-05 assumptions; no field or safety claim.

## 5. Next

1. Add battery capacity and recharge limits.
2. Add joint multi-asset failure schedules and lookahead.
3. Include operating demand and a survival-horizon definition over repeated
   events.
4. Compare greedy recovery against a pre-registered survival-aware baseline.
