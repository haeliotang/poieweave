# S5-V N14-R — Canonical Multi-Task Representability & Controlled v7 Migration

Baseline: `161273d4599ff07fdd90f7a4be9e3bd6e221a94f`.

```text
N14R-0 REPRESENTABILITY DECISION — VERIFIED
N14R-A CANONICAL GROUP EXECUTION — ESTABLISHED (SCOPED)
N14R-B ORIGINAL N14-B RECOVERY — ESTABLISHED (SCOPED)
N14R-C GENERATED CROSS-MODEL EXTENSION — NOT ESTABLISHED
N14R-D CERTIFICATE GOVERNANCE — IN PROGRESS
N14R-E REPRODUCIBILITY — IN PROGRESS
```

The canonical engine is extended with an explicit repair-group API that reuses
the existing event loop, scheduler, dependency closure, inventory ledger and
atomic commit path. Legacy `run_engine`/`run_engine_event_observed` entry points
are unchanged.
