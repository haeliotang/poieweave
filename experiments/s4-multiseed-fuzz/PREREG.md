# S4 Multi-Seed Fuzz — pre-registration

- Experiment ID: `s4-multiseed-fuzz`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: five seeds × 100 combined Base-05 mutations.

## Question

Does the no-divergence result from one seed hold across multiple independent
seeds of combined asset/inventory/verification mutations?

## Method

Seeds `20261114`…`20261118`, 100 models each, 9 targets each: 500 models,
4500 cases. Compare Rust and Python.

Prediction: agreement; any mismatch investigated as a semantic finding.

## Limitation

Five seeds, one model family, same mutation generator.
