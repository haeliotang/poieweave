# S4 Tool-Cycle Model — pre-registration

- Experiment ID: `s4-tool-cycle-model`
- Date: 2026-11-15
- Status: completed; structural cycle/unblocker model matches 2/2
- Scope: Base-08 lathe repair with nut stock 0 vs 1.

## Question

Can an independent structural model derive the tool-repair cycle and unblocker
from the TOML alone?

## Method

Python finds the process that restores `lathe-1` and needs `lathe-nut`, then the
producer of `lathe-nut` that occupies `lathe-1`; with no stock this is a cycle,
with one stock the repair duration applies. Compare with Rust.

Prediction: two rows match.

## Limitation

Structural only; no search over alternatives.
