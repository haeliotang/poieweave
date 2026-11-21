# N12 Revalidation (N13 Phase 0)

Original N12 report SHA-256 is recorded in the N12 evidence commit history.

## Corrections

- Removed the stale N12 "未建立" block that still listed state projection, overflow,
  step invariance and both-invalid as open after those gates were closed.
- Removed `N10-OR2` case-specific acceptance from the projection comparator;
  policy classification now derives from actual selected resources and failure target.
- Resource validation now loops over every task, not only Task 1.
- `validate_stress.py` fails when multi-instance verification is required but fmpy
  is unavailable; CI invokes it with `--require-multi-instance`.

## Preserved

N12 frozen journals, projections and claims are not rewritten except corrected
status text and comparator evidence.
