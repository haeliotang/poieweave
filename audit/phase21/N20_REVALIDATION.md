# N20 Revalidation (N21 Phase 0)

## Original preregistration
N20 PREREG unchanged.

## Final actual state
- N20-A/B/E/F evidence exists and main CI passed.
- N20 `REPORT.zh-CN.md` still contains `N20-F IN PROGRESS`; `CLAIMS.json` says
  VERIFIED. This addendum resolves consistency: N20-F is accepted as VERIFIED
  by the completed CI/clean run, while the report text is stale.
- N20 reference oracle has a shared-tool false pass: different robots but a
  shared required tool may be scheduled in parallel.
- N20 held-out dedup rule mutated baseline failures for duplicate candidates,
  which can make an intervention compound. Historical ledger is preserved.

## N21
Qualify the reference oracle and audit post-interruption representability
before any replanning claim.
