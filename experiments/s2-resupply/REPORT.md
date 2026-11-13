# S2 Resupply Sensitivity — report

- Date: 2026-11-14
- Status: model-level sensitivity; not a physical claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-resupply`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
before           independent=25  joint=24  gap=1
after_resupply   independent=25  joint=25  gap=0
```

## 2. Finding

- The Base-06 joint coverage gap is caused by exactly one missing sleeve.
- Adding one accepted sleeve closes the gap without changing the independent
  envelope.
- This is a clean **sensitivity result**: external resupply is sufficient for
  the pump-pair conflict in this model, but the experiment says nothing about
  delivery time, quantity uncertainty, or real logistics.
- It also confirms that the gap is a material-availability problem, not a
  planner or capability-closure bug.

## 3. Next

1. Introduce delivery time and resupply failure probability.
2. Add local material regeneration as an alternative to external resupply.
3. Measure how much resupply is needed to move the 30/90-day survival horizon.
