# Canonical C12 Components — report

- Status: MIGRATED_WITH_SEMANTIC_CORRECTION
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-c12-components`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
horizon:        legacy 6/16 -> canonical 2 days
inserts 1..6:   legacy 6/8/10/12/12 (instant), 15/20/26/31/31 (event) -> canonical 2 days
repair <=1 d:   6 days in all semantics
repair >=2 d:   legacy 6/6/6 (instant), 7/10/16 (event) -> canonical 2 days
```

## 2. Classification

- horizon / inserts / repair >=2 d: `SEMANTIC_CORRECTION_IN_REPAIR_FAILURE`
- repair 0.125/0.5/1 d: `EXPECTED_AGREEMENT` (6 days)

The 2-day failures interrupt 2-day-or-longer repairs at the real event time. With
repairs at or below one day there is no overlap, so all semantics agree.

## 3. Consequence

C12 is now fully accounted for: grease, horizon, insert and repair sweep all
have canonical evidence. The old 6/16/15/20/26/31 boundaries are superseded,
not recoverable under canonical semantics.
