# S4 Time Model — pre-registration

- Experiment ID: `s4-time-model`
- Date: 2026-11-14
- Status: completed; independent dense-event arithmetic matches 2/2
- Scope: two failures at 0/1 h with the model repair duration.

## Question

Can an independent model reproduce the clock-advanced event times without
calling Rust?

## Method

Python reads the repair duration from the TOML, applies `start = max(event,
clock)` and `clock = start + repair` for events 0 h and 1 h, and compares with
the Rust dense rows.

Prediction: both event-time lists match.

## Limitation

Dense schedule only; no resource or horizon model.
