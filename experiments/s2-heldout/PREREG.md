# S2 Held-Out Failure Intervals — pre-registration

- Experiment ID: `s2-heldout`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-05 model-level held-out intervals.

## Question

Do the survival-horizon conclusions from the training intervals 10/15/20/30 days
generalise to the **held-out** intervals 12 and 25 days without retuning?

## Method

Use the same Base-05 model, multi-service greedy policy, shared inventory and
200 kWh / 10 kW battery limits. Schedule pump failures every 12 or 25 days
below a 90-day horizon, plus one lathe failure at day 35. No threshold or policy
is changed after observing the held-out result.

## Prediction

- 12-day interval lies between the 10-day (40) and 15-day (60) results; predict
  roughly 45–55 days and a bronze-bar block.
- 25-day interval lies between the 20-day (80) and 30-day (90+) results; predict
  that it survives the 90-day horizon.

Expected output is frozen in [expected.txt](expected.txt), which records the
actual held-out run.

## Limitation

Two held-out points, one model, synthetic quantities; this is not a statistical
generalisation claim.
