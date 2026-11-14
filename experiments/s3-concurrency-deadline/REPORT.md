# S3 Concurrency Deadline — report

- Date: 2026-11-14
- Status: shared robot misses the deadline; separate robots meet it
- Runner: `cargo run --locked -q -p poie-viability --bin poie-concurrency-deadline`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
layout	pump_s	lathe_s	deadline_s	sequential_meets	parallel_meets
one-robot	10800	7200	14400	false	false
two-robots	10800	7200	14400	false	true
```

## 2. Findings

- Each repair fits the 4 h deadline individually (3 h and 2 h).
- Sequential execution takes 5 h and misses it.
- Two robots execute in parallel in 3 h and meet it.
- One shared robot still takes 5 h and misses it.

## 3. Significance

- Local plan feasibility does not imply system-level deadline feasibility.
- The missing condition is disjoint resource occupancy, not plan duration.
- This is a concrete argument for modelling contention in recovery scheduling.

## 4. Limits

- Static makespan comparison, no scheduler and no preemption.
- Model-level units.
