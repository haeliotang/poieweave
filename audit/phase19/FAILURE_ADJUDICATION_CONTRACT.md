# Failure Adjudication Contract

Every historical unexpected effect must be replayed from frozen input and
classified into exactly one primary category:

ORACLE_FALSE_POSITIVE, CANONICAL_EXECUTION_DEFECT, EXTERNAL_EXECUTION_DEFECT,
SHARED_MODEL_CONTRACT_VIOLATION, VALID_POLICY_DEPENDENT_DIFFERENCE,
INVALID_INTERVENTION_CONTRACT, EVIDENCE_OR_PROVENANCE_FAILURE,
NOT_REPRESENTABLE, UNRESOLVED.

Secondary reasons (e.g. EXPECTED_EFFECT_DIRECTION_ERROR) are retained.
The original N18 ledger is immutable.
