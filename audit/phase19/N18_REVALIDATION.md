# N18 Revalidation (N19 Phase 0)

## Original preregistration
N18 PREREG remains unchanged.

## N18 final actual state
- N18-0/A/B/C/E/F were completed.
- N18-D was marked VERIFIED for the tested 16-control domain.
- N18 generated run: 64 attempted, 55 accepted, 9 `UNEXPECTED_EFFECT`.
- Those 9 were not adjudicated in N18. They are not hidden and remain in the
  frozen N18 ledger.

## Stale text
The trailing "未完成" section of `audit/phase18/REPORT.zh-CN.md` was written
before the final N18 CI/certificate work and is stale. It is preserved as
historical text; this addendum supersedes it for final status.

## N19 objective
Adjudicate all 9 unexpected effects, correct the asymmetric expected-effect
rule, and test oracle soundness against wrong explanations, reverse effects,
invalid event order, tampered evidence and shared model errors.
