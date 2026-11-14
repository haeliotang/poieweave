# S2 Redundancy vs Shared Filter — pre-registration

- Experiment ID: `s2-redundancy-filter`
- Date: 2026-11-14
- Status: completed; redundancy does not cover a shared failure
- Scope: redundant pump plus a filter failure.

## Question

Does a redundant pump help when the water service's single filter fails?

## Method

Base-08; same redundant-pump configuration as `s2-redundancy`; add a filter-1
failure at day 30. 10-day pump failures, 90 days.

Prediction: both modes block at the filter failure.

## Limitation

One shared component; no filter restorer in the model.
