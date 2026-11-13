# S2 Local Regeneration — pre-registration

- Experiment ID: `s2-regeneration`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-06 vs Base-07 model-level regeneration sensitivity.

## Question

Can a local bronze-swarf recycling process replace the external resupply and
close the Base-06 joint coverage gap?

## Setup

Base-07 is Base-06 plus:

- 1.2 kg initial bronze-swarf,
- `recycle-bronze`: 1.2 kg swarf + 1 kWh → 1.2 kg bronze-bar, occupying lathe
  and gauge, requiring `machine-tending`, 2 h.

Compare the 0/1/2-failure joint coverage at a 24 h horizon.

## Prediction

Base-06: independent 25/56, joint 24/56, gap 1.
Base-07: independent 25/56, joint 25/56, gap 0.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

Single recycling pathway, no energy/stock competition analysis, synthetic mass
balance, no physical yield loss.
