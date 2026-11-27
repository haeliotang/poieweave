# N23 Evidence

- `native-checkpoints/native.json`: real canonical checkpoint projection
- `choice-set-certificates/e2e.json`: checkpoint-bound choice set
- `choice-set-certificates/trust.json`: trusted certificate + payload verifier
- `temporal-comparisons/strict.json`: 2 temporal cases + 11 false-pass negatives
- `continuation-faults/results.json`: 4 second-failure cases
- `continuation-faults/canonical-persistence.json`: consumed authorization after second failure
- `N23_EXTERNAL_VALIDATION_CERTIFICATE.json`: external certificate + tamper

Remaining limitation: canonical native event journal is not compared field-by-field;
Gate N23-E remains partial.
