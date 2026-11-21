# Certificate Impact Matrix

Change: explicit same-target repair-group execution API and task schedule
observation fields. Classification: `EXECUTION_SEMANTIC_EXTENSION`.

| Area | Impact |
|---|---|
| `run_engine` legacy entry | NO_IMPACT (calls inner with `None`) |
| event loop / failure batch / horizon / fail-stop | NO_IMPACT |
| closure / scheduler | NO_IMPACT (reused) |
| inventory / energy ledger | NO_IMPACT |
| `EngineTrace` | NO_IMPACT |
| `TaskResourceObservation` | OBSERVATION_ONLY new task_id/start/end fields |
| `RunningRepair` | unchanged |
| 41 v6 claims | execution expected outputs unchanged; replay passes |
| v6 certificates | frozen bytes; verified against baseline hash |
| v7 certificates | `EXECUTION_SEMANTIC_EXTENSION_SUCCESSOR` |

Legacy noninterference: `scripts/check-experiment-replays.py` passes 254
generated artifacts after the change.
