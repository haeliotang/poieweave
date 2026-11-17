# S5-V N5 — Independent Repair Coordination & Endogenous Recovery

基线：`41728951ef7e2f1e5beb6482f2e402b725bd29cb`（PR #575）。
范围：N5-00/01/02/03/04，固定两任务契约、单/双机器人、选中/未选中资源故障。

## 状态

```text
N5-A INDEPENDENT RESOURCE SCHEDULING — ESTABLISHED (SCOPED: N5-00/01/02)
N5-B EXECUTION DEPENDENCY VALIDITY — ESTABLISHED (SCOPED: N5-03/04)
N5-C REPAIR-DRIVEN RESTORATION — ESTABLISHED (SCOPED: fixed task set, group commit)
N5-D EVIDENCE PACKAGE — ESTABLISHED (local + Linux CI)

S5-V N5 INDEPENDENT REPAIR COORDINATION
AND ENDOGENOUS RECOVERY CHALLENGE
  ESTABLISHED (SCOPED: N5-00..04, fixed two-task contract)

N5 POST-INTERRUPTION REPLANNING — NOT ESTABLISHED
N5 AUTONOMOUS RECOVERY — NOT ESTABLISHED
N5 LEVEL 3 PLANNING INDEPENDENCE — NOT ESTABLISHED
```

保留：

```text
S5-V TRAJECTORY-LEVEL CROSS-MODEL VALIDITY — NOT ESTABLISHED
S5 CROSS-MODEL VALIDITY (FULL S2) — NOT ESTABLISHED
PHYSICAL-WORLD CORRECTNESS — NOT ESTABLISHED
```

## Common task contract

两套模型从相同契约独立计算：

```text
lathe task: replace-lathe-nut, 7200 s,  priority 1
pump task:  replace-sleeve-p1, 10800 s, priority 2
each task requires one robot from {robot-m1, robot-m2}
policy: earliest available eligible robot, deterministic
```

契约不包含 canonical 选择的机器人、开始/完成时间或资源表。

## 独立调度结果

| case | tasks | robots | canonical | external | decision |
| --- | --- | ---: | --- | --- | --- |
| N5-00 | pump | 1 | 0–10800 robot-m1 | 0–10800 robot-m1 | SAME_DECISION |
| N5-01 | lathe+pump | 1 | lathe 0–7200, pump 7200–18000, robot-m1 | same | SAME_DECISION |
| N5-02 | lathe+pump | 2 | lathe 0–7200 robot-m1, pump 0–10800 robot-m2 | same | SAME_DECISION |
| N5-03 | lathe+pump | 2 | robot-m1 failure at 7200 → interrupted | interrupted at 7200, no restore | SAME_DECISION |
| N5-04 | pump | 2 | robot-m2 failure at 7200 → pump completes 10800 | same | SAME_DECISION |

契约验证：

```text
每个任务恰好一次
资源来自显式池
无同一机器人时间重叠
lathe 优先于 pump
无提前开始/释放
canonical_valid = true
external_valid = true
```

同刻事件/完成规则：

- N5-03：选中执行资源失效，整组维修在 7200 s 中断；未提交恢复。
- N5-04：未选中 robot-m2 失效，仅记录故障；实际执行机器人 robot-m1 继续，pump 在 10800 s 完成并恢复。

## 内生恢复（Repair-Driven Restoration）

N5-02 的因果链：

```text
RepairStarted
→ ResourcesAcquired (robot-m1, robot-m2)
→ RepairExecuting
→ RepairCompleted at 10800 s
→ RestorationCommitted (pump-p1, lathe-1)
→ ResourcesReleased
```

外部模型根据任务时长与机器人可用性自行计算完成时间，没有读取 canonical 完成时刻。
N5-03 中断场景没有提交任何恢复。

这不等于：

```text
post-interruption replanning
autonomous recovery
independent task generation / planning
```

## 负例

| control | 检测 |
| --- | --- |
| 两任务占用同一机器人重叠 | RESOURCE_CONTRACT_VIOLATION |
| 选中机器人失效后维修仍成功 | EXECUTION_DEPENDENCY_MISMATCH |
| 未选中机器人失效导致中断 | EXECUTION_DEPENDENCY_MISMATCH |
| 维修完成前恢复 | RESTORATION_PRECONDITION_VIOLATION |
| 完成后未恢复 | RESTORATION_COMMIT_MISMATCH |
| 完成时间被改写 | EVENT_TIME_DISPLACEMENT |

全部 `detected=true`，见 `negative-controls/results.json`。

## FMI / 独立性

- 独立 C99 FMI 3.0 scheduler FMU；不链接 Rust scheduler/engine
- C harness 与 fmpy 0.3.32 当次执行，全部 case 差异 0
- 采用 900 s 步长；事件时间由模型内部精确处理
- Linux CI 重建、执行、比较并上传证据
- 共享的只有任务契约、资源池、持续时间、公开优先级策略和容差

## 观测非干扰

canonical runner 对每个 case 同时执行 `run_engine` 与 `run_engine_observed`，
完整 `EngineOutcome` 相等；N5-03 中断和 N5-04 未选中故障均包括在内。

## 未覆盖

- 中断后重新规划/恢复执行
- 自主生成维修任务（Level 3）
- 多步骤维修、工具/上游依赖、共享工具竞争
- 负载阶跃（canonical 不支持）
- 动态任务到达
- 物理世界正确性
