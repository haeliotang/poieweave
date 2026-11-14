# S2 Repair-Order Sensitivity — pre-registration

- Experiment ID: `s2-order-sensitivity`
- Date: 2026-11-14
- Status: completed; negative result
- Scope: can repair order close the Base-06 joint gap?

## Question

The Base-06 fixed-order joint coverage is 24/25 recoverable cases. Does trying
every repair order for each 2- and 3-failure case improve coverage?

## Method

For each independently recoverable case, simulate every permutation of the
failed assets and accept the case if any order keeps the service unblocked.

Prediction: the gap may close if order matters; otherwise 24/24.

## Limitation

At most two failures; one service; model-level.
