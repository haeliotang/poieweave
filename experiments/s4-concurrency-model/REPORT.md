# S4 Concurrency Model — report

- Date: 2026-11-14
- Status: independent scheduling model matches 2/2
- Runner: `cargo run --locked -q -p poie-viability --bin poie-concurrency-deadline > /tmp/cd.tsv`
  then `python3 experiments/s4-concurrency-model/verify.py /tmp/cd.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=2 matched=2
```

## 2. Findings

- Python reads 3 h and 2 h from the TOML.
- Sequential is 5 h; one-robot parallel is also 5 h; two-robot parallel is 3 h.
- With a 4 h deadline, only the two-robot layout meets it.
- All durations, makespans and flags match Rust.

## 3. Significance

- This extends the independent-model set from energy/material to scheduling.
- It derives the result from scenario durations and the layout, not from the
  Rust output alone.

## 4. Limits

- Two repairs, static makespan, no scheduler or preemption.
