# S2 Regeneration Horizon — pre-registration

- Experiment ID: `s2-regeneration-horizon`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: Base-05 vs Base-08 model-level regeneration comparison.

## Question

Does adding a local bronze-swarf recycling process extend the survival horizon
under the same pump-failure schedules?

## Method

Base-08 is Base-05 plus 1.2 kg initial swarf and a `recycle-bronze` process
(1.2 kg swarf + 1 kWh → 1.2 kg bronze-bar, lathe+gauge, 2 h). Run the multi-service
greedy policy for pump intervals 10, 15, 20 and 30 days, with one lathe failure
at day 35 and a 200 kWh / 10 kW battery cap.

## Prediction

- Base-05: 40/60/80/90 days.
- Base-08 should extend the 10-day case (regeneration supplies more sleeve
  material) and should survive the longer intervals through the horizon.
- The 10-day Base-08 case may still block on a process cycle when swarf cannot
  be regenerated fast enough.

Expected output is frozen in [expected.txt](expected.txt).

## Limitation

Synthetic mass balance, no recycling loss rate, no competing demand, one
regeneration pathway.
