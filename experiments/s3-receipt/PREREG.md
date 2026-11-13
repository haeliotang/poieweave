# S3 Execution Receipt — pre-registration

- Experiment ID: `s3-receipt`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: isolated simulation; no hardware authority.

## Question

Can every applied recovery produce a machine-checkable receipt bound to the plan
digest and post-repair inventory?

## Method

Base-05, `potable-water`, approved 8 h lease, one pump failure at day 10. Each
successful application emits a receipt containing the time, failed asset,
repaired assets, plan digests, inventory after repair, verification flag and a
SHA-256 receipt digest. The runner prints and verifies each receipt.

## Prediction

Two receipts (initial pump and day-10 repair), both verify, with distinct plan
and receipt digests. Tampering with a receipt body should fail verification.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

The receipt is self-issued by the simulator; it is not a hardware execution
proof or a third-party attestation.
