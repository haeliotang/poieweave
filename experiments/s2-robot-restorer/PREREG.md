# S2 Robot Restorer — pre-registration

- Experiment ID: `s2-robot-restorer`
- Date: 2026-11-14
- Status: completed; manual restorer removes the repair-executor blocker
- Scope: add a manual robot replacement process.

## Question

Does a manual restorer for the shared repair robot remove the blocker?

## Method

Base-08 with redundant pump; add robot-module/failed-robot resources, one module
lot, and `replace-robot` (module -> failed-robot, no occupation, restores
robot-m1). Robot fails at day 20; 90 days.

Prediction: robot repair restores the repair chain and the horizon extends.

## Limitation

Manual/off-robot replacement; one module.
