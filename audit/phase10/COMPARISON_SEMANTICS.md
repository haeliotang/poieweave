# N10 Comparison Semantics

- task masks are evaluated per task, never from the group union
- missing task mapping is `TASK_MAPPING_NOT_OBSERVED` / not comparable
- different valid decisions are not failures
- same final state is not full trajectory evidence
- first causal divergence is reported by dimension
