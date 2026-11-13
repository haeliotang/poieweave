# Certificate Bundle Tamper Test — pre-registration

- Experiment ID: `certificate-bundle-tamper`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: negative integrity test.

## Question

If one certificate in the bundle is modified, does independent bundle
verification fail?

## Method

Copy the bundle and all six certificates into a temporary root. Verify the clean
copy (must pass). Append one byte to the S1 certificate and verify again (must
fail). Expected: `tamper=detected`.

## Limitation

Tests content integrity only; no signature, author identity or rollback threat.
