# S2 Canonical C12 Migration — report

- Date: 2026-11-16
- Status: SUPERSEDED — SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c12`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
grease  instant  event  canonical  event_energy  canonical_energy  termination
0.10    4        10     2          200.0         160.0             interrupted at day 2
0.15    6        16     2          200.0         160.0             interrupted at day 2
0.20    8        21     2          200.0         160.0             interrupted at day 2
0.30    8        21     2          200.0         160.0             interrupted at day 2
0.50    8        21     2          200.0         160.0             interrupted at day 2
```

## 2. Classification

`SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`

- A pump failure at day 2 occurs while a 5-day pump repair is running.
- Canonical processes it at day 2, invalidates the repair, and blocks.
- Legacy event-driven defers the failure until repair completion, producing
  10/16/21 days.
- Therefore the old C12 grease boundary is **superseded**, not preserved.

## 3. Consequence

- C12's grease-boundary numbers cannot be used as canonical evidence.
- The canonical model needs a separate boundary study if re-evaluated with
  different failure/repair timing.
- No certificate may be issued from the legacy numbers.

## 4. Limits

- Only the grease component of C12; insert/time-horizon components remain.
