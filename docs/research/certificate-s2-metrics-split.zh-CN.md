# certificate-s2-metrics Split

This splits the aggregate metrics certificate into metric groups. It does not reissue it.

| group | claims | canonical | disposition |
| --- | --- | --- | --- |
| `adversarial` | C11 | s2-canonical-c11 | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `adversarial_resupply` | C11 | s2-canonical-c11 | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `base05` | C5 | s2-canonical-c5, s2-canonical-c6 | `MIGRATED` |
| `base08` | C7 | s2-canonical-c7 | `MIGRATED` |
| `energy_interventions` | C8, C13 | s2-canonical-c8, s2-canonical-c13 | `PARTIAL` |
| `insert_sharpening` | C7 | s2-canonical-c7 | `MIGRATED` |
| `recharge_boundary` | NONE | NONE | `NOT_MIGRATED` |
| `recovery_coverage` | NONE | NONE | `NOT_MIGRATED` |
| `redundancy` | C9, C14 | s2-canonical-c9, s2-canonical-c14 | `PARTIAL` |
| `redundancy_time` | C14 | s2-canonical-c14 | `SUPERSEDED_BASELINE` |
| `regenerative_capacity_days` | NONE | NONE | `NOT_MIGRATED` |
| `resupply` | C10 | s2-canonical-c10 | `MIGRATED` |
| `yield_sensitivity` | NONE | NONE | `NOT_MIGRATED` |

## Summary

- groups: 13
- migrated: 4
- migrated with semantic correction: 2
- partial: 2
- superseded baseline: 1
- not migrated: 4
- reissued: 0

## Reasons

### `adversarial`

minimal set filter-1; day-0 exact 0.125 d and day-10 legacy row invalid

### `adversarial_resupply`

same C11 adapter; resupply does not close the capability gap

### `base05`

Base-05 pump intervals and coverage agree

### `base08`

Base-08 regeneration 40->60 and 50% yield validated

### `energy_interventions`

load rows agree; recharge intervention rows not canonicalised

### `insert_sharpening`

2/3 inserts 40/50->60, 4 already 60

### `recharge_boundary`

average-recharge threshold not yet canonicalised

### `recovery_coverage`

Base-05 5/10 independent coverage lacks a canonical coverage adapter

### `redundancy`

redundant pump agrees; filter/robot restorer rows not canonicalised

### `redundancy_time`

redundant 90 agrees; baseline 31->2 d under canonical in-repair failures

### `regenerative_capacity_days`

single scalar without scenario binding

### `resupply`

interval->horizon mapping agrees 4/4

### `yield_sensitivity`

yield rows not canonicalised

