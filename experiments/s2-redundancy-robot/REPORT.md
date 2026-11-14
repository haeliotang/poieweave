# S2 Redundancy vs Shared Repair Robot — report

- Date: 2026-11-14
- Status: shared repair robot still blocks
- Runner: `cargo run --locked -q -p poie-viability --bin poie-redundancy-robot`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
mode	survived_days	blocked_by
baseline	30	no restorer for robot-m1
redundant-pump	35	no restorer for robot-m1
```

## 2. Findings

- The redundant pump delays the terminal block from day 30 to day 35 by removing
  one repair path, but both modes still end on `no restorer for robot-m1`.
- Redundancy in the served asset does not cover a failure of the shared repair
  executor.

## 3. Significance

- This is a second single-point-of-failure counterexample, now in the repair
  layer rather than the process chain.
- It suggests repair-capacity redundancy (or a robot restorer) is required for
  the redundancy claim to hold.

## 4. Limits

- One repair asset; no robot redundancy.
- Model-level units.
