# S2 Grease Cost — report

- Date: 2026-11-14
- Status: restorer input cost directly changes survival
- Runner: `cargo run --locked -q -p poie-viability --bin poie-grease-cost`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
grease_kg	mode	survived_days	blocked_by
0.10	cheap	20	missing resource grease: need 0.05, have 0
0.10	expensive	0	missing resource grease: need 0.2, have 0.1
0.15	cheap	30	horizon exceeded: need 10800s, horizon 0s
0.15	expensive	0	missing resource grease: need 0.2, have 0.15
0.20	cheap	30	horizon exceeded: need 10800s, horizon 0s
0.20	expensive	10	missing resource grease: need 0.2, have 0
```

## 2. Findings

- With 0.10 kg grease, the cheap restorer supports two repairs (20 days); the
  expensive one cannot even perform the first repair (0 days).
- With 0.20 kg, expensive supports one repair (10 days), cheap reaches 30 days.
- The 0.15 kg expensive case still fails the first repair.

## 3. Significance

- The S1 cost tie-break is not cosmetic: a more expensive equal-duration
  restorer can turn a repairable event into an immediate failure.
- This is the S2 consequence of `s1-restorer-tie`.

## 4. Limits

- One consumable, three stock levels; model-level units.
