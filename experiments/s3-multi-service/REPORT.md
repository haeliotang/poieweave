# S3 Multi-Service Recovery — report

- Date: 2026-11-14
- Status: multi-service approval loop; isolated simulation
- Runner: `cargo run --locked -q -p poie-viability --bin poie-multi-recovery`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
approved-8h: services=potable-water,workshop-operability survived_days=30.0 receipts=2 blocked=
lease-1h: services=potable-water,workshop-operability survived_days=0.0 receipts=0 blocked=potable-water:approval_lease_expired:pump-p1: need 10800s, lease 3600s
two-events-8h: services=potable-water,workshop-operability survived_days=30.0 receipts=3 blocked=
missing-sleeve: services=potable-water,workshop-operability survived_days=10.0 receipts=1 blocked=potable-water:missing resource bronze-bar: need 1.2, have 0
```

## 2. Findings

- The 8 h approval keeps both services verified through 30 days with two
  receipts.
- A 1 h lease blocks before the first 3 h pump repair; no side effects.
- The loop stops on the first blocked service and records its reason.
- With no accepted sleeve and no bronze, the day-10 repair blocks on
  `missing resource bronze-bar`; the run ends at day 10 with one receipt.

## 3. Limits

- Isolated simulation, not a hardware authority path.
- Two services; one or two failure events; one blocked scenario.
- No concurrent repairs or renewal.
