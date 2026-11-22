# S5-V N19 — Counterfactual Failure Adjudication & Oracle Soundness

基线：`7644886e15e9404bfdf3ab9672a5025a0a3d3a08`。

```text
N19-0 HISTORY/CLAIM/CERT INTEGRITY   VERIFIED
N19-A NINE-FAILURE ADJUDICATION       9/9 ORACLE_FALSE_POSITIVE
N19-B SYMMETRIC RELATIONS             VERIFIED (SCOPED)
N19-C ORACLE SOUNDNESS                VERIFIED
N19-D FOUR-RUN PROVENANCE             VERIFIED
N19-E ADVERSARIAL GENERALIZATION      VERIFIED (TESTED DOMAIN)
N19-F EVIDENCE/REPRODUCIBILITY        VERIFIED
```

## 9 个历史失败

全部从冻结输入重放，canonical/external 哈希一致；独立预期方向均为
`HIT → NO_HIT`（基线中断、干预完成）。N18 生成器只覆盖
`NO_HIT → HIT`，因此 9/9 均为 `ORACLE_FALSE_POSITIVE`
（次级原因 `EXPECTED_EFFECT_DIRECTION_ERROR`），不是 canonical/external 执行缺陷。

## 双向关系

10 组固定干预四路执行，覆盖插入、移除、双向时间、目标交换、split/shared、
Tool AND 移除；关系包括 `FAILURE_INDUCED_INTERRUPTION`、
`FAILURE_REMOVAL_RECOVERY_OF_OUTCOME`、`REPAIR_OUTCOME_UNCHANGED`。

## Oracle soundness

拒绝：错误解释、反向效应误分类、非法同刻顺序、证据篡改、双模型共同错误。
接受：合法双向反事实。

## 四路 provenance

10 对 × 4 = 40 份独立运行 manifest，哈希验证通过；确定性 replay 使用新 run_id。

## Mutants

M1 spurious interruption、M2 OR path misbinding、M3 premature shared release
均被正式 oracle 拒绝。

## Held-out

seed 1901/1919：64 attempted，50 accepted，14 unexpected 已记录，未隐藏。

## 限制

- held-out 的 14 个 unexpected 尚未逐项裁决；N19-A 只要求历史 9 个。
- 固定双向案例为 10 组，未覆盖任务表中全部 18 个 ID。
- v10 未变，无 v11。
