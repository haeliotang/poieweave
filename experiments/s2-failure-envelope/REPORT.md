# S2 Failure Envelope — report

- Date: 2026-11-14
- Status: model-level independent recovery coverage; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-survival`
- Reproduced output: [expected.txt](expected.txt)

## 1. Model

`scenarios/base-05/model.toml` is derived from Base-04 for an isolated outpost.
It keeps the Base-01 water/workshop/repair loop, the Base-04 spare lathe nut,
adds an accepted spare sleeve and a modelled PV recharge process. All quantities
are synthetic.

## 2. Result

```text
service=potable-water horizon_s=43200 total=10 recoverable=5 coverage=0.500000
case	failed=none	RESTORABLE	max_plan_s=10800
case	failed=bat-1	BLOCKED	reason=cyclic asset dependency at bus-1
case	failed=bus-1	BLOCKED	reason=cyclic asset dependency at bus-1
case	failed=charger-1	BLOCKED	reason=no restorer for robot-m1
case	failed=filter-1	BLOCKED	reason=no restorer for filter-1
case	failed=gauge-1	RESTORABLE	max_plan_s=10800
case	failed=lathe-1	RESTORABLE	max_plan_s=10800
case	failed=pump-p1	RESTORABLE	max_plan_s=10800
case	failed=pv-1	RESTORABLE	max_plan_s=10800
case	failed=robot-m1	BLOCKED	reason=no restorer for robot-m1
```

Coverage: **5/10 = 0.5** for single-asset failures at a 12 h horizon.

## 3. Findings

- The water service survives a pump failure because the accepted sleeve and the
  maintenance robot are enough for a 3 h replacement.
- Filter failure is outside the repair closure: no process restores `filter-1`.
- Power-side failure (`bat-1`/`bus-1`) blocks through the support cycle.
- Maintenance-side failure (`charger-1`/`robot-m1`) blocks the only repair
  capability.
- The S2 envelope is useful as a **failure map**, but its coverage is an
  optimistic independent estimate: no two repairs currently share a resource
  budget or a schedule.

## 4. Limitations

- Single-failure envelope only; no double failures yet.
- Independent checks, not a joint repair plan; inventory double-counting can
  overstate coverage.
- No energy depletion or recharge scheduling in this first slice, despite the
  Base-05 recharge process.
- No uncertainty or stochastic failure timing.
- Synthetic model; no physical safety or lifetime inference.

## 5. Next S2 work

1. Add double-failure combinations and a joint inventory ledger.
2. Simulate repeated failures over 30/90 days with recharge and consumption.
3. Compute a survival horizon rather than only per-state recovery coverage.
4. Require all service requirements to share one plan and one evidence bundle.
