# Bounded Reference Semantics

Domain: 1-2 tasks, 2 robots, 0-4 failures, split/shared upstream, tool AND.
The reference oracle enumerates admissible robot assignments, task orders and
start times; computes dependency closure and failure hits from the input
contract only; and emits policy-conditional admissible schedules/outcomes.

Forbidden: canonical/FMU output, task_resources, expected_first_hit, model_outcome.
