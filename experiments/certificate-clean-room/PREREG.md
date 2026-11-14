# Certificate Clean-Room Verification — pre-registration

- Experiment ID: `certificate-clean-room`
- Date: 2026-11-15
- Status: completed; bundle verifies in an isolated temporary copy
- Scope: bundle + 13 certificate dirs + scenarios, no other workspace files.

## Question

Does the certificate bundle verify without depending on other workspace files?

## Method

Copy the bundle, all 13 certificate directories and `scenarios/` into a fresh
temporary directory, then run the bundle verifier from there with the temporary
root and run the summary verifier.

Prediction: `clean-room=ok`.

## Limitation

Same verifier code; isolation is at the file-tree level.
