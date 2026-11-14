# S2 Adversarial Failure Search — report

- Date: 2026-11-14
- Status: a single filter failure is the minimal 0-day set
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-failures`
- Output: [expected.txt](expected.txt)

## 1. Key result

```text
filter-1	1	0	no restorer for filter-1
worst	1	0	filter-1
```

Baseline is 60 days with no blocker.

## 2. Findings

- `filter-1` alone stops the run at day 0: it has no restorer.
- Every pair containing `filter-1` also stops at 0 with the same blocker.
- `robot-m1 + lathe-1` stops at 0 via `no restorer for robot-m1`.
- Pump, PV, robot, and lathe alone are absorbed by repair/restore paths and keep
  60 days.

## 3. Significance

- The automatically selected minimal failure set is a single shared component,
  not a larger combination.
- This is automated adversarial evidence for the shared-component restorer
  requirement, consistent with the manual redundancy experiments.

## 4. Limits

- Five assets, singles/pairs only, all failures at t=0.
- Model-level units.
