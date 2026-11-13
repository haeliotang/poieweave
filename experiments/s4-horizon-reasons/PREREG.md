# S4 Horizon Reason Classes — pre-registration

- Experiment ID: `s4-horizon-reasons`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: short-horizon reason-class check.

## Question

Can the independent checker agree with Rust on `no_restorer` vs
`horizon_exceeded` classes when the horizon is deliberately shorter than every
repair plan?

## Method

Set a 900 s horizon for every asset in Base-01/04/05/06/07/08. Export Rust
blocked reasons. Python classifies:

- no restorer → `no_restorer`;
- all restorer durations exceed the horizon → `horizon`;
- otherwise `cycle` for this corpus.

Prediction: all blocked cases agree.

## Limitation

No missing-resource or unverified-process cases; horizon class uses nominal
duration only.
