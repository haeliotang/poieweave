# S5 Migration Batch 2 — report

- Date: 2026-11-15
- Status: contention explained; dense difference traced to a missing-key inventory defect
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-batch2`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	canonical_days	legacy_days	delta_days	canonical_termination	canonical_events	legacy_events	canonical_last_h	legacy_last_h
dense-long-repair	15.6	15.6	0.0	blocked:[MissingResource { resource: "grease", needed: 0.05, available: 0.0 }]	3	4	375.5	375.5
contention-1-robot	1.000	1.000	0.000	completed	3	3	5.0	3.0
contention-2-robots	1.000	1.000	0.000	completed	3	3	3.0	3.0
```

## 2. Difference records

### C-1: contention, one robot

- First difference: repair completion at 5 h vs legacy 3 h.
- Canonical serializes the 3 h and 2 h repairs on one robot.
- Legacy event-driven path logs completion at 3 h and does not model shared
  occupancy.
- Classification: **semantic fix** (resource contention now affects time).
- Affected claim: none yet; S3 concurrency claims already describe this.

### C-2: contention, two robots

- No difference: both complete at 3 h.
- Classification: **expected agreement**.

### D-1: dense long repair

- Before fix: canonical 20.9 d, `Cycle { process: turn-sleeve-p1 }`, last at 502 h.
- Root cause: `Plan.remaining_inventory` drops a key when a resource reaches zero;
  `apply_plans()`/`commit_delta()` skipped missing keys, so the last sleeve was
  never deducted.
- Fix: deltas iterate the union of planning keys and planned keys, treating a
  missing key as zero.
- After fix: canonical 15.6 d, `MissingResource { grease, needed 0.05 }`, last at
  375.5 h, identical to legacy.
- Classification: **migration implementation defect**, not a real capability cycle.
- Migration action: accepted with the timing caveat below.

## 3. Timing caveat

Both paths still defer failures that occur during a 5-day repair until the repair
completes. Equal horizons do not prove correct in-repair event timing.

## 4. Decision

- Batch 2a/2b accepted.
- Batch 2c accepted as a migration defect fix.
- No formal S2 certificate is regenerated; time certificate migration remains
  batch 4.

## 4. Limits

- Two scenarios; no resupply/load migration.
