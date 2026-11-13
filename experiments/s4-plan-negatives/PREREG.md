# S4 Plan Negative Cases — pre-registration

- Experiment ID: `s4-plan-negatives`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: verifier negative controls.

## Question

Does the independent plan verifier reject deliberately corrupted plans, rather
than accepting everything?

## Method

Take the four exported S1 plans and three restorable cases. Mutate:

1. drop the last step,
2. add 1 second to the reported total duration,
3. append a step naming a nonexistent process.

Run the same Python `verify_plan` used in `s4-independent-plan`.

Prediction: all three mutations rejected.

## Limitation

These are structural mutations, not adversarial search for subtle but
physically plausible plan errors.
