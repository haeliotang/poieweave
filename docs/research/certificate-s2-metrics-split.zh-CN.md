# certificate-s2-metrics Split

This splits the aggregate metrics certificate into metric groups. It does not reissue it.

| group | claims | canonical | disposition |
| --- | --- | --- | --- |
| `adversarial` | C11 | s2-canonical-c11 | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `adversarial_resupply` | C11 | s2-canonical-c11 | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `base05` | C5 | s2-canonical-c5, s2-canonical-c6 | `MIGRATED` |
| `base08` | C7 | s2-canonical-c7 | `MIGRATED` |
| `energy_interventions` | C8, C13 | s2-canonical-c8, s2-canonical-c13 | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `insert_sharpening` | C7 | s2-canonical-c7 | `MIGRATED` |
| `recharge_boundary` | C71 | s2-canonical-metrics-groups | `MIGRATED` |
| `recovery_coverage` | C71 | s2-canonical-metrics-groups | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `redundancy` | C9, C14 | s2-canonical-c9, s2-canonical-c14 | `MIGRATED_WITH_SEMANTIC_CORRECTION` |
| `redundancy_time` | C14 | s2-canonical-c14 | `SUPERSEDED_BASELINE` |
| `regenerative_capacity_days` | C71 | s2-canonical-metrics-groups | `MIGRATED` |
| `resupply` | C10 | s2-canonical-c10 | `MIGRATED` |
| `yield_sensitivity` | C71 | s2-canonical-metrics-groups | `MIGRATED` |

## Summary

- groups: 13
- migrated: 7
- migrated with semantic correction: 5
- partial: 0
- superseded baseline: 1
- not migrated: 0
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

12/12 day-level rows agree; energy-limited rows shift 16 -> 16.25 d

### `insert_sharpening`

2/3 inserts 40/50->60, 4 already 60

### `recharge_boundary`

average recharge 2.50 kW and first energy-limited load 2.6 kW agree

### `recovery_coverage`

legacy 5/10 = asset restorability; canonical 9/10 = service impact; definition versioned

### `redundancy`

7/8 rows agree; shared repair robot baseline 30->20 d at event time

### `redundancy_time`

redundant 90 agrees; baseline 31->2 d under canonical in-repair failures

### `regenerative_capacity_days`

canonical Base-08 minus Base-05 horizon = 20 days

### `resupply`

interval->horizon mapping agrees 4/4

### `yield_sensitivity`

yield 1.0/0.8/0.5 -> 60/60/50 days in both paths

