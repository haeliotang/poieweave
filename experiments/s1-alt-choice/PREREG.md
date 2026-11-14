# S1 Alternative Choice — pre-registration

- Experiment ID: `s1-alt-choice`
- Date: 2026-11-15
- Status: completed; closure picks the cheaper spare path when both are available
- Scope: cycle / spare-only / alternative-only / both.

## Question

When a spare-based and an alternative restorer are both feasible, does closure
pick the cheaper one?

## Method

Base-08 lathe failed; lathe-nut spare 0/1 and external part 0/1; a synthetic
alternative restorer takes 4 h and the spare path takes 2 h.

Prediction: `both` yields 7200 s, i.e. the spare path.

## Limitation

One asset and one synthetic alternative.
