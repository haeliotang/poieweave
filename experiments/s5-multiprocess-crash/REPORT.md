# S5 Multi-Process Crash Tail — report

- Date: 2026-11-15
- Status: child-process crash tail truncated on reopen
- Runner: `python3 experiments/s5-multiprocess-crash/run.py`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
multiprocess-crash=ok
```

## 2. Findings

- A different process leaves `{"record_id":"site-a/1","kind":` without newline.
- Reopening `AppendLog` truncates the tail to length 0 and returns no records.
- Recovery happens in a different process from the writer.

## 3. Significance

- This is the first multi-process fault test, not just an in-process unit test.
- It supports crash-tail recovery only; it does not establish multi-writer safety.

## 4. Limits

- One writer, one local file, no partition or clock skew.
