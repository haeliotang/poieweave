# S5-V N10 — Boundary, Identity & Alternative-Resource Causal Challenge

基线：`492e20f838c33076dc6b36cc13d298390c8e68a7`。

## 状态

```text
N10 TASK-LEVEL CONTRACT HARDENING
  VERIFIED

N9 REPORT CONSISTENCY
  VERIFIED

N10 RESOURCE-CAUSAL EVENT BOUNDARIES
  ESTABLISHED (SCOPED: N10-00..N10-06)

N10 NAMED UPSTREAM DEPENDENCY VALIDITY
  ESTABLISHED (SCOPED: N10-U01..U04)

N10 ALTERNATIVE-RESOURCE CAUSAL VALIDITY
  ESTABLISHED (SCOPED: N10-OR1..OR3)

N10 BOUNDARY-AWARE RESOURCE-CAUSAL TRAJECTORY
  ESTABLISHED (SCOPED: N10-U01..U04, N10-OR1..OR3)

N10 EVIDENCE AND CERTIFICATE INTEGRITY
  ESTABLISHED (SCOPED; see N11 N10_REVALIDATION)
```

N9 accepted claims remain scoped.

## 边界结果

```text
N10-00 no failure, completion 7200                ACCEPTED
N10-01 robot failure 7199, interruption 7199      ACCEPTED
N10-02 failure == completion 7200                 ACCEPTED (completion wins)
N10-03 failure 7201 after completion              ACCEPTED
N10-04 failure == group completion 10800          ACCEPTED (completion wins)
N10-05 failure 12000 after commit                 ACCEPTED (no interruption)
N10-06 failure == horizon 7200                    ACCEPTED (not applied)
```

- 7199/7201 使用 1 s 真实 step 与 exact event times，不做最近邻或取整。
- 同刻 completion/failure 顺序由 canonical native event_sequence 校验。
- 外部仅具 endpoint final row；轨迹深度标记 `ENDPOINT_ONLY`。
- canonical native resource release 仍为 `NOT_OBSERVED`。
- 外部 `release_s` 只是 FMU group bookkeeping，不作为原生 release 证据。

边界负例：7199 事件位移、horizon 伪应用、提交后故障中断、同刻顺序反转、故障未应用，全部检出。

## 命名上游与 OR 结果

```text
N10-U01 bus-1 active dependency @3600       ACCEPTED (both interrupt)
N10-U02 unrelated sensor-x @3600            ACCEPTED (both continue)
N10-U03 unselected bus-2 path @3600         ACCEPTED (both continue)
N10-U04 bus-1 == completion @7200           ACCEPTED (both complete)
N10-OR1 two robots, preferred robot2        ACCEPTED (DIFFERENT_BUT_VALID_DECISION)
N10-OR2 robot1 @3600, canonical robot1      ACCEPTED (VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE)
N10-OR3 robot OR + tool AND, preferred 2    ACCEPTED (different valid, tool required both)
```

- 外部 N10 FMU 新增命名 `failureAssetId`、`robot1/2DependencyMask`、
  `selectedDependencyMask`、`failureTargetId`、`dependencyPathCode`。
- 外部依赖闭包由契约输入 + 实际选中机器人独立计算，不读取 canonical witness。
- N10-U02/U03 的失败命中不同 bit，外部不中断；canonical witness 不含该资产。
- N10-OR1 中 canonical robot1 / external robot2，mask 均允许；不判 FAIL。
- N10-OR2 中合法选择差异导致 canonical 中断、external 完成；标记
  `VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE`，不是契约违规。
- N10-OR3 证明机器人 OR 与工具 AND 复合：两模型都需要 tool-t1。
- 8 个上游/OR 负例全部检出，包括两模型共同选择非法 robot。

## 事件轨迹结果

```text
N10-U01..U04, N10-OR1, N10-OR3   SAME_CAUSAL_TRAJECTORY
N10-OR2                           VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE
```

- canonical 事件为原生事件。
- external 事件由 900 s step trace 派生，标记 `DERIVED_FROM_EXECUTION_OUTPUT`。
- 事件时间严格比较，无 nearest-neighbor。
- 中间 Before 状态无精确同刻样本，标记 `NOT_OBSERVED_FOR_INTERMEDIATE_BOUNDARY`。
- 同刻 completion/failure 的外部顺序无法从 endpoint trace 建立，标记 PARTIAL。
- 轨迹负例：failure time shift、exact failure row removed、interruption state mismatch、同刻 order swap，全部检出。
- 合法 OR2 策略差异保持 ACCEPTED，不误判为失败。

## 未建立

```text
GENERAL RESOURCE-CAUSAL VALIDITY — NOT ESTABLISHED
STEP-LEVEL RESOURCE RELEASE — NOT ESTABLISHED
EVENT-ENGINE PRIORITY INPUT — NOT ESTABLISHED
FULL EVENT TRACE / SAME-PLEX CAUSALITY — NOT ESTABLISHED
```
