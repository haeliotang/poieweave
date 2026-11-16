# C19 deterministic seed and sample set

- Seed: `0x9e3779b97f4a7c15`
- Generator: xorshift64 with shifts 13, 7, 17
- Trials: 200
- Per trial: `mask = 1 + (next() % 31)`, then `day = next() % 11` (0–10).
- Assets: `pump-p1, filter-1, robot-m1, pv-1, lathe-1`.
- The extra window draw means C19 sample sequence differs from C18.
