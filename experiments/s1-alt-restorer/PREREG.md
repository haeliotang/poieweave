# S1 Alternative Restorer — pre-registration

- Experiment ID: `s1-alt-restorer`
- Date: 2026-11-15
- Status: completed; an alternative restorer bypasses the tool cycle
- Scope: Base-08 lathe failed with no lathe-nut, optional external part.

## Question

Can closure choose an alternative restorer to bypass a cycle when that restorer's
input is stocked?

## Method

Add a synthetic `replace-lathe-nut-external` restorer that consumes an external
part and 4 h; stock it 0 or 1; analyze 24 h.

Prediction: 0 part reports the cycle; 1 part restores in 4 h, ignoring the cycle.

## Limitation

Synthetic alternative process; one asset.
