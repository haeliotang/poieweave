# Viability Certificate — S2 Load — pre-registration

- Experiment ID: `certificate-s2-load`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: machine-checkable evidence for one model result.

## Question

Can one result be packaged as a machine-checkable certificate containing model
identity, assumptions, result rows and an explicit claim scope?

## Method

Generate a certificate from Base-05 site-load runs:

- model path and SHA-256;
- 90-day horizon;
- 200 kWh / 10 kW battery assumptions;
- load rows 0/1/2/3/4 kW;
- conclusion and claim scope.

The Python verifier recomputes the model hash, checks schema/assumptions/rows
and the material→energy regime switch.

Prediction: `certificate=ok`.

## Limitation

Self-issued certificate; no signature, third-party attestation or physical
evidence.
