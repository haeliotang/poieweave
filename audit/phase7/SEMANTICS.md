# N7 Semantics and Event Boundaries

- AssetStatus is read from the canonical `current.assets` at observation time.
- `Before` is sampled after ledger advance and before completion/exogenous event processing.
- `After` is sampled after the event batch at the timestamp (or after a sample-only boundary).
- Same-timestamp completion+failure is a batch; N7 does not invent per-event phases.
- Repair interruption samples `After` before returning `Blocked`, so the failure state is visible.
- Post-termination requests are `UNOBSERVABLE_AFTER_TERMINATION` with no statuses; they are never
  backfilled with `Available` or a terminal snapshot.
- `Unknown` is a model status; unobservable is an observation-system status.
