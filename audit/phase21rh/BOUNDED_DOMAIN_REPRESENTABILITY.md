# Bounded Domain Representability Audit

| Case | Decision | Reason | Impact |
|---|---|---|---|
| N21-05 Tool AND | NOT_REPRESENTABLE | FMI model has no tool availability input/state | add toolAvailable + failure state, update modelDescription |
| N21-07 Interrupted Task A | NOT_REPRESENTABLE | FMI model assumes A completed before checkpoint; no per-task progress | add partial task state/restart-from-zero semantics |
| N21-16 Inventory/Energy | NOT_REPRESENTABLE | FMI model has no inventory/energy state | add resource ledger and shortage rejection |

Scope accounting:

```text
previously NOT_REPRESENTABLE = 3
newly REPRESENTABLE = 0
still NOT_REPRESENTABLE = 3
```

No matrix item was renamed to PASS.
