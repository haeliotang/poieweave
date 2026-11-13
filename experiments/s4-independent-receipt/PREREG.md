# S4 Independent Receipt Verification — pre-registration

- Experiment ID: `s4-independent-receipt`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: independent implementation check, not a full process-model replacement.

## Question

Can a separate implementation verify the S3 recovery receipts without reusing
the Rust verifier?

## Method

Rust exports the two Base-05 recovery receipts as JSON. A Python verifier:

- loads the JSON,
- removes the `digest` field,
- reimplements `poie-digest`'s canonical JSON (sorted keys, compact separators,
  zero as `0`),
- prepends the domain separator `poie.recovery.receipt/1\n`,
- recomputes SHA-256 and compares,
- checks that each receipt's `inventory_after` equals the next receipt's
  `inventory_before`.

Prediction: `receipts=2 verified=2 chain=ok`, `OK`.

## Limitation

This independently verifies the receipt encoding and hash, not the industrial
model, process semantics or physical execution. It is an S4 seed, not full
independent model validation.
