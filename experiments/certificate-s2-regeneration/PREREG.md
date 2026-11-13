# Regeneration Certificate — pre-registration

- Experiment ID: `certificate-s2-regeneration`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: model-level regeneration horizon certificate.

## Question

Can the Base-05 → Base-08 regeneration result (40 → 60 days) be packaged with
model identities and checked independently?

## Method

Certificate binds Base-05/Base-08 paths and SHA-256, survived days 40/60, the
conclusion and a model-only claim scope. Python verifies hashes, days and scope.

Prediction: `certificate=ok`.

## Limitation

Model-level regeneration only; no physical yield or material-balance claim.
