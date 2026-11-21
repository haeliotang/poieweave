# Multi-Failure Contract

Inputs: `failureCount`, first failure from `failureTime_s/failureKind/failureIndex/failureAssetId`,
second failure from `f2Time_s/f2Kind/f2Index/f2AssetId`.

Ordering: simulation time, then stable input sequence. All failures at the same
timestamp are applied as one batch before interruption is evaluated, matching the
canonical batch semantics. A failure after termination is never applied.

Persistent state: each failure sets its target bit in `failed_mask`; the mask is
monotonic until an explicit model recovery (none in N13). `failed_before/after`
are recorded per event.
