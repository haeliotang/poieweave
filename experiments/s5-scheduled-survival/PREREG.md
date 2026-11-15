# S5 Scheduled Survival — pre-registration

- Experiment ID: `s5-scheduled-survival`
- Date: 2026-11-15
- Status: completed; two-robot batch completes 8 h vs 6 h
- Scope: pump and lathe fail together; shared robot pool; one clock.

## Question

Does the survival loop use the shared scheduler and a monotonic clock rather than
summing repair durations?

## Method

Base-08 with a second robot asset; pump and lathe fail at t=0; 30-day horizon;
one vs two robots. Report the last log timestamp (repair completion) and horizon.

Prediction: one robot needs 3 h initial + 5 h batch = 8 h; two robots 3 h + 3 h = 6 h.

## Limitation

Inventory is applied once per failure batch; per-step material interlocks are not
modeled.
