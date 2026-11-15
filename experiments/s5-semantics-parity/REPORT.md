# S5 Semantics Parity Guard — report

- Date: 2026-11-15
- Status: no violation in three tested scenarios
- Runner: `cargo run --locked -q -p poie-viability --bin poie-semantics-parity`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	instant_days	advanced_days	verdict
base-05@2d->5d	4	10	ok
base-05@3d->5d	6	10	ok
base-08@2d->5d	6	16	ok
parity=ok
```

## 2. Findings

- Clock-advanced horizons are 10-16 days where instantaneous returns 4-6.
- No scenario violates the causal guard `advanced >= instant`.

## 3. Significance

- It makes the semantics difference explicit instead of silently mixing paths.
- It is a regression guard: a future path that reports a shorter horizon under
  clock-advanced semantics fails the command.

## 4. Limits

- Three scenarios; not a theorem; instantaneous mode remains legacy.
