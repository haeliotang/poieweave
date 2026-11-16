# S2 Canonical C11 Migration — pre-registration

- Experiment ID: `s2-canonical-c11`
- Date: 2026-11-16
- Status: BLOCKED; minima agree at day 0 but minimal sets differ; day-10 legacy input is defective
- Scope: 31 adversarial subsets, with/without insert resupply.

## Question

Does canonical engine reproduce 0-day minima and the filter-1 minimal set?

## Method

Run all 31 subsets at day 0 and day 10; legacy uses its original day-0 input
because the legacy runner ignores `at_day`; canonical uses the labelled day.

Prediction: at day 0 minima agree; minimal set may expose t=0 ordering; day 10
must expose the legacy runner bug.

## Limitation

C11 is not accepted until the t=0 initial-repair ordering is resolved.
