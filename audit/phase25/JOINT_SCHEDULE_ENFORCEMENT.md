# Joint Schedule Enforcement

`resume_engine_with_authorization` now reads the selected candidate's
`task_schedule` from the trusted choice-set certificate payload and compares it
with the native scheduler's per-task start/end before any event or state change.

```text
authorized:    B 9000-19800, C 19800-23400   -> CompletedHorizon
unauthorized:  B 9000-19800, C 9000-12600    -> Blocked full schedule mismatch
clock on rejection = 9000 (no advance)
```

Missing tasks, malformed windows, task-set mismatch and per-task start/end
mismatch all reject with `replan_rejected: authorized full schedule ...`.
The payload is covered by the trusted certificate commitment and registry.
