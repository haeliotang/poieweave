# N16 Revalidation (N17 Phase 0)

## False-pass reproduced

With a legal OR policy divergence (N16-09), removing the external
`AssetFailure` event was accepted by the N16 validator because every
`EXTERNAL_FAILURE` finding was filtered by the blanket policy exemption.

Additional mutated controls:

```text
valid policy + wrong external closure  -> EXTERNAL_SHARED_DEPENDENCY_LOSS
valid policy + missing failure         -> EXTERNAL_FAILURE_MISSING_0
valid policy + early restoration       -> PREMATURE_GROUP_COMMIT_EXTERNAL
```

## Fixes

- Blanket policy findings suppression removed for event sets, failures,
  closure and restoration preconditions.
- Event trajectories compared in original order with a registered same-time
  priority order; same-time event sets compared per timestamp.
- Same-time closure uses per-event aligned external dep_after for strict cases;
  policy cases use independent per-model closure invariants.
- Generated closure oracle now uses the last event at Task A completion and
  excludes selected resources from upstream closure.
- Task upstream paths derive from each model's actual selected robot, not task
  index.
- N16 final claim/report status reconciled in the completion addendum.

## N16 final status

N16-0/A/B/C/D/E/F evidence was already present; after revalidation:

```text
N16-0 VERIFIED
N16-A ESTABLISHED (SCOPED)
N16-B ESTABLISHED (SCOPED)
N16-C ESTABLISHED (SCOPED)
N16-D VERIFIED (SCOPED)
N16-E VERIFIED
N16-F VERIFIED
```
