# S5-V N24 — Compositional Multi-Task Recovery & Causal Trace Continuity

基线：`3048e3af`。

```text
N24-0 HISTORY/BOUNDARY        IN PROGRESS — pre-start false pass corrected in N24 FMU
N24-A CAUSAL TRAJECTORY       NOT ESTABLISHED
N24-B MULTI-TASK              NOT ESTABLISHED
N24-C SCHEDULING              NOT ESTABLISHED
N24-D AUTHORIZED SCHEDULE     NOT ESTABLISHED
N24-E NATIVE EXECUTION        NOT ESTABLISHED
N24-F CROSS-MODEL             NOT ESTABLISHED
N24-G ADVERSARIAL             NOT ESTABLISHED
```

## Phase 0 已复现并纠正 pre-start failure false pass

N23 FMU 对 robot-m3 @9500（计划开始 10000）的故障忽略并完成 @20800。
N24 FMU 正确应用该故障：

```text
completed=0 restored=0 preStartInvalidated=1
executedRobot=3 (no automatic switch)
termination at 9500
```

证据：`failing-witness/PRESTART_FAILURE_FALSE_PASS/`。
N23 expected legacy 单任务回归仍需后续确认：N24 扩展必须保持 v17 默认路径不变。
