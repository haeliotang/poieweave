# S2 Filter Restorer — pre-registration

- Experiment ID: `s2-filter-restorer`
- Date: 2026-11-14
- Status: completed; restorer removes the shared blocker
- Scope: add a local filter replacement process.

## Question

Does adding a local filter restorer remove the shared-filter blocker?

## Method

Base-08 with redundant pump; add filter-cartridge/failed-filter resources, one
cartridge lot, and `replace-filter` (cartridge -> failed-filter, occupies
robot, restores filter-1). Filter fails at day 30; 90 days.

Prediction: the restorer repairs filter-1 and the full horizon is reached.

## Limitation

One restorer; one cartridge.
