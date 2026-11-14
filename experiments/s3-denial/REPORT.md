# S3 Per-Event Denial — report

- Date: 2026-11-14
- Status: denial blocks the matching event and preserves prior repairs
- Runner: `cargo run --locked -q -p poie-viability --bin poie-denial`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
deny_day	executed	survived_days	blocked_by
none	true	30	none
10	true	10	approval_denied_at_event:pump-p1
20	true	20	approval_denied_at_event:pump-p1
5	true	30	none
```

## 2. Findings

- Denying at day 10 stops at day 10; denying at day 20 stops at day 20.
- A denial time with no matching event (day 5) has no effect: the run still
  reaches 30 days.
- `executed` is true because the initial t=0 repair is applied before any denial.

## 3. Significance

- This adds a time-scoped denial path distinct from revocation.
- Prior repairs remain applied; the run stops at the denied event.

## 4. Limits

- Denial stops the run; later events are not separately considered.
- Model-level units.
