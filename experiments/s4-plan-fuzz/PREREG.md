# S4 Systematic Plan Fuzzing — pre-registration

- Experiment ID: `s4-plan-fuzz`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: deterministic negative mutation suite.

## Question

Does the independent plan verifier reject systematic mutations across **all**
RESTORABLE corpus plans, not only three hand-picked cases?

## Method

For each of the 26 RESTORABLE plans, generate five mutations:

1. remove all steps,
2. append an unknown process,
3. add 1 s to total duration,
4. change the target to a missing asset,
5. set the horizon to 1 s.

Expected: `mutations=130 rejected=130`.

## Limitation

The mutations are structural and deterministic; they do not search for subtle
physically plausible errors or valid alternative plans.
