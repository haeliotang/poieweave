# N22 Evidence

- `fixed-cases/matrix.json`: 18-item fixed matrix (16 PASS, 2 NOT_REPRESENTABLE)
- `choice-set-certificates/decisive.json`: frozen choice-set certificate
- `importer-comparisons/strict.json`: 4 C/fmpy cases + 9 false-pass negatives
- `executable-mutants/results.json`: M1/M2/M3, negatives, reducer 5->3
- `authorization-tests/proposal-commitment.json`: v13 proposal mutations
- `N22_EXTERNAL_VALIDATION_CERTIFICATE.json`: external certificate + tamper

Unrepresentable bounded-domain cases: N22-07 two-task shared resource,
N22-14 second failure after continuation. Both require extending the current
one-task/single-failure bounded model; not claimed as PASS.
