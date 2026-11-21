# N16 Policy False-Pass Reproduction

Base: N16-09 legal OR policy divergence.

| Mutation | Old behavior | Fixed behavior |
|---|---|---|
| external AssetFailure removed | ACCEPTED | EXTERNAL_FAILURE_MISSING_0 |
| external shared closure removed | ACCEPTED | EXTERNAL_SHARED_DEPENDENCY_LOSS |
| external early restoration | ACCEPTED | PREMATURE_GROUP_COMMIT_EXTERNAL |

The three controls are executed by `false_pass_controls.py` against the formal
N16 validator.
