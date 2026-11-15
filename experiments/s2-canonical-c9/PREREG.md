# S2 Canonical C9 Migration — pre-registration

- Experiment ID: `s2-canonical-c9`
- Date: 2026-11-16
- Status: completed; 6/6 rows agree after redundant-repair fix
- Scope: Base-08 pump redundancy at 2.5/2.6/3.0 kW.

## Question

Does canonical engine preserve the redundant-pump 60→90 extension and 2.6 kW
energy limit?

## Method

Add pump-p2 and make potable-water use requires_any[pump-p1,pump-p2]; run legacy
and canonical with identical events/load. Compare days/energy/termination.

Prediction: initial canonical over-repairs failed pump despite the alternative;
fix required, then 6/6 agreement.

## Limitation

Model-level; one redundancy layout.
