# S4 Recursive Closure — pre-registration

- Experiment ID: `s4-recursive-closure`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: independent recursive production/repair search for five cases.

## Question

Can an independent implementation reproduce Rust S1 verdicts when production
chains and recursive repair are required, not only direct restorers?

## Method

Python searches direct restorers and production processes with explicit cycle
guards. It checks process verification, duration, occupied assets, capability
providers and input chains. Cases:

| Model | Target | Expected |
| --- | --- | --- |
| Base-01 | pump-p1 | RESTORABLE |
| Base-01 | lathe-1 | BLOCKED |
| Base-04 | lathe-1 | RESTORABLE |
| Base-05 | pump-p1 | RESTORABLE |
| Base-05 | filter-1 | BLOCKED |

Prediction: all five verdict classes match Rust.

## Limitation

The Python search still does not model `requires_any`, shared time across
branches, parallel occupancy or full yield semantics. It is an independent
verdict check, not a formal equivalence proof.
