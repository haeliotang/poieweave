# v10 Certificate Semantic Errata

The 41 v10 certificate bytes, index, predecessor hashes and source hashes are
unchanged.

The `semantics` field inherited from v9 describes the pre-N15 execution model:
it mentions explicit same-target repair groups and says step-level release is
not implemented. It does not describe the v10 task-level execution dependency
release correction.

Clarification, not re-signing:

```text
TASK-LEVEL EXECUTION DEPENDENCY RELEASE
  implemented: completed task's selected-resource upstream closure exits

PHYSICAL RESOURCE RELEASE
  NOT established and NOT implied by the certificate
```

No physical asset release, shutdown, repair or industrial safety claim is made.
The original certificate hashes remain authoritative for the frozen bytes.
