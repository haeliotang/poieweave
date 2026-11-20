# S5-V N11 — Native External Event Evidence & Bidirectional Validation

Baseline: `3f9fff1b618ebade815f10a6e321e2cc3a169788`.

## Scope

- Native events are recorded inside the independent C99 FMU at the execution site.
- Journal uses fixed capacity 32; overflow is explicit.
- Before/after states are captured from the same FMU instance.
- FMI route: Co-Simulation without Event Mode; journal exported as FMI 3.0 String output.
- No endpoint reconstruction is used for N11 native claims.

## Gates

```text
N11-0 PRECONDITION EVIDENCE HARDENING — VERIFIED
N11-A EXTERNAL NATIVE EVENT JOURNAL — IN PROGRESS
N11-B EXTERNAL TRUE BEFORE/AFTER — IN PROGRESS
N11-C BIDIRECTIONAL NATIVE STATE-TRANSITION — NOT ESTABLISHED
N11-D ADVERSARIAL NATIVE EVENT VALIDITY — NOT ESTABLISHED
N11-E EVIDENCE/CERTIFICATE INTEGRITY — NOT ESTABLISHED
```
