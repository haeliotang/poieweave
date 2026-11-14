# S4 Time Model — report

- Date: 2026-11-14
- Status: independent dense-event arithmetic matches 2/2
- Runner: `cargo run --locked -q -p poie-viability --bin poie-time-semantics > /tmp/ts.tsv`
  then `python3 experiments/s4-time-model/verify.py /tmp/ts.tsv`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
rows=2 matched=2
```

## 2. Findings

- Instantaneous times are the raw events `[0, 1]`.
- Clock-advanced times are `[3, 6]` for the 3 h repair.
- Python derives both lists from the TOML duration and the event schedule.

## 3. Significance

- The time arithmetic itself now has an independent implementation.
- It complements the certificate invariant checks with a model-driven derivation.

## 4. Limits

- Dense schedule only; no resources/horizon.
