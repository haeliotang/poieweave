# S4 Horizon Reason Classes — report

- Date: 2026-11-14
- Status: short-horizon reason-class check; not full reason equivalence
- Runner: `cargo run --locked -q -p poie-viability --bin poie-horizon-reasons`
  then `python3 experiments/s4-horizon-reasons/compare.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
blocked=56 matched=56 mismatched=0
```

Distribution of Rust reasons in this corpus:

- 20 `horizon_exceeded`
- 36 `no_restorer`

## 2. Findings

- The independent checker agrees on all 56 blocked cases for the coarse
  `no_restorer` / `horizon` / `cycle` classes.
- This adds the `horizon_exceeded` class that was absent from the 112-case
  corpus.
- Missing-resource and unverified-process reason classes remain uncovered.

## 3. Limits

- 900 s only, one model family.
- Nominal process duration, no wait/occupancy time.
- No missing-resource or unverified-process cases.
- Not full reason or minimal-witness equivalence.
