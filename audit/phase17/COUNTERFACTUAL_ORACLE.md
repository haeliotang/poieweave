# Counterfactual Oracle

Layers:

1. input contract oracle (task/robot/tool/failure legality);
2. per-model execution oracle (schedule, closure, fault sensitivity,
   interruption/commit/restoration);
3. counterfactual relation oracle.

Relation classes: SAME_OUTCOME_VALID, CHANGED_OUTCOME_EXPLAINED,
VALID_POLICY_DEPENDENT_DIVERGENCE, INVALID_EXECUTION,
UNEXPLAINED_CAUSAL_DIVERGENCE.

A model is validated against its own selected resources and contract, never
against the other model's output.
