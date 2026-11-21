# Temporal Ownership Contract

At any event time t:

```text
unfinished_tasks(t) = tasks whose end_s > t
needed_resources(t) = union resources of unfinished_tasks(t)
```

A `ResourceReleased(r,t)` is valid only if:

- r was previously held by the group;
- r is not in needed_resources(t);
- r was not already released;
- the event follows TaskCompleted/ResourceReleased priority.

Final-completion sets are not valid substitutes for unfinished_tasks(t).
