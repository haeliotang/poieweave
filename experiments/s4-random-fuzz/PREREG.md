# S4 Random Combined Fuzz — pre-registration

- Experiment ID: `s4-random-fuzz`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: deterministic random combined mutations of Base-05.

## Question

Can a seeded random combination of asset failures, lot removals, process
downgrades and capability downgrades expose further Rust/Python divergence?

## Method

Seed `20261114`; 100 models with 0–3 failed assets, 0–2 removed lots, 0–2
downgraded processes and 0–2 downgraded capabilities. Compare all 9 targets:
900 cases.

Prediction: agreement after the capability fix; any mismatch investigated.

## Limitation

One seed, one model family, 100 models.
