# N10 Evidence

Phase 0 task-level hardening:

```text
audit/phase9/canonical/*-summary.json     task_resources
audit/phase9/comparisons/resource-trajectories.json
audit/phase9/negative-controls/resource-results.json
```

N10-A boundary evidence:

```text
audit/phase10/canonical/N10-0*-events.csv
audit/phase10/canonical/N10-0*-summary.json
audit/phase10/external/N10-0*-fmpy-1.csv
audit/phase10/comparisons/boundaries.json
audit/phase10/negative-controls/boundary-results.json
```

N10-B/C evidence:

```text
audit/phase10/canonical/N10-U0*-events.csv
audit/phase10/canonical/N10-OR*-summary.json
audit/phase10/external/N10-U0*-fmpy-900.csv
audit/phase10/external/N10-OR*-fmpy-900.csv
audit/phase10/comparisons/causal.json
audit/phase10/negative-controls/causal-results.json
external/fmu-n10-scheduler/
```

N10-D trajectory evidence:

```text
audit/phase10/comparisons/trajectories.json
audit/phase10/negative-controls/trajectory-results.json
audit/phase10/canonical/N10-U0*-events.csv
audit/phase10/external/N10-U0*-fmpy-900.csv
```

External events remain `DERIVED_FROM_EXECUTION_OUTPUT`; intermediate before-state
is `NOT_OBSERVED_FOR_INTERMEDIATE_BOUNDARY`.
