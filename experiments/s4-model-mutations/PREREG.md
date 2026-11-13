# S4 Model Mutations — pre-registration

- Experiment ID: `s4-model-mutations`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one-lot removal mutations of Base-05.

## Question

Do Rust S1 and the independent checker agree when accepted inventory is removed
from the model, rather than when one asset is marked failed?

## Method

For each accepted lot in Base-05:

1. write a mutated TOML with that lot removed;
2. run Rust verdicts for every asset at 24 h;
3. run the Python recursive checker on the same file;
4. compare verdicts.

Prediction: agreement; any mismatch is investigated as a model-semantics finding.

## Limitation

Only accepted-lot removal; no process removal, source corruption or multi-lot
mutations.
