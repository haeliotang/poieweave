# S4 Concurrency Model — pre-registration

- Experiment ID: `s4-concurrency-model`
- Date: 2026-11-14
- Status: completed; independent scheduling model matches 2/2
- Scope: two repairs, shared vs separate robot, 4 h deadline.

## Question

Can an independent model derive the makespans and deadline flags from the
scenario durations and layout?

## Method

Python reads the pump/lathe repair durations from the TOML, computes sequential
and layout-specific parallel makespans, and compares with the Rust
`poie-concurrency-deadline` table.

Prediction: both rows match.

## Limitation

Two repairs, static makespan; no scheduler.
