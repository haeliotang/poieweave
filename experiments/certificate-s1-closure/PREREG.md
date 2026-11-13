# Closure Certificate — pre-registration

- Experiment ID: `certificate-s1-closure`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: one S1 closure result packaged as evidence.

## Question

Can a single S1 closure verdict and plan be packaged so a separate
implementation can verify model identity and plan replay?

## Method

Generate a Base-04 lathe certificate containing model hash, target, horizon,
RESTORABLE verdict, plan steps, total duration and claim scope. Python checks
the hash and independently replays the plan.

Prediction: `certificate=ok`.

## Limitation

One case; model-level closure only, not physical repair.
