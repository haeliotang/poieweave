# S1 Recursive Capability Closure — pre-registration

- Experiment ID: `s1-capability-closure`
- Date: 2026-11-14
- Status: completed; final table in [REPORT.md](REPORT.md)
- Scope: model-level analysis only; synthetic Base-01/Base-04 assumptions.

## Question

Given a failed asset in one of the frozen Base models, can the system restore it
using only:

- assets that are initially usable (including their `requires` chain),
- accepted inventory,
- local processes whose verification is at least `modeled`,
- capabilities provided by usable/restorable assets,
- and a serial time horizon?

If not, what is the minimal blocking condition found by the search?

## Method

A depth-first AND/OR search over restore processes, resource production
processes, capability providers and occupied assets. Inventory is consumed;
cycles in asset, resource or process chains are reported, not guessed away.
Processes below `modeled` verification are rejected. Durations are serial and
must fit the requested horizon.

## Cases and predictions before the final run

| Case | Scenario | Target | Horizon | Prediction |
| --- | --- | --- | --- | --- |
| s1-base04-lathe | Base-04, lathe failed | `lathe-1` | 12 h | restorable via spare `lathe-nut` in 2 h |
| s1-base04-pump | Base-04, pump degraded | `pump-p1` | 12 h | restorable via accepted sleeve in 3 h |
| s1-base04-pump-too-short | Base-04, pump degraded | `pump-p1` | 2 h | blocked by horizon |
| s1-base01-lathe | Base-01, lathe failed | `lathe-1` | 12 h | blocked by self-repair cycle |
| s1-base04-lathe-no-spare | Base-04, no lathe-nut lot | `lathe-1` | 12 h | blocked by self-repair cycle |
| s1-base01-pump-no-sleeve | Base-01, no sleeve lot | `pump-p1` | 12 h | restorable via turn + inspect + replace, 7.5 h |
| s1-base01-bus-no-breaker | Base-01, bus failed, no breaker lot | `bus-1` | 12 h | blocked; robot is not available without the failed bus |

The expected output is frozen in [expected.txt](expected.txt).

## Success criterion

The runner reproduces the table exactly in CI, and the cases distinguish:

1. a positive recursive repair (spare part),
2. a positive production chain (make a missing part),
3. a self-repair cycle,
4. a missing external input, and
5. a time-horizon failure.

A PASS is not evidence of physical feasibility; it is a model-level existence
result under the serial, synthetic assumptions above.
