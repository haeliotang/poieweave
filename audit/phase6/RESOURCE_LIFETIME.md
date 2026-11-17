# N6-A — Resource Lifetime Audit Result

## Contract found in canonical

Canonical `RunningRepair` has one group `end_s`. Selected resources are added to
`execution_dependencies` until group commit. Plans are applied together at
`end_s`. A failure of any selected resource or restoration target before group
commit returns `Blocked("repair interrupted ...")`.

Therefore canonical resource lifetime is **GROUP_LEVEL**:

```text
resource_acquired_s = 0
resource_released_s = group_commit_s (or run termination on interruption)
step_completed_lathe_s = 7200
step_completed_pump_s  = 10800
group_commit_s         = 10800
step-level release     = NOT_REPRESENTABLE
```

## Boundary results

| failure | failure applied | group commit | interrupted | restored |
| ---: | ---: | ---: | --- | --- |
| 7199 | 7199 | none | yes | no |
| 7200 | 7200 | none | yes | no |
| 7201 | 7201 | none | yes | no |
| 9000 | 9000 | none | yes | no |
| 10800 | 10800 | 10800 | no | yes |

At 7200 s the lathe step has reached its nominal end, but canonical does not
release robot-m1 until group commit. Failure at 7200 or 7201 therefore interrupts
the whole group. At 10800 s, group completion is processed before the exogenous
failure, so restoration commits and the failure is traced after commit.

## N5-03 interpretation

N5-03 demonstrates **group-level interruption consistency**, not step-level
resource lifetime correctness. The external FMU reproduces the group-level
policy. Neither model supports per-step release; N6 does not claim it.

## Status

```text
N6 RESOURCE LIFETIME SEMANTICS
  ESTABLISHED (SCOPED: group-level hold until group commit)

N6 STEP-LEVEL RESOURCE RELEASE
  NOT REPRESENTABLE / NOT ESTABLISHED
```

Negative controls cover step-release-at-7200, after-commit interruption, moved
step completion, late failure application and missing restoration.
