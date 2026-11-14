# S3 Concurrency — report

- Date: 2026-11-14
- Status: shared robot serializes; separate robots do not overlap
- Runner: `cargo run --locked -q -p poie-viability --bin poie-concurrency`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
layout	pump_s	lathe_s	robot_overlap	sequential_s	ideal_parallel_s
one-robot	10800	7200	true	18000	10800
two-robots	10800	7200	false	18000	10800
```

## 2. Findings

- Pump repair takes 3 h, lathe repair 2 h.
- With one robot both repairs use robot-m1 in `[0, duration)` and overlap; the
  contended makespan is the sequential 5 h.
- With separate robots there is no overlap; the ideal parallel makespan is 3 h.

## 3. Significance

- This is the first explicit concurrency/contention observation in the core.
- It gives a concrete scheduling target: parallel repair can save 2 h only when
  the occupied assets are disjoint.

## 4. Limits

- Static interval check, no concurrent scheduler and no deadlines.
- Model-level units.
