# S5-V N7 — Native Asset-State Observatory & Certificate v3 Migration

基线：`117ba8169995c83a0d4225925e3faf6025913a9e`。

## 状态

```text
S5-V N7 NATIVE ASSET-STATE OBSERVATORY
AND CERTIFICATE V3 MIGRATION
  ESTABLISHED (SCOPED)

N7 NATIVE ASSET-STATUS OBSERVATORY
  ESTABLISHED (SCOPED: 14 preregistered rows; N7-07 batch semantics documented)

N7 OBSERVATION NONINTERFERENCE
  ESTABLISHED (TESTED SCENARIOS)

N7 NATIVE REPAIR-DRIVEN RESTORATION EVIDENCE
  ESTABLISHED (SCOPED: pump-p1 + N6-A-10800 overlap MATCH)

N7 CERTIFICATE V3 MIGRATION
  ESTABLISHED (SCOPED: 41 current canonical claims; v2 frozen/historical)

N7 REPRODUCIBILITY
  ESTABLISHED (SCOPED: frozen/execution manifests; clean checkout)

V1 / V2 HISTORICAL CERTIFICATE INTEGRITY
  VERIFIED
```

## Native observations

| ID | covered | result |
| --- | --- | --- |
| N7-00 | yes | no-failure control, 0/3600/7200/10800/14400 |
| N7-01 | yes | initial state |
| N7-02 | yes | completion Before Degraded / After Available |
| N7-03 | yes | selected robot fails, repair interrupted |
| N7-04 | yes | unselected lathe failure, repair continues |
| N7-05 | yes | required tool-t1 failure at 9000; tool Failed, target Failed, repair interrupted |
| N7-06 | yes | bus-1 failure interrupts repair |
| N7-07 | yes, scoped | same-time completion+failure needs a post-batch sample; see limitation |
| N7-08 | yes | completion exactly at horizon |
| N7-09 | yes | failure at horizon not processed |
| N7-10 | yes | post-termination request unobservable |
| N7-11 | yes | multiple failures |
| N7-12 | yes | Degraded/Unknown statuses |
| N7-13 | yes | multiple same-time requests |
| N7-14 | yes | overlap with N6-A-10800 derived restoration (lathe-1, pump-p1 MATCH) |

Post-termination requests at 20000: `UNOBSERVABLE_AFTER_TERMINATION`, no backfill.
All observed rows have `status_source=NATIVE_CANONICAL_RUNTIME`.

## Event-boundary limitation (N7-07)

At `t=10800`, canonical processes completion in one loop batch and the exogenous
failure in the next batch at the same clock time. Three `After` requests at
10800 all observe the post-completion state; a sample at 10800.001 observes the
post-failure state (`robot-m1=Failed`). N7 therefore records both states but
does not claim a per-event phase label for same-timestamp batches.

## Noninterference

`run_engine` and `run_engine_observed` EngineOutcome equality is asserted for every
N7 observation case and by unit tests for completion, interruption, sequential calls,
thread isolation and error cleanup.

## Not established

- step-level resource release
- event-engine priority input
- post-interruption replanning
- autonomous recovery
- Level 3 independent planning
- general trajectory-level / full S2 cross-model validity
- physical-world correctness
- per-event phase labels for same-timestamp batches
