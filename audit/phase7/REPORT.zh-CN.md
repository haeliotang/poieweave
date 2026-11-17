# S5-V N7 — Native Asset-State Observatory & Certificate v3 Migration

基线：`117ba8169995c83a0d4225925e3faf6025913a9e`。

## 状态

```text
N7 NATIVE ASSET-STATUS OBSERVATORY
  ESTABLISHED (SCOPED: base-08 completion and interruption scenarios)

N7 OBSERVATION NONINTERFERENCE
  ESTABLISHED (TESTED SCENARIOS: base-08 control/completion/interruption/error/thread)

N7 NATIVE REPAIR-DRIVEN RESTORATION EVIDENCE
  ESTABLISHED (SCOPED: pump-p1 completion and interruption)

N7 CERTIFICATE V3 MIGRATION
  ESTABLISHED (SCOPED: 41 current canonical claims; v2 frozen/historical)

N7 REPRODUCIBILITY
  ESTABLISHED (SCOPED: frozen/execution manifests; clean checkout)
```

## Native observations

- `N7-00`: no-failure control; at 0/3600/7200/10800/14400 assets are native.
- `N7-02` (covered by N7-00 at 10800): `pump-p1` Before `Degraded`, After `Available`.
- `N7-03`: selected `robot-m1` fails at 7200; Before `Available`, After `Failed`;
  `pump-p1` remains `Degraded`; running repair remains interrupted.
- Post-termination request at 20000: `UNOBSERVABLE_AFTER_TERMINATION`, no backfill.
- All rows have `status_source=NATIVE_CANONICAL_RUNTIME` except explicit unobservable rows.

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
- all 14 preregistered N7 scenario rows; current evidence is scoped to the listed cases
