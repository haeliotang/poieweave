# S3 Approval Digest — pre-registration

- Experiment ID: `s3-approval-digest`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: isolated simulation; no hardware authority.

## Question

Can an approval be bound to the exact deterministic repair plan, so a changed
plan invalidates the approval?

## Method

Base-05, `potable-water`. Compute the initial S1 repair plan digest. Run three
isolated cases:

1. Approved with the correct digest and no new failures.
2. Approved with a wrong digest.
3. Approved with the original digest, but a new pump failure changes the plan
   at day 10.

## Prediction

- Correct digest: repair and survive 30 days.
- Wrong digest: stop with `approval_digest_mismatch`.
- Original digest after a new failure: stop at day 10 with
  `approval_digest_mismatch`, because the new plan has a different digest.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

No signed approval, no human identity, no independent execution receipt.
