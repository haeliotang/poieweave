# S2 Local Regeneration — report

- Date: 2026-11-14
- Status: model-level regeneration sensitivity; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-regeneration`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
base-06	independent=25	joint=24	gap=1
base-07	independent=25	joint=25	gap=0
```

## 2. Finding

- A local bronze-swarf recycling process closes the Base-06 pump-pair joint gap
  in the model.
- This is the first **Autonomous Regeneration** positive result in the S1/S2
  line: the system can recover a production input from its own waste stream.
- The result is narrow: one recycling path, synthetic mass balance, no yield
  losses, no competing demand for the lathe/gauge.

## 3. What changes vs external resupply

| Path | Mechanism | Gap closed |
| --- | --- | --- |
| External resupply | add one accepted sleeve | yes |
| Local regeneration | recycle swarf into bar, then machine sleeve | yes |

Local regeneration is a stronger self-maintenance result but also more
fragile: it consumes energy, lathe/gauge time and machine-tending capability.

## 4. Limitations

- No recycling yield/loss or contamination.
- No concurrent production, maintenance or operating demand.
- Synthetic swarf stock; no calibrated quantity.
- No physical or safety implication.
