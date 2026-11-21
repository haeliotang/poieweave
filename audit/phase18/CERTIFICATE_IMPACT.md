# N18 Certificate Impact

Canonical execution semantics changed: per-task upstream dependency closure is
now derived from the scheduler's actual selected resources, not static
`process.occupies`.

- v9 certificates: frozen historically.
- v10: 41 `EXECUTION_SEMANTIC_CORRECTION_SUCCESSOR` successors.
- legacy non-OR behavior reproduced by `check-experiment-replays.py`.
- no replanning, recovery or event-priority changes.
