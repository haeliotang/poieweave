# S3 Concurrent Scheduler — report

- Date: 2026-11-15
- Status: 7/4/3 h makespans over 1/2/3 robots
- Runner: `cargo run --locked -q -p poie-viability --bin poie-scheduler`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
robots	makespan_h	plan_ends_h
1	7	3,5,7
2	4	3,2,4
3	3	3,2,2
```

## 2. Findings

- One robot serializes: ends at 3/5/7 h.
- Two robots: plan ends 3/2/4, makespan 4 h.
- Three robots: plan ends 3/2/2, makespan 3 h.
- Plans remain internally sequential; only different plans overlap.

## 3. Significance

- This is the first shared scheduler implementation, replacing the earlier
  hard-coded greedy sweep in experiments.
- It gives a reusable library primitive for later survival integration.

## 4. Limits

- Greedy list scheduling; no preemption; model-level units.
