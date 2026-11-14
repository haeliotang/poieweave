# S4 Path-Choice Model — pre-registration

- Experiment ID: `s4-path-choice-model`
- Date: 2026-11-15
- Status: completed; independent path choice matches 4/4
- Scope: cycle / spare / alternative / both.

## Question

Can an independent model derive the cheapest recovery path from the TOML and the
declared alternative duration?

## Method

Python reads the `replace-lathe-nut` duration from Base-08, declares the synthetic
alternative as 4 h, and predicts blocked/7200/14400/7200 for the four cases.

Prediction: four rows match.

## Limitation

Alternative duration is a declared constant; no search.
