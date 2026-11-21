# Intervention Contract

Each pair binds:

```text
intervention_id, type, baseline_contract_hash, intervened_contract_hash,
changed_variables, unchanged_variables, seed, replay command
```

Supported types: FAILURE_TARGET_CHANGE, FAILURE_TIME_CHANGE,
FAILURE_INSERTION, FAILURE_REMOVAL, RESOURCE_ELIGIBILITY_CHANGE,
DEPENDENCY_EDGE_CHANGE.

Every accepted pair executes four real runs:

```text
Canonical(B), Canonical(I(B)), External(B), External(I(B))
```

No counterfactual output is synthesized from baseline evidence.
