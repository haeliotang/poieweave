# S3 Robot Count Sweep — report

- Date: 2026-11-14
- Status: two robots are the minimum for the 4 h deadline
- Runner: `cargo run --locked -q -p poie-viability --bin poie-robot-sweep`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
repairs	deadline_h	robots	makespan_h	meets
3	4	1	7	false
3	4	2	4	true
3	4	3	3	true
```

## 2. Findings

- One robot serializes the repairs: 7 h, deadline missed.
- Two robots: 3 h + 2 h on one, 2 h on the other, makespan exactly 4 h, meets.
- Three robots: makespan 3 h, meets with margin.

## 3. Significance

- This gives a concrete capacity threshold: two repair executors are enough for
  this three-repair workload under a 4 h deadline.
- It complements the two-repair contention experiment with a count sweep.

## 4. Limits

- Synthetic durations and greedy scheduling; no optimality proof.
- Model-level units.
