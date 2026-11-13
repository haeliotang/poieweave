# S4 Independent Plan Verification — pre-registration

- Experiment ID: `s4-independent-plan`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: independent plan replay, not an independent closure solver.

## Question

Can a separate implementation verify exported S1 plans against the model
resources, without calling the Rust planner or closure?

## Method

Rust exports four deterministic plans:

1. Base-04 `lathe-1`, 7200 s
2. Base-01 `pump-p1`, 43200 s
3. Base-05 `pump-p1`, 10800 s
4. Base-05 `lathe-1`, 7200 s

The Python verifier reads the model TOML and each plan JSON, starts from the
accepted-lot inventory, and replays every step: checks input availability,
subtracts inputs, adds outputs, records restored assets, sums step durations, and
checks the final target plus horizon.

Prediction: `plans=4 verified=4`, `OK`.

## Limitation

This verifies plan replay, not the closure search that selected the plan. It also
does not model yield, capability contention or physical execution.
