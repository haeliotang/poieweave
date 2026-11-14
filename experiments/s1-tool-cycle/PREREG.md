# S1 Tool Repair Cycle — pre-registration

- Experiment ID: `s1-tool-cycle`
- Date: 2026-11-15
- Status: completed; a lathe repair cycles on itself without a spare nut
- Scope: Base-08 lathe failed, lathe-nut stock 0 vs 1.

## Question

Can the closure engine detect a recursive tool-repair dependency and show the
minimal unblocking condition?

## Method

Fail `lathe-1`; set the lathe-nut stock to 0 or 1; analyze the repair at a 24 h
horizon and report the blocker detail.

Prediction: 0 stock reports a cycle on `lathe-1`; 1 stock restores in 2 h.

## Limitation

One asset and one stocked part; model-level.
