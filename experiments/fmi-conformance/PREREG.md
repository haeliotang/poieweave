# FMI 3.0 Conformance — pre-registration

- Experiment ID: `fmi-conformance`
- Date: 2026-11-15
- Status: modelDescription/container contract only; no co-simulation
- Scope: one minimal energy model description.

## Question

Can Poieweave validate a minimal FMI 3.0 modelDescription and package it as an
FMU container?

## Method

Parse a hand-written FMI 3.0 description, check required attributes, variable
causality/variability and unique value references, then build and inspect a ZIP
container.

Prediction: description checks pass; co-simulation is explicitly `not_run`.

## Limitation

No FMU binary, no C ABI, no numerical stepping. This is not an FMI execution
claim.
