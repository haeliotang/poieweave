# S2 Two Restorers — report

- Date: 2026-11-14
- Status: both restorer layers are required
- Runner: `cargo run --locked -q -p poie-viability --bin poie-two-restorers`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
redundant-only	30	no restorer for filter-1
filter-only	30	no restorer for robot-m1
robot-only	30	no restorer for filter-1
filter+robot	90	none
```

## 2. Findings

- A filter restorer alone does not help: the shared robot still fails and cannot
  be repaired.
- A robot restorer alone does not help: the filter still fails without a
  restorer.
- Only the combination reaches 90 days with no blocker.

## 3. Significance

- This makes the redundancy requirement precise: service-chain resilience needs
  restorers at every shared layer, not just more served assets.
- It also confirms the two restorers compose correctly in one plan.

## 4. Limits

- One restorer each; model-level units.
