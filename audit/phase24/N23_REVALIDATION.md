# N23 Revalidation

## Original preregistration
Preserved in `audit/phase23/PREREG.md` and the N23 goal text.

## Intermediate research state
`audit/phase23/REPORT.zh-CN.md` retained early IN PROGRESS / PARTIAL / NOT ESTABLISHED
lines before final claims.

## Final reported claims
`audit/phase23/CLAIMS.json` marks N23-0 and A–G VERIFIED.

## Post-hoc revalidation
| Claim | Evidence | Status |
|---|---|---|
| N23-0 | `manifests/n23-freeze.json`, failing witnesses | VERIFIED_WITH_CORRECTION |
| A | `native-checkpoints/native.json`, projection negatives | VERIFIED |
| B | `candidate-generation/resource-discovery.json` | VERIFIED |
| C | choice-set trust + canonical trusted certificate tests | VERIFIED |
| D | canonical committed planned start test | VERIFIED |
| E | `temporal-comparisons/strict.json`, native event test | VERIFIED |
| F | `continuation-faults/results.json`, canonical persistence | VERIFIED |
| G | `executable-mutants`, `m2-reducer`, certificate, clean check | VERIFIED |

## Corrections required by N24
1. N23 external certificate label said v16 while binding v17; see `N23_CERTIFICATE_ERRATA.md`.
2. Pre-start continuation failure was skipped and execution completed; reproduced in
   `failing-witness/PRESTART_FAILURE_FALSE_PASS/`. This is a real N23 defect, not a PASS.
3. N23 CI did not contain a dedicated N23 replay job; N24 adds one.
