# N19 Revalidation (N20 Phase 0)

## Original preregistration
N19 PREREG is preserved unchanged.

## Final acceptance
N19-0/A/B/C/D/F were completed. N19-E was scoped to tested mutants/held-out
stress; its 64 held-out attempts were 50 accepted and 14 `UNEXPECTED_EFFECT`,
all retained in the historical ledger.

## Post-hoc audit
The N19 held-out generator reused the asymmetric N18 expected-effect rule
(`NO_HIT -> HIT` only), so its 14 failures were not yet adjudicated. N19 mutant
results M2/M3 were inherited historical signatures, not current-run
re-executions. N20 addresses both.

## N20 objective
Adjudicate all 14 held-out failures, build a pre-execution independent
reference oracle, and re-execute all three mutants in the current run.
