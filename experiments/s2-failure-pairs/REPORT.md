# S2 Double-Failure Envelope — report

- Date: 2026-11-14
- Status: model-level independent coverage; not a joint plan or physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-pairs`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
service=potable-water horizon_s=43200 total=46 recoverable=11 coverage=0.239130
```

The 46 cases are 1 empty + 9 single + 36 pairs. Only 11 are recoverable.

Representative blocked pairs:

```text
bus-1,filter-1       BLOCKED  cyclic asset dependency at bus-1
filter-1,pump-p1     BLOCKED  no restorer for filter-1
bus-1,robot-m1       BLOCKED  cyclic asset dependency at bus-1
charger-1,robot-m1   BLOCKED  no restorer for robot-m1
```

Recoverable pairs are mostly those that do not touch the filter or the power /
maintenance support chain, e.g. `gauge-1,lathe-1`, `lathe-1,pump-p1`,
`pump-p1,pv-1`.

## 2. Findings

- Double failures collapse coverage from 5/10 to 11/46; the filter and power
  support cycles dominate the failure map.
- The result is an **optimistic** envelope: each direct requirement is checked
  independently, so shared-inventory conflicts are not visible here.
- A pair can be "recoverable" even though two repairs would compete for the same
  sleeve, nut or bronze stock; the joint long-horizon runner is the next layer
  that exposes such conflicts.

## 3. Limitations

- Independent service requirements; no joint inventory ledger.
- No scheduling or concurrency.
- No stochastic timing or operating demand.
- Synthetic Base-05; no physical safety implication.

## 4. Next

1. Check pairs with the shared long-horizon policy rather than independent
   closure.
2. Add a joint repair schedule for pairs that compete for the same stock.
3. Add filter/bus external-input assumptions to distinguish "unrepairable" from
   "requires resupply".
