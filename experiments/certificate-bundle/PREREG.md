# Certificate Bundle — pre-registration

- Experiment ID: `certificate-bundle`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: content-addressed index of S1/S2/S3 certificates.

## Question

Can the three layer certificates be bound into one bundle that detects any
changed certificate or wrong layer identity?

## Method

Rust computes SHA-256 for each certificate and emits a bundle with layer, path,
hash and schema. Python verifies the bundle schema, exact layer set, identities,
file hashes and claim scope.

Prediction: `bundle=ok`; changing one certificate should fail the hash check.

## Limitation

Unsigned index; only detects content changes, not authorship.
