# S4 Counterfactual Threshold — pre-registration

- Experiment ID: `s4-counterfactual`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one model-level counterfactual, not a general minimal-repair solver.

## Question

For the Base-09 missing-bronze blocker, what is the minimal added stock that
turns the pump repair from blocked to restorable?

## Method

Add 1.1/1.2/1.5 kg accepted bronze to Base-09 and run The S1 closure for
`pump-p1`. The independent Python checker repeats the same counterfactual using
its recursive model.

Prediction: 1.1 kg remains blocked; 1.2 kg and 1.5 kg become restorable.
The 1.2 kg threshold should align with the observed `need 1.2` witness.

## Limitation

One resource and one target; no combined stock/asset counterfactuals.
