# S4 Reason Classes — pre-registration

- Experiment ID: `s4-reason-classes`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: coarse blocking-reason classes, not full reason equivalence.

## Question

For blocked cases in the 112-case corpus, does the independent checker agree on
the coarse reason class: `no_restorer` vs `cycle`?

## Method

Rust exports the first closure blocker reason. Classify it:

- `no restorer ...` → `no_restorer`
- `cyclic ...` → `cycle`
- anything else → `other`

Python checks whether the target has any process that restores it. No restorer →
`no_restorer`; otherwise treat as cycle for this corpus. Compare all 86 blocked
cases.

Prediction: 86/86 category agreement.

## Limitation

Only two classes and the first reason; no minimal witness or exact reason text.
