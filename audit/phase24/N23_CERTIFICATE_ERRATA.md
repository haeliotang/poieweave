# N23 Certificate Errata

Original bytes unchanged (SHA-256 `da4cf8e7d7a7e192cd8271660228a7f37b47ba79994a407ff74a2db411317c4f`).

The certificate label said canonical v16 while binding v17. It also retained the older event-journal limitation. Supplement: v17 event-observed resume covers resume-time events (ReplanAuthorized/RepairResumed/TaskRestarted/TaskCompleted); initial repair events remain via the existing event-observed API. Status: SUPPLEMENTED_NOT_REWRITTEN.
