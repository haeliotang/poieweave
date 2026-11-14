# S2 Adversarial Timing — report

- Date: 2026-11-14
- Status: same minimal set, horizon follows the failure time
- Runner: `cargo run --locked -q -p poie-viability --bin poie-adversarial-timing`
- Output: [expected.txt](expected.txt)

## 1. Key result

```text
0	worst	1	0	filter-1
10	worst	1	10	filter-1
```

## 2. Findings

- At t=0 the minimal 0-day set is `filter-1` alone.
- At day 10 the same set stops at day 10; the horizon equals the failure time.
- All filter-containing sets fail at the same time with the same blocker.
- `robot-m1 + lathe-1` fails at the failure time via `no restorer for robot-m1`.

## 3. Significance

- The minimal failure set is timing-invariant in this scenario; only the
  horizon shifts with the failure time.
- This separates "which set" from "when it occurs".

## 4. Limits

- Two times, five assets, singles/pairs only.
- Model-level units.
