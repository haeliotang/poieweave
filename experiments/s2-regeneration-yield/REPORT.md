# S2 Regeneration Yield — report

- Date: 2026-11-14
- Status: model-level sensitivity; not a physical yield claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-yield`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recycle_yield	survived_days	blocked_by
1.00	60	cyclic process dependency at turn-sleeve-p1
0.75	60	cyclic process dependency at turn-sleeve-p1
0.50	50	missing resource bronze-bar: need 1.2, have 0.6
```

## 2. Findings

- At 100% and 75% recycling yield, the 10-day Base-08 case survives 60 days.
- At 50% yield, the horizon drops to 50 days because recycled bar no longer
  reaches the 1.2 kg sleeve input.
- So the S2 regeneration benefit is not binary: it degrades smoothly with yield
  and becomes binding below 75%.
- The failure mode changes from a process-cycle block to a material shortage,
  which is useful for diagnosis.

## 3. Limitation

The experiment scales only the process output; it does not rebalance mass or
model contamination, energy per kg, or recycling time changes. It is a
sensitivity probe, not a physical model.

## 4. Next

1. Make recycling mass-conserving with explicit loss/residue outputs.
2. Add recycling time/energy as a function of yield.
3. Cross yield with failure interval and initial stock.
