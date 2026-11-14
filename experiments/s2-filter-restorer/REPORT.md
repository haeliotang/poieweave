# S2 Filter Restorer — report

- Date: 2026-11-14
- Status: local restorer removes the shared-filter blocker
- Runner: `cargo run --locked -q -p poie-viability --bin poie-filter-restorer`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
redundant-only	30	no restorer for filter-1
redundant+filter-restorer	90	none
```

## 2. Findings

- Redundant pump alone still stops at day 30 when filter-1 fails.
- Adding a modeled `replace-filter` process (one cartridge, robot-executed,
  restores filter-1) removes the blocker and the run reaches 90 days.
- The shared single point of failure is addressed by a restorer, not by more
  pumps.

## 3. Significance

- This is the positive counterpart to `s2-redundancy-filter`: service-chain
  redundancy needs a restorer for the shared component.
- It shows the model can represent a one-shot filter replacement closure.

## 4. Limits

- One restorer/cartridge; no filter degradation or contamination.
- Model-level units.
