# S2 Regeneration Cost — report

- Date: 2026-11-14
- Status: negative result; cost is not binding in this model
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-cost`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
duration_x	energy_x	survived_days	blocked_by
1.0	1.0	60	cyclic process dependency at turn-sleeve-p1
1.0	2.0	60	cyclic process dependency at turn-sleeve-p1
1.0	100.0	60	cyclic process dependency at turn-sleeve-p1
2.0	1.0	60	cyclic process dependency at turn-sleeve-p1
2.0	2.0	60	cyclic process dependency at turn-sleeve-p1
2.0	100.0	60	cyclic process dependency at turn-sleeve-p1
4.0	1.0	60	cyclic process dependency at turn-sleeve-p1
4.0	2.0	60	cyclic process dependency at turn-sleeve-p1
4.0	100.0	60	cyclic process dependency at turn-sleeve-p1
```

## 2. Findings

- Scaling recycling duration up to 4× and energy input up to 100× does not
  change the 60-day horizon in this model.
- The binding constraint remains the process-cycle/material path; recycling is
  not scheduled on the critical path at this horizon, so its cost is inert.
- This is a useful negative result: the current model cannot yet show a
  cost-limited regeneration regime.

## 3. Limits

- Only duration and electricity are varied.
- No mass loss, contamination or residue.
- Model-level only.
