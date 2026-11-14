# S4 Capability Subsets — pre-registration

- Experiment ID: `s4-capability-subsets`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: every non-empty subset of Base-05 capabilities downgraded.

## Question

Does the capability-verification fix hold when several capabilities are
downgraded at once, not only one?

## Method

5 capabilities → 31 non-empty subsets. For each, set the selected entries to
`declared`, run 9 target verdicts and compare: 279 cases.

Prediction: agreement; any mismatch is a semantic finding.

## Limitation

Base-05 only; process evidence unchanged.
