# S2 Failure Envelope — pre-registration

- Experiment ID: `s2-failure-envelope`
- Date: 2026-11-14
- Status: completed; final table in [REPORT.md](REPORT.md)
- Scope: model-level, independent recovery coverage; not a joint schedule or
  physical claim.

## Question

For Base-05 `potable-water`, which single-asset failures are recoverable within
12 h using S1 capability closure, and what is the resulting recovery coverage?

## Method

Enumerate every failure subset of size 0 or 1 over the model's assets. Set each
selected asset to `failed` in a cloned model. For every direct service
requirement, run the S1 recursive closure analysis. A case is recoverable only
when all direct requirements are restorable within the horizon.

This is **independent** coverage: every requirement is checked against the same
initial inventory, so two repair paths that compete for one lot can both count.
The report records this as an optimistic envelope, not a feasible joint plan.

## Cases and predictions

Base-05 has 10 assets, so the envelope contains 1 + 10 = 11 cases.

- No failure: recoverable; pump is degraded but can be repaired via the accepted
  sleeve and robot capability.
- `filter-1`: blocked; no restorer.
- `bus-1` or `bat-1`: blocked through the power support chain.
- `charger-1` or `robot-m1`: blocked through the maintenance support chain.
- `pump-p1`, `lathe-1`, `gauge-1`, `pv-1`: recoverable for `potable-water`.
- Predicted coverage: 5/10 = 0.5.

Expected output is frozen in [expected.txt](expected.txt).

## Limitations

No concurrency, no consumption sharing between requirements, no stochastic
failures, no recharge scheduling, and no physical calibration. S2 long-horizon
planning remains future work; this experiment only measures the static recovery
envelope.
