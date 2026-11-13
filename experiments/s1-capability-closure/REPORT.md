# S1 Recursive Capability Closure — report

- Date: 2026-11-14
- Status: model-level research result; not a physical or safety claim
- Runner: `cargo run --locked -q -p poie-viability --bin poie-closure`
- Reproduced output: [expected.txt](expected.txt)

## 1. What was added

`poie-viability::closure` implements a bounded recursive closure search over the
validated ontology model. It does not generate a new capability graph; it
answers whether a failed target can be restored under the model's own resources,
capabilities, occupied assets, inventory and time.

Inputs are the model's initial asset statuses, accepted lots, restorers,
producers, capability providers and process durations. The search is serial:
process durations add; inventory is consumed; alternatives are tried in model
order. A cycle is returned as `cycle:asset:<id>`, `cycle:resource:<id>` or
`cycle:process:<id>`.

## 2. Result table

```text
case	scenario	target	verdict	detail
s1-base04-lathe	base-04	lathe-1	RESTORABLE	steps=restore:lathe-1=replace-lathe-nut	total_s=7200
s1-base04-pump	base-04	pump-p1	RESTORABLE	steps=restore:pump-p1=replace-sleeve-p1	total_s=10800
s1-base04-pump-too-short	base-04	pump-p1	BLOCKED	reasons=horizon_exceeded
s1-base01-lathe	base-01	lathe-1	BLOCKED	reasons=cycle:asset:lathe-1
s1-base04-lathe-no-spare	base-04	lathe-1	BLOCKED	reasons=cycle:asset:lathe-1
s1-base01-pump-no-sleeve	base-01	pump-p1	RESTORABLE	steps=produce:sleeve-p1-raw=turn-sleeve-p1|produce:sleeve-p1=inspect-sleeve-p1|restore:pump-p1=replace-sleeve-p1	total_s=27000
s1-base01-bus-no-breaker	base-01	bus-1	BLOCKED	reasons=cycle:asset:bus-1
```

## 3. Findings

- **Spare-part closure works.** Base-04's accepted `lathe-nut` makes
  `replace-lathe-nut` feasible in 2 h.
- **A production chain can replace a missing part.** Without a sleeve lot,
  Base-01 can still turn a raw sleeve, inspect it and replace the pump sleeve:
  4 h + 0.5 h + 3 h = 7.5 h.
- **Self-repair cycles are found.** Without a spare lathe-nut, restoring the
  lathe requires `turn-lathe-nut`, which occupies the failed lathe: the search
  reports `cycle:asset:lathe-1` instead of assuming success.
- **The failed bus exposes a support-cycle blocker.** Replacing the breaker
  needs `electrical-repair` from `robot-m1`, but the robot needs `charger-1`,
  which needs the failed `bus-1`; the search now reports
  `cycle:asset:bus-1` instead of the shallower "no restorer" message.
- **The horizon matters.** The pump repair is infeasible under a 2 h horizon.

## 4. Assumptions and limitations

- Synthetic Base-01/Base-04 quantities; no calibrated plant model.
- Serial execution, no concurrency or preemption; durations are sums.
- No partial degradation of capability providers beyond `available`/not.
- No resource partial-failure or stochastic failure propagation.
- `modeled` is the minimum accepted process verification; this is a structural
  research choice, not a safety claim.
- A positive result is a model-level existence result, not permission to
  dispatch actions to hardware.
- No-restorer assets now inspect their support chain before returning a local
  blocker; this is what exposes the bus/robot/charger cycle.

## 5. Reproduction

```bash
cargo run --locked -q -p poie-viability --bin poie-closure
```

CI diffs the output against `expected.txt`. The analysis tests are in
`crates/poie-viability/src/closure.rs` (`cargo test -p poie-viability`).

## 6. Next S1 work

1. Report support-chain cycles explicitly for assets that have no restorer.
2. Add resource-level quantities and concurrent scheduling.
3. Add Base-05 with energy recharge, wear and multiple repair rounds.
4. Only later connect recovery decisions to execution and evidence certificates.
