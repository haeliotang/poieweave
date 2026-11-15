# S5 Canonical Engine — report

- Date: 2026-11-15
- Status: sparse scenario matches; dense migration not yet done
- Runner: `cargo run --locked -q -p poie-viability --bin poie-canonical-engine`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	canonical_days	legacy_days	termination	verdict
base-08-sparse	30	30	completed	same
```

## 2. Kernel shape

The kernel has one clock, one inventory ledger and one commit rule for:

- initial repair
- failure batches
- resupply
- horizon

Termination is a typed enum (`CompletedHorizon`, `Blocked`, `IncompleteAtHorizon`),
not inferred from log strings.

## 3. Significance

- This is the first S5 entry point that is not a patch to `survival.rs`.
- It records before/after inventory and applied plan processes per event.

## 4. Limits

- One sparse scenario; only failure and initial-repair events are exercised.
- Dense/resource-contended scenarios are expected to differ; each difference
  must be recorded before migrating a formal S2 experiment.
