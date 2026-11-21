# Event Order Semantics

- Failures at different times execute in time order.
- Failures at the same time share the batch; each gets a distinct event_sequence.
- Canonical applies all failures in a batch, then evaluates repair interruption.
- After interruption the run terminates; later failures are not executed and are
  not fabricated in the external journal.
- Completion at makespan precedes a same-timestamp failure (N10/N12 scoped rule).
