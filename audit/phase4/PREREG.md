# S5-V N4 — Discrete-Event Cross-Model Challenge (Preregistration)

Status: PREREGISTERED before N4 FMU implementation.
Baseline: `33583f53ac046a435b6756b4a448bdc9b3ff23d9` (PR #573).

## Question

When canonical and an independently implemented FMI 3.0 model receive the same
scheduled PV failure, do they produce causally consistent state transitions at
the event time, and can every divergence be attributed to a registered model
assumption, numerical integration, or event timing?

## Event contract

```text
event_id: string
scheduled_time_s: float (simulation seconds)
event_type: AssetFailure | HorizonReached | RepairCompleted | Observation
target: asset id or null
parameters: object
source: EXOGENOUS | ENDOGENOUS | OBSERVATION
phase: Before | After
```

Only `pv-1` failures are exogenous in the minimal set. Initial pump repair and
its completion are endogenous canonical events. FMU repair completion is a
fixed, independently implemented 3 h task with uniform energy draw; it does not
consume canonical planning results.

## Fixed scenario

- Model: `scenarios/base-08/model.toml`
- Battery: 200 kWh capacity, 10 kW charge limit, initial 40 kWh
- PV: 2500 W while `pv-1` is available; load 3000 W
- Initial pump repair: 0.5 kWh over 0–10800 s
- Horizon: 20 h
- Canonical repair energy: committed at completion
- FMU repair energy: uniform 0.5 kWh over the 3 h window
- Failure effect: from the exact failure time, PV generation is 0 until horizon
- Canonical same-time order: repair completion is considered before exogenous
  failure events at the same timestamp (engine loop order)

## Cases

| id | exogenous PV failure | purpose |
| --- | --- | --- |
| N4-00 | none | control; original energy path unchanged |
| N4-01 | 7200 s | failure during repair, exact event boundary |
| N4-02 | 4500 s | non-grid-aligned failure time |
| N4-03 | 10800 s | failure at repair completion; event order |

Expected canonical depletion:

```text
N4-00: horizon 20 h, final energy 29.5 kWh
N4-01: 14.833333 h = 53400 s
N4-02: 14.208333 h = 51150 s
N4-03: 15.666667 h = 56400 s
```

Expected registered repair-assumption difference:

```text
FMU - canonical = -0.5 kWh * t / 3 h   for 0 < t < 3 h
FMU - canonical = 0                    for t >= 3 h
```

This expression applies only while the initial repair is not interrupted and
the repair draw is unchanged. It does not apply to repair interruption cases.

## Samples

Common samples: 0, 3600, 7200, 10800, 14400 s and terminal.
Event samples: failure-time Before/After; 10800 s Before/After.

## Tolerances

- Energy: 1 J absolute after subtracting the registered repair assumption
- Event time: 1e-6 s
- Conservation residual per interval: 1 J
- Numerical step sensitivity: 3600, 900, 1000 s; failure at 4500 s must be
  reached by real step splitting, not rounding

## Required negative controls

| mutation | expected classification |
| --- | --- |
| FMU PV failure delayed 1 h | EVENT_TIME_DISPLACEMENT |
| FMU PV failure ignored | STATE_TRANSITION_MISMATCH |
| FMU keeps generating after failure | RESOURCE_LEDGER_DIVERGENCE |
| repaired completion delayed 1 h | EVENT_TIME_DISPLACEMENT |
| same-time order reversed | EVENT_ORDER_MISMATCH |
| reference/FMU hash changed | EVIDENCE_HASH_MISMATCH |

## Scope

This preregistration does not claim trajectory-level or full S2 cross-model
validity. It covers only the listed PV-failure events, initial pump repair,
energy trajectory, terminal state and first-divergence classification.
Restoration, repair interruption, shared-resource scheduling and load steps are
outside this minimal set.
