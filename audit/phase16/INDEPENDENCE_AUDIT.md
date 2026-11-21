# Independence

- Canonical closure: canonical runtime execution dependencies.
- External closure: N16 FMU's own dep1/dep2 masks and task completion logic.
- Validator recomputes expected task schedule, resource ownership, release
  legality, shared/future dependency persistence, and physical failure history
  from the contract, not from canonical task_resources alone.
