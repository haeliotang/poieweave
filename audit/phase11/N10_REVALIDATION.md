# N10 Revalidation (N11 Phase 0)

## Findings

- N10 report marked N10-E `NOT ESTABLISHED` while CLAIMS.json marked it `ESTABLISHED`.
- N10 trajectory reconstructed external events from endpoint state and sorted them by
  `(time, kind)`; this cannot establish external execution order.
- N10-OR2 acceptance used a case-specific branch.

## Corrected interpretation

N10-E is established for its recorded scope: frozen/current manifests, CI, historical
certificates and clean check-all. The external event order remains `NOT_OBSERVED`;
N10-D remains scoped event-level + endpoint state, not native external event order.

## Changes

- N10 report aligned with CLAIMS.
- N10 policy classifier made case-independent.
- N10 trajectory output records `external_event_order = NOT_OBSERVED_FROM_ENDPOINT_TRACE`.
- Original N10 frozen evidence bytes are not modified.
