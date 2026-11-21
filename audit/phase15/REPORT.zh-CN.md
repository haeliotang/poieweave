# S5-V N15 — Task-Level Resource Lifetime

基线：`b6a4b605f0d3a0449525731cb729f1c1452e4c71`。

```text
N15-0 EVIDENCE CLOSURE                 VERIFIED
N15-A CANONICAL TASK LIFECYCLE         ESTABLISHED (SCOPED)
N15-B RELEASE-AWARE FAULT SENSITIVITY  ESTABLISHED (SCOPED)
N15-C CROSS-MODEL RESOURCE LIFETIME    ESTABLISHED (SCOPED)
N15-D EXECUTION-DIVERGENCE REDUCTION    VERIFIED (SCOPED)
N15-E CERTIFICATE INTEGRITY            ESTABLISHED (v9)
N15-F REPRODUCIBILITY                  VERIFIED
```

## Canonical 实现

opt-in `RepairLifecycleMode::{GroupHold,TaskRelease}`。真实任务结束边界记录
`TaskCompleted`；当资源不再被任何未完成任务需要时记录 `ResourceReleased`；
执行依赖集合随之更新。默认 GROUP_HOLD 路径不变。

## 实验

- 并行双机器人：robot-m1 于7200释放；7199故障中断、7201故障不因已释放的m1中断；
  robot-m2@7201仍中断。
- 同机器人串行：A完成不释放仍被B使用的robot-m1；7201故障中断。
- Tool AND：tool@7199中断；tool@7201在tool真正释放后不中断。
- OR 双任务：选择 m1/m2，完成与释放时刻与 external 一致。
- 原生事件、release mask、dep mask 均由两侧独立生成。
- executable mutant：提前释放仍被未来任务使用的 robot-m1，验证器拒绝为
  `EXTERNAL_RELEASE_MISMATCH_robot-m1`。
- 自动 reducer 将原始 4 故障缩减到 1 故障且保留双任务与差异签名。

## 限制

```text
upstream release external = NOT_COMPARABLE (excluded)
N15-C dual importer executed: 7/7 MATCH (completion addendum).
v9 migration completed: 41 successors.
```
