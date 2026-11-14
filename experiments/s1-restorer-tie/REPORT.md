# S1 Restorer Tie-Break — report

- Date: 2026-11-14
- Status: equal-duration tie is order-sensitive, not cost-aware
- Runner: `cargo run --locked -q -p poie-viability --bin poie-restorer-tie`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
cheap-first	steps=["replace-sleeve-p1"] duration=10800s grease_left=0.95
expensive-first	steps=["replace-sleeve-expensive"] duration=10800s grease_left=0.80
```

## 2. Findings

- Both plans take 3 h, but the chosen restorer follows index order.
- Cheap-first leaves 0.95 kg grease; expensive-first leaves 0.80 kg.
- S1 does not compare input cost when durations tie.

## 3. Significance

- This is a real limitation for long-horizon survival: an equal-duration but
  more expensive plan can be selected purely by process order, consuming scarce
  inventory earlier than necessary.
- It complements the slow-restorer result: duration is optimized, resource cost
  is not part of the tie-break.

## 4. Limits

- One tie pair and one differing input.
- Model-level units.
