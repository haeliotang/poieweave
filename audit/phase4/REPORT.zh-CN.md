# S5-V N4 — Independent Discrete-Event Cross-Model Challenge

基线：`33583f53ac046a435b6756b4a448bdc9b3ff23d9`（PR #573）
范围：N4-00/01/02/03 PV 故障 + N4-05 初始泵维修中断。

## 状态

```text
N4-A EXTERNAL DISCRETE-EVENT EXECUTION — ESTABLISHED (SCOPED: PV failure)
N4-B CAUSAL EVENT AGREEMENT — ESTABLISHED (SCOPED: N4-00/01/02/03)
N4-C REPAIR INTERRUPTION — ESTABLISHED (SCOPED: fixed initial repair, robot-m1 dependency)
N4-C REPAIR / RECOVERY CROSS-MODEL VALIDITY — NOT ESTABLISHED (no recovery test)
N4-C INDEPENDENT REPAIR PLANNING — NOT ESTABLISHED (fixed schedule only)
N4-D EVIDENCE PACKAGE — ESTABLISHED (local + Linux CI)

S5-V N4 INDEPENDENT DISCRETE-EVENT CHALLENGE
  ESTABLISHED (SCOPED: tested PV failure + repair interruption)
```

保留：

```text
S5-V TRAJECTORY-LEVEL CROSS-MODEL VALIDITY — NOT ESTABLISHED
S5 CROSS-MODEL VALIDITY (FULL S2) — NOT ESTABLISHED
PHYSICAL-WORLD CORRECTNESS — NOT ESTABLISHED
```

## 实验

| case | PV failure | canonical terminal | FMU terminal |
| --- | ---: | ---: | ---: |
| N4-00 | none | 72000 s completed | 72000 s |
| N4-01 | 7200 s | 53400 s energy depleted | 53400 s |
| N4-02 | 4500 s | 51150 s energy depleted | 51150 s |
| N4-03 | 10800 s | 56400 s energy depleted | 56400 s |

CANONICAL/FMU 事件时间差为 0 s（在 1e-6 s 容差内）。
PV 故障后 FMU `pvAvailable=0`；故障前为 1。
N4-03 同刻顺序：

```text
canonical: repair_completed:pump-p1 -> failure:pv-1
FMU lastEventCode: failure after repair
```

## 差异分解

已登记维修假设：

```text
FMU - canonical = -0.5 kWh * t / 3 h    (0 < t < 3 h)
```

四案例在 0/1/2/3/4 h 样本上的残差：

```text
max unexplained residual = 0 J
worst per-interval conservation residual = 0 J
```

第一处差异为 `MODEL_ASSUMPTION_DIFFERENCE`，位于 3600 s；事件边界本身无未解释差异。
没有把模型假设差异误归为数值或事件错误。

## 独立执行与步长

- C harness 与 fmpy：所有 case/step 的终端、故障时间、能量列差异 0
- 步长 3600/900/1000 s 全部通过
- 4500 s 非网格故障由模型内部真实切分，未四舍五入
- FMI 3.0 Co-Simulation；fmpy 0.3.32

## 负例

| control | 检测 |
| --- | --- |
| 故障推迟 1 h | `EVENT_TIME_DISPLACEMENT` |
| 故障被忽略 | `STATE_TRANSITION_MISMATCH` + `RESOURCE_LEDGER_DIVERGENCE` |
| 故障后继续发电 | `RESOURCE_LEDGER_DIVERGENCE` |
| 维修完成推迟 1 h | `EVENT_TIME_DISPLACEMENT` |
| 同刻顺序反转 | `EVENT_ORDER_MISMATCH` |

全部 `detected=true`，原始结果在 `negative-controls/results.json`。

## 观测非干扰

canonical runner 对每个 case 同时执行：

```text
run_engine(...)
run_engine_observed(...)
```

并要求完整 `EngineOutcome` 相等；四个 case 均通过。N1 的 thread-local 收集器
不改变故障时间、维修完成、资源选择或能量状态。

## N4-05 repair interruption

canonical：

```text
failure:robot-m1 at 7200 s
termination: blocked:repair interrupted at 7200s: robot-m1 failed
survived_s: 7200
final_energy: 39.000 kWh
repair completion: none
```

FMU：

```text
robotFailureAppliedTime_s: 7200
repairState: 2 (interrupted)
robotAvailable: 0
repairCompletedTime_s: -1
terminal: 7200
final_energy: 38.6667 kWh
```

差额：

```text
FMU - canonical = -0.5 kWh * 7200 / 10800 = -0.3333 kWh
unexplained residual = 0 J
```

外部维修调度是固定 0–10800 s 任务，不是独立规划器；因此该结果只证明
“共同固定维修计划下的执行时序一致”，不证明调度决策独立一致。
没有发生恢复提交，也没有把执行资源失效与恢复目标混淆。

负例：故障推迟、故障忽略均被检测为事件时间或状态/依赖差异。

## 未建立 / 下一步

- N4-C：维修中断、恢复、共享机器人竞争尚未纳入本 N4 FMU
- 负载阶跃不受 canonical 支持
- 未覆盖多个并发维修、恢复后重新规划
- 未证明 trajectory-level 或 full S2 跨模型有效性
- 未证明物理世界正确性

本报告不把“最终能源一致”扩展为完整轨迹一致，也不把“事件时间一致”扩展为
调度决策独立一致。
