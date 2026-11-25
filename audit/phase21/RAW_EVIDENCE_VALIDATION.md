# Raw Evidence Validation

`validate_execution_bundle` computes validity from input spec, pre-execution
reference prediction, schedule observation, native events and manifest hashes.
Error classes include EVENT_ORDER_VIOLATION, SHARED_RESOURCE_CONTENTION,
SHARED_DEPENDENCY_LOSS, FUTURE_DEPENDENCY_LOSS, REQUIRED_TOOL_MISSING,
PREMATURE_GROUP_COMMIT and EVIDENCE_INTEGRITY_FAILURE.
