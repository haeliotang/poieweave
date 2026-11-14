# S1 Restorer Tie-Break — report

- Date: 2026-11-14
- Status: fixed; lower input cost wins the equal-duration tie
- Runner: `cargo run --locked -q -p poie-viability --bin poie-restorer-tie`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
cheap-first	steps=["replace-sleeve-p1"] duration=10800s grease_left=0.95
expensive-first	steps=["replace-sleeve-p1"] duration=10800s grease_left=0.95
```

## 2. Before the fix

Expensive-first selected `replace-sleeve-expensive` and left 0.80 kg grease,
because equal-duration ties followed index order.

## 3. Fix

After duration comparison, closure compares relative consumption of the initial
stock (`relative_consumption`) and keeps the lower-cost state. A regression test
covers the expensive-first order.

## 4. Findings

- Both orders now select the 0.05 kg restorer and leave 0.95 kg grease.
- Duration is the primary criterion; input cost breaks ties.
- This makes restorer choice independent of process order for this pair.

## 5. Limits

- One tie pair; relative-consumption metric is a heuristic, not a formal cost
  optimum across all resources.
- Model-level units.
