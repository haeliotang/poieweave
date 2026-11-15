# S5 Migration Batch 2 — report

- Date: 2026-11-15
- Status: contention explained; dense now applies in-repair failures at event time
- Runner: `cargo run --locked -q -p poie-viability --bin poie-migration-batch2`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
scenario	canonical_days	legacy_days	delta_days	canonical_termination	canonical_events	legacy_events	canonical_last_h	legacy_last_h
dense-long-repair	2.0	15.6	-13.6	blocked:repair interrupted at 172800s: pump-p1 failed	1	4	48.0	375.5
contention-1-robot	1.000	1.000	0.000	completed	4	3	5.0	3.0
contention-2-robots	1.000	1.000	0.000	completed	4	3	3.0	3.0
```

## 2. Difference records

### C-1: contention, one robot

- First difference: repair completion at 5 h vs legacy 3 h.
- Canonical serializes 3 h and 2 h repairs on one robot.
- Legacy event-driven path does not model shared occupancy.
- Classification: **semantic fix**.

### C-2: contention, two robots

- Both complete at 3 h.
- Classification: **expected agreement**.

### D-1: dense long repair

- Canonical: 2.0 d, `Blocked("repair interrupted at 172800s: pump-p1 failed")`.
- Legacy: 15.6 d, deferring the day-2 failure until the 5-day repair completes.
- First divergence: day 2, while the initial pump repair is still running.
- Classification: **semantic fix** — the failure takes effect at its own time and
  invalidates the running repair; it is not deferred.
- Consequence: the expected 15.6 d is **not preserved**. Batch 2c is accepted for
  event-time semantics, not for numeric parity with legacy.
- No S2 certificate is regenerated.

## 3. Timing caveat

## 3. Timing caveat

The canonical path now processes failures during repair at their event time. The
legacy path still defers them; numeric parity is therefore not expected. The
canonical result is the one that matches the intended causal semantics.

## 4. Decision

- Batch 2a/2b accepted.
- Batch 2c accepted as an event-time semantic change; legacy 15.6 d is superseded.
- No formal S2 certificate is regenerated; time certificate migration remains
  batch 4.

## 4. Limits

- Two scenarios; no resupply/load migration.
