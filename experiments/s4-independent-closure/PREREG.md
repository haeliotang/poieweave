# S4 Independent Direct Closure — pre-registration

- Experiment ID: `s4-independent-closure`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: independent direct-restorer verdicts, not full recursive closure.

## Question

Can a separate implementation reach the same verdict classes as Rust S1 for a
small set of direct-repair cases?

## Method

Python loads the TOML model and checks only a single direct restorer:

- the target has a process that restores it,
- process verification is at least `modeled`,
- duration fits the horizon,
- all occupied assets are available,
- required capabilities are provided at `modeled` or better,
- all process inputs are available in the initial accepted inventory.

Cases:

| Model | Target | Rust expectation |
| --- | --- | --- |
| Base-04 | lathe-1 | RESTORABLE |
| Base-01 | lathe-1 | BLOCKED |
| Base-05 | lathe-1 | RESTORABLE |
| Base-05 | filter-1 | BLOCKED |
| Base-05 | pump-p1 | RESTORABLE |

Prediction: Python verdict classes match all five.

## Limitation

The Python checker does not search production chains, recursive repairs,
requires_any or inventory consumption across steps. Agreement is only about
direct-repair verdicts.
