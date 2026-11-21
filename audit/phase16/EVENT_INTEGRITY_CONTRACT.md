# Event Integrity Contract

Validated event families:

```text
TaskCompleted
ResourceReleased
AssetFailure
RepairInterrupted
RepairCompleted
```

Checks: identity, time, same-time order, before/after presence, state continuity,
duplicate events, missing releases, post-termination events, release
preconditions.
