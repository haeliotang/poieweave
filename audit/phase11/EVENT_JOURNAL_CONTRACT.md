# Native Event Journal Contract

FMI output `eventJournal` (String, VR 100), `eventJournalCount` (101),
`eventJournalOverflow` (102).

Schema `poie.s5v-n11-native-events/1`:

```text
capacity, count, overflow
events[]:
  seq            strictly increasing within one run
  at             real simulation time
  kind           InitialRepairStarted | AssetFailure | RepairInterrupted |
                 RepairCompleted | RestorationCommitted | RunTerminated
  origin         NATIVE_EXTERNAL_FMU
  target         named asset when applicable
  before[7]      selected_robot, asset_health, dependency_mask, failure_applied,
                 repair_state, restoration_state, termination_state
  after[7]       same dimensions after the transition
```

Capacity exhaustion sets `overflow=true` and does not overwrite old events.
No wall-clock or random event IDs.
