# S4 Approval-Schedule Model — pre-registration

- Experiment ID: `s4-approval-schedule-model`
- Date: 2026-11-14
- Status: completed; independent receipts/horizon model matches 4/4
- Scope: single/multi service, all-approved vs reject-day20.

## Question

Can an independent model reproduce the approval-schedule receipts and horizon?

## Method

Python applies the schedule to events at days 10/20 with an 8 h approval policy,
counts one initial receipt plus accepted events, and compares four rows.

Prediction: all four rows match.

## Limitation

Schedule and events are fixed; no lease/digest logic.
