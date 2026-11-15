# S5 Migration Batch 2 — report

- Date: 2026-11-15
- Status: contention explained; dense blocked by unexplained cycle
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-batch2`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	canonical_days	legacy_days	delta_days	canonical_termination	canonical_events	legacy_events	canonical_last_h	legacy_last_h
dense-long-repair	20.9	15.6	5.3	blocked:[Cycle { kind: "process", id: "turn-sleeve-p1" }]	4	4	502.0	375.5
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

- Canonical horizon 20.9 d vs legacy 15.6 d.
- Canonical terminates with an unexplained `Cycle { process: turn-sleeve-p1 }`.
- Last committed event: 502 h vs 375.5 h.
- Classification: **unexplained / blocking**.
- Affected results: S2 dense long-repair horizons, time certificate, and the
  corresponding claims cannot be marked as migrated until the cycle is explained.
- Migration action: do not change expected outputs; keep batch 2 blocked.

## 3. Decision

- Batch 2 is **not accepted** for dense repair.
- Only the contention sub-batch is explained.
- No formal S2 certificate is regenerated in this batch.

## 4. Limits

- Two scenarios; no resupply/load migration.
