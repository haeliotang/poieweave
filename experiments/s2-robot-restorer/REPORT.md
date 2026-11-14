# S2 Robot Restorer — report

- Date: 2026-11-14
- Status: manual robot restorer removes the repair-executor blocker
- Runner: `cargo run --locked -q -p poie-viability --bin poie-robot-restorer`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
redundant-only	35	no restorer for robot-m1
redundant+robot-restorer	90	none
```

## 2. Findings

- Without a robot restorer the redundant-pump configuration still stops at 35
  days because the shared repair robot has no restorer.
- Adding `replace-robot` (manual, no robot occupation) removes that blocker and
  the run reaches 90 days.
- The repair-executor single point of failure is addressed by adding a restorer
  at the repair layer.

## 3. Significance

- Positive counterpart to `s2-redundancy-robot`.
- Together with the filter restorer, it shows both the served-component and the
  repair-executor layers need their own restorers.

## 4. Limits

- Manual replacement, no robot redundancy.
- One module; model-level units.
