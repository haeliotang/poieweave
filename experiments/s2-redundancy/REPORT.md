# S2 Redundant Pump — report

- Date: 2026-11-14
- Status: redundancy extends 60 -> 90 days
- Runner: `cargo run --locked -q -p poie-viability --bin poie-redundancy`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
baseline	60	cyclic process dependency at turn-sleeve-p1
redundant-pump	90	none
```

## 2. Findings

- Baseline Base-08 stops at 60 days on the cyclic turn-sleeve dependency.
- With a second available pump and `potable-water` expressed as
  `requires_any = [pump-p1, pump-p2]`, the same failure sequence reaches the
  full 90-day horizon with no blocker.
- Redundancy removes the pump-repair path from the critical chain, so the
  remaining schedule has enough time and materials.

## 3. Significance

- This is the first S2 result where a service-level N+1 design change removes the
  terminal blocker entirely.
- It also shows the model can represent redundancy via `requires_any` rather than
  inventing a new mechanism.

## 4. Limits

- Service-level redundancy only; no shared pump controller, plumbing, or power.
- One policy; model-level units.
