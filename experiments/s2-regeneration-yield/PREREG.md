# S2 Regeneration Yield — pre-registration

- Experiment ID: `s2-regeneration-yield`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-08 model-level yield sensitivity.

## Question

How sensitive is the 60-day regeneration horizon to the bronze recycling yield?

## Method

Clone Base-08 and scale the `recycle-bronze` bronze-bar output by 1.00, 0.75 and
0.50. Run the 10-day pump-failure schedule with the day-35 lathe failure, shared
inventory and the 200 kWh / 10 kW battery cap.

## Prediction

- 1.00 and 0.75 should survive 60 days.
- 0.50 should reduce the horizon to about 50 days on the 1.2 kg bronze-bar
  requirement.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

This is a synthetic output scaling, not a mass-conserving recycling model; it
measures sensitivity, not physical yield.
