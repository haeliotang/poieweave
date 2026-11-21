# N15 Revalidation (N16 Phase 0)

## Temporal ownership bug reproduced

Old N15 release logic used the final completed-task set. In a no-failure
same-robot serial group, an injected `ResourceReleased(robot-m1) @7200` was
accepted because Task B was in the final completed set. The corrected validator
computes `unfinished_tasks(t)` from task end times and rejects it as
`SHARED_RESOURCE_RELEASE_VIOLATION_robot-m1`.

New regression case: `N15-D2-SERIAL-NOFAIL`.

## Other corrections

- Independent task schedule contract checks task start/end, robot mask/OR,
  tool AND, contention, makespan, selected resource identity.
- Current-run generation script `audit/phase15/run_cases.py` regenerates N15
  canonical and external evidence from contracts before validation.
- N15 CI now runs current-run generation, C harness, fmpy comparison, strict
  validator, negative controls and reducer.
- N15 REPORT stale v8/v9 and dual-importer limitation text replaced by a dated
  completion addendum; original PREREG hypotheses are retained.
