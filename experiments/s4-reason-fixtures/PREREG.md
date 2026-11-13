# S4 Reason Fixtures — pre-registration

- Experiment ID: `s4-reason-fixtures`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: missing-resource and unverified-process reason classes.

## Question

Can the independent checker reproduce the two reason classes absent from the
112-case corpus: `missing_resource` and `unverified_process`?

## Fixtures

- Base-09: Base-05 with accepted sleeve and bronze stock removed. Pump repair
  must fail on missing sleeve/bronze.
- Base-10: Base-05 with `replace-lathe-nut` verification downgraded from
  demonstrated to declared. Lathe repair must fail on the verification gate.

Prediction: Rust and Python agree on both reason classes.

## Limitation

Two purpose-built fixtures; no minimal witness or full reason text.
