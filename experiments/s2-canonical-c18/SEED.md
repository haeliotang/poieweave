# C18 deterministic seed and sample set

- Seed: `0x9e3779b97f4a7c15`
- Generator: xorshift64 with shifts 13, 7, 17
- Trials: 200
- Per trial: `mask = 1 + (next() % 31)`; assets ordered
  `pump-p1, filter-1, robot-m1, pv-1, lathe-1`; pump-p1 also fails every 2 days.
- The seed is consumed once per trial before any scenario events.
