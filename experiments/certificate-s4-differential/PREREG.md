# S4 Differential Certificate — pre-registration

- Experiment ID: `certificate-s4-differential`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: evidence certificate, not a rerun.

## Question

Can the S4 differential verdict/plan evidence be packaged with content hashes so
a verifier can detect drift?

## Method

Certificate binds:

- `expected.txt` SHA-256 and `cases=112 matched=112 mismatched=0`;
- `expected-plans.txt` SHA-256 and `plans=26 verified=26`;
- explicit independent-implementation claim scope.

The certificate is also added to the S1–S4 bundle.

Prediction: both certificate and bundle verify.

## Limitation

Evidence certificate only; it does not rerun Python/Rust or prove equivalence.
