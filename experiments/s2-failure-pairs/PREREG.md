# S2 Double-Failure Envelope — pre-registration

- Experiment ID: `s2-failure-pairs`
- Date: 2026-11-14
- Status: completed; final table in [REPORT.md](REPORT.md)
- Scope: model-level independent coverage; not a joint repair schedule.

## Question

On Base-05, what is the 12 h recovery coverage for `potable-water` when any
**two** assets fail simultaneously?

## Method

Enumerate all subsets of size 0, 1 and 2 over the nine Base-05 assets. Set the
selected statuses to `failed` in a cloned model and run S1 closure for each
direct service requirement. A pair is recoverable only when all requirements
are restorable within 12 h. Each requirement is checked independently against
the same inventory, so the result is an optimistic coverage envelope rather
than a joint plan.

## Prediction

There are 46 cases (1 + 9 + 36 pairs). Pairs containing `filter-1` should be
blocked because no process restores the filter. Pairs containing `bus-1` or
`bat-1` should be blocked through the power support cycle. Many pairs involving
only pump/lathe/gauge/pv should remain recoverable. Predicted coverage is
around 11/46.

Expected output is frozen in [expected.txt](expected.txt).

## Limitations

Independent requirements, no shared resource ledger, no scheduling, no
stochastic failure timing, no physical calibration.
