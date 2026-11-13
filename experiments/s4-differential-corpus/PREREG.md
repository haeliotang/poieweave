# S4 Differential Corpus — pre-registration

- Experiment ID: `s4-differential-corpus`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: differential verdict corpus, not a formal equivalence proof.

## Question

Does the independent recursive Python checker agree with Rust S1 across a
broader target corpus?

## Method

Rust exports verdicts for **every asset** in Base-01, Base-04, Base-05, Base-06,
Base-07 and Base-08 at 12 h and 24 h horizons (112 cases). Python runs its
recursive production/repair checker on the same cases. Compare verdict classes
and report mismatches.

Prediction: most cases agree; any mismatch is investigated and recorded rather
than hidden.

## Limitation

The corpus is small and uses one horizon/model family. Verdict agreement does
not establish identical blocking reasons or full semantic equivalence.
