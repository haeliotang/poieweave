# State-Transition Contract

For every event `e`:

```text
restoration_target(e) = available  =>  exists RestorationCommitted r:
    r.time <= e.time  AND  all unfinished tasks completed by r.time
GroupCommit.time >= max(TaskCompleted.time)
shared robot/tool intervals must not overlap
shared dependency remains active until its last user exits
event ids unique, sequences contiguous, times non-decreasing
```

The independent oracle is `state_oracle.py`. It checks event sequence,
time monotonicity, group commit ordering, restoration state snapshots and
task interval overlap.
