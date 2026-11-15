# S5 Multi-Process Crash Tail — pre-registration

- Experiment ID: `s5-multiprocess-crash`
- Date: 2026-11-15
- Status: completed; child-process crash tail truncated on reopen
- Scope: one child process, one local append log.

## Question

Does a half-written line from a different process get recovered on reopen?

## Method

Spawn `poie-partial-writer` as a child; it appends a partial JSON line and exits.
Reopen `AppendLog` in the test process and inspect records and file length.

Prediction: records empty, length 0, no error.

## Limitation

Crash tail only; no concurrent writers, partitions or clock skew.
