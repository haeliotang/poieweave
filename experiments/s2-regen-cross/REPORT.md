# S2 Regeneration Yield × Interval — report

- Date: 2026-11-14
- Status: yield/failure-frequency interaction; model-level
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regen-cross`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
recycle_yield	interval_days	survived_days	blocked_by
1.00	10	60	cyclic process dependency at turn-sleeve-p1
1.00	15	90	none
1.00	20	90	none
1.00	30	90	none
0.75	10	60	cyclic process dependency at turn-sleeve-p1
0.75	15	90	none
0.75	20	90	none
0.75	30	90	none
0.50	10	50	missing resource bronze-bar: need 1.2, have 0.6
0.50	15	75	missing resource bronze-bar: need 1.2, have 0.6
0.50	20	90	none
0.50	30	90	none
```

## 2. Findings

- At 100% and 75% yield, a 15-day interval is enough for 90 days.
- At 50% yield, the 15-day interval only reaches 75 days, while 20 days reaches
  90: yield and failure frequency interact.
- The blocker remains material shortage at 50%; the regime changes from process
  cycle at 10 days to material shortage at 15 days.

## 3. Limits

- Scales only process output; no mass conservation, contamination or recycling
  energy/time.
- One model family and one horizon.
